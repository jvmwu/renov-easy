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

// Re-export commonly used types
pub use sms_service::{
    SmsService,
    mask_phone_number,
    is_valid_phone_number,
};
pub use mock_sms::MockSmsService;

// Future implementations will be added here:
// pub mod twilio;
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
        // Future providers:
        // "twilio" => Box::new(TwilioSmsService::new(config)),
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