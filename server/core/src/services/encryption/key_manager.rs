//! Key management utilities for OTP encryption

use aes_gcm::{Aes256Gcm, Key};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, RngCore};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::{DomainError, DomainResult};

/// Key rotation configuration
#[derive(Debug, Clone)]
pub struct KeyRotationConfig {
    /// Maximum age of a key in days before rotation
    pub max_key_age_days: u32,
    /// Whether to keep old keys for decryption
    pub keep_old_keys: bool,
    /// Maximum number of old keys to retain
    pub max_old_keys: usize,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            max_key_age_days: 30,
            keep_old_keys: true,
            max_old_keys: 3,
        }
    }
}

/// Encryption key with metadata
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// The actual key material
    pub key: Vec<u8>,
    /// Key identifier
    pub id: String,
    /// When the key was created
    pub created_at: DateTime<Utc>,
    /// Whether this is the active key for encryption
    pub is_active: bool,
}

/// Key manager for handling encryption keys
pub struct KeyManager {
    /// Active key for encryption
    active_key: Arc<RwLock<EncryptionKey>>,
    /// All keys (including old ones for decryption)
    all_keys: Arc<RwLock<HashMap<String, EncryptionKey>>>,
    /// Key rotation configuration
    config: KeyRotationConfig,
}

impl KeyManager {
    /// Create a new key manager with a generated key
    pub fn new(config: KeyRotationConfig) -> DomainResult<Self> {
        let key = Self::generate_key()?;
        let key_id = Self::generate_key_id();
        
        let encryption_key = EncryptionKey {
            key: key.clone(),
            id: key_id.clone(),
            created_at: Utc::now(),
            is_active: true,
        };
        
        let mut all_keys = HashMap::new();
        all_keys.insert(key_id.clone(), encryption_key.clone());
        
        Ok(Self {
            active_key: Arc::new(RwLock::new(encryption_key)),
            all_keys: Arc::new(RwLock::new(all_keys)),
            config,
        })
    }
    
    /// Create a new key manager with a provided key (for testing or recovery)
    pub fn with_key(key: Vec<u8>, config: KeyRotationConfig) -> DomainResult<Self> {
        if key.len() != 32 {
            return Err(DomainError::Validation {
                message: "Encryption key must be 32 bytes (256 bits)".to_string(),
            });
        }
        
        let key_id = Self::generate_key_id();
        let encryption_key = EncryptionKey {
            key: key.clone(),
            id: key_id.clone(),
            created_at: Utc::now(),
            is_active: true,
        };
        
        let mut all_keys = HashMap::new();
        all_keys.insert(key_id.clone(), encryption_key.clone());
        
        Ok(Self {
            active_key: Arc::new(RwLock::new(encryption_key)),
            all_keys: Arc::new(RwLock::new(all_keys)),
            config,
        })
    }
    
    /// Generate a new 256-bit key
    pub fn generate_key() -> DomainResult<Vec<u8>> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        Ok(key)
    }
    
    /// Generate a unique key identifier
    fn generate_key_id() -> String {
        let mut bytes = [0u8; 8];
        OsRng.fill_bytes(&mut bytes);
        BASE64.encode(bytes)
    }
    
    /// Get the active encryption key
    pub fn get_active_key(&self) -> DomainResult<EncryptionKey> {
        self.active_key
            .read()
            .map(|key| key.clone())
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to read active key: {}", e),
            })
    }
    
    /// Get a specific key by ID
    pub fn get_key(&self, key_id: &str) -> DomainResult<EncryptionKey> {
        self.all_keys
            .read()
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to read keys: {}", e),
            })?
            .get(key_id)
            .cloned()
            .ok_or_else(|| DomainError::NotFound {
                resource: format!("Encryption key: {}", key_id),
            })
    }
    
    /// Rotate to a new encryption key
    pub fn rotate_key(&self) -> DomainResult<String> {
        let new_key = Self::generate_key()?;
        let new_key_id = Self::generate_key_id();
        
        let new_encryption_key = EncryptionKey {
            key: new_key,
            id: new_key_id.clone(),
            created_at: Utc::now(),
            is_active: true,
        };
        
        // Update active key
        {
            let mut active_key = self.active_key.write().map_err(|e| {
                DomainError::Internal {
                    message: format!("Failed to write active key: {}", e),
                }
            })?;
            
            // Mark old key as inactive
            active_key.is_active = false;
            
            // Update all_keys map
            let mut all_keys = self.all_keys.write().map_err(|e| {
                DomainError::Internal {
                    message: format!("Failed to write keys: {}", e),
                }
            })?;
            
            // Add old key if we're keeping them
            if self.config.keep_old_keys {
                all_keys.insert(active_key.id.clone(), active_key.clone());
                
                // Remove oldest keys if we exceed the limit
                if all_keys.len() > self.config.max_old_keys + 1 {
                    let mut keys_by_age: Vec<_> = all_keys
                        .values()
                        .filter(|k| !k.is_active)
                        .map(|k| (k.created_at, k.id.clone()))
                        .collect();
                    keys_by_age.sort_by_key(|k| k.0);
                    
                    let keys_to_remove = keys_by_age.len() - self.config.max_old_keys;
                    for i in 0..keys_to_remove {
                        all_keys.remove(&keys_by_age[i].1);
                    }
                }
            }
            
            // Add new key
            all_keys.insert(new_key_id.clone(), new_encryption_key.clone());
            
            // Update active key reference
            *active_key = new_encryption_key;
        }
        
        Ok(new_key_id)
    }
    
    /// Check if key rotation is needed
    pub fn should_rotate(&self) -> bool {
        let active_key = match self.active_key.read() {
            Ok(key) => key.clone(),
            Err(_) => return false,
        };
        
        let age_days = (Utc::now() - active_key.created_at).num_days();
        age_days >= self.config.max_key_age_days as i64
    }
    
    /// Get all key IDs (for debugging/monitoring)
    pub fn get_all_key_ids(&self) -> DomainResult<Vec<String>> {
        self.all_keys
            .read()
            .map(|keys| keys.keys().cloned().collect())
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to read keys: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key = KeyManager::generate_key().unwrap();
        assert_eq!(key.len(), 32);
        
        // Ensure keys are different each time
        let key2 = KeyManager::generate_key().unwrap();
        assert_ne!(key, key2);
    }
    
    #[test]
    fn test_key_manager_creation() {
        let config = KeyRotationConfig::default();
        let manager = KeyManager::new(config).unwrap();
        
        let active_key = manager.get_active_key().unwrap();
        assert_eq!(active_key.key.len(), 32);
        assert!(active_key.is_active);
    }
    
    #[test]
    fn test_key_rotation() {
        let config = KeyRotationConfig {
            keep_old_keys: true,
            max_old_keys: 2,
            ..Default::default()
        };
        let manager = KeyManager::new(config).unwrap();
        
        let original_key_id = manager.get_active_key().unwrap().id;
        
        // Rotate key
        let new_key_id = manager.rotate_key().unwrap();
        assert_ne!(original_key_id, new_key_id);
        
        // Check active key changed
        let active_key = manager.get_active_key().unwrap();
        assert_eq!(active_key.id, new_key_id);
        assert!(active_key.is_active);
        
        // Check old key is still available
        let old_key = manager.get_key(&original_key_id).unwrap();
        assert!(!old_key.is_active);
    }
    
    #[test]
    fn test_key_rotation_with_limit() {
        let config = KeyRotationConfig {
            keep_old_keys: true,
            max_old_keys: 2,
            ..Default::default()
        };
        let manager = KeyManager::new(config).unwrap();
        
        let mut key_ids = vec![manager.get_active_key().unwrap().id];
        
        // Rotate keys multiple times
        for _ in 0..3 {
            let new_id = manager.rotate_key().unwrap();
            key_ids.push(new_id);
        }
        
        // Check we have at most max_old_keys + 1 (active) keys
        let all_ids = manager.get_all_key_ids().unwrap();
        assert!(all_ids.len() <= 3); // 2 old + 1 active
        
        // Oldest key should be removed
        assert!(manager.get_key(&key_ids[0]).is_err());
    }
}