//! Configuration for the authentication service

/// Configuration for the authentication service
#[derive(Debug, Clone)]
pub struct AuthServiceConfig {
    /// Maximum SMS requests per phone number per hour
    pub max_sms_per_hour: i64,
    /// Hour duration in seconds for rate limiting
    pub rate_limit_window_seconds: i64,
    /// Whether to allow registration of new users
    pub allow_registration: bool,
    /// Whether to require user type selection immediately after registration
    pub require_immediate_user_type: bool,
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            max_sms_per_hour: 3,
            rate_limit_window_seconds: 3600, // 1 hour
            allow_registration: true,
            require_immediate_user_type: false,
        }
    }
}