//! Mock implementation of UserRepository for testing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::entities::user::{User, UserType};
use crate::errors::DomainError;

use super::trait_::UserRepository;

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

impl Default for MockUserRepository {
    fn default() -> Self {
        Self::new()
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