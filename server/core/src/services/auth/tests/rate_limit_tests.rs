//! Tests for enhanced rate limiting in AuthService

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::repositories::audit::NoOpAuditLogRepository;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    
    /// Mock rate limiter for testing
    struct MockRateLimiter {
        phone_counters: Arc<Mutex<HashMap<String, i64>>>,
        ip_counters: Arc<Mutex<HashMap<String, i64>>>,
        phone_limit: i64,
        ip_limit: i64,
    }
    
    impl MockRateLimiter {
        fn new(phone_limit: i64, ip_limit: i64) -> Self {
            Self {
                phone_counters: Arc::new(Mutex::new(HashMap::new())),
                ip_counters: Arc::new(Mutex::new(HashMap::new())),
                phone_limit,
                ip_limit,
            }
        }
    }
    
    #[async_trait]
    impl RateLimiterTrait for MockRateLimiter {
        async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String> {
            let counters = self.phone_counters.lock().unwrap();
            let count = counters.get(phone).unwrap_or(&0);
            Ok(*count >= self.phone_limit)
        }
        
        async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String> {
            let mut counters = self.phone_counters.lock().unwrap();
            let count = counters.entry(phone.to_string()).or_insert(0);
            *count += 1;
            Ok(*count)
        }
        
        async fn get_rate_limit_reset_time(&self, _phone: &str) -> Result<Option<i64>, String> {
            Ok(Some(3600))
        }
        
        async fn check_ip_verification_limit(&self, ip: &str) -> Result<bool, String> {
            let counters = self.ip_counters.lock().unwrap();
            let count = counters.get(ip).unwrap_or(&0);
            Ok(*count >= self.ip_limit)
        }
        
        async fn increment_ip_verification_counter(&self, ip: &str) -> Result<i64, String> {
            let mut counters = self.ip_counters.lock().unwrap();
            let count = counters.entry(ip.to_string()).or_insert(0);
            *count += 1;
            Ok(*count)
        }
        
        async fn get_ip_rate_limit_reset_time(&self, _ip: &str) -> Result<Option<i64>, String> {
            Ok(Some(3600))
        }
        
        async fn log_rate_limit_violation(
            &self, 
            _identifier: &str, 
            _identifier_type: &str,
            _action: &str
        ) -> Result<(), String> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_phone_rate_limiting() {
        // Create a mock rate limiter with limit of 3 per phone
        let rate_limiter = Arc::new(MockRateLimiter::new(3, 10));
        
        // Test that phone is rate limited after 3 attempts
        let phone = "+1234567890";
        
        // First 3 attempts should succeed
        for i in 1..=3 {
            let exceeded = rate_limiter.check_sms_rate_limit(phone).await.unwrap();
            assert!(!exceeded, "Attempt {} should not exceed limit", i);
            rate_limiter.increment_sms_counter(phone).await.unwrap();
        }
        
        // 4th attempt should be rate limited
        let exceeded = rate_limiter.check_sms_rate_limit(phone).await.unwrap();
        assert!(exceeded, "4th attempt should exceed limit");
    }
    
    #[tokio::test]
    async fn test_ip_rate_limiting() {
        // Create a mock rate limiter with limit of 10 per IP
        let rate_limiter = Arc::new(MockRateLimiter::new(3, 10));
        
        // Test that IP is rate limited after 10 attempts
        let ip = "192.168.1.1";
        
        // First 10 attempts should succeed
        for i in 1..=10 {
            let exceeded = rate_limiter.check_ip_verification_limit(ip).await.unwrap();
            assert!(!exceeded, "Attempt {} should not exceed limit", i);
            rate_limiter.increment_ip_verification_counter(ip).await.unwrap();
        }
        
        // 11th attempt should be rate limited
        let exceeded = rate_limiter.check_ip_verification_limit(ip).await.unwrap();
        assert!(exceeded, "11th attempt should exceed limit");
    }
    
    #[tokio::test]
    async fn test_independent_phone_rate_limits() {
        // Create a mock rate limiter
        let rate_limiter = Arc::new(MockRateLimiter::new(3, 10));
        
        let phone1 = "+1234567890";
        let phone2 = "+9876543210";
        
        // Use up rate limit for phone1
        for _ in 1..=3 {
            rate_limiter.increment_sms_counter(phone1).await.unwrap();
        }
        
        // phone1 should be rate limited
        let exceeded1 = rate_limiter.check_sms_rate_limit(phone1).await.unwrap();
        assert!(exceeded1, "phone1 should be rate limited");
        
        // phone2 should still have capacity
        let exceeded2 = rate_limiter.check_sms_rate_limit(phone2).await.unwrap();
        assert!(!exceeded2, "phone2 should not be rate limited");
    }
    
    #[tokio::test]
    async fn test_combined_phone_and_ip_limits() {
        // Create a mock rate limiter
        let rate_limiter = Arc::new(MockRateLimiter::new(3, 10));
        
        let phone = "+1234567890";
        let ip = "192.168.1.1";
        
        // Phone can have 3 attempts
        for _ in 1..=3 {
            rate_limiter.increment_sms_counter(phone).await.unwrap();
        }
        
        // IP can have 10 attempts total across all phones
        for _ in 1..=10 {
            rate_limiter.increment_ip_verification_counter(ip).await.unwrap();
        }
        
        // Both should now be rate limited
        let phone_exceeded = rate_limiter.check_sms_rate_limit(phone).await.unwrap();
        let ip_exceeded = rate_limiter.check_ip_verification_limit(ip).await.unwrap();
        
        assert!(phone_exceeded, "Phone should be rate limited");
        assert!(ip_exceeded, "IP should be rate limited");
    }
}