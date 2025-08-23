//! Integration tests for Redis cache client
//! 
//! These tests require a running Redis instance to execute.
//! Run with: cargo test -p infra --test redis_integration -- --ignored

use re_infra::cache::{CacheConfig, RedisClient};

#[tokio::test]
#[ignore] // Requires Redis server
async fn test_redis_connection() {
    let config = CacheConfig {
        url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        pool_size: 5,
        default_ttl: 3600,
    };

    let client = RedisClient::new(config).await;
    assert!(client.is_ok(), "Failed to connect to Redis");
}

#[tokio::test]
#[ignore] // Requires Redis server
async fn test_set_and_get() {
    let config = CacheConfig {
        url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        pool_size: 5,
        default_ttl: 3600,
    };

    let client = RedisClient::new(config).await.unwrap();
    
    // Test verification code scenario
    let phone = "13800138000";
    let code = "123456";
    let key = format!("test:verification:{}", phone);
    
    // Set verification code with 5 minute expiry
    client.set_with_expiry(&key, code, 300).await.unwrap();
    
    // Retrieve the code
    let retrieved = client.get(&key).await.unwrap();
    assert_eq!(retrieved, Some(code.to_string()));
    
    // Clean up
    client.delete(&key).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis server
async fn test_expiry() {
    let config = CacheConfig {
        url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        pool_size: 5,
        default_ttl: 3600,
    };

    let client = RedisClient::new(config).await.unwrap();
    
    let key = "test:expiry";
    let value = "will_expire";
    
    // Set with 2 second expiry
    client.set_with_expiry(key, value, 2).await.unwrap();
    
    // Should exist immediately
    assert!(client.exists(key).await.unwrap());
    
    // Wait for expiry
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Should no longer exist
    assert!(!client.exists(key).await.unwrap());
}

#[tokio::test]
#[ignore] // Requires Redis server
async fn test_rate_limiting_counter() {
    let config = CacheConfig {
        url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        pool_size: 5,
        default_ttl: 3600,
    };

    let client = RedisClient::new(config).await.unwrap();
    
    let phone = "13900139000";
    let key = format!("test:rate_limit:{}", phone);
    
    // Clean up from previous tests
    let _ = client.delete(&key).await;
    
    // First request - counter should be 1
    let count1 = client.increment(&key, Some(60)).await.unwrap();
    assert_eq!(count1, 1);
    
    // Second request - counter should be 2
    let count2 = client.increment(&key, Some(60)).await.unwrap();
    assert_eq!(count2, 2);
    
    // Third request - counter should be 3
    let count3 = client.increment(&key, Some(60)).await.unwrap();
    assert_eq!(count3, 3);
    
    // Clean up
    client.delete(&key).await.unwrap();
}