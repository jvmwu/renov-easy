//! Example demonstrating the Redis-based rate limiter usage
//!
//! Run with: cargo run --example rate_limiter_demo

use infra::cache::redis_client::RedisClient;
use infra::services::auth::{RedisRateLimiter, RateLimitStatus};
use renov_core::services::auth::RateLimiterTrait;
use shared::config::cache::CacheConfig;
use shared::config::rate_limit::RateLimitConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create Redis client
    let cache_config = CacheConfig::new("redis://localhost:6379");
    let redis_client = RedisClient::new(cache_config).await?;
    let redis_client = Arc::new(redis_client);
    
    // Create rate limiter with default config
    let mut rate_config = RateLimitConfig::default();
    // Customize limits for demo
    rate_config.sms.per_phone_per_hour = 3;
    rate_config.auth.login_per_ip_per_hour = 5;
    
    let limiter = RedisRateLimiter::new(redis_client, rate_config);
    
    // Test phone number rate limiting
    let phone = "+1234567890";
    println!("\n=== Testing Phone SMS Rate Limiting ===");
    
    // Reset any existing limits
    limiter.reset_phone_limits(phone).await?;
    
    // Make 3 requests (should all succeed)
    for i in 1..=3 {
        let is_limited = limiter.check_sms_rate_limit(phone).await?;
        println!("Request {}: Rate limited = {}", i, is_limited);
        
        let status = limiter.check_phone_sms_limit(phone).await?;
        match status {
            RateLimitStatus::Ok { remaining, limit, .. } => {
                println!("  -> Allowed. Remaining: {}/{}", remaining, limit);
            }
            RateLimitStatus::Exceeded { retry_after_seconds, .. } => {
                println!("  -> Rate limited! Retry after {} seconds", retry_after_seconds);
            }
            RateLimitStatus::Locked { retry_after_seconds, reason } => {
                println!("  -> Locked! Reason: {}. Retry after {} seconds", reason, retry_after_seconds);
            }
        }
    }
    
    // Fourth request should be rate limited
    println!("\nAttempting 4th request (should be rate limited):");
    let is_limited = limiter.check_sms_rate_limit(phone).await?;
    println!("Request 4: Rate limited = {}", is_limited);
    
    // Test IP rate limiting
    let ip = "192.168.1.100";
    println!("\n=== Testing IP Verification Rate Limiting ===");
    
    // Reset any existing limits
    limiter.reset_ip_limits(ip).await?;
    
    // Make 5 requests (should all succeed)
    for i in 1..=5 {
        let status = limiter.check_ip_verification_limit(ip).await?;
        match status {
            RateLimitStatus::Ok { remaining, limit, .. } => {
                println!("Request {}: Allowed. Remaining: {}/{}", i, remaining, limit);
            }
            RateLimitStatus::Exceeded { retry_after_seconds, .. } => {
                println!("Request {}: Rate limited! Retry after {} seconds", i, retry_after_seconds);
            }
            RateLimitStatus::Locked { .. } => {
                println!("Request {}: IP is locked!", i);
            }
        }
    }
    
    // Sixth request should be rate limited
    println!("\nAttempting 6th request (should be rate limited):");
    let status = limiter.check_ip_verification_limit(ip).await?;
    match status {
        RateLimitStatus::Exceeded { retry_after_seconds, .. } => {
            println!("Rate limited as expected! Retry after {} seconds", retry_after_seconds);
        }
        _ => println!("Unexpected status"),
    }
    
    // Test failed attempts and lockout
    println!("\n=== Testing Failed Attempts and Lockout ===");
    let test_phone = "+9876543210";
    limiter.reset_phone_limits(test_phone).await?;
    
    // Simulate failed attempts (default threshold is 5)
    for i in 1..=5 {
        let locked = limiter.increment_failed_attempts(test_phone).await?;
        println!("Failed attempt {}: Locked = {}", i, locked);
        
        if locked {
            println!("Phone is now locked!");
            
            // Try to make a request
            let status = limiter.check_phone_sms_limit(test_phone).await?;
            match status {
                RateLimitStatus::Locked { retry_after_seconds, reason } => {
                    println!("Confirmed locked: {}. Retry after {} seconds", reason, retry_after_seconds);
                }
                _ => println!("Unexpected status"),
            }
            break;
        }
    }
    
    // Get status information
    println!("\n=== Rate Limit Status Information ===");
    
    let phone_status = limiter.get_phone_status(phone).await?;
    println!("Phone {} status:", phone_status.identifier);
    println!("  Locked: {}", phone_status.is_locked);
    println!("  Failed attempts: {}/{}", phone_status.failed_attempts, phone_status.failed_attempts_threshold);
    for limit in &phone_status.limits {
        println!("  {}: {}/{} (window: {}s)", limit.name, limit.current, limit.limit, limit.window_seconds);
    }
    
    let ip_status = limiter.get_ip_status(ip).await?;
    println!("\nIP {} status:", ip_status.identifier);
    println!("  Locked: {}", ip_status.is_locked);
    println!("  Failed attempts: {}/{}", ip_status.failed_attempts, ip_status.failed_attempts_threshold);
    for limit in &ip_status.limits {
        println!("  {}: {}/{} (window: {}s)", limit.name, limit.current, limit.limit, limit.window_seconds);
    }
    
    // Clean up
    println!("\n=== Cleaning up ===");
    limiter.reset_phone_limits(phone).await?;
    limiter.reset_phone_limits(test_phone).await?;
    limiter.reset_ip_limits(ip).await?;
    println!("All rate limits reset.");
    
    Ok(())
}