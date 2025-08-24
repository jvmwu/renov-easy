//! Audit log entity for recording authentication and security events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Event types for comprehensive authentication auditing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    // Login events
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    
    // Code sending events
    SendCodeRequest,
    SendCodeSuccess,
    SendCodeFailure,
    
    // Code verification events
    VerifyCodeAttempt,
    VerifyCodeSuccess,
    VerifyCodeFailure,
    
    // Token events
    TokenGenerated,
    TokenRefreshed,
    TokenRevoked,
    TokenValidation,
    TokenValidationFailure,
    
    // Rate limiting events
    RateLimitExceeded,
    RateLimitPhoneExceeded,
    RateLimitIpExceeded,
    
    // Account events
    AccountLocked,
    AccountUnlocked,
    
    // Session events
    Logout,
    SessionExpired,
    
    // Security events
    SuspiciousActivity,
    InvalidTokenUsage,
    
    // Refresh token events
    RefreshTokenAttempt,
    RefreshTokenSuccess,
    RefreshTokenFailure,
}

impl AuditEventType {
    /// Convert to string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginAttempt => "LOGIN_ATTEMPT",
            Self::LoginSuccess => "LOGIN_SUCCESS",
            Self::LoginFailure => "LOGIN_FAILURE",
            Self::SendCodeRequest => "SEND_CODE_REQUEST",
            Self::SendCodeSuccess => "SEND_CODE_SUCCESS",
            Self::SendCodeFailure => "SEND_CODE_FAILURE",
            Self::VerifyCodeAttempt => "VERIFY_CODE_ATTEMPT",
            Self::VerifyCodeSuccess => "VERIFY_CODE_SUCCESS",
            Self::VerifyCodeFailure => "VERIFY_CODE_FAILURE",
            Self::TokenGenerated => "TOKEN_GENERATED",
            Self::TokenRefreshed => "TOKEN_REFRESHED",
            Self::TokenRevoked => "TOKEN_REVOKED",
            Self::TokenValidation => "TOKEN_VALIDATION",
            Self::TokenValidationFailure => "TOKEN_VALIDATION_FAILURE",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::RateLimitPhoneExceeded => "RATE_LIMIT_PHONE_EXCEEDED",
            Self::RateLimitIpExceeded => "RATE_LIMIT_IP_EXCEEDED",
            Self::AccountLocked => "ACCOUNT_LOCKED",
            Self::AccountUnlocked => "ACCOUNT_UNLOCKED",
            Self::Logout => "LOGOUT",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::SuspiciousActivity => "SUSPICIOUS_ACTIVITY",
            Self::InvalidTokenUsage => "INVALID_TOKEN_USAGE",
            Self::RefreshTokenAttempt => "REFRESH_TOKEN_ATTEMPT",
            Self::RefreshTokenSuccess => "REFRESH_TOKEN_SUCCESS",
            Self::RefreshTokenFailure => "REFRESH_TOKEN_FAILURE",
        }
    }
    
    /// Parse from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LOGIN_ATTEMPT" => Some(Self::LoginAttempt),
            "LOGIN_SUCCESS" => Some(Self::LoginSuccess),
            "LOGIN_FAILURE" => Some(Self::LoginFailure),
            "SEND_CODE_REQUEST" => Some(Self::SendCodeRequest),
            "SEND_CODE_SUCCESS" => Some(Self::SendCodeSuccess),
            "SEND_CODE_FAILURE" => Some(Self::SendCodeFailure),
            "VERIFY_CODE_ATTEMPT" => Some(Self::VerifyCodeAttempt),
            "VERIFY_CODE_SUCCESS" => Some(Self::VerifyCodeSuccess),
            "VERIFY_CODE_FAILURE" => Some(Self::VerifyCodeFailure),
            "TOKEN_GENERATED" => Some(Self::TokenGenerated),
            "TOKEN_REFRESHED" => Some(Self::TokenRefreshed),
            "TOKEN_REVOKED" => Some(Self::TokenRevoked),
            "TOKEN_VALIDATION" => Some(Self::TokenValidation),
            "TOKEN_VALIDATION_FAILURE" => Some(Self::TokenValidationFailure),
            "RATE_LIMIT_EXCEEDED" => Some(Self::RateLimitExceeded),
            "RATE_LIMIT_PHONE_EXCEEDED" => Some(Self::RateLimitPhoneExceeded),
            "RATE_LIMIT_IP_EXCEEDED" => Some(Self::RateLimitIpExceeded),
            "ACCOUNT_LOCKED" => Some(Self::AccountLocked),
            "ACCOUNT_UNLOCKED" => Some(Self::AccountUnlocked),
            "LOGOUT" => Some(Self::Logout),
            "SESSION_EXPIRED" => Some(Self::SessionExpired),
            "SUSPICIOUS_ACTIVITY" => Some(Self::SuspiciousActivity),
            "INVALID_TOKEN_USAGE" => Some(Self::InvalidTokenUsage),
            "REFRESH_TOKEN_ATTEMPT" => Some(Self::RefreshTokenAttempt),
            "REFRESH_TOKEN_SUCCESS" => Some(Self::RefreshTokenSuccess),
            "REFRESH_TOKEN_FAILURE" => Some(Self::RefreshTokenFailure),
            _ => None,
        }
    }
}

/// Represents an audit log entry for authentication and security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLog {
    /// Unique identifier for the log entry
    pub id: Uuid,
    
    /// Type of authentication event
    pub event_type: AuditEventType,
    
    /// User ID if available (None for anonymous actions)
    pub user_id: Option<Uuid>,
    
    /// Masked phone number showing only last 4 digits (e.g., "****1234")
    pub phone_masked: Option<String>,
    
    /// Hashed phone number for correlation without exposing the actual number
    pub phone_hash: Option<String>,
    
    /// IP address of the request (required for security tracking)
    pub ip_address: String,
    
    /// User agent string from the request
    pub user_agent: Option<String>,
    
    /// Device information extracted from user agent or fingerprint
    pub device_info: Option<String>,
    
    /// Additional event data in JSON format for flexibility
    pub event_data: Option<JsonValue>,
    
    /// Failure reason for failed attempts
    pub failure_reason: Option<String>,
    
    /// Token ID for token-related events
    pub token_id: Option<Uuid>,
    
    /// Rate limit type if applicable
    pub rate_limit_type: Option<String>,
    
    /// Whether the action succeeded (kept for backward compatibility)
    pub success: bool,
    
    /// Action being audited (kept for backward compatibility with enhanced event_type)
    pub action: String,
    
    /// Error message if the action failed (backward compatibility, use failure_reason)
    pub error_message: Option<String>,
    
    /// Timestamp when the event occurred
    pub created_at: DateTime<Utc>,
    
    /// Whether the record has been archived (for 90-day retention policy)
    pub archived: bool,
    
    /// Timestamp when the record was archived
    pub archived_at: Option<DateTime<Utc>>,
}

impl AuditLog {
    /// Create a new audit log entry with enhanced fields
    pub fn new(
        event_type: AuditEventType,
        ip_address: impl Into<String>,
    ) -> Self {
        let action = event_type.as_str().to_string();
        let success = matches!(
            event_type,
            AuditEventType::LoginSuccess
                | AuditEventType::SendCodeSuccess
                | AuditEventType::VerifyCodeSuccess
                | AuditEventType::TokenGenerated
                | AuditEventType::TokenRefreshed
                | AuditEventType::RefreshTokenSuccess
        );
        
        Self {
            id: Uuid::new_v4(),
            event_type,
            user_id: None,
            phone_masked: None,
            phone_hash: None,
            ip_address: ip_address.into(),
            user_agent: None,
            device_info: None,
            event_data: None,
            failure_reason: None,
            token_id: None,
            rate_limit_type: None,
            success,
            action,
            error_message: None,
            created_at: Utc::now(),
            archived: false,
            archived_at: None,
        }
    }
    
    /// Create a new audit log entry (backward compatibility)
    pub fn new_legacy(
        action: impl Into<String>,
        success: bool,
    ) -> Self {
        let action_str = action.into();
        let event_type = Self::action_to_event_type(&action_str);
        
        Self {
            id: Uuid::new_v4(),
            event_type,
            user_id: None,
            phone_masked: None,
            phone_hash: None,
            ip_address: String::new(), // Will need to be set
            user_agent: None,
            device_info: None,
            event_data: None,
            failure_reason: None,
            token_id: None,
            rate_limit_type: None,
            success,
            action: action_str,
            error_message: None,
            created_at: Utc::now(),
            archived: false,
            archived_at: None,
        }
    }

    /// Add user context to the audit log
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Add phone context with automatic masking
    pub fn with_phone(mut self, phone: &str, phone_hash: impl Into<String>) -> Self {
        self.phone_masked = Some(Self::mask_phone(phone));
        self.phone_hash = Some(phone_hash.into());
        self
    }
    
    /// Add phone hash to the audit log (backward compatibility)
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
        if let Some(ip) = ip_address {
            self.ip_address = ip;
        }
        self.user_agent = user_agent.clone();
        
        // Extract device info from user agent if available
        if let Some(ua) = &user_agent {
            self.device_info = Some(Self::extract_device_info(ua));
        }
        
        self
    }
    
    /// Add device information
    pub fn with_device_info(mut self, device_info: impl Into<String>) -> Self {
        self.device_info = Some(device_info.into());
        self
    }
    
    /// Add event data as JSON
    pub fn with_event_data(mut self, data: JsonValue) -> Self {
        self.event_data = Some(data);
        self
    }
    
    /// Add failure reason for failed attempts
    pub fn with_failure_reason(mut self, reason: impl Into<String>) -> Self {
        self.failure_reason = Some(reason.into());
        self.success = false;
        self
    }

    /// Add error message for failed actions (backward compatibility)
    pub fn with_error(mut self, error_message: impl Into<String>) -> Self {
        let msg = error_message.into();
        self.error_message = Some(msg.clone());
        self.failure_reason = Some(msg);
        self.success = false;
        self
    }
    
    /// Add token ID for token-related events
    pub fn with_token_id(mut self, token_id: Uuid) -> Self {
        self.token_id = Some(token_id);
        self
    }
    
    /// Add rate limit information
    pub fn with_rate_limit(mut self, rate_limit_type: impl Into<String>) -> Self {
        self.rate_limit_type = Some(rate_limit_type.into());
        self
    }
    
    /// Mask phone number to show only last 4 digits
    pub fn mask_phone(phone: &str) -> String {
        if phone.len() <= 4 {
            return "*".repeat(phone.len());
        }
        
        // Remove any non-digit characters for consistent masking
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        
        if digits.len() <= 4 {
            return format!("****{}", digits);
        }
        
        // Show only last 4 digits
        let last_four = &digits[digits.len() - 4..];
        format!("****{}", last_four)
    }
    
    /// Extract device information from user agent string
    pub fn extract_device_info(user_agent: &str) -> String {
        // Simple extraction - can be enhanced with a proper user agent parser
        let ua_lower = user_agent.to_lowercase();
        
        let device_type = if ua_lower.contains("mobile") || ua_lower.contains("android") || ua_lower.contains("iphone") {
            "Mobile"
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            "Tablet"
        } else {
            "Desktop"
        };
        
        let os = if ua_lower.contains("windows") {
            "Windows"
        } else if ua_lower.contains("android") {
            "Android"
        } else if ua_lower.contains("ios") || ua_lower.contains("iphone") || ua_lower.contains("ipad") {
            "iOS"
        } else if ua_lower.contains("mac") {
            "macOS"
        } else if ua_lower.contains("linux") {
            "Linux"
        } else {
            "Unknown"
        };
        
        format!("{}/{}", device_type, os)
    }
    
    /// Convert legacy action string to event type
    fn action_to_event_type(action: &str) -> AuditEventType {
        match action {
            actions::SEND_CODE_ATTEMPT => AuditEventType::SendCodeRequest,
            actions::VERIFY_CODE_ATTEMPT => AuditEventType::VerifyCodeAttempt,
            actions::LOGIN_ATTEMPT => AuditEventType::LoginAttempt,
            actions::REFRESH_TOKEN_ATTEMPT => AuditEventType::RefreshTokenAttempt,
            actions::RATE_LIMIT_EXCEEDED => AuditEventType::RateLimitExceeded,
            actions::SUSPICIOUS_ACTIVITY => AuditEventType::SuspiciousActivity,
            actions::TOKEN_VALIDATION => AuditEventType::TokenValidation,
            _ => AuditEventType::LoginAttempt, // Default fallback
        }
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