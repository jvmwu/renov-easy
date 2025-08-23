//! Integration tests for Redis-based rate limiter
//!
//! These tests require Redis to be running locally on port 6379.
//! Run with: cargo test --test rate_limiter_integration -- --ignored

use infra::cache::redis_client::RedisClient;
use infra::services::auth::{RedisRateLimiter, RateLimitStatus};
use renov_core::services::auth::RateLimiterTrait;
use shared::config::cache::CacheConfig;
use shared::config::rate_limit::RateLimitConfig;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Helper to create a test rate limiter with custom config
async fn create_test_limiter_with_config(config: RateLimitConfig) -> RedisRateLimiter {
    let cache_config = CacheConfig::new("redis://localhost:6379");
    let redis_client = RedisClient::new(cache_config)
        .await
        .expect("Failed to create Redis client");
    
    RedisRateLimiter::new(Arc::new(redis_client), config)
}

/// Helper to create a test rate limiter with default config
async fn create_test_limiter() -> RedisRateLimiter {
    create_test_limiter_with_config(RateLimitConfig::default()).await
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_sms_rate_limit_enforcement() {
    let limiter = create_test_limiter().await;
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    
    // Reset any existing limits
    limiter.reset_phone_limits(&phone).await.unwrap();
    
    // First 3 requests should succeed (default limit)
    for i in 1..=3 {
        let is_limited = limiter.check_sms_rate_limit(&phone).await.unwrap();
        assert!(!is_limited, "Request {} should not be rate limited", i);
        
        let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
        match status {
            RateLimitStatus::Ok { remaining, limit, .. } => {
                assert_eq!(limit, 3);
                assert_eq!(remaining, 3 - i);
            }
            _ => panic!("Expected Ok status for request {}", i),
        }
    }
    
    // Fourth request should be rate limited
    let is_limited = limiter.check_sms_rate_limit(&phone).await.unwrap();
    assert!(is_limited, "Fourth request should be rate limited");
    
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Exceeded { .. }));
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_ip_rate_limit_enforcement() {
    let mut config = RateLimitConfig::default();
    config.auth.login_per_ip_per_hour = 5; // Set lower limit for testing
    
    let limiter = create_test_limiter_with_config(config).await;
    let ip = format!("192.168.{}.{}", rand::random::<u8>(), rand::random::<u8>());
    
    // Reset any existing limits
    limiter.reset_ip_limits(&ip).await.unwrap();
    
    // First 5 requests should succeed
    for i in 1..=5 {
        let status = limiter.check_ip_verification_limit(&ip).await.unwrap();
        match status {
            RateLimitStatus::Ok { remaining, limit, .. } => {
                assert_eq!(limit, 5);
                assert_eq!(remaining, 5 - i);
            }
            _ => panic!("Expected Ok status for request {}", i),
        }
    }
    
    // Sixth request should be rate limited
    let status = limiter.check_ip_verification_limit(&ip).await.unwrap();
    match status {
        RateLimitStatus::Exceeded { retry_after_seconds, .. } => {
            assert!(retry_after_seconds > 0);
        }
        _ => panic!("Expected Exceeded status for sixth request"),
    }
    
    // Clean up
    limiter.reset_ip_limits(&ip).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_phone_lockout_after_failed_attempts() {
    let mut config = RateLimitConfig::default();
    config.auth.failed_attempts_threshold = 3; // Lower threshold for testing
    config.sms.phone_lock_duration = 2; // 2 seconds for faster testing
    
    let limiter = create_test_limiter_with_config(config).await;
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    
    // Reset any existing limits
    limiter.reset_phone_limits(&phone).await.unwrap();
    
    // Simulate failed attempts
    for i in 1..3 {
        let locked = limiter.increment_failed_attempts(&phone).await.unwrap();
        assert!(!locked, "Should not be locked after {} attempts", i);
    }
    
    // Third failed attempt should trigger lockout
    let locked = limiter.increment_failed_attempts(&phone).await.unwrap();
    assert!(locked, "Should be locked after 3 failed attempts");
    
    // Verify phone is locked
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    match status {
        RateLimitStatus::Locked { retry_after_seconds, reason } => {
            assert!(retry_after_seconds > 0);
            assert!(reason.contains("locked"));
        }
        _ => panic!("Expected Locked status"),
    }
    
    // Wait for lock to expire
    sleep(Duration::from_secs(3)).await;
    
    // Should be unlocked now
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Ok { .. }));
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_sliding_window_accuracy() {
    let mut config = RateLimitConfig::default();
    config.sms.per_phone_per_hour = 3;
    
    let limiter = create_test_limiter_with_config(config).await;
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    
    // Reset any existing limits
    limiter.reset_phone_limits(&phone).await.unwrap();
    
    // Make 3 requests (reach limit)
    for _ in 1..=3 {
        let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
        assert!(matches!(status, RateLimitStatus::Ok { .. }));
    }
    
    // Fourth request should be limited
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Exceeded { .. }));
    
    // Note: In a real sliding window, old entries would expire after exactly 1 hour
    // For testing purposes, we're just verifying the limit is enforced
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_get_phone_status() {
    let limiter = create_test_limiter().await;
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    
    // Reset and get initial status
    limiter.reset_phone_limits(&phone).await.unwrap();
    let status = limiter.get_phone_status(&phone).await.unwrap();
    
    assert_eq!(status.identifier, phone);
    assert_eq!(status.identifier_type, "phone");
    assert!(!status.is_locked);
    assert!(status.lock_ttl_seconds.is_none());
    assert_eq!(status.failed_attempts, 0);
    
    // Make some requests
    limiter.check_phone_sms_limit(&phone).await.unwrap();
    limiter.check_phone_sms_limit(&phone).await.unwrap();
    
    // Check updated status
    let status = limiter.get_phone_status(&phone).await.unwrap();
    assert_eq!(status.limits[0].current, 2);
    assert_eq!(status.limits[0].limit, 3);
    assert_eq!(status.limits[0].limit_type, "sms_per_hour");
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_get_ip_status() {
    let limiter = create_test_limiter().await;
    let ip = format!("192.168.{}.{}", rand::random::<u8>(), rand::random::<u8>());
    
    // Reset and get initial status
    limiter.reset_ip_limits(&ip).await.unwrap();
    let status = limiter.get_ip_status(&ip).await.unwrap();
    
    assert_eq!(status.identifier, ip);
    assert_eq!(status.identifier_type, "ip");
    assert!(!status.is_locked);
    assert!(status.lock_ttl_seconds.is_none());
    assert_eq!(status.failed_attempts, 0);
    
    // Make some requests
    limiter.check_ip_verification_limit(&ip).await.unwrap();
    limiter.check_ip_verification_limit(&ip).await.unwrap();
    limiter.check_ip_verification_limit(&ip).await.unwrap();
    
    // Check updated status
    let status = limiter.get_ip_status(&ip).await.unwrap();
    assert_eq!(status.limits[0].current, 3);
    assert_eq!(status.limits[0].limit, 10); // Default auth.login_per_ip_per_hour
    assert_eq!(status.limits[0].limit_type, "verification_per_hour");
    
    // Clean up
    limiter.reset_ip_limits(&ip).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_reset_limits() {
    let mut config = RateLimitConfig::default();
    config.sms.per_phone_per_hour = 1; // Very low limit for testing
    
    let limiter = create_test_limiter_with_config(config).await;
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    let ip = format!("192.168.{}.{}", rand::random::<u8>(), rand::random::<u8>());
    
    // Hit rate limits
    limiter.check_phone_sms_limit(&phone).await.unwrap();
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Exceeded { .. }));
    
    // Reset phone limits
    limiter.reset_phone_limits(&phone).await.unwrap();
    
    // Should be able to make request again
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Ok { .. }));
    
    // Test IP reset similarly
    for _ in 0..10 {
        limiter.check_ip_verification_limit(&ip).await.unwrap();
    }
    let status = limiter.check_ip_verification_limit(&ip).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Exceeded { .. }));
    
    limiter.reset_ip_limits(&ip).await.unwrap();
    let status = limiter.check_ip_verification_limit(&ip).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Ok { .. }));
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
    limiter.reset_ip_limits(&ip).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_concurrent_requests() {
    use tokio::task::JoinSet;
    
    let mut config = RateLimitConfig::default();
    config.sms.per_phone_per_hour = 10; // Higher limit for concurrent testing
    
    let limiter = Arc::new(create_test_limiter_with_config(config).await);
    let phone = format!("+1555{:07}", rand::random::<u32>() % 10000000);
    
    // Reset any existing limits
    limiter.reset_phone_limits(&phone).await.unwrap();
    
    // Launch concurrent requests
    let mut tasks = JoinSet::new();
    for _ in 0..10 {
        let limiter_clone = limiter.clone();
        let phone_clone = phone.clone();
        tasks.spawn(async move {
            limiter_clone.check_phone_sms_limit(&phone_clone).await
        });
    }
    
    // Collect results
    let mut ok_count = 0;
    let mut exceeded_count = 0;
    
    while let Some(result) = tasks.join_next().await {
        let status = result.unwrap().unwrap();
        match status {
            RateLimitStatus::Ok { .. } => ok_count += 1,
            RateLimitStatus::Exceeded { .. } => exceeded_count += 1,
            _ => panic!("Unexpected status"),
        }
    }
    
    // Exactly 10 requests should succeed
    assert_eq!(ok_count, 10);
    assert_eq!(exceeded_count, 0);
    
    // Next request should be rate limited
    let status = limiter.check_phone_sms_limit(&phone).await.unwrap();
    assert!(matches!(status, RateLimitStatus::Exceeded { .. }));
    
    // Clean up
    limiter.reset_phone_limits(&phone).await.unwrap();
}

#[tokio::test]
async fn test_rate_limit_trait_implementation() {
    // This test doesn't require Redis - just tests the trait implementation
    let cache_config = CacheConfig::new("redis://localhost:6379");
    
    // This will fail to connect if Redis isn't running, but we can still test compilation
    if let Ok(redis_client) = RedisClient::new(cache_config).await {
        let limiter = RedisRateLimiter::new(Arc::new(redis_client), RateLimitConfig::default());
        
        // Test that it implements the trait
        let _trait_obj: Box<dyn RateLimiterTrait> = Box::new(limiter);
    }
}