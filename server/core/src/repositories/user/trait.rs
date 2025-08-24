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