//! Tests for enhanced token storage functionality

use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;
use mockall::predicate::*;
use mockall::mock;
use async_trait::async_trait;

use crate::domain::entities::token::{RefreshToken, Claims, TokenPair};
use crate::errors::{DomainError, TokenError};
use crate::repositories::TokenRepository;
use crate::services::token::{TokenService, TokenServiceConfig, TokenCleanupService, TokenCleanupConfig};

// Mock for TokenRepository
mock! {
    TokenRepository {}
    
    #[async_trait]
    impl TokenRepository for TokenRepository {
        async fn save_refresh_token(&self, token: RefreshToken) 
            -> Result<RefreshToken, DomainError>;
        async fn find_refresh_token(&self, token_hash: &str) 
            -> Result<Option<RefreshToken>, DomainError>;
        async fn find_by_id(&self, id: Uuid) 
            -> Result<Option<RefreshToken>, DomainError>;
        async fn find_by_user_id(&self, user_id: Uuid) 
            -> Result<Vec<RefreshToken>, DomainError>;
        async fn find_by_token_family(&self, token_family: &str) 
            -> Result<Vec<RefreshToken>, DomainError>;
        async fn revoke_token_family(&self, token_family: &str) 
            -> Result<usize, DomainError>;
        async fn is_token_blacklisted(&self, token_jti: &str) 
            -> Result<bool, DomainError>;
        async fn blacklist_token(&self, token_jti: &str, expires_at: chrono::DateTime<chrono::Utc>) 
            -> Result<(), DomainError>;
        async fn revoke_token(&self, token_hash: &str) 
            -> Result<bool, DomainError>;
        async fn revoke_all_user_tokens(&self, user_id: Uuid) 
            -> Result<usize, DomainError>;
        async fn delete_expired_tokens(&self) 
            -> Result<usize, DomainError>;
        async fn cleanup_blacklist(&self) 
            -> Result<usize, DomainError>;
    }
}

#[tokio::test]
async fn test_token_rotation_with_family_tracking() {
    let mut mock_repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    let old_token_hash = "old_token_hash";
    let new_token_hash = "new_token_hash";
    let token_family = Some("family_123".to_string());
    let device_fingerprint = Some("device_abc".to_string());
    
    // Setup expectations for finding old token
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
    
    mock_repo.expect_find_refresh_token()
        .with(eq(old_token_hash))
        .times(1)
        .returning(move |_| Ok(Some(old_token.clone())));
    
    // Expect old token to be revoked
    mock_repo.expect_revoke_token()
        .with(eq(old_token_hash))
        .times(1)
        .returning(|_| Ok(true));
    
    // Expect new token to be saved with family
    mock_repo.expect_save_refresh_token()
        .withf(move |token| {
            token.token_hash == new_token_hash &&
            token.token_family == token_family &&
            token.device_fingerprint == device_fingerprint &&
            token.previous_token_id == Some(old_token.id)
        })
        .times(1)
        .returning(|token| Ok(token));
    
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
    assert_eq!(token_pair.token_family, token_family);
    assert_eq!(token_pair.device_fingerprint, device_fingerprint);
}

#[tokio::test]
async fn test_token_family_revocation_on_reuse() {
    let mut mock_repo = MockTokenRepository::new();
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
    
    mock_repo.expect_find_refresh_token()
        .with(eq(revoked_token_hash))
        .times(1)
        .returning(move |_| Ok(Some(revoked_token.clone())));
    
    // Expect entire family to be revoked
    mock_repo.expect_revoke_token_family()
        .with(eq(token_family))
        .times(1)
        .returning(|_| Ok(3)); // 3 tokens in family revoked
    
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
    matches!(result.unwrap_err(), DomainError::Token(TokenError::TokenRevoked));
}

#[tokio::test]
async fn test_device_fingerprint_mismatch_detection() {
    let mut mock_repo = MockTokenRepository::new();
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
    
    mock_repo.expect_find_refresh_token()
        .with(eq(token_hash))
        .times(1)
        .returning(move |_| Ok(Some(token.clone())));
    
    // Expect family revocation due to fingerprint mismatch
    mock_repo.expect_revoke_token_family()
        .with(eq(token_family.as_ref().unwrap()))
        .times(1)
        .returning(|_| Ok(2));
    
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
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_blacklist_check() {
    let mut mock_repo = MockTokenRepository::new();
    let jti = "test_jti_123";
    
    // Token is blacklisted
    mock_repo.expect_is_token_blacklisted()
        .with(eq(jti))
        .times(1)
        .returning(|_| Ok(true));
    
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
    matches!(result.unwrap_err(), DomainError::Token(TokenError::TokenRevoked));
}

#[tokio::test]
async fn test_cleanup_expired_tokens() {
    let mut mock_repo = MockTokenRepository::new();
    
    // Expect cleanup operations
    mock_repo.expect_delete_expired_tokens()
        .times(1)
        .returning(|| Ok(15)); // 15 expired tokens deleted
    
    mock_repo.expect_cleanup_blacklist()
        .times(1)
        .returning(|| Ok(8)); // 8 blacklist entries cleaned
    
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
    let mut mock_repo = MockTokenRepository::new();
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
    
    mock_repo.expect_find_refresh_token()
        .with(eq(token1_hash))
        .times(1)
        .returning(move |_| Ok(Some(token1.clone())));
    
    mock_repo.expect_revoke_token()
        .with(eq(token1_hash))
        .times(1)
        .returning(|_| Ok(true));
    
    // New token should create a family
    mock_repo.expect_save_refresh_token()
        .withf(|token| {
            token.token_family.is_some() && // Family created
            token.previous_token_id.is_some() // Links to previous
        })
        .times(1)
        .returning(|token| Ok(token));
    
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
    assert!(result.unwrap().token_family.is_some());
}

#[tokio::test]
async fn test_concurrent_token_usage_detection() {
    let mut mock_repo = MockTokenRepository::new();
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
    
    // First call finds the revoked token
    mock_repo.expect_find_refresh_token()
        .with(eq(token_hash))
        .times(1)
        .returning(move |_| Ok(Some(rotated_token.clone())));
    
    // Should revoke entire family due to concurrent use
    mock_repo.expect_revoke_token_family()
        .with(eq(token_family.as_ref().unwrap()))
        .times(1)
        .returning(|_| Ok(5)); // All tokens in family revoked
    
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