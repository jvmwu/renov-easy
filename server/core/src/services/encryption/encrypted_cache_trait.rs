//! Trait for encrypted OTP cache service with fallback support

use async_trait::async_trait;
use super::otp_encryption::EncryptedOtp;
use crate::errors::DomainResult;

/// Storage backend type for encrypted OTP
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageBackend {
    /// Redis primary storage
    Redis,
    /// Database fallback storage
    Database,
}

/// Trait for encrypted OTP cache service with fallback support
#[async_trait]
pub trait EncryptedCacheServiceTrait: Send + Sync {
    /// Store an encrypted OTP
    async fn store_encrypted_otp(
        &self,
        encrypted_otp: &EncryptedOtp,
    ) -> DomainResult<StorageBackend>;
    
    /// Retrieve an encrypted OTP
    async fn get_encrypted_otp(&self, phone: &str) -> DomainResult<Option<EncryptedOtp>>;
    
    /// Update attempt count for an OTP
    async fn increment_attempt_count(&self, phone: &str) -> DomainResult<u32>;
    
    /// Check if an OTP exists for a phone number
    async fn encrypted_otp_exists(&self, phone: &str) -> DomainResult<bool>;
    
    /// Get time-to-live for an OTP in seconds
    async fn get_encrypted_otp_ttl(&self, phone: &str) -> DomainResult<Option<i64>>;
    
    /// Clear encrypted OTP data for a phone number
    async fn clear_encrypted_otp(&self, phone: &str) -> DomainResult<()>;
    
    /// Get current storage backend (for monitoring)
    async fn get_current_backend(&self) -> StorageBackend;
    
    /// Check if Redis is available
    async fn is_redis_available(&self) -> bool;
}