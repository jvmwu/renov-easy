//! User repository trait defining the interface for user data persistence.
//!
//! This module defines the repository pattern interface for User entities,
//! following Domain-Driven Design principles. The trait is async-first and
//! uses Result types for proper error handling.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::user::{User, UserType};
use crate::errors::DomainError;

/// Repository trait for User entity persistence operations
///
/// This trait defines the contract for data access operations related to users.
/// Implementations of this trait should handle the actual database operations
/// while maintaining the abstraction boundary between domain and infrastructure layers.
///
/// # Example Implementation
/// ```no_run
/// use async_trait::async_trait;
/// use uuid::Uuid;
/// use renov_core::repositories::UserRepository;
/// use renov_core::domain::entities::user::{User, UserType};
/// use renov_core::errors::DomainError;
///
/// struct MySqlUserRepository {
///     // database connection pool
/// }
///
/// #[async_trait]
/// impl UserRepository for MySqlUserRepository {
///     async fn find_by_phone(
///         &self,
///         phone_hash: &str,
///         country_code: &str
///     ) -> Result<Option<User>, DomainError> {
///         // Implementation here
///         Ok(None)
///     }
///     
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their phone number hash and country code
    ///
    /// # Arguments
    /// * `phone_hash` - SHA-256 hash of the phone number
    /// * `country_code` - International country code (e.g., "+86", "+61")
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - No user found with given phone
    /// * `Err(DomainError)` - Database or other error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::UserRepository;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let phone_hash = "sha256_hash_of_phone";
    /// let country_code = "+61";
    /// 
    /// match repo.find_by_phone(phone_hash, country_code).await? {
    ///     Some(user) => println!("User found: {:?}", user.id),
    ///     None => println!("User not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn find_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<Option<User>, DomainError>;

    /// Find a user by their unique identifier
    ///
    /// # Arguments
    /// * `id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - No user found with given ID
    /// * `Err(DomainError)` - Database or other error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::UserRepository;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// 
    /// if let Some(user) = repo.find_by_id(user_id).await? {
    ///     println!("User type: {:?}", user.user_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;

    /// Create a new user in the repository
    ///
    /// # Arguments
    /// * `user` - The User entity to persist
    ///
    /// # Returns
    /// * `Ok(User)` - The created user with any database-generated fields
    /// * `Err(DomainError)` - Creation failed (e.g., duplicate phone number)
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::UserRepository;
    /// # use renov_core::domain::entities::user::User;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let new_user = User::new(
    ///     "sha256_hash_of_phone".to_string(),
    ///     "+61".to_string(),
    /// );
    /// 
    /// let created_user = repo.create(new_user).await?;
    /// println!("Created user with ID: {}", created_user.id);
    /// # Ok(())
    /// # }
    /// ```
    async fn create(&self, user: User) -> Result<User, DomainError>;

    /// Update an existing user in the repository
    ///
    /// # Arguments
    /// * `user` - The User entity with updated fields
    ///
    /// # Returns
    /// * `Ok(User)` - The updated user
    /// * `Err(DomainError)` - Update failed (e.g., user not found, validation error)
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::UserRepository;
    /// # use renov_core::domain::entities::user::UserType;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// 
    /// if let Some(mut user) = repo.find_by_id(user_id).await? {
    ///     user.set_user_type(UserType::Customer);
    ///     user.verify();
    ///     
    ///     let updated_user = repo.update(user).await?;
    ///     println!("User updated at: {}", updated_user.updated_at);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn update(&self, user: User) -> Result<User, DomainError>;

    /// Delete a user from the repository
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to delete
    ///
    /// # Returns
    /// * `Ok(true)` - User was deleted
    /// * `Ok(false)` - User not found
    /// * `Err(DomainError)` - Deletion failed
    ///
    /// # Example
    /// ```no_run
    /// # use uuid::Uuid;
    /// # use renov_core::repositories::UserRepository;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    /// 
    /// if repo.delete(user_id).await? {
    ///     println!("User deleted successfully");
    /// } else {
    ///     println!("User not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn delete(&self, id: Uuid) -> Result<bool, DomainError>;

    /// Check if a user exists with the given phone number
    ///
    /// # Arguments
    /// * `phone_hash` - SHA-256 hash of the phone number
    /// * `country_code` - International country code
    ///
    /// # Returns
    /// * `Ok(true)` - User exists
    /// * `Ok(false)` - User does not exist
    /// * `Err(DomainError)` - Database error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::UserRepository;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let phone_hash = "sha256_hash_of_phone";
    /// let country_code = "+61";
    /// 
    /// if repo.exists_by_phone(phone_hash, country_code).await? {
    ///     println!("Phone number already registered");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn exists_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<bool, DomainError>;

    /// Count total users by type
    ///
    /// # Arguments
    /// * `user_type` - Optional filter by user type (None counts all users)
    ///
    /// # Returns
    /// * `Ok(count)` - Number of users matching the criteria
    /// * `Err(DomainError)` - Database error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use renov_core::repositories::UserRepository;
    /// # use renov_core::domain::entities::user::UserType;
    /// # async fn example(repo: &impl UserRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let total_users = repo.count_by_type(None).await?;
    /// let customers = repo.count_by_type(Some(UserType::Customer)).await?;
    /// let workers = repo.count_by_type(Some(UserType::Worker)).await?;
    /// 
    /// println!("Total: {}, Customers: {}, Workers: {}", 
    ///          total_users, customers, workers);
    /// # Ok(())
    /// # }
    /// ```
    async fn count_by_type(&self, user_type: Option<UserType>) -> Result<u64, DomainError>;
}

/// Mock implementation of UserRepository for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock user repository for testing
    pub struct MockUserRepository {
        users: Arc<RwLock<HashMap<Uuid, User>>>,
    }

    impl MockUserRepository {
        /// Create a new mock repository
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_phone(
            &self,
            phone_hash: &str,
            country_code: &str,
        ) -> Result<Option<User>, DomainError> {
            let users = self.users.read().await;
            Ok(users
                .values()
                .find(|u| u.phone_hash == phone_hash && u.country_code == country_code)
                .cloned())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
            let users = self.users.read().await;
            Ok(users.get(&id).cloned())
        }

        async fn create(&self, user: User) -> Result<User, DomainError> {
            let mut users = self.users.write().await;
            
            // Check for duplicate phone
            if users.values().any(|u| {
                u.phone_hash == user.phone_hash && u.country_code == user.country_code
            }) {
                return Err(DomainError::Validation {
                    message: "Phone number already registered".to_string(),
                });
            }
            
            users.insert(user.id, user.clone());
            Ok(user)
        }

        async fn update(&self, user: User) -> Result<User, DomainError> {
            let mut users = self.users.write().await;
            
            if !users.contains_key(&user.id) {
                return Err(DomainError::NotFound {
                    resource: "User".to_string(),
                });
            }
            
            users.insert(user.id, user.clone());
            Ok(user)
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
            let mut users = self.users.write().await;
            Ok(users.remove(&id).is_some())
        }

        async fn exists_by_phone(
            &self,
            phone_hash: &str,
            country_code: &str,
        ) -> Result<bool, DomainError> {
            let users = self.users.read().await;
            Ok(users
                .values()
                .any(|u| u.phone_hash == phone_hash && u.country_code == country_code))
        }

        async fn count_by_type(&self, user_type: Option<UserType>) -> Result<u64, DomainError> {
            let users = self.users.read().await;
            let count = match user_type {
                Some(ut) => users.values().filter(|u| u.user_type == Some(ut)).count(),
                None => users.len(),
            };
            Ok(count as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::user::User;

    #[tokio::test]
    async fn test_mock_repository_create_and_find() {
        let repo = mock::MockUserRepository::new();
        
        let user = User::new(
            "test_hash".to_string(),
            "+61".to_string(),
        );
        
        let created = repo.create(user.clone()).await.unwrap();
        assert_eq!(created.id, user.id);
        
        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_mock_repository_find_by_phone() {
        let repo = mock::MockUserRepository::new();
        
        let user = User::new(
            "phone_hash_123".to_string(),
            "+86".to_string(),
        );
        
        repo.create(user.clone()).await.unwrap();
        
        let found = repo
            .find_by_phone("phone_hash_123", "+86")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_mock_repository_duplicate_phone() {
        let repo = mock::MockUserRepository::new();
        
        let user1 = User::new(
            "same_hash".to_string(),
            "+61".to_string(),
        );
        
        let user2 = User::new(
            "same_hash".to_string(),
            "+61".to_string(),
        );
        
        repo.create(user1).await.unwrap();
        let result = repo.create(user2).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_mock_repository_update() {
        let repo = mock::MockUserRepository::new();
        
        let mut user = User::new(
            "test_hash".to_string(),
            "+61".to_string(),
        );
        
        repo.create(user.clone()).await.unwrap();
        
        user.set_user_type(UserType::Customer);
        user.verify();
        
        let updated = repo.update(user.clone()).await.unwrap();
        assert_eq!(updated.user_type, Some(UserType::Customer));
        assert!(updated.is_verified);
    }

    #[tokio::test]
    async fn test_mock_repository_delete() {
        let repo = mock::MockUserRepository::new();
        
        let user = User::new(
            "test_hash".to_string(),
            "+61".to_string(),
        );
        
        repo.create(user.clone()).await.unwrap();
        
        let deleted = repo.delete(user.id).await.unwrap();
        assert!(deleted);
        
        let found = repo.find_by_id(user.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_mock_repository_count_by_type() {
        let repo = mock::MockUserRepository::new();
        
        let mut customer = User::new("customer_hash".to_string(), "+61".to_string());
        customer.set_user_type(UserType::Customer);
        
        let mut worker = User::new("worker_hash".to_string(), "+86".to_string());
        worker.set_user_type(UserType::Worker);
        
        let untyped = User::new("untyped_hash".to_string(), "+1".to_string());
        
        repo.create(customer).await.unwrap();
        repo.create(worker).await.unwrap();
        repo.create(untyped).await.unwrap();
        
        assert_eq!(repo.count_by_type(None).await.unwrap(), 3);
        assert_eq!(
            repo.count_by_type(Some(UserType::Customer)).await.unwrap(),
            1
        );
        assert_eq!(
            repo.count_by_type(Some(UserType::Worker)).await.unwrap(),
            1
        );
    }
}