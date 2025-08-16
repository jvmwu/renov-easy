//! Traits for SMS and cache service integration

use async_trait::async_trait;

/// Trait for SMS service integration
#[async_trait]
pub trait SmsServiceTrait: Send + Sync {
    /// Send a verification code via SMS
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String>;
    /// Check if the phone number format is valid
    fn is_valid_phone_number(&self, phone: &str) -> bool;
}

/// Trait for cache service integration
#[async_trait]
pub trait CacheServiceTrait: Send + Sync {
    /// Store a verification code with expiration
    async fn store_code(&self, phone: &str, code: &str) -> Result<(), String>;
    /// Verify a code and track attempts
    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String>;
    /// Get remaining verification attempts
    async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String>;
    /// Check if a code exists for a phone number
    async fn code_exists(&self, phone: &str) -> Result<bool, String>;
    /// Get time-to-live for a verification code in seconds
    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String>;
    /// Clear verification data for a phone number
    async fn clear_verification(&self, phone: &str) -> Result<(), String>;
}