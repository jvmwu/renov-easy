//! OTP encryption service using AES-256-GCM

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use constant_time_eq::constant_time_eq;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::{DomainError, DomainResult};

use super::key_manager::{KeyManager, KeyRotationConfig};

/// Configuration for OTP encryption service
#[derive(Debug, Clone)]
pub struct OtpEncryptionConfig {
    /// Key rotation configuration
    pub key_rotation: KeyRotationConfig,
    /// Whether to use database fallback when Redis fails
    pub enable_db_fallback: bool,
}

impl Default for OtpEncryptionConfig {
    fn default() -> Self {
        Self {
            key_rotation: KeyRotationConfig::default(),
            enable_db_fallback: true,
        }
    }
}

/// Encrypted OTP with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedOtp {
    /// Encrypted OTP ciphertext (base64 encoded)
    pub ciphertext: String,
    /// Nonce used for encryption (base64 encoded)
    pub nonce: String,
    /// Key ID used for encryption
    pub key_id: String,
    /// When the OTP was created
    pub created_at: DateTime<Utc>,
    /// Number of verification attempts
    pub attempt_count: u32,
    /// When the OTP expires
    pub expires_at: DateTime<Utc>,
    /// Phone number (for reference, not encrypted)
    pub phone: String,
}

/// Trait defining OTP encryption operations
pub trait OtpEncryption: Send + Sync {
    /// Encrypt an OTP
    fn encrypt_otp(
        &self,
        otp: &str,
        phone: &str,
        expiration_minutes: u32,
    ) -> DomainResult<EncryptedOtp>;
    
    /// Decrypt an OTP
    fn decrypt_otp(&self, encrypted: &EncryptedOtp) -> DomainResult<String>;
    
    /// Verify an OTP using constant-time comparison
    fn verify_otp(&self, encrypted: &EncryptedOtp, provided_otp: &str) -> DomainResult<bool>;
    
    /// Check if key rotation is needed
    fn should_rotate_key(&self) -> bool;
    
    /// Rotate encryption key
    fn rotate_key(&self) -> DomainResult<String>;
}

/// AES-GCM based OTP encryption implementation
pub struct AesGcmOtpEncryption {
    /// Key manager for handling encryption keys
    key_manager: Arc<KeyManager>,
    /// Service configuration
    config: OtpEncryptionConfig,
}

impl AesGcmOtpEncryption {
    /// Create a new OTP encryption service
    pub fn new(config: OtpEncryptionConfig) -> DomainResult<Self> {
        let key_manager = Arc::new(KeyManager::new(config.key_rotation.clone())?);
        
        Ok(Self {
            key_manager,
            config,
        })
    }
    
    /// Create with a specific key (for testing)
    pub fn with_key(key: Vec<u8>, config: OtpEncryptionConfig) -> DomainResult<Self> {
        let key_manager = Arc::new(KeyManager::with_key(key, config.key_rotation.clone())?);
        
        Ok(Self {
            key_manager,
            config,
        })
    }
    
    /// Generate a random nonce for AES-GCM
    fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }
    
    /// Perform AES-256-GCM encryption
    fn encrypt_with_key(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> DomainResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(DomainError::Internal {
                message: "Invalid key size for AES-256".to_string(),
            });
        }
        
        if nonce.len() != 12 {
            return Err(DomainError::Internal {
                message: "Invalid nonce size for AES-GCM".to_string(),
            });
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| DomainError::Internal {
                message: format!("Encryption failed: {}", e),
            })
    }
    
    /// Perform AES-256-GCM decryption
    fn decrypt_with_key(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> DomainResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(DomainError::Internal {
                message: "Invalid key size for AES-256".to_string(),
            });
        }
        
        if nonce.len() != 12 {
            return Err(DomainError::Internal {
                message: "Invalid nonce size for AES-GCM".to_string(),
            });
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DomainError::Internal {
                message: format!("Decryption failed: {}", e),
            })
    }
}

impl OtpEncryption for AesGcmOtpEncryption {
    fn encrypt_otp(
        &self,
        otp: &str,
        phone: &str,
        expiration_minutes: u32,
    ) -> DomainResult<EncryptedOtp> {
        // Get active encryption key
        let key_info = self.key_manager.get_active_key()?;
        
        // Generate nonce
        let nonce = Self::generate_nonce();
        
        // Encrypt OTP
        let ciphertext = self.encrypt_with_key(otp.as_bytes(), &key_info.key, &nonce)?;
        
        // Create encrypted OTP structure
        Ok(EncryptedOtp {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce),
            key_id: key_info.id,
            created_at: Utc::now(),
            attempt_count: 0,
            expires_at: Utc::now() + chrono::Duration::minutes(expiration_minutes as i64),
            phone: phone.to_string(),
        })
    }
    
    fn decrypt_otp(&self, encrypted: &EncryptedOtp) -> DomainResult<String> {
        // Get the key used for encryption
        let key_info = self.key_manager.get_key(&encrypted.key_id)?;
        
        // Decode base64
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to decode ciphertext: {}", e),
            })?;
        
        let nonce = BASE64
            .decode(&encrypted.nonce)
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to decode nonce: {}", e),
            })?;
        
        // Decrypt
        let plaintext = self.decrypt_with_key(&ciphertext, &key_info.key, &nonce)?;
        
        // Convert to string
        String::from_utf8(plaintext).map_err(|e| DomainError::Internal {
            message: format!("Failed to convert decrypted OTP to string: {}", e),
        })
    }
    
    fn verify_otp(&self, encrypted: &EncryptedOtp, provided_otp: &str) -> DomainResult<bool> {
        // Check if OTP has expired
        if Utc::now() > encrypted.expires_at {
            return Ok(false);
        }
        
        // Decrypt the stored OTP
        let stored_otp = self.decrypt_otp(encrypted)?;
        
        // Use constant-time comparison to prevent timing attacks
        let stored_bytes = stored_otp.as_bytes();
        let provided_bytes = provided_otp.as_bytes();
        
        // Both must be the same length for constant-time comparison
        if stored_bytes.len() != provided_bytes.len() {
            return Ok(false);
        }
        
        Ok(constant_time_eq(stored_bytes, provided_bytes))
    }
    
    fn should_rotate_key(&self) -> bool {
        self.key_manager.should_rotate()
    }
    
    fn rotate_key(&self) -> DomainResult<String> {
        self.key_manager.rotate_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt_otp() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "123456";
        let phone = "+1234567890";
        
        // Encrypt
        let encrypted = service.encrypt_otp(otp, phone, 5).unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());
        assert!(!encrypted.key_id.is_empty());
        assert_eq!(encrypted.phone, phone);
        
        // Decrypt
        let decrypted = service.decrypt_otp(&encrypted).unwrap();
        assert_eq!(decrypted, otp);
    }
    
    #[test]
    fn test_verify_otp_success() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "654321";
        let phone = "+9876543210";
        
        let encrypted = service.encrypt_otp(otp, phone, 5).unwrap();
        
        // Correct OTP should verify
        assert!(service.verify_otp(&encrypted, otp).unwrap());
        
        // Wrong OTP should not verify
        assert!(!service.verify_otp(&encrypted, "111111").unwrap());
        
        // Different length OTP should not verify
        assert!(!service.verify_otp(&encrypted, "65432").unwrap());
    }
    
    #[test]
    fn test_verify_expired_otp() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "123456";
        let phone = "+1234567890";
        
        // Create an expired OTP
        let mut encrypted = service.encrypt_otp(otp, phone, 5).unwrap();
        encrypted.expires_at = Utc::now() - chrono::Duration::minutes(1);
        
        // Should return false for expired OTP
        assert!(!service.verify_otp(&encrypted, otp).unwrap());
    }
    
    #[test]
    fn test_key_rotation() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "999999";
        let phone = "+1112223333";
        
        // Encrypt with original key
        let encrypted1 = service.encrypt_otp(otp, phone, 5).unwrap();
        
        // Rotate key
        let new_key_id = service.rotate_key().unwrap();
        assert!(!new_key_id.is_empty());
        
        // Encrypt with new key
        let encrypted2 = service.encrypt_otp(otp, phone, 5).unwrap();
        
        // Key IDs should be different
        assert_ne!(encrypted1.key_id, encrypted2.key_id);
        
        // Both should still be decryptable
        assert_eq!(service.decrypt_otp(&encrypted1).unwrap(), otp);
        assert_eq!(service.decrypt_otp(&encrypted2).unwrap(), otp);
    }
    
    #[test]
    fn test_constant_time_comparison() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "123456";
        let phone = "+1234567890";
        
        let encrypted = service.encrypt_otp(otp, phone, 5).unwrap();
        
        // Test various wrong OTPs to ensure constant-time comparison
        let wrong_otps = vec![
            "000000", // All different
            "123455", // Last digit different
            "223456", // First digit different
            "123457", // One off
        ];
        
        for wrong_otp in wrong_otps {
            assert!(!service.verify_otp(&encrypted, wrong_otp).unwrap());
        }
        
        // Correct OTP should still work
        assert!(service.verify_otp(&encrypted, otp).unwrap());
    }
    
    #[test]
    fn test_different_nonces() {
        let config = OtpEncryptionConfig::default();
        let service = AesGcmOtpEncryption::new(config).unwrap();
        
        let otp = "123456";
        let phone = "+1234567890";
        
        // Encrypt same OTP multiple times
        let encrypted1 = service.encrypt_otp(otp, phone, 5).unwrap();
        let encrypted2 = service.encrypt_otp(otp, phone, 5).unwrap();
        
        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        
        // Ciphertexts should be different (due to different nonces)
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        
        // Both should decrypt to the same OTP
        assert_eq!(service.decrypt_otp(&encrypted1).unwrap(), otp);
        assert_eq!(service.decrypt_otp(&encrypted2).unwrap(), otp);
    }
}