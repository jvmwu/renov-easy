//! Unit tests for database connection pool

use shared::config::database::DatabaseConfig;
use crate::database::connection::{DatabasePool, PoolStatistics};

#[tokio::test]
async fn test_pool_creation_with_invalid_url() {
    let config = DatabaseConfig::new("invalid://url")
        .with_max_connections(10);

    let result = DatabasePool::new(config).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires actual database
async fn test_pool_health_check() {
    let config = DatabaseConfig::new(
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/renovesy_test".to_string())
    ).with_max_connections(5);

    let pool = DatabasePool::new(config).await.unwrap();
    let health = pool.health_check().await.unwrap();
    assert!(health);
}

#[tokio::test]
#[ignore] // Requires actual database
async fn test_pool_get_connection() {
    let config = DatabaseConfig::new(
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/renovesy_test".to_string())
    ).with_max_connections(5);

    let pool = DatabasePool::new(config).await.unwrap();
    // Pool is ready if creation succeeded
    assert!(pool.health_check().await.is_ok());
}

#[test]
fn test_pool_statistics_display() {
    let stats = PoolStatistics {
        connections: 5,
        idle_connections: 3,
        max_connections: 10,
    };

    let display = format!("{}", stats);
    assert!(display.contains("5/10"));
    assert!(display.contains("3 idle"));
}