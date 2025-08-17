//! Unit tests for domain error types

use std::collections::HashMap;
use crate::errors::{AuthError, TokenError, ValidationError, ErrorResponse};
use crate::errors::{extract_english_message, extract_chinese_message};

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