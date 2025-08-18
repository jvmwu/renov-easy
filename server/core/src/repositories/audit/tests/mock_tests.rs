//! Tests for the mock audit log repository implementation

use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::domain::entities::audit::{AuditLog, actions};
use crate::repositories::audit::MockAuditLogRepository;
use crate::repositories::AuditLogRepository;

#[tokio::test]
async fn test_mock_create_and_retrieve() {
    let repo = MockAuditLogRepository::new();
    
    let audit_log = AuditLog::new(actions::LOGIN_ATTEMPT, true)
        .with_phone_hash("test_phone")
        .with_request_context(Some("192.168.1.1".to_string()), None);
    
    // Create the log
    let result = repo.create(&audit_log).await;
    assert!(result.is_ok());
    
    // Retrieve all logs
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, actions::LOGIN_ATTEMPT);
    assert_eq!(logs[0].phone_hash, Some("test_phone".to_string()));
}

#[tokio::test]
async fn test_mock_find_by_user() {
    let repo = MockAuditLogRepository::new();
    let user_id = Uuid::new_v4();
    
    // Create logs for different users
    let log1 = AuditLog::new(actions::LOGIN_ATTEMPT, true)
        .with_user(user_id);
    
    let log2 = AuditLog::new(actions::SEND_CODE_ATTEMPT, true)
        .with_user(Uuid::new_v4());
    
    let log3 = AuditLog::new(actions::VERIFY_CODE_ATTEMPT, false)
        .with_user(user_id);
    
    repo.create(&log1).await.unwrap();
    repo.create(&log2).await.unwrap();
    repo.create(&log3).await.unwrap();
    
    // Find by user
    let user_logs = repo.find_by_user(user_id, 10).await.unwrap();
    assert_eq!(user_logs.len(), 2);
    
    // Verify the logs are for the correct user
    for log in &user_logs {
        assert_eq!(log.user_id, Some(user_id));
    }
}

#[tokio::test]
async fn test_mock_find_by_phone() {
    let repo = MockAuditLogRepository::new();
    let phone_hash = "test_phone_hash";
    
    // Create logs with different phone hashes
    let log1 = AuditLog::new(actions::SEND_CODE_ATTEMPT, true)
        .with_phone_hash(phone_hash);
    
    let log2 = AuditLog::new(actions::VERIFY_CODE_ATTEMPT, true)
        .with_phone_hash("other_phone");
    
    let log3 = AuditLog::new(actions::LOGIN_ATTEMPT, true)
        .with_phone_hash(phone_hash);
    
    repo.create(&log1).await.unwrap();
    repo.create(&log2).await.unwrap();
    repo.create(&log3).await.unwrap();
    
    // Find by phone
    let phone_logs = repo.find_by_phone_hash(phone_hash, 10).await.unwrap();
    assert_eq!(phone_logs.len(), 2);
    
    // Verify the logs are for the correct phone
    for log in &phone_logs {
        assert_eq!(log.phone_hash, Some(phone_hash.to_string()));
    }
}

#[tokio::test]
async fn test_mock_count_failed_attempts() {
    let repo = MockAuditLogRepository::new();
    let phone_hash = "test_phone";
    let ip_address = "192.168.1.1";
    
    // Create some failed attempts
    for _ in 0..3 {
        let log = AuditLog::new(actions::LOGIN_ATTEMPT, false)
            .with_phone_hash(phone_hash)
            .with_request_context(Some(ip_address.to_string()), None);
        repo.create(&log).await.unwrap();
    }
    
    // Create a successful attempt (should not be counted)
    let success_log = AuditLog::new(actions::LOGIN_ATTEMPT, true)
        .with_phone_hash(phone_hash)
        .with_request_context(Some(ip_address.to_string()), None);
    repo.create(&success_log).await.unwrap();
    
    // Count failed attempts in the last 60 minutes
    let since = Utc::now() - Duration::minutes(60);
    let count = repo.count_failed_attempts(
        actions::LOGIN_ATTEMPT,
        Some(phone_hash),
        Some(ip_address),
        since,
    ).await.unwrap();
    
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_mock_count_suspicious_activity() {
    let repo = MockAuditLogRepository::new();
    let ip_address = "192.168.1.1";
    
    // Create multiple failed attempts from same IP with different phones
    for i in 0..5 {
        let log = AuditLog::new(actions::LOGIN_ATTEMPT, false)
            .with_phone_hash(format!("phone_{}", i))
            .with_request_context(Some(ip_address.to_string()), None);
        repo.create(&log).await.unwrap();
    }
    
    // Find suspicious activity (unique phones from IP) in the last 60 minutes
    let since = Utc::now() - Duration::minutes(60);
    let suspicious_logs = repo.find_suspicious_activity(
        Some(ip_address),
        since,
    ).await.unwrap();
    
    // Should find all 5 logs with different phone numbers
    assert_eq!(suspicious_logs.len(), 5);
}

#[tokio::test]
async fn test_mock_repository_failure() {
    let repo = MockAuditLogRepository::new();
    
    // Set repository to fail
    repo.set_should_fail(true);
    
    let audit_log = AuditLog::new(actions::LOGIN_ATTEMPT, true);
    
    // Should return error
    let result = repo.create(&audit_log).await;
    assert!(result.is_err());
    
    // Reset and try again
    repo.set_should_fail(false);
    let result = repo.create(&audit_log).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_clear_logs() {
    let repo = MockAuditLogRepository::new();
    
    // Add some logs
    for i in 0..3 {
        let log = AuditLog::new(format!("action_{}", i), true);
        repo.create(&log).await.unwrap();
    }
    
    assert_eq!(repo.get_all_logs().len(), 3);
    
    // Clear logs
    repo.clear();
    assert_eq!(repo.get_all_logs().len(), 0);
}