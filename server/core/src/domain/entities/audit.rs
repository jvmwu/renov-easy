//! Audit log entity for recording authentication and security events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an audit log entry for authentication and security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLog {
    /// Unique identifier for the log entry
    pub id: Uuid,
    /// User ID if available (None for anonymous actions)
    pub user_id: Option<Uuid>,
    /// Hashed phone number if available
    pub phone_hash: Option<String>,
    /// Action being audited (e.g., "login_attempt", "send_code", "verify_code")
    pub action: String,
    /// Whether the action succeeded
    pub success: bool,
    /// IP address of the request
    pub ip_address: Option<String>,
    /// User agent string from the request
    pub user_agent: Option<String>,
    /// Error message if the action failed
    pub error_message: Option<String>,
    /// Timestamp when the event occurred
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub fn new(
        action: impl Into<String>,
        success: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            phone_hash: None,
            action: action.into(),
            success,
            ip_address: None,
            user_agent: None,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    /// Add user context to the audit log
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Add phone hash to the audit log
    pub fn with_phone_hash(mut self, phone_hash: impl Into<String>) -> Self {
        self.phone_hash = Some(phone_hash.into());
        self
    }

    /// Add request context (IP and User Agent)
    pub fn with_request_context(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    /// Add error message for failed actions
    pub fn with_error(mut self, error_message: impl Into<String>) -> Self {
        self.error_message = Some(error_message.into());
        self
    }
}

/// Common audit log actions
pub mod actions {
    /// User attempts to send verification code
    pub const SEND_CODE_ATTEMPT: &str = "send_code_attempt";
    /// User attempts to verify code
    pub const VERIFY_CODE_ATTEMPT: &str = "verify_code_attempt";
    /// User attempts to login
    pub const LOGIN_ATTEMPT: &str = "login_attempt";
    /// User attempts to refresh token
    pub const REFRESH_TOKEN_ATTEMPT: &str = "refresh_token_attempt";
    /// Rate limit exceeded
    pub const RATE_LIMIT_EXCEEDED: &str = "rate_limit_exceeded";
    /// Suspicious activity detected
    pub const SUSPICIOUS_ACTIVITY: &str = "suspicious_activity";
    /// Token validation attempt
    pub const TOKEN_VALIDATION: &str = "token_validation";
}