//! Shared error types and response structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard error response structure used across all API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for client identification
    pub error: String,
    
    /// Human-readable error message (localized)
    pub message: String,
    
    /// Additional error details (field errors, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
    
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response with details
    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: Some(details),
            timestamp: Utc::now(),
        }
    }

    /// Add a detail field to the error response
    pub fn add_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let details = self.details.get_or_insert_with(HashMap::new);
        if let Ok(json_value) = serde_json::to_value(value) {
            details.insert(key.into(), json_value);
        }
        self
    }
}

/// Common error codes used across the application
pub mod error_codes {
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const BAD_REQUEST: &str = "BAD_REQUEST";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";
    pub const TOKEN_EXPIRED: &str = "TOKEN_EXPIRED";
    pub const TOKEN_INVALID: &str = "TOKEN_INVALID";
    pub const DATABASE_ERROR: &str = "DATABASE_ERROR";
    pub const CACHE_ERROR: &str = "CACHE_ERROR";
    pub const SMS_ERROR: &str = "SMS_ERROR";
    pub const PHONE_INVALID: &str = "PHONE_INVALID";
    pub const VERIFICATION_CODE_INVALID: &str = "VERIFICATION_CODE_INVALID";
    pub const VERIFICATION_CODE_EXPIRED: &str = "VERIFICATION_CODE_EXPIRED";
}

/// Trait for converting errors to ErrorResponse
pub trait IntoErrorResponse {
    fn to_error_response(&self) -> ErrorResponse;
}

/// Result type with ErrorResponse as error
pub type ApiResult<T> = Result<T, ErrorResponse>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("TEST_ERROR", "Test error message");
        assert_eq!(error.error, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let mut details = HashMap::new();
        details.insert("field".to_string(), json!("value"));
        
        let error = ErrorResponse::with_details(
            "VALIDATION_ERROR",
            "Validation failed",
            details.clone(),
        );
        
        assert_eq!(error.error, "VALIDATION_ERROR");
        assert_eq!(error.details.unwrap(), details);
    }

    #[test]
    fn test_add_detail() {
        let error = ErrorResponse::new("ERROR", "Message")
            .add_detail("field1", "value1")
            .add_detail("field2", 42);
        
        let details = error.details.unwrap();
        assert_eq!(details.get("field1").unwrap(), &json!("value1"));
        assert_eq!(details.get("field2").unwrap(), &json!(42));
    }
}