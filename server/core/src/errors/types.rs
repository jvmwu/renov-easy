//! Domain-specific error types for authentication and related operations
//! 
//! This module provides error type definitions for authentication, token management, 
//! and validation operations. The actual error messages are configured externally
//! in the presentation layer for internationalization support.

use shared::types::response::ErrorResponse;
use thiserror::Error;

// Re-export shared ErrorResponse for backward compatibility
pub use shared::types::response::ErrorResponse as DomainErrorResponse;

/// Authentication-related errors
/// 
/// These errors represent various authentication failure scenarios.
/// Error messages are configured in the presentation layer for i18n support.
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid phone format: {phone}")]
    InvalidPhoneFormat { phone: String },

    #[error("Invalid verification code")]
    InvalidVerificationCode,

    #[error("Verification code expired")]
    VerificationCodeExpired,

    #[error("Maximum attempts exceeded")]
    MaxAttemptsExceeded,

    #[error("Rate limit exceeded: {minutes} minutes")]
    RateLimitExceeded { minutes: u32 },

    #[error("SMS service failure")]
    SmsServiceFailure,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Account suspended")]
    AccountSuspended,

    #[error("Session expired")]
    SessionExpired,
    
    #[error("Registration disabled")]
    RegistrationDisabled,
    
    #[error("User blocked")]
    UserBlocked,
}

/// Token-related errors
/// 
/// These errors represent various token validation and management failures.
/// Error messages are configured in the presentation layer for i18n support.
#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Token not yet valid")]
    TokenNotYetValid,

    #[error("Invalid claims")]
    InvalidClaims,

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Refresh token expired")]
    RefreshTokenExpired,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Token generation failed")]
    TokenGenerationFailed,

    #[error("Missing claim: {claim}")]
    MissingClaim { claim: String },
}

/// Validation errors
/// 
/// These errors represent input validation failures.
/// Error messages are configured in the presentation layer for i18n support.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Required field: {field}")]
    RequiredField { field: String },

    #[error("Invalid format: {field}")]
    InvalidFormat { field: String },

    #[error("Out of range: {field} (min: {min}, max: {max})")]
    OutOfRange { 
        field: String, 
        min: String, 
        max: String 
    },

    #[error("Invalid length: {field} (expected: {expected}, actual: {actual})")]
    InvalidLength { 
        field: String, 
        expected: usize, 
        actual: usize 
    },

    #[error("Pattern mismatch: {field}")]
    PatternMismatch { field: String },

    #[error("Invalid email")]
    InvalidEmail,

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("Invalid date")]
    InvalidDate,

    #[error("Duplicate value: {field}")]
    DuplicateValue { field: String },

    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        message_en: String,
        message_zh: String,
        limit: u32,
        window_seconds: u64,
    },
}

// The ErrorResponse struct is now imported from shared module
// We only need to add any core-specific extension methods if needed

// Utility functions for extracting language-specific messages
// These are kept for backward compatibility but actual i18n is handled in the presentation layer

/// Extract English message from bilingual error string (deprecated)
#[deprecated(note = "Use i18n configuration in the presentation layer instead")]
pub fn extract_english_message(error_msg: &str) -> String {
    if let Some(pipe_index) = error_msg.find(" | ") {
        error_msg[..pipe_index].to_string()
    } else {
        error_msg.to_string()
    }
}

/// Extract Chinese message from bilingual error string (deprecated)
#[deprecated(note = "Use i18n configuration in the presentation layer instead")]
pub fn extract_chinese_message(error_msg: &str) -> String {
    if let Some(pipe_index) = error_msg.find(" | ") {
        error_msg[pipe_index + 3..].to_string()
    } else {
        error_msg.to_string()
    }
}