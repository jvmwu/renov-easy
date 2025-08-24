//! Tests for RS256 JWT token generation and validation

use uuid::Uuid;
use jsonwebtoken::Algorithm;

use crate::domain::entities::user::UserType;
use crate::services::token::{TokenService, TokenServiceConfig, Rs256KeyManager};
// Mock repository implementation
use std::sync::Arc;
use std::sync::Mutex;
use async_trait::async_trait;
use crate::domain::entities::token::RefreshToken;
use crate::errors::DomainError;
use crate::repositories::TokenRepository;

/// Mock implementation of TokenRepository for testing
pub struct MockTokenRepository {
    tokens: Arc<Mutex<Vec<RefreshToken>>>,
}

impl MockTokenRepository {
    fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TokenRepository for MockTokenRepository {
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.push(token.clone());
        Ok(token)
    }

    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens.iter().find(|t| t.token_hash == token_hash).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens.iter().find(|t| t.id == id).cloned())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, DomainError> {
        let tokens = self.tokens.lock().unwrap();
        Ok(tokens
            .iter()
            .filter(|t| t.user_id == user_id && t.is_valid())
            .cloned()
            .collect())
    }

    async fn revoke_token(&self, token_hash: &str) -> Result<bool, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.iter_mut().find(|t| t.token_hash == token_hash) {
            token.revoke();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<usize, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        let mut count = 0;
        for token in tokens.iter_mut().filter(|t| t.user_id == user_id) {
            if !token.is_revoked {
                token.revoke();
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete_expired_tokens(&self) -> Result<usize, DomainError> {
        let mut tokens = self.tokens.lock().unwrap();
        let before_count = tokens.len();
        tokens.retain(|t| !t.is_expired());
        Ok(before_count - tokens.len())
    }
}

/// Test RS256 key pair for testing
const TEST_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAyVR8ERJPASA6YK4z7KtRWLWiI7BBzDwrRy+vmzR+hIaLr+GF
c3/TnGdYNpqz+Dg8xfQYkVF1g8I1kq3ut+Sl9jCF9GrbvLQcu7dOr0xkF1mzXPWn
E7LaDjb/OqE9jmG1TYYkcnjJqVaDLEQGLvJs2VYijJRmH3bHQ3cOI+PkqsFLZmYe
EYFchm8zYvM6SDMEkjMhKJqiRq2rUWUGYy2n/aLMYuH+2q3T2M9MiB9gGI1qDj2Y
YP3oxnOkzqGvAVMXTVMcL+7Ss41OhNr4Nn7w4oquvM8TjLLFaVQmIMz8w+sPCB6K
n5WIhZhpkPdpMBWlKqsJQSiWCDdZCCxIcUMHtwIDAQABAoIBAAqOq3yFpI3HbF8/
W+8dVlbdQq+j3vSvmKBP9VQ7a8xBCnBKDiHrd8Q9V0aaCe8SmwEF2q+mvtnzHgl8
n4K0nYEz6MtLnWAw3sgz0ZQOT+XkD5L/GJOqHvDz3GXdAE+fqDsjRnlPQhzpYQmP
iHLLO4ooY8vDlehZ4d3gBwJNDFvF5MUZuFFy7hPbUqjI6qUboVlrfvzQWhlG1eTm
A4lF/I9pv/p7bXXDzzMcKqkE8NF2lBr0R8zbR6FCKqR0FK0c9xHj1RvT/u7HYTN/
ErmRQK0L8BRJnu4JKD8R0YL0vMNLNx7D5LjXjjbBvVfJX96CcRwHF8I9THBNjJPV
TXqbJQECgYEA5sRckRT2Cu/n8LF5vOznHsp5vI/u8TKqxJJPVWEiZMcM8k2n7H8b
9VJkzuMJKw7/gC5z7jlNCCqN+fkSkJQxGJPmzOqnHoLSeMLSNrLRGNHDmluFgvAC
+8TyoCHs4w6qnzcolUPSHhYSLfrgJL9gz7RRP9vGTZVXOe7hlt6VpLcCgYEA3/VF
8qy/cFZUegUxL9TL8aKiZsdSgvLwigf4eBgqvV3EApAuoN6x8wcsvhssdPGfRtzp
XxF8QIjjqAkQvqOfc6f9EQm8K7+kkVHfu0gvn2ijwM3w0ScCbrF7oMv/hB0TjOsC
AUVjJAxkOvcHA0sD3qCBsdWzKW8Jr3K6sLqgSgECgYEAiR2nt0TFK7kKV3Ce5b+I
azdM0OnyGq6FbCIulKQPw6jn5lPzhQTfKNpCD0a5qpz/6UY4Yfp0bZ0riPR1TCUK
gU8fWHkGLxY2qwOpH5M1XfpYKbKGqQq1U1OXCfH72q7VhXhH4hj5bgH4H8Qr1bFc
WGqkQoxeYkuXP8O4OLqKVt0CgYBN5RTX5vl0tYW4RhpA5glQu3KlHBq4G5dWVgAc
FrVKyi7LhAGDgrdlpQVH5aIE1K+1e2ocRF7h9dFWEz5qY5hQl7Kc+SzDkQJGlJcq
AozP0BuTTomgKpJhs0HrqN3OqYXtre8WmZ5VJCsmjj6ifYHqUhBbo9NrfIILbnu5
gQKzAQKBgDYq3Uqb7L3mtsKniG0xqiE7qBKVMqd+3TnF6H0XdDbVXxP7xr5wEW3z
G0jJRfBE8qu2w+x+qAd3a5YiJQPRJ3xGYdHlmt+9u+rvJNMb7iNEqDiXjkQcS/fs
y2R/KIPyYPkufbq5Pe8Yz8EcvGZLH+bB1tXlB6vdZbSg9KvmCmye
-----END RSA PRIVATE KEY-----"#;

const TEST_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyVR8ERJPASA6YK4z7KtR
WLWiI7BBzDwrRy+vmzR+hIaLr+GFc3/TnGdYNpqz+Dg8xfQYkVF1g8I1kq3ut+Sl
9jCF9GrbvLQcu7dOr0xkF1mzXPWnE7LaDjb/OqE9jmG1TYYkcnjJqVaDLEQGLvJs
2VYijJRmH3bHQ3cOI+PkqsFLZmYeEYFchm8zYvM6SDMEkjMhKJqiRq2rUWUGYy2n
/aLMYuH+2q3T2M9MiB9gGI1qDj2YYP3oxnOkzqGvAVMXTVMcL+7Ss41OhNr4Nn7w
4oquvM8TjLLFaVQmIMz8w+sPCB6Kn5WIhZhpkPdpMBWlKqsJQSiWCDdZCCxIcUMH
twIDAQAB
-----END PUBLIC KEY-----"#;

#[tokio::test]
async fn test_rs256_token_generation() {
    // Create RS256 key manager from test keys
    let key_manager = Rs256KeyManager::from_pem_strings(TEST_PRIVATE_KEY, TEST_PUBLIC_KEY)
        .expect("Failed to create key manager");

    // Create token service with RS256
    let repository = MockTokenRepository::new();
    let config = TokenServiceConfig {
        jwt_secret: "not-used-for-rs256".to_string(),
        algorithm: Algorithm::RS256,
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        rs256_config: None, // Not needed when using with_rs256_keys
    };

    let service = TokenService::with_rs256_keys(repository, config, key_manager);

    // Generate tokens
    let user_id = Uuid::new_v4();
    let user_type = Some(UserType::Customer);
    let token_pair = service
        .generate_tokens(user_id, user_type.clone(), true)
        .await
        .expect("Failed to generate tokens");

    // Verify we got tokens
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.access_expires_in, 15 * 60);
    assert_eq!(token_pair.refresh_expires_in, 7 * 24 * 60 * 60);
}

#[tokio::test]
async fn test_rs256_token_verification() {
    // Create RS256 key manager from test keys
    let key_manager = Rs256KeyManager::from_pem_strings(TEST_PRIVATE_KEY, TEST_PUBLIC_KEY)
        .expect("Failed to create key manager");

    // Create token service with RS256
    let repository = MockTokenRepository::new();
    let config = TokenServiceConfig {
        jwt_secret: "not-used-for-rs256".to_string(),
        algorithm: Algorithm::RS256,
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        rs256_config: None,
    };

    let service = TokenService::with_rs256_keys(repository, config, key_manager);

    // Generate tokens
    let user_id = Uuid::new_v4();
    let user_type = Some(UserType::Worker);
    let token_pair = service
        .generate_tokens(user_id, user_type.clone(), true)
        .await
        .expect("Failed to generate tokens");

    // Verify the access token
    let claims = service
        .verify_access_token(&token_pair.access_token)
        .expect("Failed to verify access token");

    // Check claims
    assert_eq!(claims.user_id().unwrap(), user_id);
    assert_eq!(claims.user_type, Some("worker".to_string()));
    assert!(claims.is_verified);
    assert!(claims.is_valid());
}

#[tokio::test]
async fn test_rs256_invalid_token_rejection() {
    // Create RS256 key manager from test keys
    let key_manager = Rs256KeyManager::from_pem_strings(TEST_PRIVATE_KEY, TEST_PUBLIC_KEY)
        .expect("Failed to create key manager");

    // Create token service with RS256
    let repository = MockTokenRepository::new();
    let config = TokenServiceConfig {
        jwt_secret: "not-used-for-rs256".to_string(),
        algorithm: Algorithm::RS256,
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        rs256_config: None,
    };

    let service = TokenService::with_rs256_keys(repository, config, key_manager);

    // Try to verify an invalid token
    let invalid_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature";
    let result = service.verify_access_token(invalid_token);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_rs256_key_manager_from_env() {
    // Set environment variables for testing
    std::env::set_var("JWT_PRIVATE_KEY_PATH", "core/keys/jwt_private_key.pem");
    std::env::set_var("JWT_PUBLIC_KEY_PATH", "core/keys/jwt_public_key.pem");

    // Try to load from environment (will fail if keys don't exist)
    let result = Rs256KeyManager::from_env();

    // Check if keys exist
    if std::path::Path::new("core/keys/jwt_private_key.pem").exists() {
        assert!(result.is_ok());
        let manager = result.unwrap();
        assert!(manager.validate());
    } else {
        // Keys don't exist in test environment, which is expected
        assert!(result.is_err());
    }

    // Clean up environment variables
    std::env::remove_var("JWT_PRIVATE_KEY_PATH");
    std::env::remove_var("JWT_PUBLIC_KEY_PATH");
}

#[test]
fn test_rs256_claims_structure() {
    use crate::domain::entities::token::Claims;

    let user_id = Uuid::new_v4();
    let claims = Claims::new_access_token(
        user_id,
        Some("customer".to_string()),
        true,
    );

    // Verify claims structure
    assert_eq!(claims.iss, "renov-easy");
    assert_eq!(claims.aud, "renov-easy-api");
    assert_eq!(claims.user_type, Some("customer".to_string()));
    assert!(claims.is_verified);
    assert!(!claims.jti.is_empty());
    assert!(claims.is_valid());
}
