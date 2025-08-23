//! Unit tests for Redis client

use crate::cache::redis_client::{RedisClient, mask_url, is_retriable_error};
use re_shared::config::cache::CacheConfig;
use redis::{RedisError, ErrorKind};

#[test]
fn test_mask_url() {
    assert_eq!(
        mask_url("redis://user:pass@localhost:6379"),
        "redis://****@localhost:6379"
    );
    assert_eq!(
        mask_url("redis://localhost:6379"),
        "redis://localhost:6379"
    );
}

#[test]
fn test_is_retriable_error() {
    // IO errors should be retriable
    let io_error = RedisError::from(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Connection refused",
    ));
    assert!(is_retriable_error(&io_error));

    // Parse errors should not be retriable
    let parse_error = RedisError::from((
        ErrorKind::TypeError,
        "Invalid type",
    ));
    assert!(!is_retriable_error(&parse_error));
}

#[tokio::test]
async fn test_client_creation_with_invalid_url() {
    let config = CacheConfig::new("invalid://url");

    let result = RedisClient::new(config).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_basic_operations() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let client = RedisClient::new(config).await.unwrap();

    // Test set and get
    let key = "test:key";
    let value = "test_value";
    
    client.set_with_expiry(key, value, 60).await.unwrap();
    
    let retrieved = client.get(key).await.unwrap();
    assert_eq!(retrieved, Some(value.to_string()));

    // Test exists
    let exists = client.exists(key).await.unwrap();
    assert!(exists);

    // Test TTL
    let ttl = client.ttl(key).await.unwrap();
    assert!(ttl.is_some());
    assert!(ttl.unwrap() > 0 && ttl.unwrap() <= 60);

    // Test delete
    let deleted = client.delete(key).await.unwrap();
    assert!(deleted);

    // Verify deletion
    let after_delete = client.get(key).await.unwrap();
    assert_eq!(after_delete, None);
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_increment_counter() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let client = RedisClient::new(config).await.unwrap();

    let key = "test:counter";
    
    // Clean up from previous tests
    let _ = client.delete(key).await;

    // First increment should return 1
    let count1 = client.increment(key, Some(60)).await.unwrap();
    assert_eq!(count1, 1);

    // Second increment should return 2
    let count2 = client.increment(key, Some(60)).await.unwrap();
    assert_eq!(count2, 2);

    // Clean up
    client.delete(key).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_health_check() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let client = RedisClient::new(config).await.unwrap();
    
    let healthy = client.health_check().await.unwrap();
    assert!(healthy);
}