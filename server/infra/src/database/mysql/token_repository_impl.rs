//! MySQL implementation of the TokenRepository trait.
//!
//! This module provides the concrete implementation of refresh token persistence
//! using MySQL database with SQLx. It handles token storage, retrieval, validation,
//! and revocation for JWT refresh tokens.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

use renov_core::domain::entities::token::RefreshToken;
use renov_core::errors::DomainError;
use renov_core::repositories::TokenRepository;

/// MySQL implementation of TokenRepository
///
/// This implementation uses SQLx for database operations and SHA-256
/// for secure token hashing before storage.
pub struct MySqlTokenRepository {
    /// Database connection pool
    pool: MySqlPool,
}

impl MySqlTokenRepository {
    /// Create a new MySQL token repository
    ///
    /// # Arguments
    /// * `pool` - MySQL connection pool from SQLx
    ///
    /// # Returns
    /// A new instance of MySqlTokenRepository
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Hash a token value using SHA-256
    ///
    /// # Arguments
    /// * `token` - Raw token value to hash
    ///
    /// # Returns
    /// Hexadecimal string representation of the SHA-256 hash
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Convert database row to RefreshToken entity
    ///
    /// Maps database columns to RefreshToken struct fields
    fn row_to_token(row: &sqlx::mysql::MySqlRow) -> Result<RefreshToken, DomainError> {
        let id: String = row.try_get("id")
            .map_err(|e| DomainError::Internal { message: format!("Failed to get id: {}", e) })?;
        
        let user_id: String = row.try_get("user_id")
            .map_err(|e| DomainError::Internal { message: format!("Failed to get user_id: {}", e) })?;

        Ok(RefreshToken {
            id: Uuid::parse_str(&id)
                .map_err(|e| DomainError::Internal { message: format!("Invalid token UUID: {}", e) })?,
            user_id: Uuid::parse_str(&user_id)
                .map_err(|e| DomainError::Internal { message: format!("Invalid user UUID: {}", e) })?,
            token_hash: row.try_get("token_hash")
                .map_err(|e| DomainError::Internal { message: format!("Failed to get token_hash: {}", e) })?,
            created_at: row.try_get::<DateTime<Utc>, _>("created_at")
                .map_err(|e| DomainError::Internal { message: format!("Failed to get created_at: {}", e) })?,
            expires_at: row.try_get::<DateTime<Utc>, _>("expires_at")
                .map_err(|e| DomainError::Internal { message: format!("Failed to get expires_at: {}", e) })?,
            is_revoked: row.try_get("is_revoked")
                .map_err(|e| DomainError::Internal { message: format!("Failed to get is_revoked: {}", e) })?,
        })
    }
}

#[async_trait]
impl TokenRepository for MySqlTokenRepository {
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError> {
        // Check for duplicate token hash first
        let check_query = "SELECT EXISTS(SELECT 1 FROM refresh_tokens WHERE token_hash = ?) as exists";
        let exists_row = sqlx::query(check_query)
            .bind(&token.token_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to check token existence: {}", e) })?;
        
        let exists: i8 = exists_row.try_get("exists")
            .map_err(|e| DomainError::Internal { message: format!("Failed to get existence result: {}", e) })?;
        
        if exists == 1 {
            return Err(DomainError::Validation { message: "Token already exists".to_string() });
        }

        let query = r#"
            INSERT INTO refresh_tokens (
                id, user_id, token_hash, created_at, expires_at, is_revoked
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(token.id.to_string())
            .bind(token.user_id.to_string())
            .bind(&token.token_hash)
            .bind(token.created_at)
            .bind(token.expires_at)
            .bind(token.is_revoked)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to save refresh token: {}", e) })?;

        Ok(token)
    }

    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError> {
        let query = r#"
            SELECT id, user_id, token_hash, created_at, expires_at, is_revoked
            FROM refresh_tokens
            WHERE token_hash = ?
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to find refresh token: {}", e) })?;

        match result {
            Some(row) => Ok(Some(Self::row_to_token(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, DomainError> {
        let query = r#"
            SELECT id, user_id, token_hash, created_at, expires_at, is_revoked
            FROM refresh_tokens
            WHERE id = ?
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to find token by id: {}", e) })?;

        match result {
            Some(row) => Ok(Some(Self::row_to_token(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError> {
        let query = r#"
            SELECT id, user_id, token_hash, created_at, expires_at, is_revoked
            FROM refresh_tokens
            WHERE user_id = ? 
                AND is_revoked = FALSE 
                AND expires_at > ?
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(Utc::now())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to find user tokens: {}", e) })?;

        let mut tokens = Vec::new();
        for row in rows {
            tokens.push(Self::row_to_token(&row)?);
        }

        Ok(tokens)
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<bool, DomainError> {
        let query = r#"
            UPDATE refresh_tokens 
            SET is_revoked = TRUE 
            WHERE token_hash = ? AND is_revoked = FALSE
        "#;

        let result = sqlx::query(query)
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to revoke token: {}", e) })?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
        let query = r#"
            UPDATE refresh_tokens 
            SET is_revoked = TRUE 
            WHERE user_id = ? AND is_revoked = FALSE
        "#;

        let result = sqlx::query(query)
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to revoke user tokens: {}", e) })?;

        Ok(result.rows_affected() as usize)
    }

    async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
        let query = r#"
            DELETE FROM refresh_tokens 
            WHERE expires_at < ? OR (is_revoked = TRUE AND created_at < DATE_SUB(?, INTERVAL 30 DAY))
        "#;

        let now = Utc::now();
        let result = sqlx::query(query)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal { message: format!("Failed to delete expired tokens: {}", e) })?;

        Ok(result.rows_affected() as usize)
    }
}

/// Helper functions for token processing
impl MySqlTokenRepository {
    /// Save a refresh token with raw token value
    ///
    /// This method accepts a raw token value and handles the hashing internally.
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `raw_token` - Raw token value (not hashed)
    ///
    /// # Returns
    /// Saved RefreshToken with hashed token value
    pub async fn save_with_raw_token(
        &self,
        user_id: Uuid,
        raw_token: &str,
    ) -> Result<RefreshToken, DomainError> {
        let token_hash = Self::hash_token(raw_token);
        let token = RefreshToken::new(user_id, token_hash);
        self.save_refresh_token(token).await
    }

    /// Find a refresh token by raw token value
    ///
    /// This method accepts a raw token value and handles the hashing internally.
    ///
    /// # Arguments
    /// * `raw_token` - Raw token value (not hashed)
    ///
    /// # Returns
    /// RefreshToken if found, None otherwise
    pub async fn find_by_raw_token(
        &self,
        raw_token: &str,
    ) -> Result<Option<RefreshToken>, DomainError> {
        let token_hash = Self::hash_token(raw_token);
        self.find_refresh_token(&token_hash).await
    }

    /// Revoke a token by raw token value
    ///
    /// # Arguments
    /// * `raw_token` - Raw token value (not hashed)
    ///
    /// # Returns
    /// true if token was revoked, false if not found
    pub async fn revoke_by_raw_token(&self, raw_token: &str) -> Result<bool, DomainError> {
        let token_hash = Self::hash_token(raw_token);
        self.revoke_token(&token_hash).await
    }

    /// Check if a raw token is valid
    ///
    /// # Arguments
    /// * `raw_token` - Raw token value (not hashed)
    ///
    /// # Returns
    /// true if token exists and is valid, false otherwise
    pub async fn is_raw_token_valid(&self, raw_token: &str) -> Result<bool, DomainError> {
        let token_hash = Self::hash_token(raw_token);
        self.is_token_valid(&token_hash).await
    }

    /// Clean up old tokens periodically
    ///
    /// This method should be called periodically (e.g., daily) to clean up:
    /// - Expired tokens
    /// - Revoked tokens older than 30 days
    ///
    /// # Returns
    /// Number of tokens deleted
    pub async fn cleanup_old_tokens(&self) -> Result<usize, DomainError> {
        self.delete_expired_tokens().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_hashing() {
        let token1 = "jwt_token_value_1";
        let token2 = "jwt_token_value_2";
        let token1_duplicate = "jwt_token_value_1";

        let hash1 = MySqlTokenRepository::hash_token(token1);
        let hash2 = MySqlTokenRepository::hash_token(token2);
        let hash1_dup = MySqlTokenRepository::hash_token(token1_duplicate);

        // Same input should produce same hash
        assert_eq!(hash1, hash1_dup);
        
        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
        
        // Hash should be 64 characters (SHA-256 in hex)
        assert_eq!(hash1.len(), 64);
        assert_eq!(hash2.len(), 64);
    }

    #[test]
    fn test_token_hash_security() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let hash = MySqlTokenRepository::hash_token(token);
        
        // Hash should not contain the original token
        assert!(!hash.contains(token));
        assert!(!hash.contains("JWT"));
        assert!(!hash.contains("eyJ"));
        
        // Hash should be irreversible (we can't test this directly,
        // but we can verify it's a proper hex string)
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_consistent_hashing() {
        let token = "test_refresh_token";
        
        // Hash the same token multiple times
        let hashes: Vec<String> = (0..10)
            .map(|_| MySqlTokenRepository::hash_token(token))
            .collect();
        
        // All hashes should be identical
        for hash in &hashes[1..] {
            assert_eq!(&hashes[0], hash);
        }
    }
}