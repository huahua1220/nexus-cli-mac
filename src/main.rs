// Copyright (c) 2024 Nexus. All rights reserved.

use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::error::Error;
use std::path::PathBuf;
use std::io;
use tokio::signal;
use tokio::task::JoinSet;
use std::sync::Arc;

mod analytics;
mod config;
mod environment;
mod keys;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod orchestrator_client;
mod prover;
mod setup;
mod task;
mod ui;
mod utils;
mod node_list;

use crate::config::Config;
use crate::environment::Environment;
use crate::setup::clear_node_config;
use crate::node_list::NodeList;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Command-line arguments
struct Args {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the prover
    Start {
        /// Node ID
        #[arg(long, value_name = "NODE_ID")]
        node_id: Option<u64>,

        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,
    },
    
    /// Start multiple provers from node list file
    BatchFile {
        /// Path to node list file (.txt)
        #[arg(long, value_name = "FILE_PATH")]
        file: String,

        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,

        /// Delay between starting each node (seconds)
        #[arg(long, default_value = "2")]
        start_delay: u64,

        /// Delay between proof submissions per node (seconds)
        #[arg(long, default_value = "3")]
        proof_interval: u64,

        /// Maximum number of concurrent nodes
        #[arg(long, default_value = "10")]
        max_concurrent: usize,

        /// Enable verbose error logging
        #[arg(long)]
        verbose: bool,
    },

    /// Create example node list files
    CreateExamples {
        /// Directory to create example files
        #[arg(long, default_value = "./examples")]
        dir: String,
    },
    
    /// Logout from the current session
    Logout,
}

/// Get the path to the Nexus config file, typically located at ~/.nexus/config.json.
fn get_config_path() -> Result<PathBuf, ()> {
    let home_path = home::home_dir().expect("Failed to get home directory");
    let config_path = home_path.join(".nexus").join("config.json");
    Ok(config_path)
}

// Fixed line display manager
#[derive(Debug)]
struct FixedLineDisplay {
    #[allow(dead_code)]
    max_lines: usize,
    node_lines: Arc<tokio::sync::RwLock<std::collections::HashMap<u64, String>>>,
    last_render_hash: Arc<tokio::sync::Mutex<u64>>,
}

impl FixedLineDisplay {
    fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            node_lines: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::with_capacity(max_lines))),
            last_render_hash: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    async fn update_node_status(&self, node_id: u64, status: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let formatted_status = format!("[{}] {}", timestamp, status);
        
        let needs_update = {
            let lines = self.node_lines.read().await;
            lines.get(&node_id) != Some(&formatted_status)
        };
        
        if needs_update {
            {
                let mut lines = self.node_lines.write().await;
                lines.insert(node_id, formatted_status.clone());
            }
            self.render_display_optimized().await;
        }
    }

    #[allow(dead_code)]
    async fn remove_node(&self, node_id: u64) {
        {
            let mut lines = self.node_lines.write().await;
            lines.remove(&node_id);
        }
    }



    async fn render_display_optimized(&self) {
        let lines = self.node_lines.read().await;
        
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for (id, status) in lines.iter() {
            std::hash::Hasher::write_u64(&mut hasher, *id);
            std::hash::Hasher::write(&mut hasher, status.as_bytes());
        }
        let current_hash = std::hash::Hasher::finish(&hasher);
        
        let mut last_hash = self.last_render_hash.lock().await;
        if *last_hash != current_hash {
            *last_hash = current_hash;
            drop(last_hash);
            self.render_display(&lines).await;
        }
    }

    async fn render_display(&self, lines: &std::collections::HashMap<u64, String>) {
        // Clear screen and move to top
        print!("\x1b[2J\x1b[H");
        
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Title
        println!("🚀 Nexus Batch Mining Monitor - {}", current_time);
        println!("═══════════════════════════════════════");
        
        // Statistics
        let total_nodes = lines.len();
        let successful_count = lines.values()
            .filter(|status| status.contains("✅"))
            .count();
        let failed_count = lines.values()
            .filter(|status| status.contains("❌"))
            .count();
        let active_count = lines.values()
            .filter(|status| status.contains("🔄") || status.contains("⚠️"))
            .count();
        
        println!("📊 Status: {} Total | {} Active | {} Success | {} Failed", 
                 total_nodes, active_count, successful_count, failed_count);
        

        
        println!("───────────────────────────────────────");
        
        // Display sorted by node ID
        let mut sorted_lines: Vec<_> = lines.iter().collect();
        sorted_lines.sort_by_key(|(id, _)| *id);
        
        for (node_id, status) in sorted_lines {
            println!("Node-{}: {}", node_id, status);
        }
        
        println!("───────────────────────────────────────");
        println!("💡 Press Ctrl+C to stop all miners");
        
        // Force output flush
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();
    
    let args = Args::parse();
    match args.command {
        Command::Start { node_id, env } => {
            let mut node_id = node_id;
            // If no node ID is provided, try to load it from the config file.
            let config_path = get_config_path().expect("Failed to get config path");
            if node_id.is_none() && config_path.exists() {
                if let Ok(config) = Config::load_from_file(&config_path) {
                    let node_id_as_u64 = config
                        .node_id
                        .parse::<u64>()
                        .expect("Failed to parse node ID");
                    node_id = Some(node_id_as_u64);
                }
            }

            let environment = env.unwrap_or_default();
            start(node_id, environment).await
        }
        
        Command::BatchFile {
            file,
            env,
            start_delay,
            proof_interval,
            max_concurrent,
            verbose,
        } => {
            if verbose {
                std::env::set_var("RUST_LOG", "debug");
                env_logger::init();
            }
            let environment = env.unwrap_or_default();
            start_batch_from_file_with_pool(&file, environment, start_delay, proof_interval, max_concurrent, verbose).await
        }

        Command::CreateExamples { dir } => {
            NodeList::create_example_files(&dir)
                .map_err(|e| -> Box<dyn Error> { Box::new(e) })?;
            
            println!("🎉 Example node list files created successfully!");
            println!("📂 Location: {}", dir);
            println!("💡 Edit these files with your actual node IDs, then use:");
            println!("   nexus batch-file --file {}/example_nodes.txt", dir);
            Ok(())
        }
        
        Command::Logout => {
            let config_path = get_config_path().expect("Failed to get config path");
            clear_node_config(&config_path).map_err(Into::into)
        }
    }
}

/// Starts the Nexus CLI application.
async fn start(node_id: Option<u64>, env: Environment) -> Result<(), Box<dyn Error>> {
    if node_id.is_some() {
        // Use headless mode for single node with ID
        start_headless_prover(node_id, env).await
    } else {
        // Use UI mode for interactive setup
        start_with_ui(node_id, env).await
    }
}

/// Start with UI (original logic)
async fn start_with_ui(
    node_id: Option<u64>,
    env: Environment,
) -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let res = ui::run(&mut terminal, ui::App::new(node_id, env, crate::orchestrator_client::OrchestratorClient::new(env)));

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn start_headless_prover(
    node_id: Option<u64>,
    env: Environment,
) -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Nexus Prover in headless mode...");
    prover::start_prover(env, node_id).await?;
    Ok(())
}

async fn start_batch_from_file_with_pool(
    file_path: &str,
    env: Environment,
    start_delay: u64,
    proof_interval: u64,
    max_concurrent: usize,
    _verbose: bool, // Unused parameter for interface compatibility
) -> Result<(), Box<dyn Error>> {
    // Load node list
    let node_list = NodeList::load_from_file(file_path)?;
    let all_nodes = node_list.node_ids().to_vec();
    
    if all_nodes.is_empty() {
        return Err("Node list is empty".into());
    }
    
    let actual_concurrent = std::cmp::min(max_concurrent, all_nodes.len());
    
    println!("🚀 Starting batch processing from file: {}", file_path);
    println!("📊 Total nodes: {}", all_nodes.len());
    println!("🔄 Max concurrent: {}", actual_concurrent);
    println!("⏱️  Start delay: {}s, Proof interval: {}s", start_delay, proof_interval);
    println!("🌍 Environment: {:?}", env);
    println!("♾️  Mode: Infinite retry (no node replacement)");
    println!("═══════════════════════════════════════");

    // Create display manager
    let display = Arc::new(FixedLineDisplay::new(actual_concurrent));
    
    // Initial display
    display.render_display(&std::collections::HashMap::new()).await;

    let mut join_set = JoinSet::new();
    
    // Start concurrent nodes
    for &node_id in all_nodes.iter().take(actual_concurrent) {
        let disp = display.clone();
        
        join_set.spawn(async move {
            let display_callback = {
                let disp = disp.clone();
                move |status: String| {
                    let disp = disp.clone();
                    let node_id = node_id;
                    tokio::spawn(async move {
                        disp.update_node_status(node_id, status).await;
                    });
                }
            };
            
            // Each node will retry infinitely until manually interrupted
            let result = prover::start_prover_with_callback(env, Some(node_id), proof_interval, display_callback).await;
            
            // This should not be reached as nodes retry infinitely
            match &result {
                Ok(_) => disp.update_node_status(node_id, "✅ Completed".to_string()).await,
                Err(e) => disp.update_node_status(node_id, format!("❌ Unexpected exit: {}", e)).await,
            }
            
            (node_id, result)
        });
        
        tokio::time::sleep(std::time::Duration::from_secs(start_delay)).await;
    }
    
    println!("✅ All {} provers started with infinite retry!", actual_concurrent);
    println!("📋 Unused nodes: {}", all_nodes.len() - actual_concurrent);
    println!("🛑 Press Ctrl+C to stop all provers");
    
    // Simplified monitoring: wait for interrupt signal
    monitor_infinite_retry(join_set, display).await;
    
    Ok(())
}



/// Monitor tasks with infinite retry (no replacement)
async fn monitor_infinite_retry(
    mut join_set: JoinSet<(u64, Result<(), prover::ProverError>)>,
    display: Arc<FixedLineDisplay>,
) {
    tokio::pin! {
        let ctrl_c = signal::ctrl_c();
    }
    
    loop {
        tokio::select! {
            // Handle completed tasks (should not happen with infinite retry)
            Some(result) = join_set.join_next() => {
                if let Ok((node_id, prover_result)) = result {
                    match prover_result {
                        Ok(_) => {
                            println!("🎯 [Node-{}] Prover completed successfully (unexpected)", node_id);
                            display.update_node_status(node_id, "✅ Completed".to_string()).await;
                        },
                        Err(e) => {
                            println!("❌ [Node-{}] Prover exited unexpectedly: {}", node_id, e);
                            display.update_node_status(node_id, format!("❌ Unexpected exit: {}", e)).await;
                        }
                    }
                }
            }
            
            // Handle shutdown signal
            _ = &mut ctrl_c => {
                println!("🛑 Shutdown signal received. Stopping all provers...");
                join_set.abort_all();
                break;
            }
            
            // Exit monitoring if all tasks unexpectedly exit
            else => {
                println!("⚠️ All nodes have exited unexpectedly.");
                break;
            }
        }
    }
    
    println!("✅ All provers stopped.");
} 