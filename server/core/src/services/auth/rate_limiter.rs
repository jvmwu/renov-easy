//! Rate limiting traits and implementations for authentication service

use async_trait::async_trait;

/// Rate limiting service trait for tracking SMS requests
#[async_trait]
pub trait RateLimiterTrait: Send + Sync {
    /// Check if a phone number has exceeded the rate limit for SMS requests
    async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String>;
    
    /// Increment the SMS request counter for a phone number
    async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String>;
    
    /// Get the remaining time until rate limit resets (in seconds)
    async fn get_rate_limit_reset_time(&self, phone: &str) -> Result<Option<i64>, String>;
}