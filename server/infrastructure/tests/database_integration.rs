//! Integration tests for database repositories

use renov_infrastructure::database::mysql::{MySqlUserRepository, MySqlTokenRepository};
use renov_infrastructure::database::DatabasePool;
use renov_infrastructure::config::DatabaseConfig;
use renov_core::repositories::user::UserRepository;
use renov_core::repositories::token::TokenRepository;
use renov_core::domain::entities::user::User;
use renov_core::domain::entities::token::RefreshToken;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
#[ignore] // Requires actual database
async fn test_user_repository_operations() {
    let config = DatabaseConfig {
        url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/renovesy_test".to_string()),
        max_connections: 5,
        connect_timeout: 10,
    };

    let pool = DatabasePool::new(config).await.unwrap();
    let repo = MySqlUserRepository::new(pool.get_pool().clone());
    
    // Create a test user
    let user = User::new(
        "test_phone_hash".to_string(),
        "+1".to_string(),
    );
    
    // Test create
    let created = repo.create(user.clone()).await.unwrap();
    assert_eq!(created.phone_hash, user.phone_hash);
    
    // Test find by id
    let found = repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);
    
    // Cleanup - delete the test user
    sqlx::query!("DELETE FROM users WHERE id = ?", created.id.to_string())
        .execute(pool.get_pool())
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // Requires actual database
async fn test_token_repository_operations() {
    let config = DatabaseConfig {
        url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/renovesy_test".to_string()),
        max_connections: 5,
        connect_timeout: 10,
    };

    let pool = DatabasePool::new(config).await.unwrap();
    let repo = MySqlTokenRepository::new(pool.get_pool().clone());
    
    let user_id = Uuid::new_v4();
    let token = RefreshToken::new(user_id, "test_token_hash".to_string());
    
    // Test create
    let created = repo.create(token.clone()).await.unwrap();
    assert_eq!(created.user_id, user_id);
    
    // Test find by token hash
    let found = repo.find_by_token_hash("test_token_hash").await.unwrap();
    assert!(found.is_some());
    
    // Cleanup
    sqlx::query!("DELETE FROM refresh_tokens WHERE id = ?", created.id.to_string())
        .execute(pool.get_pool())
        .await
        .unwrap();
}