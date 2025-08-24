//! Unit tests for token service

use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;
use chrono::{Duration, Utc};
use async_trait::async_trait;
use jsonwebtoken::Algorithm;

use crate::domain::entities::token::{Claims, RefreshToken};
use crate::domain::entities::user::UserType;
use crate::errors::{DomainError, TokenError};
use crate::repositories::TokenRepository;
use crate::services::token::{TokenService, TokenServiceConfig};

/// Mock implementation of TokenRepository for testing
struct MockTokenRepository {
    tokens: Arc<Mutex<Vec<RefreshToken>>>,
}

impl MockTokenRepository {
    fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TokenRepository for MockTokenRepository {
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.push(token.clone());
        Ok(token)
    }

    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens.iter().find(|t| t.token_hash == token_hash).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens.iter().find(|t| t.id == id).cloned())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens
            .iter()
            .filter(|t| t.user_id == user_id && t.is_valid())
            .cloned()
            .collect())
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<bool, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.iter_mut().find(|t| t.token_hash == token_hash) {
            token.revoke();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        let mut count = 0;
        for token in tokens.iter_mut() {
            if token.user_id == user_id && !token.is_revoked {
                token.revoke();
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        let before_count = tokens.len();
        tokens.retain(|t| t.is_valid());
        Ok(before_count - tokens.len())
    }

    async fn count_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
        let tokens = self.find_by_user_id(user_id).await?;
        Ok(tokens.len())
    }
}

fn create_test_service() -> TokenService<MockTokenRepository> {
    let repository = MockTokenRepository::new();
    let mut config = TokenServiceConfig::default();
    // Use HS256 for tests to avoid needing key files
    config.algorithm = Algorithm::HS256;
    config.rs256_config = None;
    TokenService::new(repository, config).expect("Failed to create token service")
}

#[tokio::test]
async fn test_generate_tokens() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    let token_pair = service
        .generate_tokens(user_id, Some(UserType::Customer), true)
        .await
        .unwrap();
    
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.access_expires_in, 15 * 60);
    assert_eq!(token_pair.refresh_expires_in, 7 * 24 * 60 * 60);
}

#[tokio::test]
async fn test_verify_access_token() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    let token_pair = service
        .generate_tokens(user_id, Some(UserType::Worker), false)
        .await
        .unwrap();
    
    let claims = service
        .verify_access_token(&token_pair.access_token)
        .unwrap();
    
    assert_eq!(claims.user_id().unwrap(), user_id);
    assert_eq!(claims.user_type, Some("worker".to_string()));
    assert!(!claims.is_verified);
}

#[tokio::test]
async fn test_verify_invalid_access_token() {
    let service = create_test_service();
    let result = service.verify_access_token("invalid_token");
    
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DomainError::Token(TokenError::InvalidTokenFormat)
    ));
}

#[tokio::test]
async fn test_verify_refresh_token() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    let token_pair = service
        .generate_tokens(user_id, None, false)
        .await
        .unwrap();
    
    let verified_user_id = service
        .verify_refresh_token(&token_pair.refresh_token)
        .await
        .unwrap();
    
    assert_eq!(verified_user_id, user_id);
}

#[tokio::test]
async fn test_refresh_access_token() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    let token_pair = service
        .generate_tokens(user_id, Some(UserType::Customer), true)
        .await
        .unwrap();
    
    let new_access_token = service
        .refresh_access_token(
            &token_pair.refresh_token,
            Some(UserType::Customer),
            true,
        )
        .await
        .unwrap();
    
    assert!(!new_access_token.is_empty());
    
    // Verify the new access token
    let claims = service.verify_access_token(&new_access_token).unwrap();
    assert_eq!(claims.user_id().unwrap(), user_id);
}

#[tokio::test]
async fn test_revoke_tokens() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    // Generate multiple tokens for the user
    for _ in 0..3 {
        service
            .generate_tokens(user_id, None, false)
            .await
            .unwrap();
    }
    
    // Revoke all tokens
    service.revoke_tokens(user_id).await.unwrap();
    
    // Verify tokens are revoked
    let count = service.repository.count_user_tokens(user_id).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_revoke_specific_refresh_token() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    let token_pair = service
        .generate_tokens(user_id, None, false)
        .await
        .unwrap();
    
    // Revoke the specific refresh token
    let revoked = service
        .revoke_refresh_token(&token_pair.refresh_token)
        .await
        .unwrap();
    
    assert!(revoked);
    
    // Verify token is revoked
    let result = service
        .verify_refresh_token(&token_pair.refresh_token)
        .await;
    
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DomainError::Token(TokenError::TokenRevoked)
    ));
}

#[tokio::test]
async fn test_cleanup_expired_tokens() {
    let service = create_test_service();
    let user_id = Uuid::new_v4();
    
    // Generate a token
    service
        .generate_tokens(user_id, None, false)
        .await
        .unwrap();
    
    // Since we can't easily expire tokens in tests,
    // just verify the cleanup method doesn't error
    let cleaned = service.cleanup_expired_tokens().await.unwrap();
    assert_eq!(cleaned, 0); // No expired tokens yet
}

#[tokio::test]
async fn test_token_hash() {
    let service = create_test_service();
    let token = "test_token";
    
    let hash1 = service.hash_token(token);
    let hash2 = service.hash_token(token);
    
    // Same token should produce same hash
    assert_eq!(hash1, hash2);
    
    // Different token should produce different hash
    let hash3 = service.hash_token("different_token");
    assert_ne!(hash1, hash3);
}

#[tokio::test]
async fn test_expired_token_validation() {
    let service = create_test_service();
    
    // Create expired claims manually
    let user_id = Uuid::new_v4();
    let mut claims = Claims::new_access_token(user_id, None, false);
    claims.exp = (Utc::now() - Duration::hours(1)).timestamp();
    
    let token = service.encode_jwt(&claims).unwrap();
    let result = service.verify_access_token(&token);
    
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DomainError::Token(TokenError::TokenExpired)
    ));
}

#[tokio::test]
async fn test_not_yet_valid_token() {
    let service = create_test_service();
    
    // Create future nbf claims manually
    let user_id = Uuid::new_v4();
    let mut claims = Claims::new_access_token(user_id, None, false);
    claims.nbf = (Utc::now() + Duration::hours(1)).timestamp();
    
    let token = service.encode_jwt(&claims).unwrap();
    let result = service.verify_access_token(&token);
    
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DomainError::Token(TokenError::TokenNotYetValid)
    ));
}