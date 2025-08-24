//! Tests for enhanced token storage functionality

use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;
use async_trait::async_trait;

use crate::domain::entities::token::{RefreshToken, Claims};
use crate::errors::{DomainError, TokenError};
use crate::repositories::TokenRepository;
use crate::services::token::{TokenService, TokenServiceConfig, TokenCleanupService, TokenCleanupConfig};

// Simple mock for TokenRepository
struct MockTokenRepository {
    find_refresh_token_response: Option<RefreshToken>,
    save_refresh_token_response: Option<RefreshToken>,
    revoke_token_response: bool,
    revoke_token_family_response: usize,
    is_token_blacklisted_response: bool,
    delete_expired_tokens_response: usize,
    cleanup_blacklist_response: usize,
}

impl MockTokenRepository {
    fn new() -> Self {
        Self {
            find_refresh_token_response: None,
            save_refresh_token_response: None,
            revoke_token_response: false,
            revoke_token_family_response: 0,
            is_token_blacklisted_response: false,
            delete_expired_tokens_response: 0,
            cleanup_blacklist_response: 0,
        }
    }

    fn with_find_refresh_token(mut self, token: RefreshToken) -> Self {
        self.find_refresh_token_response = Some(token);
        self
    }

    fn with_revoke_token(mut self, success: bool) -> Self {
        self.revoke_token_response = success;
        self
    }

    fn with_revoke_token_family(mut self, count: usize) -> Self {
        self.revoke_token_family_response = count;
        self
    }

    fn with_is_token_blacklisted(mut self, blacklisted: bool) -> Self {
        self.is_token_blacklisted_response = blacklisted;
        self
    }

    fn with_cleanup_responses(mut self, expired: usize, blacklist: usize) -> Self {
        self.delete_expired_tokens_response = expired;
        self.cleanup_blacklist_response = blacklist;
        self
    }
}

#[async_trait]
impl TokenRepository for MockTokenRepository {
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError> {
        Ok(self.save_refresh_token_response.clone().unwrap_or(token))
    }

    async fn find_refresh_token(&self, _token_hash: &str) -> Result<Option<RefreshToken>, DomainError> {
        Ok(self.find_refresh_token_response.clone())
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<RefreshToken>, DomainError> {
        Ok(None)
    }

    async fn find_by_user_id(&self, _user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError> {
        Ok(Vec::new())
    }

    async fn find_by_token_family(&self, _token_family: &str) -> Result<Vec<RefreshToken>, DomainError> {
        Ok(Vec::new())
    }

    async fn revoke_token_family(&self, _token_family: &str) -> Result<usize, DomainError> {
        Ok(self.revoke_token_family_response)
    }

    async fn is_token_blacklisted(&self, _token_jti: &str) -> Result<bool, DomainError> {
        Ok(self.is_token_blacklisted_response)
    }

    async fn blacklist_token(&self, _token_jti: &str, _expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), DomainError> {
        Ok(())
    }

    async fn revoke_token(&self, _token_hash: &str) -> Result<bool, DomainError> {
        Ok(self.revoke_token_response)
    }

    async fn revoke_all_user_tokens(&self, _user_id: Uuid) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
        Ok(self.delete_expired_tokens_response)
    }

    async fn cleanup_blacklist(&self) -> Result<usize, DomainError> {
        Ok(self.cleanup_blacklist_response)
    }
}

#[tokio::test]
async fn test_token_rotation_with_family_tracking() {
    let user_id = Uuid::new_v4();
    let old_token_hash = "old_token_hash";
    let token_family = Some("family_123".to_string());
    let device_fingerprint = Some("device_abc".to_string());

    // Setup old token
    let old_token = RefreshToken {
        id: Uuid::new_v4(),
        user_id,
        token_hash: old_token_hash.to_string(),
        created_at: Utc::now() - Duration::hours(1),
        expires_at: Utc::now() + Duration::days(29),
        is_revoked: false,
        token_family: token_family.clone(),
        device_fingerprint: device_fingerprint.clone(),
        previous_token_id: None,
    };

    let mock_repo = MockTokenRepository::new()
        .with_find_refresh_token(old_token.clone())
        .with_revoke_token(true);

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    // Perform rotation
    let result = service.refresh_tokens(
        old_token_hash,
        None,
        true,
        None,
        device_fingerprint.clone()
    ).await;

    assert!(result.is_ok());
    let token_pair = result.unwrap();
    // Token family should be created or maintained
    assert!(token_pair.token_family.is_some());
}

#[tokio::test]
async fn test_token_family_revocation_on_reuse() {
    let token_family = "family_456";
    let revoked_token_hash = "revoked_token";

    // Setup revoked token that's being reused
    let revoked_token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token_hash: revoked_token_hash.to_string(),
        created_at: Utc::now() - Duration::hours(2),
        expires_at: Utc::now() + Duration::days(28),
        is_revoked: true, // Already revoked
        token_family: Some(token_family.to_string()),
        device_fingerprint: None,
        previous_token_id: None,
    };

    let mock_repo = MockTokenRepository::new()
        .with_find_refresh_token(revoked_token)
        .with_revoke_token_family(3); // 3 tokens in family revoked

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    // Attempt to use revoked token
    let result = service.refresh_tokens(
        revoked_token_hash,
        None,
        true,
        None,
        None
    ).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::Token(TokenError::TokenRevoked)));
}

#[tokio::test]
async fn test_device_fingerprint_mismatch_detection() {
    let token_hash = "test_token";
    let original_fingerprint = Some("device_original".to_string());
    let different_fingerprint = Some("device_different".to_string());
    let token_family = Some("family_789".to_string());

    // Setup token with device fingerprint
    let token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token_hash: token_hash.to_string(),
        created_at: Utc::now() - Duration::minutes(30),
        expires_at: Utc::now() + Duration::days(29),
        is_revoked: false,
        token_family: token_family.clone(),
        device_fingerprint: original_fingerprint.clone(),
        previous_token_id: None,
    };

    let mock_repo = MockTokenRepository::new()
        .with_find_refresh_token(token)
        .with_revoke_token_family(2);

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    // Attempt refresh with different device fingerprint
    let result = service.refresh_tokens(
        token_hash,
        None,
        true,
        None,
        different_fingerprint
    ).await;

    // Should fail due to fingerprint mismatch
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_blacklist_check() {
    let jti = "test_jti_123";

    let mock_repo = MockTokenRepository::new()
        .with_is_token_blacklisted(true);

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    // Create a token with the blacklisted JTI
    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + Duration::minutes(15)).timestamp(),
        nbf: Utc::now().timestamp(),
        iss: "renov-easy".to_string(),
        aud: "renov-easy-api".to_string(),
        jti: jti.to_string(),
        user_type: None,
        is_verified: true,
        phone_hash: None,
        device_fingerprint: None,
        token_family: None,
    };

    let token = service.encode_jwt(&claims).unwrap();

    // Verify should fail due to blacklist
    let result = service.verify_access_token(&token).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::Token(TokenError::TokenRevoked)));
}

#[tokio::test]
async fn test_cleanup_expired_tokens() {
    let mock_repo = MockTokenRepository::new()
        .with_cleanup_responses(15, 8); // 15 expired tokens, 8 blacklist entries

    let cleanup_config = TokenCleanupConfig::default();
    let cleanup_service = TokenCleanupService::new(
        Arc::new(mock_repo),
        cleanup_config
    );

    let result = cleanup_service.run_cleanup().await.unwrap();

    assert_eq!(result.expired_tokens_deleted, 15);
    assert_eq!(result.blacklist_entries_deleted, 8);
    assert!(result.is_success());
    assert_eq!(result.total_cleaned(), 23);
}

#[tokio::test]
async fn test_token_rotation_creates_chain() {
    let user_id = Uuid::new_v4();

    // First rotation - no family yet
    let token1_hash = "token1";
    let token1 = RefreshToken {
        id: Uuid::new_v4(),
        user_id,
        token_hash: token1_hash.to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(30),
        is_revoked: false,
        token_family: None, // No family yet
        device_fingerprint: None,
        previous_token_id: None,
    };

    let mock_repo = MockTokenRepository::new()
        .with_find_refresh_token(token1)
        .with_revoke_token(true);

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    let result = service.refresh_tokens(
        token1_hash,
        None,
        true,
        None,
        None
    ).await;

    assert!(result.is_ok());
    // New token should have a family
    assert!(result.unwrap().token_family.is_some());
}

#[tokio::test]
async fn test_concurrent_token_usage_detection() {
    let token_hash = "concurrent_token";
    let token_family = Some("family_concurrent".to_string());

    // Token that's already been rotated
    let rotated_token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token_hash: token_hash.to_string(),
        created_at: Utc::now() - Duration::minutes(5),
        expires_at: Utc::now() + Duration::days(30),
        is_revoked: true, // Already rotated/revoked
        token_family: token_family.clone(),
        device_fingerprint: None,
        previous_token_id: None,
    };

    let mock_repo = MockTokenRepository::new()
        .with_find_refresh_token(rotated_token)
        .with_revoke_token_family(5); // All tokens in family revoked

    let config = TokenServiceConfig::default();
    let service = TokenService::new(mock_repo, config).unwrap();

    // Attempt to use already-rotated token (concurrent use scenario)
    let result = service.refresh_tokens(
        token_hash,
        None,
        true,
        None,
        None
    ).await;

    assert!(result.is_err());
    // Family should be revoked for security
}
