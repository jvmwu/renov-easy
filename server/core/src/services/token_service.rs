//! Token service for JWT generation, verification, and refresh token management.
//!
//! This service handles all token-related operations including:
//! - JWT access token generation (15 minutes expiry)
//! - Refresh token generation (7 days expiry)
//! - Token verification and validation
//! - Token refresh logic
//! - Token revocation

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::entities::token::{Claims, RefreshToken, TokenPair};
use crate::domain::entities::user::UserType;
use crate::errors::{DomainError, TokenError};
use crate::repositories::TokenRepository;

/// Configuration for the token service
#[derive(Debug, Clone)]
pub struct TokenServiceConfig {
    /// JWT signing secret
    pub jwt_secret: String,
    /// JWT signing algorithm
    pub algorithm: Algorithm,
    /// Access token expiry in minutes
    pub access_token_expiry_minutes: i64,
    /// Refresh token expiry in days
    pub refresh_token_expiry_days: i64,
}

impl Default for TokenServiceConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "development-secret-please-change-in-production".to_string(),
            algorithm: Algorithm::HS256,
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
        }
    }
}

/// Service for managing JWT tokens and refresh tokens
pub struct TokenService<R: TokenRepository> {
    repository: R,
    config: TokenServiceConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl<R: TokenRepository> TokenService<R> {
    /// Creates a new token service instance
    ///
    /// # Arguments
    ///
    /// * `repository` - Token repository for persistence
    /// * `config` - Token service configuration
    ///
    /// # Returns
    ///
    /// A new `TokenService` instance
    pub fn new(repository: R, config: TokenServiceConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&["renov-easy"]);
        validation.set_audience(&["renov-easy-api"]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        Self {
            repository,
            config,
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Generates a new token pair (access + refresh tokens) for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `user_type` - The user's type (Customer or Worker)
    /// * `is_verified` - Whether the user is verified
    ///
    /// # Returns
    ///
    /// * `Ok(TokenPair)` - The generated token pair
    /// * `Err(TokenError)` - Token generation failed
    pub async fn generate_tokens(
        &self,
        user_id: Uuid,
        user_type: Option<UserType>,
        is_verified: bool,
    ) -> Result<TokenPair, DomainError> {
        // Generate access token
        let access_claims = Claims::new_access_token(
            user_id,
            user_type.map(|t| match t {
                UserType::Customer => "customer".to_string(),
                UserType::Worker => "worker".to_string(),
            }),
            is_verified,
        );
        
        let access_token = self.encode_jwt(&access_claims)?;
        
        // Generate refresh token
        let refresh_claims = Claims::new_refresh_token(user_id);
        let refresh_token = self.encode_jwt(&refresh_claims)?;
        
        // Hash the refresh token for storage
        let token_hash = self.hash_token(&refresh_token);
        
        // Store refresh token in database
        let refresh_token_entity = RefreshToken::new(user_id, token_hash);
        self.repository.save_refresh_token(refresh_token_entity).await?;
        
        Ok(TokenPair::new(access_token, refresh_token))
    }

    /// Verifies an access token and returns the claims
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT access token to verify
    ///
    /// # Returns
    ///
    /// * `Ok(Claims)` - The validated claims
    /// * `Err(TokenError)` - Token verification failed
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        TokenError::TokenExpired
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        TokenError::InvalidTokenFormat
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        TokenError::InvalidSignature
                    }
                    jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                        TokenError::TokenNotYetValid
                    }
                    _ => TokenError::InvalidTokenFormat,
                }
            })?;

        // Additional validation
        if !token_data.claims.is_valid() {
            return Err(TokenError::TokenExpired.into());
        }

        Ok(token_data.claims)
    }

    /// Verifies a refresh token and returns the user ID
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT refresh token to verify
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - The user ID from the token
    /// * `Err(TokenError)` - Token verification failed
    pub async fn verify_refresh_token(&self, token: &str) -> Result<Uuid, DomainError> {
        // First verify the JWT structure and signature
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        TokenError::RefreshTokenExpired
                    }
                    _ => TokenError::InvalidRefreshToken,
                }
            })?;

        // Hash the token to check against database
        let token_hash = self.hash_token(token);
        
        // Check if token exists and is valid in database
        let stored_token = self.repository
            .find_refresh_token(&token_hash)
            .await?
            .ok_or(TokenError::InvalidRefreshToken)?;

        if !stored_token.is_valid() {
            if stored_token.is_expired() {
                return Err(TokenError::RefreshTokenExpired.into());
            }
            if stored_token.is_revoked {
                return Err(TokenError::TokenRevoked.into());
            }
            return Err(TokenError::InvalidRefreshToken.into());
        }

        // Parse user ID from claims
        token_data.claims
            .user_id()
            .map_err(|_| TokenError::InvalidClaims.into())
    }

    /// Refreshes an access token using a valid refresh token
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token to use
    /// * `user_type` - The user's type (Customer or Worker)
    /// * `is_verified` - Whether the user is verified
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The new access token
    /// * `Err(TokenError)` - Token refresh failed
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
        user_type: Option<UserType>,
        is_verified: bool,
    ) -> Result<String, DomainError> {
        // Verify the refresh token and get user ID
        let user_id = self.verify_refresh_token(refresh_token).await?;
        
        // Generate new access token
        let access_claims = Claims::new_access_token(
            user_id,
            user_type.map(|t| match t {
                UserType::Customer => "customer".to_string(),
                UserType::Worker => "worker".to_string(),
            }),
            is_verified,
        );
        
        self.encode_jwt(&access_claims)
    }

    /// Revokes all tokens for a user (used during logout)
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Tokens revoked successfully
    /// * `Err(TokenError)` - Revocation failed
    pub async fn revoke_tokens(&self, user_id: Uuid) -> Result<(), DomainError> {
        self.repository.revoke_all_user_tokens(user_id).await?;
        Ok(())
    }

    /// Revokes a specific refresh token
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token to revoke
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - Whether the token was found and revoked
    /// * `Err(TokenError)` - Revocation failed
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<bool, DomainError> {
        let token_hash = self.hash_token(refresh_token);
        self.repository.revoke_token(&token_hash).await
    }

    /// Cleans up expired tokens from the database
    ///
    /// This should be called periodically by a background job
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of tokens cleaned up
    /// * `Err(DomainError)` - Cleanup failed
    pub async fn cleanup_expired_tokens(&self) -> Result<usize, DomainError> {
        self.repository.delete_expired_tokens().await
    }

    /// Encodes claims into a JWT token
    fn encode_jwt(&self, claims: &Claims) -> Result<String, DomainError> {
        let header = Header::new(self.config.algorithm);
        encode(&header, claims, &self.encoding_key)
            .map_err(|_| TokenError::TokenGenerationFailed.into())
    }

    /// Hashes a token for secure storage
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::token_repository::mock::MockTokenRepository;
    use chrono::{Duration, Utc};

    fn create_test_service() -> TokenService<MockTokenRepository> {
        let repository = MockTokenRepository::new();
        let config = TokenServiceConfig::default();
        TokenService::new(repository, config)
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
}