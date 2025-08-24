//! Comprehensive tests for the AuditService.

use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entities::audit::{AuditLog, actions};
use crate::errors::DomainError;
use crate::repositories::AuditLogRepository;
use crate::services::audit::{AuditService, AuditServiceConfig};

/// Mock implementation of AuditLogRepository for testing
struct MockAuditLogRepository {
    logs: Arc<Mutex<Vec<AuditLog>>>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockAuditLogRepository {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    fn get_all_logs(&self) -> Vec<AuditLog> {
        self.logs.lock().unwrap().clone()
    }
}

#[async_trait]
impl AuditLogRepository for MockAuditLogRepository {
    async fn create(&self, audit_log: &AuditLog) -> Result<(), DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal { message: "Mock failure".to_string() });
        }
        self.logs.lock().unwrap().push(audit_log.clone());
        Ok(())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal { message: "Mock failure".to_string() });
        }
        let logs = self.logs.lock().unwrap();
        let mut user_logs: Vec<AuditLog> = logs
            .iter()
            .filter(|log| log.user_id == Some(user_id))
            .cloned()
            .collect();
        user_logs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        user_logs.truncate(limit);
        Ok(user_logs)
    }

    async fn find_by_phone_hash(
        &self,
        phone_hash: &str,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal { message: "Mock failure".to_string() });
        }
        let logs = self.logs.lock().unwrap();
        let mut phone_logs: Vec<AuditLog> = logs
            .iter()
            .filter(|log| log.phone_hash.as_deref() == Some(phone_hash))
            .cloned()
            .collect();
        phone_logs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        phone_logs.truncate(limit);
        Ok(phone_logs)
    }

    async fn count_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal { message: "Mock failure".to_string() });
        }
        let logs = self.logs.lock().unwrap();
        let count = logs
            .iter()
            .filter(|log| {
                log.action == action
                    && !log.success
                    && log.created_at >= since
                    && (phone_hash.is_none() || log.phone_hash.as_deref() == phone_hash)
                    && (ip_address.is_none() || log.ip_address.as_deref() == ip_address)
            })
            .count();
        Ok(count)
    }

    async fn find_suspicious_activity(
        &self,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal { message: "Mock failure".to_string() });
        }
        let logs = self.logs.lock().unwrap();
        let suspicious_logs: Vec<AuditLog> = logs
            .iter()
            .filter(|log| {
                log.created_at >= since
                    && (ip_address.is_none() || log.ip_address.as_deref() == ip_address)
                    && !log.success
            })
            .cloned()
            .collect();
        Ok(suspicious_logs)
    }
}

#[tokio::test]
async fn test_log_auth_attempt() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false, // Disable async for testing
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_auth_attempt(
            "test_action",
            true,
            None,
            Some("phone_hash".to_string()),
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            None,
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "test_action");
    assert!(logs[0].success);
    assert_eq!(logs[0].phone_hash, Some("phone_hash".to_string()));
    assert_eq!(logs[0].ip_address, Some("192.168.1.1".to_string()));
    assert_eq!(logs[0].user_agent, Some("Mozilla/5.0".to_string()));
}

#[tokio::test]
async fn test_log_send_code() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_send_code(
            "phone_hash",
            true,
            Some("192.168.1.1".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, actions::SEND_CODE_ATTEMPT);
    assert_eq!(logs[0].phone_hash, Some("phone_hash".to_string()));
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_log_verify_code_success() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let user_id = Uuid::new_v4();
    let result = service
        .log_verify_code(
            "phone_hash_123",
            true,
            Some(user_id),
            Some("10.0.0.1".to_string()),
            Some("Chrome/100.0".to_string()),
            None,
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    
    let log = &logs[0];
    assert_eq!(log.action, actions::VERIFY_CODE_ATTEMPT);
    assert!(log.success);
    assert_eq!(log.user_id, Some(user_id));
    assert_eq!(log.phone_hash, Some("phone_hash_123".to_string()));
    assert_eq!(log.ip_address, Some("10.0.0.1".to_string()));
    assert_eq!(log.user_agent, Some("Chrome/100.0".to_string()));
    assert!(log.error_message.is_none());
}

#[tokio::test]
async fn test_log_verify_code_failure() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_verify_code(
            "phone_hash_456",
            false,
            None,
            Some("10.0.0.2".to_string()),
            Some("Firefox/95.0".to_string()),
            Some("Invalid verification code".to_string()),
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    
    let log = &logs[0];
    assert_eq!(log.action, actions::VERIFY_CODE_ATTEMPT);
    assert!(!log.success);
    assert!(log.user_id.is_none());
    assert_eq!(log.phone_hash, Some("phone_hash_456".to_string()));
    assert_eq!(log.error_message, Some("Invalid verification code".to_string()));
}

#[tokio::test]
async fn test_log_rate_limit_exceeded() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_rate_limit_exceeded(
            Some("phone_hash_789".to_string()),
            Some("10.0.0.3".to_string()),
            Some("Safari/14.0".to_string()),
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    
    let log = &logs[0];
    assert_eq!(log.action, actions::RATE_LIMIT_EXCEEDED);
    assert!(!log.success);
    assert_eq!(log.phone_hash, Some("phone_hash_789".to_string()));
    assert_eq!(log.error_message, Some("Rate limit exceeded".to_string()));
}

#[tokio::test]
async fn test_log_suspicious_activity() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_suspicious_activity(
            Some("phone_hash_suspicious".to_string()),
            Some("10.0.0.4".to_string()),
            Some("Bot/1.0".to_string()),
            "Multiple phone numbers from same IP",
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    
    let log = &logs[0];
    assert_eq!(log.action, actions::SUSPICIOUS_ACTIVITY);
    assert!(!log.success);
    assert_eq!(log.error_message, Some("Multiple phone numbers from same IP".to_string()));
}

#[tokio::test]
async fn test_check_failed_attempts_threshold() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        max_failed_attempts: 3,
        failed_attempts_window_minutes: 15,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    // Add 2 failed attempts - should not trigger
    for _ in 0..2 {
        service
            .log_login(
                None,
                Some("phone_hash_test".to_string()),
                false,
                Some("10.0.0.5".to_string()),
                None,
                Some("Invalid credentials".to_string()),
            )
            .await
            .unwrap();
    }

    let too_many = service
        .check_failed_attempts(
            actions::LOGIN_ATTEMPT,
            Some("phone_hash_test"),
            Some("10.0.0.5"),
        )
        .await
        .unwrap();

    assert!(!too_many);

    // Add one more failed attempt - should trigger
    service
        .log_login(
            None,
            Some("phone_hash_test".to_string()),
            false,
            Some("10.0.0.5".to_string()),
            None,
            Some("Invalid credentials".to_string()),
        )
        .await
        .unwrap();

    let too_many = service
        .check_failed_attempts(
            actions::LOGIN_ATTEMPT,
            Some("phone_hash_test"),
            Some("10.0.0.5"),
        )
        .await
        .unwrap();

    assert!(too_many);
}

#[tokio::test]
async fn test_detect_suspicious_activity_multiple_phones() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        suspicious_activity_window_minutes: 60,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    // Add failed attempts from same IP with 6 different phone numbers
    for i in 0..6 {
        service
            .log_login(
                None,
                Some(format!("phone_hash_{}", i)),
                false,
                Some("10.0.0.6".to_string()),
                None,
                Some("Invalid credentials".to_string()),
            )
            .await
            .unwrap();
    }

    let suspicious = service
        .detect_suspicious_activity(Some("10.0.0.6"))
        .await
        .unwrap();

    assert!(suspicious);
}

#[tokio::test]
async fn test_detect_suspicious_activity_many_failures() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        suspicious_activity_window_minutes: 60,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    // Add 11 failed attempts (threshold is 10)
    for i in 0..11 {
        service
            .log_login(
                None,
                Some(format!("phone_hash_{}", i % 3)), // Only 3 different phones
                false,
                Some("10.0.0.7".to_string()),
                None,
                Some("Invalid credentials".to_string()),
            )
            .await
            .unwrap();
    }

    let suspicious = service
        .detect_suspicious_activity(Some("10.0.0.7"))
        .await
        .unwrap();

    assert!(suspicious);
}

#[tokio::test]
async fn test_get_user_audit_logs() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let user_id = Uuid::new_v4();
    
    // Add some logs for the user
    for i in 0..5 {
        service
            .log_login(
                Some(user_id),
                Some("phone_hash".to_string()),
                i % 2 == 0,
                Some("10.0.0.8".to_string()),
                None,
                if i % 2 == 0 { None } else { Some("Failed".to_string()) },
            )
            .await
            .unwrap();
    }

    let logs = service.get_user_audit_logs(user_id, 3).await.unwrap();
    
    assert_eq!(logs.len(), 3);
    assert!(logs.iter().all(|log| log.user_id == Some(user_id)));
}

#[tokio::test]
async fn test_get_phone_audit_logs() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let phone_hash = "phone_hash_test_123";
    
    // Add some logs for the phone
    for i in 0..5 {
        service
            .log_send_code(
                phone_hash,
                i % 2 == 0,
                Some("10.0.0.9".to_string()),
                None,
                if i % 2 == 0 { None } else { Some("Failed".to_string()) },
            )
            .await
            .unwrap();
    }

    let logs = service.get_phone_audit_logs(phone_hash, 10).await.unwrap();
    
    assert_eq!(logs.len(), 5);
    assert!(logs.iter().all(|log| log.phone_hash == Some(phone_hash.to_string())));
}

#[tokio::test]
async fn test_async_writes_enabled() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: true, // Enable async writes
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_login(
            None,
            Some("phone_hash_async".to_string()),
            true,
            Some("10.0.0.10".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    
    // Wait a bit for async write to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, actions::LOGIN_ATTEMPT);
}

#[tokio::test]
async fn test_log_login_success() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let user_id = Uuid::new_v4();
    let result = service
        .log_login(
            Some(user_id),
            Some("phone_hash".to_string()),
            true,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            None,
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, actions::LOGIN_ATTEMPT);
    assert_eq!(logs[0].user_id, Some(user_id));
    assert_eq!(logs[0].phone_hash, Some("phone_hash".to_string()));
    assert!(logs[0].success);
    assert!(logs[0].error_message.is_none());
}

#[tokio::test]
async fn test_log_login_failure() {
    let repo = Arc::new(MockAuditLogRepository::new());
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_login(
            None,
            Some("phone_hash".to_string()),
            false,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            Some("Invalid credentials".to_string()),
        )
        .await;

    assert!(result.is_ok());
    
    let logs = repo.get_all_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, actions::LOGIN_ATTEMPT);
    assert!(!logs[0].success);
    assert_eq!(logs[0].error_message, Some("Invalid credentials".to_string()));
}

#[tokio::test]
async fn test_repository_error_handling() {
    let repo = Arc::new(MockAuditLogRepository::new());
    repo.set_should_fail(true);
    
    let config = AuditServiceConfig {
        async_writes: false,
        ..Default::default()
    };
    let service = AuditService::new(Arc::clone(&repo), config);

    let result = service
        .log_login(
            None,
            Some("phone_hash".to_string()),
            true,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
}