//! Key management for the Nexus Network client
//!
//! Handles Ed25519 signing keys for node authentication

use ed25519_dalek::{SigningKey, VerifyingKey};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Get key storage path
pub fn get_key_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_path = home::home_dir().ok_or("Failed to get home directory")?;
    let key_path = home_path.join(".nexus").join("node.key");
    
    // Ensure directory exists
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    Ok(key_path)
}

/// Load or generate signing key
pub fn load_or_generate_signing_key() -> Result<SigningKey, Box<dyn std::error::Error>> {
    let key_path = get_key_path()?;
    
    if key_path.exists() {
        // Try to load existing key
        match load_signing_key(&key_path) {
            Ok(key) => return Ok(key),
            Err(_) => {
                // If loading fails, remove corrupted file and generate new key
                let _ = fs::remove_file(&key_path);
            }
        }
    }
    
    // Generate new key and save
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    save_signing_key(&key_path, &signing_key)?;
    
    println!("🔑 Generated new signing key: {}", key_path.display());
    Ok(signing_key)
}

/// Load signing key from file
fn load_signing_key(path: &Path) -> Result<SigningKey, Box<dyn std::error::Error>> {
    let key_bytes = fs::read(path)?;
    if key_bytes.len() != 32 {
        return Err("Invalid key file length".into());
    }
    
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    Ok(SigningKey::from_bytes(&key_array))
}

/// Save signing key to file
fn save_signing_key(path: &Path, signing_key: &SigningKey) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, signing_key.to_bytes())?;
    
    // Set file permissions to owner read/write only (600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions)?;
    }
    
    Ok(())
}

/// Validate Ethereum address format
#[allow(dead_code)]
pub fn is_valid_eth_address(address: &str) -> bool {
    address.len() == 42 && 
    address.starts_with("0x") && 
    address.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

/// Get public key from signing key
#[allow(dead_code)]
pub fn get_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
    signing_key.verifying_key()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_valid_eth_address() {
        assert!(is_valid_eth_address("0x1234567890abcdef1234567890abcdef12345678"));
        assert!(!is_valid_eth_address("0x123")); // Too short
        assert!(!is_valid_eth_address("1234567890abcdef1234567890abcdef12345678")); // No 0x prefix
        assert!(!is_valid_eth_address("0x1234567890abcdef1234567890abcdef1234567g")); // Invalid character
    }

    #[test]
    fn test_key_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test.key");
        
        let original_key = SigningKey::generate(&mut rand::thread_rng());
        save_signing_key(&key_path, &original_key).unwrap();
        
        let loaded_key = load_signing_key(&key_path).unwrap();
        assert_eq!(original_key.to_bytes(), loaded_key.to_bytes());
    }
} 