//! Domain-specific error types for authentication and related operations
//! 
//! This module provides comprehensive error types with bilingual support (English and Chinese)
//! for authentication, token management, and validation operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Authentication-related errors with bilingual messages
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid phone format: {phone} | 无效的手机号码格式: {phone}")]
    InvalidPhoneFormat { phone: String },

    #[error("Invalid verification code | 验证码错误")]
    InvalidVerificationCode,

    #[error("Verification code expired | 验证码已过期")]
    VerificationCodeExpired,

    #[error("Maximum attempts exceeded. Please request a new code | 尝试次数超限，请重新获取验证码")]
    MaxAttemptsExceeded,

    #[error("Too many requests. Please try again in {minutes} minutes | 请求过于频繁，请在 {minutes} 分钟后重试")]
    RateLimitExceeded { minutes: u32 },

    #[error("SMS service failure. Please try again later | 短信服务失败，请稍后重试")]
    SmsServiceFailure,

    #[error("User not found | 用户不存在")]
    UserNotFound,

    #[error("User already exists | 用户已存在")]
    UserAlreadyExists,

    #[error("Authentication failed | 认证失败")]
    AuthenticationFailed,

    #[error("Insufficient permissions | 权限不足")]
    InsufficientPermissions,

    #[error("Account suspended | 账户已被暂停")]
    AccountSuspended,

    #[error("Session expired. Please login again | 会话已过期，请重新登录")]
    SessionExpired,
    
    #[error("Registration is currently disabled | 注册功能暂时关闭")]
    RegistrationDisabled,
    
    #[error("User account is blocked | 用户账户已被封禁")]
    UserBlocked,
}

/// Token-related errors with bilingual messages
#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Token expired | 令牌已过期")]
    TokenExpired,

    #[error("Invalid token format | 无效的令牌格式")]
    InvalidTokenFormat,

    #[error("Token signature verification failed | 令牌签名验证失败")]
    InvalidSignature,

    #[error("Token not yet valid | 令牌尚未生效")]
    TokenNotYetValid,

    #[error("Invalid token claims | 无效的令牌声明")]
    InvalidClaims,

    #[error("Token revoked | 令牌已被撤销")]
    TokenRevoked,

    #[error("Refresh token expired | 刷新令牌已过期")]
    RefreshTokenExpired,

    #[error("Invalid refresh token | 无效的刷新令牌")]
    InvalidRefreshToken,

    #[error("Token generation failed | 令牌生成失败")]
    TokenGenerationFailed,

    #[error("Missing required claim: {claim} | 缺少必需的声明: {claim}")]
    MissingClaim { claim: String },
}

/// Validation errors with bilingual messages
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field required: {field} | 必填字段: {field}")]
    RequiredField { field: String },

    #[error("Invalid format for field: {field} | 字段格式无效: {field}")]
    InvalidFormat { field: String },

    #[error("Value out of range for field: {field} (min: {min}, max: {max}) | 字段值超出范围: {field} (最小: {min}, 最大: {max})")]
    OutOfRange {
        field: String,
        min: String,
        max: String,
    },

    #[error("Invalid length for field: {field} (expected: {expected}, actual: {actual}) | 字段长度无效: {field} (期望: {expected}, 实际: {actual})")]
    InvalidLength {
        field: String,
        expected: usize,
        actual: usize,
    },

    #[error("Pattern mismatch for field: {field} | 字段模式不匹配: {field}")]
    PatternMismatch { field: String },

    #[error("Invalid email format | 无效的邮箱格式")]
    InvalidEmail,

    #[error("Invalid URL format | 无效的URL格式")]
    InvalidUrl,

    #[error("Invalid date format | 无效的日期格式")]
    InvalidDate,

    #[error("Duplicate value for field: {field} | 字段值重复: {field}")]
    DuplicateValue { field: String },

    #[error("Business rule violation: {rule} | 业务规则违反: {rule}")]
    BusinessRuleViolation { rule: String },

    #[error("{message_en} | {message_zh}")]
    RateLimitExceeded { 
        message_en: String,
        message_zh: String,
    },
}

/// Unified error response structure for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub error: String,
    /// Human-readable error message (bilingual)
    pub message: String,
    /// Additional error details if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: impl ToString, message: impl ToString) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
        }
    }

    /// Add details to the error response
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a single detail to the error response
    pub fn with_detail(mut self, key: impl ToString, value: serde_json::Value) -> Self {
        let mut details = self.details.unwrap_or_default();
        details.insert(key.to_string(), value);
        self.details = Some(details);
        self
    }
}

/// Convert AuthError to ErrorResponse
impl From<AuthError> for ErrorResponse {
    fn from(err: AuthError) -> Self {
        let error_code = match &err {
            AuthError::InvalidPhoneFormat { .. } => "INVALID_PHONE_FORMAT",
            AuthError::InvalidVerificationCode => "INVALID_VERIFICATION_CODE",
            AuthError::VerificationCodeExpired => "VERIFICATION_CODE_EXPIRED",
            AuthError::MaxAttemptsExceeded => "MAX_ATTEMPTS_EXCEEDED",
            AuthError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AuthError::SmsServiceFailure => "SMS_SERVICE_FAILURE",
            AuthError::UserNotFound => "USER_NOT_FOUND",
            AuthError::UserAlreadyExists => "USER_ALREADY_EXISTS",
            AuthError::AuthenticationFailed => "AUTHENTICATION_FAILED",
            AuthError::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS",
            AuthError::AccountSuspended => "ACCOUNT_SUSPENDED",
            AuthError::SessionExpired => "SESSION_EXPIRED",
            AuthError::RegistrationDisabled => "REGISTRATION_DISABLED",
            AuthError::UserBlocked => "USER_BLOCKED",
        };

        ErrorResponse::new(error_code, err.to_string())
    }
}

/// Convert TokenError to ErrorResponse
impl From<TokenError> for ErrorResponse {
    fn from(err: TokenError) -> Self {
        let error_code = match &err {
            TokenError::TokenExpired => "TOKEN_EXPIRED",
            TokenError::InvalidTokenFormat => "INVALID_TOKEN_FORMAT",
            TokenError::InvalidSignature => "INVALID_SIGNATURE",
            TokenError::TokenNotYetValid => "TOKEN_NOT_YET_VALID",
            TokenError::InvalidClaims => "INVALID_CLAIMS",
            TokenError::TokenRevoked => "TOKEN_REVOKED",
            TokenError::RefreshTokenExpired => "REFRESH_TOKEN_EXPIRED",
            TokenError::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            TokenError::TokenGenerationFailed => "TOKEN_GENERATION_FAILED",
            TokenError::MissingClaim { .. } => "MISSING_CLAIM",
        };

        ErrorResponse::new(error_code, err.to_string())
    }
}

/// Convert ValidationError to ErrorResponse
impl From<ValidationError> for ErrorResponse {
    fn from(err: ValidationError) -> Self {
        let error_code = match &err {
            ValidationError::RequiredField { .. } => "REQUIRED_FIELD",
            ValidationError::InvalidFormat { .. } => "INVALID_FORMAT",
            ValidationError::OutOfRange { .. } => "OUT_OF_RANGE",
            ValidationError::InvalidLength { .. } => "INVALID_LENGTH",
            ValidationError::PatternMismatch { .. } => "PATTERN_MISMATCH",
            ValidationError::InvalidEmail => "INVALID_EMAIL",
            ValidationError::InvalidUrl => "INVALID_URL",
            ValidationError::InvalidDate => "INVALID_DATE",
            ValidationError::DuplicateValue { .. } => "DUPLICATE_VALUE",
            ValidationError::BusinessRuleViolation { .. } => "BUSINESS_RULE_VIOLATION",
            ValidationError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
        };

        ErrorResponse::new(error_code, err.to_string())
    }
}

/// Helper function to extract English message from bilingual error
pub fn extract_english_message(message: &str) -> &str {
    message.split(" | ").next().unwrap_or(message)
}

/// Helper function to extract Chinese message from bilingual error
pub fn extract_chinese_message(message: &str) -> &str {
    message.split(" | ").nth(1).unwrap_or(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_messages() {
        let error = AuthError::InvalidPhoneFormat {
            phone: "123".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Invalid phone format"));
        assert!(message.contains("无效的手机号码格式"));
    }

    #[test]
    fn test_token_error_conversion() {
        let error = TokenError::TokenExpired;
        let response: ErrorResponse = error.into();
        assert_eq!(response.error, "TOKEN_EXPIRED");
        assert!(response.message.contains("Token expired"));
        assert!(response.message.contains("令牌已过期"));
    }

    #[test]
    fn test_validation_error_with_fields() {
        let error = ValidationError::RequiredField {
            field: "phone".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("phone"));
        assert!(message.contains("必填字段"));
    }

    #[test]
    fn test_error_response_with_details() {
        let mut details = HashMap::new();
        details.insert("attempts".to_string(), serde_json::json!(3));
        details.insert("max_attempts".to_string(), serde_json::json!(5));

        let response = ErrorResponse::new("TEST_ERROR", "Test error message")
            .with_details(details.clone());

        assert_eq!(response.error, "TEST_ERROR");
        assert_eq!(response.message, "Test error message");
        assert!(response.details.is_some());
        assert_eq!(response.details.unwrap()["attempts"], 3);
    }

    #[test]
    fn test_message_extraction() {
        let bilingual = "Invalid token | 无效的令牌";
        assert_eq!(extract_english_message(bilingual), "Invalid token");
        assert_eq!(extract_chinese_message(bilingual), "无效的令牌");

        let english_only = "Only English";
        assert_eq!(extract_english_message(english_only), "Only English");
        assert_eq!(extract_chinese_message(english_only), "Only English");
    }

    #[test]
    fn test_rate_limit_error() {
        let error = AuthError::RateLimitExceeded { minutes: 5 };
        let message = error.to_string();
        assert!(message.contains("5 minutes"));
        assert!(message.contains("5 分钟"));
    }
}