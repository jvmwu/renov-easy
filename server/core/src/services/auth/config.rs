//! Configuration for the authentication service

use shared::config::rate_limit::RateLimitConfig;

/// Configuration for the authentication service
#[derive(Debug, Clone)]
pub struct AuthServiceConfig {
    /// Rate limit configuration
    pub rate_limit: RateLimitConfig,
    /// Whether to allow registration of new users
    pub allow_registration: bool,
    /// Whether to require user type selection immediately after registration
    pub require_immediate_user_type: bool,
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            rate_limit: RateLimitConfig::default(),
            allow_registration: true,
            require_immediate_user_type: false,
        }
    }
}

impl AuthServiceConfig {
    /// Get maximum SMS requests per hour (backward compatibility)
    pub fn max_sms_per_hour(&self) -> i64 {
        self.rate_limit.max_requests() as i64
    }
    
    /// Get rate limit window in seconds (backward compatibility)
    pub fn rate_limit_window_seconds(&self) -> i64 {
        self.rate_limit.window_seconds() as i64
    }
}