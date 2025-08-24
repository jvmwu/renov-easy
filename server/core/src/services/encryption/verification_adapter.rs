//! Adapter to integrate OTP encryption with the verification service

use async_trait::async_trait;
use std::sync::Arc;

use crate::errors::{DomainError, DomainResult};
use crate::services::verification::CacheServiceTrait;

use super::{
    encrypted_cache_trait::{EncryptedCacheServiceTrait, StorageBackend},
    otp_encryption::{OtpEncryption, EncryptedOtp},
};

/// Adapter that bridges the verification service with encrypted OTP storage
pub struct EncryptedVerificationAdapter<E, C> 
where
    E: OtpEncryption,
    C: EncryptedCacheServiceTrait,
{
    /// OTP encryption service
    encryption_service: Arc<E>,
    /// Encrypted cache service
    cache_service: Arc<C>,
    /// Default OTP expiration in minutes
    default_expiration_minutes: u32,
    /// Maximum verification attempts
    max_attempts: u32,
}

impl<E, C> EncryptedVerificationAdapter<E, C>
where
    E: OtpEncryption,
    C: EncryptedCacheServiceTrait,
{
    /// Create a new encrypted verification adapter
    pub fn new(
        encryption_service: Arc<E>,
        cache_service: Arc<C>,
        default_expiration_minutes: u32,
        max_attempts: u32,
    ) -> Self {
        Self {
            encryption_service,
            cache_service,
            default_expiration_minutes,
            max_attempts,
        }
    }
    
    /// Log a warning for fallback to database
    fn log_fallback_warning(&self, backend: StorageBackend) {
        if backend == StorageBackend::Database {
            // In production, this would integrate with your logging system
            eprintln!(
                "WARNING: OTP storage fallen back to database. Redis connection may be unavailable."
            );
        }
    }
}

#[async_trait]
impl<E, C> CacheServiceTrait for EncryptedVerificationAdapter<E, C>
where
    E: OtpEncryption + Send + Sync,
    C: EncryptedCacheServiceTrait + Send + Sync,
{
    async fn store_code(&self, phone: &str, code: &str) -> Result<(), String> {
        // Encrypt the OTP
        let encrypted_otp = self.encryption_service
            .encrypt_otp(code, phone, self.default_expiration_minutes)
            .map_err(|e| format!("Failed to encrypt OTP: {:?}", e))?;
        
        // Store the encrypted OTP
        let backend = self.cache_service
            .store_encrypted_otp(&encrypted_otp)
            .await
            .map_err(|e| format!("Failed to store encrypted OTP: {:?}", e))?;
        
        // Log warning if using database fallback
        self.log_fallback_warning(backend);
        
        Ok(())
    }
    
    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String> {
        // Retrieve the encrypted OTP
        let encrypted_otp = self.cache_service
            .get_encrypted_otp(phone)
            .await
            .map_err(|e| format!("Failed to retrieve encrypted OTP: {:?}", e))?;
        
        match encrypted_otp {
            Some(mut encrypted) => {
                // Check attempt count
                if encrypted.attempt_count >= self.max_attempts {
                    // Clear the OTP if max attempts exceeded
                    let _ = self.cache_service.clear_encrypted_otp(phone).await;
                    return Ok(false);
                }
                
                // Increment attempt count
                let new_count = self.cache_service
                    .increment_attempt_count(phone)
                    .await
                    .map_err(|e| format!("Failed to increment attempt count: {:?}", e))?;
                
                encrypted.attempt_count = new_count;
                
                // Verify using constant-time comparison
                let is_valid = self.encryption_service
                    .verify_otp(&encrypted, code)
                    .map_err(|e| format!("Failed to verify OTP: {:?}", e))?;
                
                if is_valid {
                    // Clear the OTP after successful verification
                    let _ = self.cache_service.clear_encrypted_otp(phone).await;
                }
                
                Ok(is_valid)
            }
            None => Ok(false),
        }
    }
    
    async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String> {
        let encrypted_otp = self.cache_service
            .get_encrypted_otp(phone)
            .await
            .map_err(|e| format!("Failed to retrieve encrypted OTP: {:?}", e))?;
        
        match encrypted_otp {
            Some(encrypted) => {
                let remaining = self.max_attempts.saturating_sub(encrypted.attempt_count) as i64;
                Ok(remaining)
            }
            None => Ok(0),
        }
    }
    
    async fn code_exists(&self, phone: &str) -> Result<bool, String> {
        self.cache_service
            .encrypted_otp_exists(phone)
            .await
            .map_err(|e| format!("Failed to check OTP existence: {:?}", e))
    }
    
    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String> {
        self.cache_service
            .get_encrypted_otp_ttl(phone)
            .await
            .map_err(|e| format!("Failed to get OTP TTL: {:?}", e))
    }
    
    async fn clear_verification(&self, phone: &str) -> Result<(), String> {
        self.cache_service
            .clear_encrypted_otp(phone)
            .await
            .map_err(|e| format!("Failed to clear encrypted OTP: {:?}", e))
    }
}