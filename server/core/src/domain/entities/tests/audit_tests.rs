//! Tests for the AuditLog entity

use uuid::Uuid;
use crate::domain::entities::audit::{AuditLog, actions};

#[test]
fn test_create_audit_log() {
    let log = AuditLog::new(actions::LOGIN_ATTEMPT, true);
    
    assert_eq!(log.action, actions::LOGIN_ATTEMPT);
    assert!(log.success);
    assert!(log.user_id.is_none());
    assert!(log.phone_hash.is_none());
    assert!(log.error_message.is_none());
}

#[test]
fn test_builder_pattern() {
    let user_id = Uuid::new_v4();
    let log = AuditLog::new(actions::LOGIN_ATTEMPT, false)
        .with_user(user_id)
        .with_phone_hash("hashed_phone")
        .with_request_context(
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        )
        .with_error("Invalid credentials");

    assert_eq!(log.user_id, Some(user_id));
    assert_eq!(log.phone_hash, Some("hashed_phone".to_string()));
    assert_eq!(log.ip_address, Some("192.168.1.1".to_string()));
    assert_eq!(log.user_agent, Some("Mozilla/5.0".to_string()));
    assert_eq!(log.error_message, Some("Invalid credentials".to_string()));
}

#[test]
fn test_audit_log_with_user() {
    let user_id = Uuid::new_v4();
    let log = AuditLog::new(actions::REFRESH_TOKEN_ATTEMPT, true)
        .with_user(user_id);
    
    assert_eq!(log.user_id, Some(user_id));
    assert_eq!(log.action, actions::REFRESH_TOKEN_ATTEMPT);
}

#[test]
fn test_audit_log_with_phone_hash() {
    let log = AuditLog::new(actions::SEND_CODE_ATTEMPT, true)
        .with_phone_hash("hashed_12345");
    
    assert_eq!(log.phone_hash, Some("hashed_12345".to_string()));
}

#[test]
fn test_audit_log_with_error() {
    let log = AuditLog::new(actions::VERIFY_CODE_ATTEMPT, false)
        .with_error("Invalid verification code");
    
    assert!(!log.success);
    assert_eq!(log.error_message, Some("Invalid verification code".to_string()));
}

#[test]
fn test_audit_log_actions() {
    assert_eq!(actions::SEND_CODE_ATTEMPT, "send_code_attempt");
    assert_eq!(actions::VERIFY_CODE_ATTEMPT, "verify_code_attempt");
    assert_eq!(actions::LOGIN_ATTEMPT, "login_attempt");
    assert_eq!(actions::REFRESH_TOKEN_ATTEMPT, "refresh_token_attempt");
    assert_eq!(actions::RATE_LIMIT_EXCEEDED, "rate_limit_exceeded");
    assert_eq!(actions::SUSPICIOUS_ACTIVITY, "suspicious_activity");
    assert_eq!(actions::TOKEN_VALIDATION, "token_validation");
}