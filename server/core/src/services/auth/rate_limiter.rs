//! Rate limiting traits and implementations for authentication service

use async_trait::async_trait;

/// Rate limiting service trait for tracking SMS and verification requests
#[async_trait]
pub trait RateLimiterTrait: Send + Sync {
    /// Check if a phone number has exceeded the rate limit for SMS requests
    async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String>;
    
    /// Increment the SMS request counter for a phone number
    async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String>;
    
    /// Get the remaining time until rate limit resets (in seconds)
    async fn get_rate_limit_reset_time(&self, phone: &str) -> Result<Option<i64>, String>;
    
    /// Check if an IP has exceeded the rate limit for verification attempts
    async fn check_ip_verification_limit(&self, ip: &str) -> Result<bool, String>;
    
    /// Increment the verification counter for an IP address
    async fn increment_ip_verification_counter(&self, ip: &str) -> Result<i64, String>;
    
    /// Get the remaining time until IP rate limit resets (in seconds)
    async fn get_ip_rate_limit_reset_time(&self, ip: &str) -> Result<Option<i64>, String>;
    
    /// Log a rate limit violation to audit log (if available)
    async fn log_rate_limit_violation(
        &self, 
        identifier: &str, 
        identifier_type: &str,
        action: &str
    ) -> Result<(), String>;
}