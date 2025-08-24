//! Unit tests for rate limiter functionality

use super::mocks::MockRateLimiter;
use crate::services::auth::rate_limiter::RateLimiterTrait;

#[tokio::test]
async fn test_sms_rate_limiting() {
    let limiter = MockRateLimiter::new(3); // Max 3 requests
    
    // Should not be limited initially
    let is_limited = limiter.check_sms_rate_limit("+1234567890").await.unwrap();
    assert!(!is_limited);
    
    // Increment counter 3 times
    for _ in 0..3 {
        limiter.increment_sms_counter("+1234567890").await.unwrap();
    }
    
    // Should now be rate limited
    let is_limited = limiter.check_sms_rate_limit("+1234567890").await.unwrap();
    assert!(is_limited);
    
    // Different phone should not be limited
    let is_limited = limiter.check_sms_rate_limit("+0987654321").await.unwrap();
    assert!(!is_limited);
}

#[tokio::test]
async fn test_ip_verification_limiting() {
    let mut limiter = MockRateLimiter::new(3);
    limiter.max_ip_attempts = 5; // Max 5 attempts from same IP
    
    let test_ip = "192.168.1.1";
    
    // Should not be limited initially
    let is_limited = limiter.check_ip_verification_limit(test_ip).await.unwrap();
    assert!(!is_limited);
    
    // Increment counter 5 times
    for i in 1..=5 {
        let count = limiter.increment_ip_verification_counter(test_ip).await.unwrap();
        assert_eq!(count, i);
    }
    
    // Should now be rate limited
    let is_limited = limiter.check_ip_verification_limit(test_ip).await.unwrap();
    assert!(is_limited);
    
    // Different IP should not be limited
    let is_limited = limiter.check_ip_verification_limit("10.0.0.1").await.unwrap();
    assert!(!is_limited);
}

#[tokio::test]
async fn test_rate_limit_reset_time() {
    let limiter = MockRateLimiter::new(3);
    
    // Should return reset time for phone
    let reset_time = limiter.get_rate_limit_reset_time("+1234567890").await.unwrap();
    assert_eq!(reset_time, Some(3600)); // 1 hour in seconds
    
    // Should return reset time for IP
    let reset_time = limiter.get_ip_rate_limit_reset_time("192.168.1.1").await.unwrap();
    assert_eq!(reset_time, Some(3600)); // 1 hour in seconds
}

#[tokio::test]
async fn test_rate_limit_violation_logging() {
    let limiter = MockRateLimiter::new(3);
    
    // Log a phone rate limit violation
    limiter.log_rate_limit_violation(
        "+1234567890",
        "phone",
        "sms_send"
    ).await.unwrap();
    
    // Log an IP rate limit violation
    limiter.log_rate_limit_violation(
        "192.168.1.1",
        "ip",
        "verify_code"
    ).await.unwrap();
    
    // Check that logs were recorded
    let logs = limiter.rate_limit_logs.lock().unwrap();
    assert_eq!(logs.len(), 2);
    
    // Verify first log
    assert_eq!(logs[0].0, "+1234567890");
    assert_eq!(logs[0].1, "phone");
    assert_eq!(logs[0].2, "sms_send");
    
    // Verify second log
    assert_eq!(logs[1].0, "192.168.1.1");
    assert_eq!(logs[1].1, "ip");
    assert_eq!(logs[1].2, "verify_code");
}

#[tokio::test]
async fn test_increment_counters() {
    let limiter = MockRateLimiter::new(5);
    
    // Test phone counter increment
    let count1 = limiter.increment_sms_counter("+1234567890").await.unwrap();
    assert_eq!(count1, 1);
    
    let count2 = limiter.increment_sms_counter("+1234567890").await.unwrap();
    assert_eq!(count2, 2);
    
    // Test IP counter increment
    let count1 = limiter.increment_ip_verification_counter("192.168.1.1").await.unwrap();
    assert_eq!(count1, 1);
    
    let count2 = limiter.increment_ip_verification_counter("192.168.1.1").await.unwrap();
    assert_eq!(count2, 2);
    
    // Different IP should have separate counter
    let count1 = limiter.increment_ip_verification_counter("10.0.0.1").await.unwrap();
    assert_eq!(count1, 1);
}