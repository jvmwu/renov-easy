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
//! - **Twilio Support**: Production SMS via Twilio API (future)
//! - **AWS SNS Support**: Alternative SMS provider (future)
//! - **Phone Number Validation**: E.164 format validation
//! - **Security**: Phone number masking in logs

pub mod sms_service;
pub mod mock_sms;

// Twilio SMS service (feature-gated)
#[cfg(feature = "twilio-sms")]
pub mod twilio;
#[cfg(feature = "twilio-sms")]
pub mod twilio_trait_adapter;

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

// Future implementations will be added here:
// pub mod aws_sns;

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
pub fn create_sms_service(config: &crate::config::SmsConfig) -> Box<dyn SmsService> {
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
        // Future providers:
        // "aws-sns" => Box::new(AwsSnsService::new(config)),
        _ => {
            tracing::warn!(
                "Unknown SMS provider '{}', using mock implementation",
                config.provider
            );
            Box::new(MockSmsService::new())
        }
    }
}