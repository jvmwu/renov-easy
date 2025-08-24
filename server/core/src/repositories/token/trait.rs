//! Token repository trait defining the interface for refresh token persistence.

use async_trait::async_trait;
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
    /// # async fn example(repo: &impl TokenRepository, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_tokens = repo.find_by_user_id(user_id).await?;
    /// println!("User has {} active tokens", user_tokens.len());
    /// 
    /// for token in user_tokens {
    ///     println!("Token expires at: {:?}", token.expires_at);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError>;
    
    /// Find refresh tokens by token family
    ///
    /// # Arguments
    /// * `token_family` - The token family ID
    ///
    /// # Returns
    /// * `Ok(Vec<RefreshToken>)` - List of tokens in the family
    /// * `Err(DomainError)` - Database error occurred
    async fn find_by_token_family(&self, token_family: &str) -> Result<Vec<RefreshToken>, DomainError>;
    
    /// Revoke all tokens in a token family
    ///
    /// # Arguments
    /// * `token_family` - The token family ID to revoke
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of tokens revoked
    /// * `Err(DomainError)` - Revocation failed
    async fn revoke_token_family(&self, token_family: &str) -> Result<usize, DomainError>;
    
    /// Check if a token is blacklisted
    ///
    /// # Arguments
    /// * `token_jti` - The JWT ID to check
    ///
    /// # Returns
    /// * `Ok(bool)` - True if blacklisted, false otherwise
    /// * `Err(DomainError)` - Database error occurred
    async fn is_token_blacklisted(&self, token_jti: &str) -> Result<bool, DomainError>;
    
    /// Add a token to the blacklist
    ///
    /// # Arguments
    /// * `token_jti` - The JWT ID to blacklist
    /// * `expires_at` - When the blacklist entry expires
    ///
    /// # Returns
    /// * `Ok(())` - Token blacklisted successfully
    /// * `Err(DomainError)` - Blacklisting failed
    async fn blacklist_token(&self, token_jti: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), DomainError>;

    /// Revoke a specific refresh token
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
    /// # async fn example(repo: &impl TokenRepository, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    /// let revoked_count = repo.revoke_all_user_tokens(user_id).await?;
    /// println!("Revoked {} tokens for user", revoked_count);
    /// # Ok(())
    /// # }
    /// ```
    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError>;

    /// Delete expired refresh tokens from the repository
    ///
    /// This method should be called periodically to clean up expired tokens.
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of expired tokens deleted
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
    
    /// Clean up expired blacklist entries
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of entries cleaned up
    /// * `Err(DomainError)` - Cleanup failed
    async fn cleanup_blacklist(&self) -> Result<usize, DomainError>;
}