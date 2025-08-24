//! SMS Service Module
//!
//! This module provides SMS service implementations for sending verification codes
//! and other SMS messages. It includes support for multiple providers and a mock
//! implementation for development.
//!
//! ## Features
//!
//! - **SMS Service Trait**: Common interface for all SMS providers
//! - **Mock Implementation**: Console output for development
//! - **Twilio Support**: Production SMS via Twilio API
//! - **AWS SNS Support**: Alternative SMS provider with automatic failover
//! - **Phone Number Validation**: E.164 format validation
//! - **Security**: Phone number masking in logs

use std::time::Duration;

pub mod sms_service;
pub mod mock_sms;

// Twilio SMS service (feature-gated)
#[cfg(feature = "twilio-sms")]
pub mod twilio;
#[cfg(feature = "twilio-sms")]
pub mod twilio_trait_adapter;

// AWS SNS SMS service (feature-gated)
#[cfg(feature = "aws-sns")]
pub mod aws_sns;
#[cfg(feature = "aws-sns")]
pub mod aws_sns_trait_adapter;

// Failover SMS service
pub mod failover_sms;

// Re-export commonly used types
pub use sms_service::{
    SmsService,
    mask_phone_number,
    is_valid_phone_number,
};
pub use mock_sms::MockSmsService;

#[cfg(feature = "twilio-sms")]
pub use twilio::{TwilioSmsService, TwilioConfig};
#[cfg(feature = "twilio-sms")]
pub use twilio_trait_adapter::TwilioSmsServiceAdapter;

#[cfg(feature = "aws-sns")]
pub use aws_sns::{AwsSnsSmsService, AwsSnsConfig};
#[cfg(feature = "aws-sns")]
pub use aws_sns_trait_adapter::AwsSnsSmsServiceAdapter;

pub use failover_sms::{FailoverSmsService, FailoverSmsServiceAdapter};

#[cfg(test)]
mod tests;

/// Create an SMS service based on configuration
///
/// Returns the appropriate SMS service implementation based on the
/// provider specified in the configuration.
///
/// # Arguments
///
/// * `config` - SMS configuration containing provider settings
///
/// # Returns
///
/// A boxed SMS service implementation
pub async fn create_sms_service(config: &crate::config::SmsConfig) -> Box<dyn SmsService> {
    match config.provider.as_str() {
        "mock" => Box::new(MockSmsService::new()),
        #[cfg(feature = "twilio-sms")]
        "twilio" => {
            // Create Twilio configuration from the generic SMS config
            let twilio_config = TwilioConfig {
                account_sid: config.api_key.clone(),
                auth_token: config.api_secret.clone(),
                from_number: config.from_number.clone(),
                max_retries: 3,
                retry_delay_ms: 1000,
                request_timeout_secs: 30,
            };
            
            match TwilioSmsService::new(twilio_config) {
                Ok(service) => Box::new(service),
                Err(e) => {
                    tracing::error!("Failed to initialize Twilio SMS service: {}", e);
                    tracing::warn!("Falling back to mock SMS service");
                    Box::new(MockSmsService::new())
                }
            }
        }
        #[cfg(feature = "aws-sns")]
        "aws-sns" => {
            // Create AWS SNS configuration from the generic SMS config
            let aws_config = AwsSnsConfig {
                access_key_id: config.api_key.clone(),
                secret_access_key: config.api_secret.clone(),
                region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                sender_id: config.from_number.clone().into(),
                sms_type: "Transactional".to_string(),
                max_retries: 3,
                retry_delay_ms: 1000,
                request_timeout_secs: 30,
            };
            
            match AwsSnsSmsService::new(aws_config).await {
                Ok(service) => Box::new(service),
                Err(e) => {
                    tracing::error!("Failed to initialize AWS SNS SMS service: {}", e);
                    tracing::warn!("Falling back to mock SMS service");
                    Box::new(MockSmsService::new())
                }
            }
        }
        "failover" => {
            // Create failover service with Twilio as primary and AWS SNS as backup
            create_failover_sms_service().await
        }
        _ => {
            tracing::warn!(
                "Unknown SMS provider '{}', using mock implementation",
                config.provider
            );
            Box::new(MockSmsService::new())
        }
    }
}

/// Create a failover SMS service with Twilio as primary and AWS SNS as backup
///
/// This function creates a resilient SMS service that automatically switches
/// from Twilio to AWS SNS if the primary service fails.
pub async fn create_failover_sms_service() -> Box<dyn SmsService> {
    #[cfg(all(feature = "twilio-sms", feature = "aws-sns"))]
    {
        // Try to create Twilio service
        let twilio_service = match TwilioConfig::from_env() {
            Ok(config) => match TwilioSmsService::new(config) {
                Ok(service) => Some(Box::new(service) as Box<dyn SmsService>),
                Err(e) => {
                    tracing::warn!("Failed to initialize Twilio SMS service: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to load Twilio configuration: {}", e);
                None
            }
        };
        
        // Try to create AWS SNS service
        let aws_service = match AwsSnsConfig::from_env() {
            Ok(config) => match AwsSnsSmsService::new(config).await {
                Ok(service) => Some(Box::new(service) as Box<dyn SmsService>),
                Err(e) => {
                    tracing::warn!("Failed to initialize AWS SNS SMS service: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to load AWS SNS configuration: {}", e);
                None
            }
        };
        
        // Create failover service if we have at least one service
        match (twilio_service, aws_service) {
            (Some(primary), Some(backup)) => {
                tracing::info!("Created failover SMS service with Twilio (primary) and AWS SNS (backup)");
                Box::new(FailoverSmsService::new(primary, backup, Duration::from_secs(30)))
            }
            (Some(service), None) | (None, Some(service)) => {
                tracing::warn!("Only one SMS service available, failover disabled");
                service
            }
            (None, None) => {
                tracing::error!("No SMS services available, using mock implementation");
                Box::new(MockSmsService::new())
            }
        }
    }
    
    #[cfg(not(all(feature = "twilio-sms", feature = "aws-sns")))]
    {
        tracing::warn!("Failover SMS service requires both twilio-sms and aws-sns features");
        Box::new(MockSmsService::new())
    }
}