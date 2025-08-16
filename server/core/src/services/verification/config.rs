//! Configuration for the verification service

use crate::domain::entities::verification_code::{DEFAULT_EXPIRATION_MINUTES, MAX_ATTEMPTS};

/// Configuration for the verification service
#[derive(Debug, Clone)]
pub struct VerificationServiceConfig {
    /// Number of minutes before a verification code expires
    pub code_expiration_minutes: i64,
    /// Maximum number of verification attempts allowed
    pub max_attempts: i32,
    /// Whether to use mock SMS service (for development)
    pub use_mock_sms: bool,
    /// Minimum seconds between code resend requests
    pub resend_cooldown_seconds: i64,
}

impl Default for VerificationServiceConfig {
    fn default() -> Self {
        Self {
            code_expiration_minutes: DEFAULT_EXPIRATION_MINUTES,
            max_attempts: MAX_ATTEMPTS,
            use_mock_sms: false,
            resend_cooldown_seconds: 60,
        }
    }
}