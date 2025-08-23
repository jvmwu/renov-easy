//! Twilio SMS Service Trait Adapter
//!
//! This module provides an adapter that implements the core SmsServiceTrait
//! for the Twilio SMS service, bridging the infrastructure implementation
//! with the core domain trait.

use async_trait::async_trait;
use re_core::services::verification::SmsServiceTrait;

use crate::sms::twilio::{TwilioSmsService, TwilioConfig};
use crate::sms::sms_service::SmsService;

/// Adapter that implements the core SmsServiceTrait for Twilio
pub struct TwilioSmsServiceAdapter {
    inner: TwilioSmsService,
}

impl TwilioSmsServiceAdapter {
    /// Create a new Twilio SMS service adapter
    pub fn new(config: TwilioConfig) -> Result<Self, crate::InfrastructureError> {
        let inner = TwilioSmsService::new(config)?;
        Ok(Self { inner })
    }
    
    /// Create from environment variables
    pub fn from_env() -> Result<Self, crate::InfrastructureError> {
        let config = TwilioConfig::from_env()?;
        Self::new(config)
    }
}

#[async_trait]
impl SmsServiceTrait for TwilioSmsServiceAdapter {
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
        // Use the infrastructure SmsService trait method
        match self.inner.send_verification_code(phone, code).await {
            Ok(message_id) => Ok(message_id),
            Err(e) => Err(e.to_string()),
        }
    }
    
    fn is_valid_phone_number(&self, phone: &str) -> bool {
        // Use the same validation logic
        crate::sms::sms_service::is_valid_phone_number(phone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phone_validation() {
        // Set up test environment
        std::env::set_var("TWILIO_ACCOUNT_SID", "ACtest");
        std::env::set_var("TWILIO_AUTH_TOKEN", "test_token");
        std::env::set_var("TWILIO_FROM_NUMBER", "+15551234567");
        
        let adapter = TwilioSmsServiceAdapter::from_env().unwrap();
        
        // Test valid phone numbers
        assert!(adapter.is_valid_phone_number("+14155552671"));
        assert!(adapter.is_valid_phone_number("+919876543210"));
        
        // Test invalid phone numbers
        assert!(!adapter.is_valid_phone_number("1234567890")); // Missing +
        assert!(!adapter.is_valid_phone_number("+123")); // Too short
        assert!(!adapter.is_valid_phone_number("+1234567890123456")); // Too long
        assert!(!adapter.is_valid_phone_number("+123abc4567")); // Contains letters
        
        // Clean up
        std::env::remove_var("TWILIO_ACCOUNT_SID");
        std::env::remove_var("TWILIO_AUTH_TOKEN");
        std::env::remove_var("TWILIO_FROM_NUMBER");
    }
}