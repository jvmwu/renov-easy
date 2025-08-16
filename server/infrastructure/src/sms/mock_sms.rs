//! Mock SMS Service Implementation
//!
//! A mock implementation of the SMS service for development and testing.
//! This implementation logs SMS messages to the console instead of sending them.

use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::InfrastructureError;
use super::sms_service::{SmsService, mask_phone_number, is_valid_phone_number};

/// Mock SMS service for development and testing
///
/// This implementation:
/// - Logs SMS messages to console
/// - Validates phone numbers
/// - Generates mock message IDs
/// - Tracks message count for testing
#[derive(Clone)]
pub struct MockSmsService {
    /// Counter for tracking number of messages sent
    message_count: Arc<AtomicU64>,
    /// Whether to simulate failures (for testing)
    simulate_failure: bool,
    /// Whether to print messages to console
    console_output: bool,
}

impl MockSmsService {
    /// Create a new mock SMS service
    pub fn new() -> Self {
        Self {
            message_count: Arc::new(AtomicU64::new(0)),
            simulate_failure: false,
            console_output: true,
        }
    }

    /// Create a mock service with configurable options
    pub fn with_options(console_output: bool, simulate_failure: bool) -> Self {
        Self {
            message_count: Arc::new(AtomicU64::new(0)),
            simulate_failure,
            console_output,
        }
    }

    /// Get the total number of messages sent
    pub fn get_message_count(&self) -> u64 {
        self.message_count.load(Ordering::SeqCst)
    }

    /// Reset the message counter
    pub fn reset_counter(&self) {
        self.message_count.store(0, Ordering::SeqCst);
    }

    /// Enable or disable failure simulation
    pub fn set_simulate_failure(&mut self, simulate: bool) {
        self.simulate_failure = simulate;
    }
}

impl Default for MockSmsService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SmsService for MockSmsService {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<String, InfrastructureError> {
        // Validate phone number format
        if !is_valid_phone_number(phone_number) {
            return Err(InfrastructureError::Sms(format!(
                "Invalid phone number format: {}",
                mask_phone_number(phone_number)
            )));
        }

        // Simulate failure if configured
        if self.simulate_failure {
            warn!(
                "Mock SMS service simulating failure for phone: {}",
                mask_phone_number(phone_number)
            );
            return Err(InfrastructureError::Sms(
                "Simulated SMS sending failure".to_string()
            ));
        }

        // Generate mock message ID
        let message_id = format!("mock_{}", Uuid::new_v4());
        
        // Increment message counter
        let count = self.message_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Log the SMS details
        let masked_phone = mask_phone_number(phone_number);
        
        if self.console_output {
            // Console output for development - show full message
            println!("\n{}", "=".repeat(60));
            println!("📱 MOCK SMS SERVICE - MESSAGE #{}", count);
            println!("{}", "=".repeat(60));
            println!("To: {} (masked: {})", phone_number, masked_phone);
            println!("Message ID: {}", message_id);
            println!("Content: {}", message);
            println!("{}\n", "=".repeat(60));
        }

        // Structured logging for production
        info!(
            target: "sms_service",
            provider = "mock",
            phone = %masked_phone,
            message_id = %message_id,
            message_length = message.len(),
            "SMS sent successfully (mock)"
        );

        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(message_id)
    }

    fn provider_name(&self) -> &str {
        "Mock"
    }

    async fn is_available(&self) -> bool {
        !self.simulate_failure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_sms_send_success() {
        let service = MockSmsService::with_options(false, false);
        let result = service.send_sms("+1234567890", "Test message").await;
        
        assert!(result.is_ok());
        let message_id = result.unwrap();
        assert!(message_id.starts_with("mock_"));
        assert_eq!(service.get_message_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_sms_invalid_phone() {
        let service = MockSmsService::new();
        let result = service.send_sms("1234567890", "Test message").await;
        
        assert!(result.is_err());
        if let Err(InfrastructureError::Sms(msg)) = result {
            assert!(msg.contains("Invalid phone number"));
        } else {
            panic!("Expected Sms error");
        }
    }

    #[tokio::test]
    async fn test_mock_sms_simulate_failure() {
        let mut service = MockSmsService::new();
        service.set_simulate_failure(true);
        
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_err());
        assert!(!service.is_available().await);
    }

    #[tokio::test]
    async fn test_mock_sms_verification_code() {
        let service = MockSmsService::with_options(false, false);
        let result = service.send_verification_code("+1234567890", "123456").await;
        
        assert!(result.is_ok());
        assert_eq!(service.get_message_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_sms_counter() {
        let service = MockSmsService::new();
        
        for i in 1..=3 {
            let _ = service.send_sms("+1234567890", &format!("Message {}", i)).await;
            assert_eq!(service.get_message_count(), i);
        }
        
        service.reset_counter();
        assert_eq!(service.get_message_count(), 0);
    }

    #[test]
    fn test_provider_name() {
        let service = MockSmsService::new();
        assert_eq!(service.provider_name(), "Mock");
    }
}