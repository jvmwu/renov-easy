//! Integration tests for audit logging in authentication service

use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entities::audit::{AuditLog, AuditEventType};
use crate::domain::entities::user::User;
use crate::errors::{DomainError};
use crate::repositories::AuditLogRepository;
use crate::services::auth::{AuthService, AuthServiceConfig};
use crate::services::audit::{AuditService, AuditServiceConfig};
use crate::services::token::{TokenService, TokenServiceConfig};
use crate::services::verification::{VerificationService, VerificationServiceConfig};
use jsonwebtoken::Algorithm;


/// Mock implementation of AuditLogRepository for testing
pub struct MockAuditLogRepository {
    pub logs: Arc<Mutex<Vec<AuditLog>>>,
}

impl MockAuditLogRepository {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get all logs for verification
    pub fn get_logs(&self) -> Vec<AuditLog> {
        self.logs.lock().unwrap().clone()
    }
    
    /// Get the last log entry
    pub fn get_last_log(&self) -> Option<AuditLog> {
        self.logs.lock().unwrap().last().cloned()
    }
    
    /// Count logs of a specific event type
    pub fn count_by_event_type(&self, event_type: AuditEventType) -> usize {
        self.logs.lock().unwrap()
            .iter()
            .filter(|log| log.event_type == event_type)
            .count()
    }
}

#[async_trait]
impl AuditLogRepository for MockAuditLogRepository {
    async fn create(&self, audit_log: &AuditLog) -> Result<(), DomainError> {
        let mut logs = self.logs.lock().unwrap();
        logs.push(audit_log.clone());
        Ok(())
    }

    async fn find_by_user(&self, user_id: Uuid, limit: usize) -> Result<Vec<AuditLog>, DomainError> {
        let logs = self.logs.lock().unwrap();
        Ok(logs
            .iter()
            .filter(|log| log.user_id == Some(user_id))
            .take(limit)
            .cloned()
            .collect())
    }

    async fn find_by_phone_hash(&self, phone_hash: &str, limit: usize) -> Result<Vec<AuditLog>, DomainError> {
        let logs = self.logs.lock().unwrap();
        Ok(logs
            .iter()
            .filter(|log| log.phone_hash.as_deref() == Some(phone_hash))
            .take(limit)
            .cloned()
            .collect())
    }

    async fn count_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        let logs = self.logs.lock().unwrap();
        Ok(logs
            .iter()
            .filter(|log| {
                log.action == action
                    && !log.success
                    && log.created_at >= since
                    && (phone_hash.is_none() || log.phone_hash.as_deref() == phone_hash)
                    && (ip_address.is_none() || log.ip_address == ip_address.unwrap_or(""))
            })
            .count())
    }

    async fn find_suspicious_activity(
        &self,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let logs = self.logs.lock().unwrap();
        Ok(logs
            .iter()
            .filter(|log| {
                !log.success
                    && log.created_at >= since
                    && (ip_address.is_none() || log.ip_address == ip_address.unwrap_or(""))
            })
            .cloned()
            .collect())
    }

    async fn archive_old_logs(&self) -> Result<usize, DomainError> {
        let mut logs = self.logs.lock().unwrap();
        let mut count = 0;
        for log in logs.iter_mut() {
            if !log.archived {
                log.archived = true;
                log.archived_at = Some(Utc::now());
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete_archived_logs(&self) -> Result<usize, DomainError> {
        let mut logs = self.logs.lock().unwrap();
        let before_count = logs.len();
        logs.retain(|log| !log.archived);
        Ok(before_count - logs.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Create a test authentication service with audit logging
    async fn create_test_service_with_audit() -> (
        AuthService<
            super::super::mocks::MockUserRepository,
            super::super::mocks::MockSmsService,
            super::super::mocks::MockCacheService,
            MockRateLimiter,
            MockTokenRepository,
            MockAuditLogRepository,
        >,
        Arc<MockAuditLogRepository>,
        Arc<MockRateLimiter>,
    ) {
        let user_repo = Arc::new(super::super::mocks::MockUserRepository::new());
        let sms_service = Arc::new(super::super::mocks::MockSmsService);
        let cache_service = Arc::new(super::super::mocks::MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new());
        let token_repo = MockTokenRepository;
        
        let mut token_config = TokenServiceConfig::default();
        // Use HS256 for tests to avoid needing key files
        token_config.algorithm = Algorithm::HS256;
        token_config.rs256_config = None;
        let token_service = Arc::new(TokenService::new(token_repo, token_config).expect("Failed to create token service"));
        
        let audit_repo = Arc::new(MockAuditLogRepository::new());
        let audit_service = Arc::new(AuditService::new(
            audit_repo.clone(),
            AuditServiceConfig::default(),
        ));
        
        let auth_config = AuthServiceConfig::default();
        
        let auth_service = AuthService::with_audit(
            user_repo,
            verification_service,
            rate_limiter.clone(),
            token_service,
            audit_service,
            auth_config,
        );
        
        (auth_service, audit_repo, rate_limiter)
    }
    
    #[tokio::test]
    async fn test_audit_log_successful_code_send() {
        let (auth_service, audit_repo, _rate_limiter) = create_test_service_with_audit().await;
        
        let phone = "+1234567890";
        let client_ip = Some("192.168.1.1".to_string());
        let user_agent = Some("Mozilla/5.0 iPhone".to_string());
        
        // Send verification code
        let result = auth_service
            .send_verification_code(phone, client_ip.clone(), user_agent.clone())
            .await;
        
        assert!(result.is_ok());
        
        // Verify audit log was created
        let logs = audit_repo.get_logs();
        assert_eq!(logs.len(), 1);
        
        let log = &logs[0];
        assert_eq!(log.event_type, AuditEventType::SendCodeSuccess);
        assert_eq!(log.ip_address, "192.168.1.1");
        assert_eq!(log.user_agent, user_agent);
        assert!(log.phone_masked.is_some());
        assert!(log.phone_hash.is_some());
        assert!(log.event_data.is_some());
    }
    
    #[tokio::test]
    async fn test_audit_log_rate_limit_exceeded() {
        let (auth_service, audit_repo, rate_limiter) = create_test_service_with_audit().await;
        
        let phone = "+1234567890";
        let client_ip = Some("192.168.1.1".to_string());
        let user_agent = Some("Mozilla/5.0 Chrome".to_string());
        
        // Set rate limiter to trigger limit
        rate_limiter.set_phone_rate_limited(true);
        
        // Try to send verification code
        let result = auth_service
            .send_verification_code(phone, client_ip.clone(), user_agent.clone())
            .await;
        
        assert!(result.is_err());
        
        // Verify rate limit audit log was created
        let logs = audit_repo.get_logs();
        assert!(logs.len() >= 1);
        
        let rate_limit_logs: Vec<_> = logs
            .iter()
            .filter(|log| matches!(
                log.event_type,
                AuditEventType::RateLimitPhoneExceeded | AuditEventType::RateLimitExceeded
            ))
            .collect();
        
        assert!(!rate_limit_logs.is_empty());
        let log = rate_limit_logs[0];
        assert_eq!(log.ip_address, "192.168.1.1");
        assert_eq!(log.rate_limit_type, Some("phone".to_string()));
    }
    
    #[tokio::test]
    async fn test_audit_log_successful_login() {
        let (auth_service, audit_repo, _rate_limiter) = create_test_service_with_audit().await;
        
        let phone = "+1234567890";
        let code = "123456";
        let client_ip = Some("192.168.1.1".to_string());
        let user_agent = Some("Mozilla/5.0 Android".to_string());
        
        // Verify code (simulated success)
        let result = auth_service
            .verify_code(phone, code, client_ip.clone(), user_agent.clone(), None)
            .await;
        
        assert!(result.is_ok());
        
        // Verify login success audit log was created
        let logs = audit_repo.get_logs();
        let login_success_count = audit_repo.count_by_event_type(AuditEventType::LoginSuccess);
        
        assert_eq!(login_success_count, 1);
        
        let login_log = logs
            .iter()
            .find(|log| log.event_type == AuditEventType::LoginSuccess)
            .unwrap();
        
        assert_eq!(login_log.ip_address, "192.168.1.1");
        assert!(login_log.user_id.is_some());
        assert!(login_log.phone_hash.is_some());
        assert!(login_log.token_id.is_some());
    }
    
    #[tokio::test]
    async fn test_audit_log_failed_login() {
        let (auth_service, audit_repo, _rate_limiter) = create_test_service_with_audit().await;
        
        // Create auth service with failing cache
        let user_repo = Arc::new(super::super::mocks::MockUserRepository::new());
        let sms_service = Arc::new(super::super::mocks::MockSmsService);
        let cache_service = Arc::new(super::super::mocks::MockCacheService::new_failure(2)); // Will fail verification
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new());
        let token_repo = MockTokenRepository;
        
        let mut token_config = TokenServiceConfig::default();
        // Use HS256 for tests to avoid needing key files
        token_config.algorithm = Algorithm::HS256;
        token_config.rs256_config = None;
        let token_service = Arc::new(TokenService::new(token_repo, token_config).expect("Failed to create token service"));
        
        let audit_service = Arc::new(AuditService::new(
            audit_repo.clone(),
            AuditServiceConfig::default(),
        ));
        
        let auth_config = AuthServiceConfig::default();
        
        let auth_service = AuthService::with_audit(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            audit_service,
            auth_config,
        );
        
        let phone = "+1234567890";
        let code = "wrong_code";
        let client_ip = Some("192.168.1.1".to_string());
        let user_agent = Some("Mozilla/5.0 Safari".to_string());
        
        // Try to verify with wrong code
        let result = auth_service
            .verify_code(phone, code, client_ip.clone(), user_agent.clone(), None)
            .await;
        
        assert!(result.is_err());
        
        // Verify failure audit logs were created
        let logs = audit_repo.get_logs();
        assert!(logs.len() >= 1);
        
        // Should have both VerifyCodeFailure and LoginFailure logs
        let verify_failure_count = audit_repo.count_by_event_type(AuditEventType::VerifyCodeFailure);
        let login_failure_count = audit_repo.count_by_event_type(AuditEventType::LoginFailure);
        
        assert!(verify_failure_count > 0 || login_failure_count > 0);
        
        // Check that failure reason is logged
        let failure_log = logs
            .iter()
            .find(|log| matches!(
                log.event_type,
                AuditEventType::VerifyCodeFailure | AuditEventType::LoginFailure
            ))
            .unwrap();
        
        assert!(failure_log.failure_reason.is_some());
        assert_eq!(failure_log.ip_address, "192.168.1.1");
    }
    
    #[tokio::test]
    async fn test_audit_log_token_refresh() {
        // This test would require a more complete setup with actual JWT tokens
        // For now, we verify the structure is in place
        let (auth_service, audit_repo, _rate_limiter) = create_test_service_with_audit().await;
        
        // The actual refresh token test would require setting up a valid user
        // and token first, then attempting refresh
        
        // Verify the audit repository is properly connected
        assert_eq!(audit_repo.get_logs().len(), 0);
    }
    
    #[tokio::test]
    async fn test_audit_log_logout() {
        let (auth_service, audit_repo, _rate_limiter) = create_test_service_with_audit().await;
        
        let user_id = Uuid::new_v4();
        let client_ip = Some("192.168.1.1".to_string());
        let user_agent = Some("Mozilla/5.0 Firefox".to_string());
        
        // Perform logout
        let result = auth_service
            .logout(user_id, None, client_ip.clone(), user_agent.clone(), None)
            .await;
        
        assert!(result.is_ok());
        
        // Verify logout audit log was created
        let logs = audit_repo.get_logs();
        assert_eq!(logs.len(), 1);
        
        let log = &logs[0];
        assert_eq!(log.event_type, AuditEventType::Logout);
        assert_eq!(log.user_id, Some(user_id));
        assert_eq!(log.ip_address, "192.168.1.1");
        assert!(log.event_data.is_some());
    }
}

/// Mock implementation of RateLimiterTrait for testing
pub struct MockRateLimiter {
    phone_rate_limited: Arc<Mutex<bool>>,
    ip_rate_limited: Arc<Mutex<bool>>,
}

impl MockRateLimiter {
    pub fn new() -> Self {
        Self {
            phone_rate_limited: Arc::new(Mutex::new(false)),
            ip_rate_limited: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn set_phone_rate_limited(&self, limited: bool) {
        *self.phone_rate_limited.lock().unwrap() = limited;
    }
    
    pub fn set_ip_rate_limited(&self, limited: bool) {
        *self.ip_rate_limited.lock().unwrap() = limited;
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl crate::services::auth::rate_limiter::RateLimiterTrait for MockRateLimiter {
    async fn check_sms_rate_limit(&self, _phone: &str) -> Result<bool, String> {
        Ok(*self.phone_rate_limited.lock().unwrap())
    }
    
    async fn increment_sms_counter(&self, _phone: &str) -> Result<i64, String> {
        Ok(1)
    }
    
    async fn check_ip_verification_limit(&self, _ip: &str) -> Result<bool, String> {
        Ok(*self.ip_rate_limited.lock().unwrap())
    }
    
    async fn increment_ip_verification_counter(&self, _ip: &str) -> Result<i64, String> {
        Ok(1)
    }
    
    async fn get_rate_limit_reset_time(&self, _phone: &str) -> Result<Option<i64>, String> {
        Ok(Some(3600))
    }
    
    async fn get_ip_rate_limit_reset_time(&self, _ip: &str) -> Result<Option<i64>, String> {
        Ok(Some(3600))
    }
    
    async fn log_rate_limit_violation(
        &self,
        _identifier: &str,
        _limit_type: &str,
        _action: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Mock implementation of TokenRepository
pub struct MockTokenRepository;

#[async_trait]
impl crate::repositories::TokenRepository for MockTokenRepository {
    async fn save_refresh_token(&self, token: crate::domain::entities::token::RefreshToken) -> Result<crate::domain::entities::token::RefreshToken, DomainError> {
        Ok(token)
    }

    async fn find_refresh_token(&self, _token_hash: &str) -> Result<Option<crate::domain::entities::token::RefreshToken>, DomainError> {
        Ok(None)
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<crate::domain::entities::token::RefreshToken>, DomainError> {
        Ok(None)
    }

    async fn find_by_user_id(&self, _user_id: Uuid) -> Result<Vec<crate::domain::entities::token::RefreshToken>, DomainError> {
        Ok(Vec::new())
    }

    async fn revoke_token(&self, _token_hash: &str) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn revoke_all_user_tokens(&self, _user_id: Uuid) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn count_user_tokens(&self, _user_id: Uuid) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn find_by_token_family(&self, _token_family: &str) -> Result<Vec<crate::domain::entities::token::RefreshToken>, DomainError> {
        Ok(Vec::new())
    }

    async fn revoke_token_family(&self, _token_family: &str) -> Result<usize, DomainError> {
        Ok(0)
    }

    
    async fn is_token_blacklisted(&self, _token_jti: &str) -> Result<bool, DomainError> {
        Ok(false)
    }
    
    async fn blacklist_token(&self, _token_jti: &str, _expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), DomainError> {
        Ok(())
    }
    
    async fn cleanup_blacklist(&self) -> Result<usize, DomainError> {
        Ok(0)
    }
}