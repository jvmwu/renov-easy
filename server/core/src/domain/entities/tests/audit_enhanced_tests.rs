//! Tests for enhanced audit log functionality

use serde_json::json;
use uuid::Uuid;

use crate::domain::entities::audit::{AuditLog, AuditEventType};

#[test]
fn test_enhanced_audit_log_creation() {
    let ip = "192.168.1.1";
    let audit_log = AuditLog::new(AuditEventType::LoginSuccess, ip);
    
    assert_eq!(audit_log.event_type, AuditEventType::LoginSuccess);
    assert_eq!(audit_log.ip_address, ip);
    assert!(audit_log.success);
    assert_eq!(audit_log.action, "LOGIN_SUCCESS");
    assert!(!audit_log.archived);
    assert!(audit_log.archived_at.is_none());
}

#[test]
fn test_phone_masking() {
    // Test various phone number formats
    assert_eq!(AuditLog::mask_phone("+1234567890"), "****7890");
    assert_eq!(AuditLog::mask_phone("123-456-7890"), "****7890");
    assert_eq!(AuditLog::mask_phone("(123) 456-7890"), "****7890");
    assert_eq!(AuditLog::mask_phone("1234"), "****");
    assert_eq!(AuditLog::mask_phone("12345"), "****2345");
    assert_eq!(AuditLog::mask_phone(""), "");
}

#[test]
fn test_device_info_extraction() {
    // Test mobile detection
    let mobile_ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)";
    assert_eq!(AuditLog::extract_device_info(mobile_ua), "Mobile/iOS");
    
    // Test Android detection
    let android_ua = "Mozilla/5.0 (Linux; Android 10; SM-G960F)";
    assert_eq!(AuditLog::extract_device_info(android_ua), "Mobile/Android");
    
    // Test desktop detection
    let desktop_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
    assert_eq!(AuditLog::extract_device_info(desktop_ua), "Desktop/Windows");
    
    // Test macOS detection
    let mac_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
    assert_eq!(AuditLog::extract_device_info(mac_ua), "Desktop/macOS");
    
    // Test tablet detection
    let ipad_ua = "Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)";
    assert_eq!(AuditLog::extract_device_info(ipad_ua), "Tablet/iOS");
}

#[test]
fn test_audit_log_with_phone() {
    let phone = "+1234567890";
    let phone_hash = "hash123";
    let ip = "192.168.1.1";
    
    let audit_log = AuditLog::new(AuditEventType::SendCodeRequest, ip)
        .with_phone(phone, phone_hash);
    
    assert_eq!(audit_log.phone_masked, Some("****7890".to_string()));
    assert_eq!(audit_log.phone_hash, Some(phone_hash.to_string()));
}

#[test]
fn test_audit_log_with_event_data() {
    let ip = "192.168.1.1";
    let event_data = json!({
        "attempt_count": 3,
        "verification_method": "sms"
    });
    
    let audit_log = AuditLog::new(AuditEventType::VerifyCodeAttempt, ip)
        .with_event_data(event_data.clone());
    
    assert_eq!(audit_log.event_data, Some(event_data));
}

#[test]
fn test_audit_log_with_failure() {
    let ip = "192.168.1.1";
    let failure_reason = "Invalid verification code";
    
    let audit_log = AuditLog::new(AuditEventType::VerifyCodeFailure, ip)
        .with_failure_reason(failure_reason);
    
    assert_eq!(audit_log.failure_reason, Some(failure_reason.to_string()));
    assert!(!audit_log.success);
}

#[test]
fn test_audit_log_with_token() {
    let ip = "192.168.1.1";
    let token_id = Uuid::new_v4();
    
    let audit_log = AuditLog::new(AuditEventType::TokenGenerated, ip)
        .with_token_id(token_id);
    
    assert_eq!(audit_log.token_id, Some(token_id));
}

#[test]
fn test_audit_log_with_rate_limit() {
    let ip = "192.168.1.1";
    let rate_limit_type = "phone";
    
    let audit_log = AuditLog::new(AuditEventType::RateLimitPhoneExceeded, ip)
        .with_rate_limit(rate_limit_type);
    
    assert_eq!(audit_log.rate_limit_type, Some(rate_limit_type.to_string()));
}

#[test]
fn test_event_type_string_conversion() {
    // Test conversion to string
    assert_eq!(AuditEventType::LoginSuccess.as_str(), "LOGIN_SUCCESS");
    assert_eq!(AuditEventType::RateLimitExceeded.as_str(), "RATE_LIMIT_EXCEEDED");
    assert_eq!(AuditEventType::TokenGenerated.as_str(), "TOKEN_GENERATED");
    
    // Test parsing from string
    assert_eq!(AuditEventType::from_str("LOGIN_SUCCESS"), Some(AuditEventType::LoginSuccess));
    assert_eq!(AuditEventType::from_str("RATE_LIMIT_EXCEEDED"), Some(AuditEventType::RateLimitExceeded));
    assert_eq!(AuditEventType::from_str("INVALID"), None);
}

#[test]
fn test_comprehensive_audit_log() {
    let ip = "192.168.1.1";
    let user_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let phone = "+1234567890";
    let phone_hash = "hash123";
    let user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)";
    
    let event_data = json!({
        "login_method": "passwordless",
        "token_type": "access"
    });
    
    let mut audit_log = AuditLog::new(AuditEventType::LoginSuccess, ip)
        .with_user(user_id)
        .with_phone(phone, phone_hash)
        .with_token_id(token_id)
        .with_event_data(event_data.clone());
    
    audit_log.user_agent = Some(user_agent.to_string());
    audit_log.device_info = Some(AuditLog::extract_device_info(user_agent));
    
    // Verify all fields
    assert_eq!(audit_log.event_type, AuditEventType::LoginSuccess);
    assert_eq!(audit_log.ip_address, ip);
    assert_eq!(audit_log.user_id, Some(user_id));
    assert_eq!(audit_log.phone_masked, Some("****7890".to_string()));
    assert_eq!(audit_log.phone_hash, Some(phone_hash.to_string()));
    assert_eq!(audit_log.token_id, Some(token_id));
    assert_eq!(audit_log.event_data, Some(event_data));
    assert_eq!(audit_log.user_agent, Some(user_agent.to_string()));
    assert_eq!(audit_log.device_info, Some("Mobile/iOS".to_string()));
    assert!(audit_log.success);
    assert!(!audit_log.archived);
}

#[test]
fn test_backward_compatibility() {
    // Test that the legacy new method still works
    let action = "login_attempt";
    let success = false;
    
    let mut audit_log = AuditLog::new_legacy(action, success);
    audit_log.ip_address = "192.168.1.1".to_string();
    
    assert_eq!(audit_log.action, action);
    assert_eq!(audit_log.success, success);
    assert_eq!(audit_log.event_type, AuditEventType::LoginAttempt);
}