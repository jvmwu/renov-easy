//! AWS SNS SMS Service Trait Adapter
//!
//! This module provides an adapter that implements the core SmsServiceTrait
//! for the AWS SNS SMS service, bridging the infrastructure implementation
//! with the core domain trait.

use async_trait::async_trait;
use re_core::services::verification::SmsServiceTrait;

use crate::sms::aws_sns::{AwsSnsSmsService, AwsSnsConfig};
use crate::sms::sms_service::SmsService;

/// Adapter that implements the core SmsServiceTrait for AWS SNS
pub struct AwsSnsSmsServiceAdapter {
    inner: AwsSnsSmsService,
}

impl AwsSnsSmsServiceAdapter {
    /// Create a new AWS SNS SMS service adapter
    pub async fn new(config: AwsSnsConfig) -> Result<Self, crate::InfrastructureError> {
        let inner = AwsSnsSmsService::new(config).await?;
        Ok(Self { inner })
    }
    
    /// Create from environment variables
    pub async fn from_env() -> Result<Self, crate::InfrastructureError> {
        let config = AwsSnsConfig::from_env()?;
        Self::new(config).await
    }
}

#[async_trait]
impl SmsServiceTrait for AwsSnsSmsServiceAdapter {
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
    
    #[tokio::test]
    async fn test_phone_validation() {
        // Set up test environment
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
        std::env::set_var("AWS_REGION", "us-east-1");
        
        // Test that we can create the adapter
        let adapter = AwsSnsSmsServiceAdapter::from_env().await;
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        
        // Test valid phone numbers using the trait's validation method
        assert!(adapter.is_valid_phone_number("+14155552671"));
        assert!(adapter.is_valid_phone_number("+919876543210"));
        
        // Test invalid phone numbers
        assert!(!adapter.is_valid_phone_number("1234567890")); // Missing +
        assert!(!adapter.is_valid_phone_number("+123")); // Too short
        assert!(!adapter.is_valid_phone_number("+1234567890123456")); // Too long
        assert!(!adapter.is_valid_phone_number("+123abc4567")); // Contains letters
        
        // Clean up
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
    }
}