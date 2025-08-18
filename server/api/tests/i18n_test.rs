// Integration tests for i18n error message system

extern crate core as renov_core;

use api::i18n::{get_error_message, format_message, Language};
use std::collections::HashMap;

#[test]
fn test_simple_error_message_english() {
    let result = get_error_message("auth", "user_not_found", Language::English);
    assert!(result.is_some());
    
    if let Some((code, message, status)) = result {
        assert_eq!(code, "user_not_found");
        assert_eq!(message, "User not found");
        assert_eq!(status, 404);
    }
}

#[test]
fn test_simple_error_message_chinese() {
    let result = get_error_message("auth", "user_not_found", Language::Chinese);
    assert!(result.is_some());
    
    if let Some((code, message, status)) = result {
        assert_eq!(code, "user_not_found");
        assert_eq!(message, "用户不存在");
        assert_eq!(status, 404);
    }
}

#[test]
fn test_formatted_message_with_single_param() {
    let result = get_error_message("auth", "invalid_phone_format", Language::English);
    assert!(result.is_some());
    
    if let Some((code, message_template, status)) = result {
        let mut params = HashMap::new();
        params.insert("phone", "+1234567890".to_string());
        let formatted = format_message(&message_template, &params);
        
        assert_eq!(code, "invalid_phone_format");
        assert_eq!(formatted, "Invalid phone number format: +1234567890");
        assert_eq!(status, 400);
    }
}

#[test]
fn test_formatted_message_chinese_with_param() {
    let result = get_error_message("auth", "rate_limit_exceeded", Language::Chinese);
    assert!(result.is_some());
    
    if let Some((code, message_template, status)) = result {
        let mut params = HashMap::new();
        params.insert("minutes", "5".to_string());
        let formatted = format_message(&message_template, &params);
        
        assert_eq!(code, "rate_limit_exceeded");
        assert_eq!(formatted, "请求过于频繁，请在5分钟后重试");
        assert_eq!(status, 429);
    }
}

#[test]
fn test_validation_error_with_multiple_params() {
    let result = get_error_message("validation", "out_of_range", Language::English);
    assert!(result.is_some());
    
    if let Some((code, message_template, status)) = result {
        let mut params = HashMap::new();
        params.insert("field", "age".to_string());
        params.insert("min", "18".to_string());
        params.insert("max", "100".to_string());
        let formatted = format_message(&message_template, &params);
        
        assert_eq!(code, "out_of_range");
        assert_eq!(formatted, "Field age out of range (min: 18, max: 100)");
        assert_eq!(status, 400);
    }
}

#[test]
fn test_token_error() {
    let result = get_error_message("token", "token_expired", Language::English);
    assert!(result.is_some());
    
    if let Some((code, message, status)) = result {
        assert_eq!(code, "token_expired");
        assert_eq!(message, "Token has expired");
        assert_eq!(status, 401);
    }
}

#[test]
fn test_general_error_with_resource() {
    let result = get_error_message("general", "not_found", Language::Chinese);
    assert!(result.is_some());
    
    if let Some((code, message_template, status)) = result {
        let mut params = HashMap::new();
        params.insert("resource", "订单".to_string());
        let formatted = format_message(&message_template, &params);
        
        assert_eq!(code, "not_found");
        assert_eq!(formatted, "订单不存在");
        assert_eq!(status, 404);
    }
}

#[test]
fn test_nonexistent_error_returns_none() {
    let result = get_error_message("auth", "nonexistent_error", Language::English);
    assert!(result.is_none());
}

#[test]
fn test_language_detection_from_header() {
    assert_eq!(Language::from_header(Some("zh-CN")), Language::Chinese);
    assert_eq!(Language::from_header(Some("zh")), Language::Chinese);
    assert_eq!(Language::from_header(Some("en-US")), Language::English);
    assert_eq!(Language::from_header(Some("fr-FR")), Language::English); // Default to English
    assert_eq!(Language::from_header(None), Language::English);
}

#[test]
fn test_message_format_with_missing_params() {
    // Test that missing parameters are left as-is in the template
    let template = "Field {field} is required, min: {min}";
    let mut params = HashMap::new();
    params.insert("field", "username".to_string());
    // Note: "min" parameter is missing
    
    let result = format_message(template, &params);
    assert_eq!(result, "Field username is required, min: {min}");
}