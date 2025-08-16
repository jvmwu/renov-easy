//! Token repository trait defining the interface for refresh token persistence.
//!
//! This module defines the repository pattern interface for RefreshToken entities,
//! managing JWT refresh tokens in the database for secure authentication.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::token::RefreshToken;
use crate::errors::DomainError;

/// Repository trait for RefreshToken entity persistence operations
///
/// This trait defines the contract for managing refresh tokens in the database.
/// Implementations should handle token storage, retrieval, validation, and revocation.
///
/// # Security Considerations
/// - Tokens should be hashed before storage
/// - Expired tokens should be periodically cleaned up
/// - Revoked tokens should be immediately invalidated
#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Save a new refresh token to the repository
    ///
    /// # Arguments
    /// * `token` - The RefreshToken entity to persist
    ///
    /// # Returns
    /// * `Ok(RefreshToken)` - The saved token with any database-generated fields
    /// * `Err(DomainError)` - Save failed (e.g., duplicate token)
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::TokenRepository;
    /// # use renov_core::domain::entities::token::RefreshToken;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::new_v4();
    /// let token = RefreshToken::new(user_id, "hashed_token_value".to_string());
    /// 
    /// let saved = repo.save_refresh_token(token).await?;
    /// println!("Token saved with ID: {}", saved.id);
    /// # Ok(())
    /// # }
    /// ```
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError>;

    /// Find a refresh token by its hashed value
    ///
    /// # Arguments
    /// * `token_hash` - The hashed token value to search for
    ///
    /// # Returns
    /// * `Ok(Some(RefreshToken))` - Token found
    /// * `Ok(None)` - No token found with given hash
    /// * `Err(DomainError)` - Database error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::TokenRepository;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let token_hash = "sha256_hash_of_token";
    /// 
    /// match repo.find_refresh_token(token_hash).await? {
    ///     Some(token) => {
    ///         if token.is_valid() {
    ///             println!("Token is valid for user: {}", token.user_id);
    ///         }
    ///     }
    ///     None => println!("Token not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError>;

    /// Find a refresh token by its ID
    ///
    /// # Arguments
    /// * `id` - The UUID of the refresh token
    ///
    /// # Returns
    /// * `Ok(Some(RefreshToken))` - Token found
    /// * `Ok(None)` - No token found with given ID
    /// * `Err(DomainError)` - Database error occurred
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, DomainError>;

    /// Find all valid refresh tokens for a user
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(Vec<RefreshToken>)` - List of valid (non-expired, non-revoked) tokens
    /// * `Err(DomainError)` - Database error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::TokenRepository;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// 
    /// let tokens = repo.find_by_user_id(user_id).await?;
    /// println!("User has {} valid tokens", tokens.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError>;

    /// Revoke a refresh token
    ///
    /// # Arguments
    /// * `token_hash` - The hashed token value to revoke
    ///
    /// # Returns
    /// * `Ok(true)` - Token was revoked
    /// * `Ok(false)` - Token not found
    /// * `Err(DomainError)` - Revocation failed
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::TokenRepository;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let token_hash = "sha256_hash_of_token";
    /// 
    /// if repo.revoke_token(token_hash).await? {
    ///     println!("Token revoked successfully");
    /// } else {
    ///     println!("Token not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn revoke_token(&self, token_hash: &str) -> Result<bool, DomainError>;

    /// Revoke all refresh tokens for a user
    ///
    /// Used during logout or security events.
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of tokens revoked
    /// * `Err(DomainError)` - Revocation failed
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::TokenRepository;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// 
    /// let count = repo.revoke_all_user_tokens(user_id).await?;
    /// println!("Revoked {} tokens for user", count);
    /// # Ok(())
    /// # }
    /// ```
    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError>;

    /// Delete expired tokens from the repository
    ///
    /// This should be called periodically to clean up expired tokens.
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of tokens deleted
    /// * `Err(DomainError)` - Deletion failed
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::TokenRepository;
    /// # async fn example(repo: &impl TokenRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let deleted = repo.delete_expired_tokens().await?;
    /// println!("Cleaned up {} expired tokens", deleted);
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_expired_tokens(&self) -> Result<usize, DomainError>;

    /// Check if a token exists and is valid
    ///
    /// # Arguments
    /// * `token_hash` - The hashed token value to check
    ///
    /// # Returns
    /// * `Ok(true)` - Token exists and is valid (not expired, not revoked)
    /// * `Ok(false)` - Token doesn't exist or is invalid
    /// * `Err(DomainError)` - Database error occurred
    async fn is_token_valid(&self, token_hash: &str) -> Result<bool, DomainError> {
        match self.find_refresh_token(token_hash).await? {
            Some(token) => Ok(token.is_valid()),
            None => Ok(false),
        }
    }

    /// Count active tokens for a user
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of active (valid) tokens
    /// * `Err(DomainError)` - Database error occurred
    async fn count_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
        let tokens = self.find_by_user_id(user_id).await?;
        Ok(tokens.len())
    }
}

/// Mock implementation of TokenRepository for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock token repository for testing
    pub struct MockTokenRepository {
        tokens: Arc<RwLock<HashMap<String, RefreshToken>>>,
    }

    impl MockTokenRepository {
        /// Create a new mock repository
        pub fn new() -> Self {
            Self {
                tokens: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl TokenRepository for MockTokenRepository {
        async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError> {
            let mut tokens = self.tokens.write().await;
            
            // Check for duplicate
            if tokens.contains_key(&token.token_hash) {
                return Err(DomainError::Validation {
                    message: "Token already exists".to_string(),
                });
            }
            
            tokens.insert(token.token_hash.clone(), token.clone());
            Ok(token)
        }

        async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError> {
            let tokens = self.tokens.read().await;
            Ok(tokens.get(token_hash).cloned())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, DomainError> {
            let tokens = self.tokens.read().await;
            Ok(tokens.values().find(|t| t.id == id).cloned())
        }

        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError> {
            let tokens = self.tokens.read().await;
            Ok(tokens
                .values()
                .filter(|t| t.user_id == user_id && t.is_valid())
                .cloned()
                .collect())
        }

        async fn revoke_token(&self, token_hash: &str) -> Result<bool, DomainError> {
            let mut tokens = self.tokens.write().await;
            
            if let Some(token) = tokens.get_mut(token_hash) {
                token.revoke();
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
            let mut tokens = self.tokens.write().await;
            let mut count = 0;
            
            for token in tokens.values_mut() {
                if token.user_id == user_id && !token.is_revoked {
                    token.revoke();
                    count += 1;
                }
            }
            
            Ok(count)
        }

        async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
            let mut tokens = self.tokens.write().await;
            let initial_count = tokens.len();
            
            tokens.retain(|_, token| !token.is_expired());
            
            Ok(initial_count - tokens.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::token::RefreshToken;

    #[tokio::test]
    async fn test_mock_save_and_find_token() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        let token = RefreshToken::new(user_id, "test_hash".to_string());
        
        let saved = repo.save_refresh_token(token.clone()).await.unwrap();
        assert_eq!(saved.id, token.id);
        
        let found = repo.find_refresh_token("test_hash").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, token.id);
    }

    #[tokio::test]
    async fn test_mock_duplicate_token() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        let token1 = RefreshToken::new(user_id, "same_hash".to_string());
        let token2 = RefreshToken::new(user_id, "same_hash".to_string());
        
        repo.save_refresh_token(token1).await.unwrap();
        let result = repo.save_refresh_token(token2).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_mock_find_by_user_id() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        
        let token1 = RefreshToken::new(user_id, "hash1".to_string());
        let token2 = RefreshToken::new(user_id, "hash2".to_string());
        let token3 = RefreshToken::new(Uuid::new_v4(), "hash3".to_string());
        
        repo.save_refresh_token(token1).await.unwrap();
        repo.save_refresh_token(token2).await.unwrap();
        repo.save_refresh_token(token3).await.unwrap();
        
        let user_tokens = repo.find_by_user_id(user_id).await.unwrap();
        assert_eq!(user_tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_revoke_token() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        let token = RefreshToken::new(user_id, "test_hash".to_string());
        
        repo.save_refresh_token(token).await.unwrap();
        
        let revoked = repo.revoke_token("test_hash").await.unwrap();
        assert!(revoked);
        
        let found = repo.find_refresh_token("test_hash").await.unwrap().unwrap();
        assert!(found.is_revoked);
        assert!(!found.is_valid());
    }

    #[tokio::test]
    async fn test_mock_revoke_all_user_tokens() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        
        for i in 0..3 {
            let token = RefreshToken::new(user_id, format!("hash_{}", i));
            repo.save_refresh_token(token).await.unwrap();
        }
        
        let count = repo.revoke_all_user_tokens(user_id).await.unwrap();
        assert_eq!(count, 3);
        
        let valid_tokens = repo.find_by_user_id(user_id).await.unwrap();
        assert_eq!(valid_tokens.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_is_token_valid() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        let token = RefreshToken::new(user_id, "test_hash".to_string());
        
        repo.save_refresh_token(token).await.unwrap();
        
        assert!(repo.is_token_valid("test_hash").await.unwrap());
        
        repo.revoke_token("test_hash").await.unwrap();
        
        assert!(!repo.is_token_valid("test_hash").await.unwrap());
        assert!(!repo.is_token_valid("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_count_user_tokens() {
        let repo = mock::MockTokenRepository::new();
        let user_id = Uuid::new_v4();
        
        assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 0);
        
        for i in 0..3 {
            let token = RefreshToken::new(user_id, format!("hash_{}", i));
            repo.save_refresh_token(token).await.unwrap();
        }
        
        assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 3);
        
        repo.revoke_token("hash_0").await.unwrap();
        
        assert_eq!(repo.count_user_tokens(user_id).await.unwrap(), 2);
    }
}