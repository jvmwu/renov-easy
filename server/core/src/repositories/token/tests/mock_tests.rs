//! Unit tests for mock token repository implementation

use uuid::Uuid;
use chrono::Utc;

use crate::domain::entities::token::RefreshToken;
use crate::repositories::token::{TokenRepository, MockTokenRepository};

#[tokio::test]
async fn test_save_and_find_refresh_token() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    let token = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "test_hash".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    // Save token
    let saved = repo.save_refresh_token(token.clone()).await.unwrap();
    assert_eq!(saved.id, token.id);
    
    // Find token by hash
    let found = repo.find_refresh_token("test_hash").await.unwrap();
    assert!(found.is_some());
    
    let found_token = found.unwrap();
    assert_eq!(found_token.id, token.id);
    assert_eq!(found_token.user_id, token.user_id);
    assert_eq!(found_token.token_hash, token.token_hash);
}

#[tokio::test]
async fn test_duplicate_token() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    let token1 = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "same_hash".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    let token2 = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "same_hash".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    // First save should succeed
    repo.save_refresh_token(token1).await.unwrap();
    
    // Second save with same hash should fail
    let result = repo.save_refresh_token(token2).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_by_id() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    
    let token = RefreshToken {
        id: token_id,
        token_hash: "test_hash".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    repo.save_refresh_token(token).await.unwrap();
    
    // Find by ID
    let found = repo.find_by_id(token_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, token_id);
    
    // Find non-existent ID
    let not_found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_by_user_id() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    
    // Create tokens for first user
    for i in 0..3 {
        let token = RefreshToken {
            id: Uuid::new_v4(),
            token_hash: format!("hash_{}", i),
            user_id,
            expires_at: Utc::now() + chrono::Duration::days(7),
            is_revoked: false,
            created_at: Utc::now(),
        };
        repo.save_refresh_token(token).await.unwrap();
    }
    
    // Create token for second user
    let other_token = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "other_hash".to_string(),
        user_id: other_user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    repo.save_refresh_token(other_token).await.unwrap();
    
    // Find tokens for first user
    let user_tokens = repo.find_by_user_id(user_id).await.unwrap();
    assert_eq!(user_tokens.len(), 3);
    
    // Find tokens for second user
    let other_tokens = repo.find_by_user_id(other_user_id).await.unwrap();
    assert_eq!(other_tokens.len(), 1);
}

#[tokio::test]
async fn test_revoke_token() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    let token = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "test_hash".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    repo.save_refresh_token(token).await.unwrap();
    
    // Revoke token
    let revoked = repo.revoke_token("test_hash").await.unwrap();
    assert!(revoked);
    
    // Check token is revoked
    let found = repo.find_refresh_token("test_hash").await.unwrap().unwrap();
    assert!(found.is_revoked);
    assert!(!found.is_valid());
    
    // Revoking non-existent token returns false
    let not_revoked = repo.revoke_token("nonexistent").await.unwrap();
    assert!(!not_revoked);
}

#[tokio::test]
async fn test_revoke_all_user_tokens() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    
    // Create tokens for first user
    for i in 0..3 {
        let token = RefreshToken {
            id: Uuid::new_v4(),
            token_hash: format!("user1_hash_{}", i),
            user_id,
            expires_at: Utc::now() + chrono::Duration::days(7),
            is_revoked: false,
            created_at: Utc::now(),
        };
        repo.save_refresh_token(token).await.unwrap();
    }
    
    // Create token for second user
    let other_token = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "user2_hash".to_string(),
        user_id: other_user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    repo.save_refresh_token(other_token).await.unwrap();
    
    // Revoke all tokens for first user
    let count = repo.revoke_all_user_tokens(user_id).await.unwrap();
    assert_eq!(count, 3);
    
    // First user should have no valid tokens
    let valid_tokens = repo.find_by_user_id(user_id).await.unwrap();
    assert_eq!(valid_tokens.len(), 0);
    
    // Second user should still have valid tokens
    let other_valid = repo.find_by_user_id(other_user_id).await.unwrap();
    assert_eq!(other_valid.len(), 1);
}

#[tokio::test]
async fn test_delete_expired_tokens() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    // Create expired tokens
    for i in 0..3 {
        let token = RefreshToken {
            id: Uuid::new_v4(),
            token_hash: format!("expired_{}", i),
            user_id,
            expires_at: Utc::now() - chrono::Duration::days(1), // Expired
            is_revoked: false,
            created_at: Utc::now() - chrono::Duration::days(8),
        };
        repo.save_refresh_token(token).await.unwrap();
    }
    
    // Create valid tokens
    for i in 0..2 {
        let token = RefreshToken {
            id: Uuid::new_v4(),
            token_hash: format!("valid_{}", i),
            user_id,
            expires_at: Utc::now() + chrono::Duration::days(7), // Valid
            is_revoked: false,
            created_at: Utc::now(),
        };
        repo.save_refresh_token(token).await.unwrap();
    }
    
    // Delete expired tokens
    let deleted = repo.delete_expired_tokens().await.unwrap();
    assert_eq!(deleted, 3);
    
    // Expired tokens should be gone
    for i in 0..3 {
        let found = repo.find_refresh_token(&format!("expired_{}", i)).await.unwrap();
        assert!(found.is_none());
    }
    
    // Valid tokens should still exist
    for i in 0..2 {
        let found = repo.find_refresh_token(&format!("valid_{}", i)).await.unwrap();
        assert!(found.is_some());
    }
}

#[tokio::test]
async fn test_is_token_valid() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    let valid_token = RefreshToken {
        id: Uuid::new_v4(),
        token_hash: "valid_token".to_string(),
        user_id,
        expires_at: Utc::now() + chrono::Duration::days(7),
        is_revoked: false,
        created_at: Utc::now(),
    };
    
    repo.save_refresh_token(valid_token).await.unwrap();
    
    // Check valid token
    assert!(repo.is_token_valid("valid_token").await.unwrap());
    
    // Revoke token
    repo.revoke_token("valid_token").await.unwrap();
    
    // Check revoked token
    assert!(!repo.is_token_valid("valid_token").await.unwrap());
    
    // Check non-existent token
    assert!(!repo.is_token_valid("nonexistent").await.unwrap());
}

#[tokio::test]
async fn test_count_user_tokens() {
    let repo = MockTokenRepository::new();
    let user_id = Uuid::new_v4();
    
    // Initially no tokens
    assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 0);
    
    // Add tokens
    for i in 0..3 {
        let token = RefreshToken {
            id: Uuid::new_v4(),
            token_hash: format!("hash_{}", i),
            user_id,
            expires_at: Utc::now() + chrono::Duration::days(7),
            is_revoked: false,
            created_at: Utc::now(),
        };
        repo.save_refresh_token(token).await.unwrap();
    }
    
    // Count should be 3
    assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 3);
    
    // Revoke one token
    repo.revoke_token("hash_0").await.unwrap();
    
    // Count should be 2 (only valid tokens)
    assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 2);
}