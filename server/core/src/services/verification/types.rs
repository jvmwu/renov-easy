//! Types for verification service results

use chrono::{DateTime, Utc};
use crate::domain::entities::verification_code::VerificationCode;

/// Result of sending a verification code
#[derive(Debug, Clone)]
pub struct SendCodeResult {
    /// The verification code entity that was created
    pub verification_code: VerificationCode,
    /// The SMS message ID from the provider
    pub message_id: String,
    /// When the user can request another code
    pub next_resend_at: DateTime<Utc>,
}

/// Result of verifying a code
#[derive(Debug, Clone)]
pub struct VerifyCodeResult {
    /// Whether the verification was successful
    pub success: bool,
    /// Number of remaining attempts (if verification failed)
    pub remaining_attempts: Option<i32>,
    /// Error message if verification failed
    pub error_message: Option<String>,
}