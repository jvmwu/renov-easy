//! MySQL implementation of the UserRepository trait.
//!
//! This module provides the concrete implementation of user data persistence
//! using MySQL database with SQLx. It handles all database operations including
//! phone number hashing for security.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

use renov_core::domain::entities::user::{User, UserType};
use renov_core::errors::DomainError;
use renov_core::repositories::UserRepository;

/// MySQL implementation of UserRepository
///
/// This implementation uses SQLx for database operations and SHA-256
/// for secure phone number hashing.
pub struct MySqlUserRepository {
    /// Database connection pool
    pool: MySqlPool,
}

impl MySqlUserRepository {
    /// Create a new MySQL user repository
    ///
    /// # Arguments
    /// * `pool` - MySQL connection pool from SQLx
    ///
    /// # Returns
    /// A new instance of MySqlUserRepository
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Hash a phone number using SHA-256
    ///
    /// # Arguments
    /// * `phone` - Raw phone number to hash
    ///
    /// # Returns
    /// Hexadecimal string representation of the SHA-256 hash
    fn hash_phone(phone: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(phone.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Convert database row to User entity
    ///
    /// Maps database columns to User struct fields
    fn row_to_user(row: &sqlx::mysql::MySqlRow) -> Result<User, DomainError> {
        let id: String = row.try_get("id")
            .map_err(|e| DomainError::Database(format!("Failed to get id: {}", e)))?;
        
        let user_type_str: Option<String> = row.try_get("user_type")
            .map_err(|e| DomainError::Database(format!("Failed to get user_type: {}", e)))?;
        
        let user_type = user_type_str.map(|s| match s.as_str() {
            "customer" => UserType::Customer,
            "worker" => UserType::Worker,
            _ => UserType::Customer, // Default fallback
        });

        Ok(User {
            id: Uuid::parse_str(&id)
                .map_err(|e| DomainError::Database(format!("Invalid UUID: {}", e)))?,
            phone_hash: row.try_get("phone_hash")
                .map_err(|e| DomainError::Database(format!("Failed to get phone_hash: {}", e)))?,
            country_code: row.try_get("country_code")
                .map_err(|e| DomainError::Database(format!("Failed to get country_code: {}", e)))?,
            user_type,
            created_at: row.try_get::<DateTime<Utc>, _>("created_at")
                .map_err(|e| DomainError::Database(format!("Failed to get created_at: {}", e)))?,
            updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")
                .map_err(|e| DomainError::Database(format!("Failed to get updated_at: {}", e)))?,
            last_login_at: row.try_get("last_login_at")
                .map_err(|e| DomainError::Database(format!("Failed to get last_login_at: {}", e)))?,
            is_verified: row.try_get("is_verified")
                .map_err(|e| DomainError::Database(format!("Failed to get is_verified: {}", e)))?,
            is_blocked: row.try_get("is_blocked")
                .map_err(|e| DomainError::Database(format!("Failed to get is_blocked: {}", e)))?,
        })
    }
}

#[async_trait]
impl UserRepository for MySqlUserRepository {
    async fn find_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<Option<User>, DomainError> {
        let query = r#"
            SELECT id, phone_hash, country_code, user_type, 
                   created_at, updated_at, last_login_at, 
                   is_verified, is_blocked
            FROM users
            WHERE phone_hash = ? AND country_code = ?
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .bind(phone_hash)
            .bind(country_code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Database query failed: {}", e)))?;

        match result {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let query = r#"
            SELECT id, phone_hash, country_code, user_type,
                   created_at, updated_at, last_login_at,
                   is_verified, is_blocked
            FROM users
            WHERE id = ?
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Database query failed: {}", e)))?;

        match result {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, user: User) -> Result<User, DomainError> {
        // Check for duplicate phone first
        if self.exists_by_phone(&user.phone_hash, &user.country_code).await? {
            return Err(DomainError::Validation(
                "Phone number already registered".to_string(),
            ));
        }

        let user_type_str = user.user_type.map(|ut| match ut {
            UserType::Customer => "customer",
            UserType::Worker => "worker",
        });

        let query = r#"
            INSERT INTO users (
                id, phone_hash, country_code, user_type,
                created_at, updated_at, last_login_at,
                is_verified, is_blocked
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(user.id.to_string())
            .bind(&user.phone_hash)
            .bind(&user.country_code)
            .bind(user_type_str)
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.last_login_at)
            .bind(user.is_verified)
            .bind(user.is_blocked)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Failed to create user: {}", e)))?;

        Ok(user)
    }

    async fn update(&self, user: User) -> Result<User, DomainError> {
        let user_type_str = user.user_type.map(|ut| match ut {
            UserType::Customer => "customer",
            UserType::Worker => "worker",
        });

        let query = r#"
            UPDATE users SET
                phone_hash = ?,
                country_code = ?,
                user_type = ?,
                updated_at = ?,
                last_login_at = ?,
                is_verified = ?,
                is_blocked = ?
            WHERE id = ?
        "#;

        let result = sqlx::query(query)
            .bind(&user.phone_hash)
            .bind(&user.country_code)
            .bind(user_type_str)
            .bind(Utc::now()) // Always update the timestamp
            .bind(user.last_login_at)
            .bind(user.is_verified)
            .bind(user.is_blocked)
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Failed to update user: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound("User not found".to_string()));
        }

        // Return the updated user with new timestamp
        let mut updated_user = user;
        updated_user.updated_at = Utc::now();
        Ok(updated_user)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
        let query = "DELETE FROM users WHERE id = ?";

        let result = sqlx::query(query)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Failed to delete user: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<bool, DomainError> {
        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE phone_hash = ? AND country_code = ?
            ) as user_exists
        "#;

        let result = sqlx::query(query)
            .bind(phone_hash)
            .bind(country_code)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(format!("Failed to check user existence: {}", e)))?;

        let exists: i8 = result.try_get("user_exists")
            .map_err(|e| DomainError::Database(format!("Failed to get existence result: {}", e)))?;

        Ok(exists == 1)
    }

    async fn count_by_type(&self, user_type: Option<UserType>) -> Result<u64, DomainError> {
        let query = match user_type {
            Some(_) => {
                r#"
                SELECT COUNT(*) as count
                FROM users
                WHERE user_type = ?
                "#
            }
            None => {
                r#"
                SELECT COUNT(*) as count
                FROM users
                "#
            }
        };

        let result = if let Some(ut) = user_type {
            let user_type_str = match ut {
                UserType::Customer => "customer",
                UserType::Worker => "worker",
            };
            sqlx::query(query)
                .bind(user_type_str)
                .fetch_one(&self.pool)
                .await
        } else {
            sqlx::query(query)
                .fetch_one(&self.pool)
                .await
        };

        let row = result
            .map_err(|e| DomainError::Database(format!("Failed to count users: {}", e)))?;

        let count: i64 = row.try_get("count")
            .map_err(|e| DomainError::Database(format!("Failed to get count: {}", e)))?;

        Ok(count as u64)
    }
}

/// Helper functions for phone number processing
impl MySqlUserRepository {
    /// Create a new user with phone number hashing
    ///
    /// This method accepts a raw phone number and handles the hashing internally.
    ///
    /// # Arguments
    /// * `phone` - Raw phone number (not hashed)
    /// * `country_code` - Country code
    ///
    /// # Returns
    /// Created user with hashed phone number
    pub async fn create_with_phone(
        &self,
        phone: &str,
        country_code: String,
    ) -> Result<User, DomainError> {
        let phone_hash = Self::hash_phone(phone);
        let user = User::new(phone_hash, country_code);
        self.create(user).await
    }

    /// Find a user by raw phone number
    ///
    /// This method accepts a raw phone number and handles the hashing internally.
    ///
    /// # Arguments
    /// * `phone` - Raw phone number (not hashed)
    /// * `country_code` - Country code
    ///
    /// # Returns
    /// User if found, None otherwise
    pub async fn find_by_raw_phone(
        &self,
        phone: &str,
        country_code: &str,
    ) -> Result<Option<User>, DomainError> {
        let phone_hash = Self::hash_phone(phone);
        self.find_by_phone(&phone_hash, country_code).await
    }

    /// Check if a user exists by raw phone number
    ///
    /// # Arguments
    /// * `phone` - Raw phone number (not hashed)
    /// * `country_code` - Country code
    ///
    /// # Returns
    /// true if user exists, false otherwise
    pub async fn exists_by_raw_phone(
        &self,
        phone: &str,
        country_code: &str,
    ) -> Result<bool, DomainError> {
        let phone_hash = Self::hash_phone(phone);
        self.exists_by_phone(&phone_hash, country_code).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_hashing() {
        let phone1 = "1234567890";
        let phone2 = "0987654321";
        let phone1_duplicate = "1234567890";

        let hash1 = MySqlUserRepository::hash_phone(phone1);
        let hash2 = MySqlUserRepository::hash_phone(phone2);
        let hash1_dup = MySqlUserRepository::hash_phone(phone1_duplicate);

        // Same input should produce same hash
        assert_eq!(hash1, hash1_dup);
        
        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
        
        // Hash should be 64 characters (SHA-256 in hex)
        assert_eq!(hash1.len(), 64);
        assert_eq!(hash2.len(), 64);
    }

    #[test]
    fn test_phone_hash_security() {
        let phone = "0412345678";
        let hash = MySqlUserRepository::hash_phone(phone);
        
        // Hash should not contain the original phone number
        assert!(!hash.contains(phone));
        
        // Hash should be irreversible (we can't test this directly,
        // but we can verify it's a proper hex string)
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}