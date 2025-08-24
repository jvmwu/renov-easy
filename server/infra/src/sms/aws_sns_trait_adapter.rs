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