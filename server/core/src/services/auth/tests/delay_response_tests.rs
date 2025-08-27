//! Integration tests for delay response service

#[cfg(test)]
mod tests {
    use crate::services::auth::{DelayResponseService, DelayResponseConfig, DelayInfo};
    use std::time::Duration;

    #[test]
    fn test_integration_with_default_config() {
        let service = DelayResponseService::with_defaults();
        
        // Simulate progressive failed attempts
        let mut attempts = vec![];
        for i in 0..10 {
            let delay = service.calculate_delay(i);
            attempts.push((i, delay.as_millis()));
        }
        
        // Verify progressive delays
        assert_eq!(attempts[0], (0, 0));      // No delay on first attempt
        assert_eq!(attempts[1], (1, 500));     // 500ms after first failure
        assert_eq!(attempts[2], (2, 1000));    // 1s after second failure
        assert_eq!(attempts[3], (3, 2000));    // 2s after third failure
        assert_eq!(attempts[4], (4, 4000));    // 4s after fourth failure
        assert_eq!(attempts[5], (5, 8000));    // 8s after fifth failure
        assert_eq!(attempts[6], (6, 16000));   // 16s after sixth failure
        assert_eq!(attempts[7], (7, 30000));   // Max 30s
        assert_eq!(attempts[8], (8, 30000));   // Still max 30s
        assert_eq!(attempts[9], (9, 30000));   // Still max 30s
    }

    #[test]
    fn test_custom_config_for_strict_security() {
        let strict_config = DelayResponseConfig {
            base_delay_ms: 1000,       // Start with 1 second
            backoff_multiplier: 3.0,   // Triple each time (more aggressive)
            max_delay_ms: 60000,        // 1 minute max
            delay_after_attempts: 0,    // Delay from the first failure
        };
        
        let service = DelayResponseService::new(strict_config);
        
        assert_eq!(service.calculate_delay(0).as_millis(), 1000);   // 1s
        assert_eq!(service.calculate_delay(1).as_millis(), 3000);   // 3s
        assert_eq!(service.calculate_delay(2).as_millis(), 9000);   // 9s
        assert_eq!(service.calculate_delay(3).as_millis(), 27000);  // 27s
        assert_eq!(service.calculate_delay(4).as_millis(), 60000);  // Capped at 60s
    }

    #[test]
    fn test_custom_config_for_user_friendly() {
        let friendly_config = DelayResponseConfig {
            base_delay_ms: 200,         // Start with 200ms
            backoff_multiplier: 1.5,    // Slower increase
            max_delay_ms: 5000,          // 5 seconds max
            delay_after_attempts: 3,     // Give users 3 free attempts
        };
        
        let service = DelayResponseService::new(friendly_config);
        
        // First 3 attempts have no delay
        assert_eq!(service.calculate_delay(0).as_millis(), 0);
        assert_eq!(service.calculate_delay(1).as_millis(), 0);
        assert_eq!(service.calculate_delay(2).as_millis(), 0);
        
        // Delays start after 3 attempts
        assert_eq!(service.calculate_delay(3).as_millis(), 200);
        assert_eq!(service.calculate_delay(4).as_millis(), 300);
        assert_eq!(service.calculate_delay(5).as_millis(), 450);
        assert_eq!(service.calculate_delay(6).as_millis(), 675);
    }

    #[tokio::test]
    async fn test_apply_delay_timing() {
        let config = DelayResponseConfig {
            base_delay_ms: 50,
            backoff_multiplier: 2.0,
            max_delay_ms: 500,
            delay_after_attempts: 1,
        };
        let service = DelayResponseService::new(config);
        
        // Test that delays are actually applied
        let test_cases = vec![
            (0, 0),     // No delay
            (1, 50),    // 50ms delay
            (2, 100),   // 100ms delay
            (3, 200),   // 200ms delay
            (4, 400),   // 400ms delay
            (5, 500),   // Max 500ms delay
        ];
        
        for (attempts, expected_delay_ms) in test_cases {
            let start = tokio::time::Instant::now();
            service.apply_delay(attempts).await;
            let elapsed = start.elapsed();
            
            if expected_delay_ms == 0 {
                assert!(elapsed.as_millis() < 10, 
                    "Expected no delay for {} attempts, but got {}ms", 
                    attempts, elapsed.as_millis());
            } else {
                assert!(
                    elapsed.as_millis() >= expected_delay_ms as u128 &&
                    elapsed.as_millis() < (expected_delay_ms + 20) as u128,
                    "Expected ~{}ms delay for {} attempts, but got {}ms",
                    expected_delay_ms, attempts, elapsed.as_millis()
                );
            }
        }
    }

    #[test]
    fn test_delay_info_provides_useful_metrics() {
        let service = DelayResponseService::with_defaults();
        
        // Test info at different stages
        let info_0 = service.get_delay_info(0);
        assert_eq!(info_0.failed_attempts, 0);
        assert!(!info_0.is_delayed);
        assert!(!info_0.at_max_delay);
        assert_eq!(info_0.current_delay_ms, 0);
        assert_eq!(info_0.next_delay_ms, 500);
        
        let info_5 = service.get_delay_info(5);
        assert_eq!(info_5.failed_attempts, 5);
        assert!(info_5.is_delayed);
        assert!(!info_5.at_max_delay);
        assert_eq!(info_5.current_delay_ms, 8000);
        assert_eq!(info_5.next_delay_ms, 16000);
        
        let info_max = service.get_delay_info(20);
        assert_eq!(info_max.failed_attempts, 20);
        assert!(info_max.is_delayed);
        assert!(info_max.at_max_delay);
        assert_eq!(info_max.current_delay_ms, 30000);
        assert_eq!(info_max.next_delay_ms, 30000);
    }

    /// Example of how the service would be integrated with AuthService
    #[tokio::test]
    async fn test_example_auth_integration() {
        let delay_service = DelayResponseService::with_defaults();
        
        // Simulate authentication flow
        async fn verify_code_with_delay(
            delay_service: &DelayResponseService,
            failed_attempts: u32,
        ) -> Result<(), String> {
            // Apply delay before verification
            delay_service.apply_delay(failed_attempts).await;
            
            // Simulate verification (always fails in this test)
            Err("Invalid code".to_string())
        }
        
        // Track timing of multiple failed attempts
        let mut total_delay = Duration::from_millis(0);
        
        for attempt in 0..5 {
            let start = tokio::time::Instant::now();
            let _ = verify_code_with_delay(&delay_service, attempt).await;
            let elapsed = start.elapsed();
            total_delay += elapsed;
            
            // Log delay info for monitoring
            let info = delay_service.get_delay_info(attempt);
            println!(
                "Attempt {}: delayed={}ms, next_delay={}ms", 
                attempt, info.current_delay_ms, info.next_delay_ms
            );
        }
        
        // After 5 attempts, total delay should be at least 0 + 500 + 1000 + 2000 + 4000 = 7500ms
        assert!(total_delay.as_millis() >= 7500);
    }
}