//! Mock implementation of TokenRepository for testing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::entities::token::RefreshToken;
use crate::errors::DomainError;

use super::r#trait::TokenRepository;

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

impl Default for MockTokenRepository {
    fn default() -> Self {
        Self::new()
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