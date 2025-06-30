use std::collections::VecDeque;
use tokio::sync::Mutex;
use crate::task::Task;

pub struct TaskPool {
    tasks: Mutex<VecDeque<Task>>,
    completed_tasks: Mutex<std::collections::HashSet<String>>,
    capacity: usize,
}

impl TaskPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            tasks: Mutex::new(VecDeque::with_capacity(capacity)),
            completed_tasks: Mutex::new(std::collections::HashSet::new()),
            capacity,
        }
    }

    pub async fn get_task(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().await;
        tasks.pop_front()
    }

    pub async fn add_tasks(&self, new_tasks: Vec<Task>) -> usize {
        let mut tasks = self.tasks.lock().await;
        let completed = self.completed_tasks.lock().await;
        
        let mut added = 0;
        for task in new_tasks {
            if !completed.contains(&task.task_id) && !tasks.iter().any(|t| t.task_id == task.task_id) {
                if tasks.len() < self.capacity {
                    tasks.push_back(task);
                    added += 1;
                }
            }
        }
        added
    }

    pub async fn mark_completed(&self, task_id: &str) {
        let mut completed = self.completed_tasks.lock().await;
        completed.insert(task_id.to_string());
        
        if completed.len() > 1000 {
            let keys_to_remove: Vec<_> = completed.iter().take(500).cloned().collect();
            for key in keys_to_remove {
                completed.remove(&key);
            }
        }
    }

    pub async fn len(&self) -> usize {
        let tasks = self.tasks.lock().await;
        tasks.len()
    }
} 