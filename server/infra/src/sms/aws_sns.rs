//! AWS SNS SMS Service Implementation
//!
//! This module provides SMS sending capabilities using the AWS SNS API.
//! It implements the SmsService trait for production SMS delivery and serves
//! as a backup provider when Twilio fails.
//!
//! ## Features
//!
//! - International SMS support with E.164 format validation
//! - Automatic retry logic with exponential backoff
//! - Rate limiting handling
//! - Delivery status tracking
//! - Comprehensive error handling
//! - Security: Phone number masking in logs
//! - Automatic failover from Twilio

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_sns::{
    Client as SnsClient,
    config::Region,
    types::MessageAttributeValue,
};
use phonenumber::{Mode, PhoneNumber};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::{
    sms::sms_service::{mask_phone_number, SmsService},
    InfrastructureError,
};

/// AWS SNS SMS service configuration
#[derive(Debug, Clone)]
pub struct AwsSnsConfig {
    /// AWS Access Key ID
    pub access_key_id: String,
    /// AWS Secret Access Key
    pub secret_access_key: String,
    /// AWS Region (e.g., "us-east-1")
    pub region: String,
    /// SMS sender ID (optional, may not be supported in all regions)
    pub sender_id: Option<String>,
    /// SMS type: "Transactional" or "Promotional"
    pub sms_type: String,
    /// Maximum retry attempts for failed requests
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Timeout for API requests in seconds
    pub request_timeout_secs: u64,
}

impl AwsSnsConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, InfrastructureError> {
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
            .or_else(|_| std::env::var("AWS_SNS_ACCESS_KEY_ID"))
            .map_err(|_| InfrastructureError::Config("AWS_ACCESS_KEY_ID or AWS_SNS_ACCESS_KEY_ID not set".to_string()))?;
        
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| std::env::var("AWS_SNS_SECRET_ACCESS_KEY"))
            .map_err(|_| InfrastructureError::Config("AWS_SECRET_ACCESS_KEY or AWS_SNS_SECRET_ACCESS_KEY not set".to_string()))?;
        
        let region = std::env::var("AWS_REGION")
            .or_else(|_| std::env::var("AWS_SNS_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string());
        
        let sender_id = std::env::var("AWS_SNS_SENDER_ID").ok();
        
        let sms_type = std::env::var("AWS_SNS_SMS_TYPE")
            .unwrap_or_else(|_| "Transactional".to_string());
        
        // Validate SMS type
        if sms_type != "Transactional" && sms_type != "Promotional" {
            return Err(InfrastructureError::Config(
                "AWS_SNS_SMS_TYPE must be either 'Transactional' or 'Promotional'".to_string()
            ));
        }
        
        Ok(Self {
            access_key_id,
            secret_access_key,
            region,
            sender_id,
            sms_type,
            max_retries: std::env::var("AWS_SNS_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            retry_delay_ms: std::env::var("AWS_SNS_RETRY_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            request_timeout_secs: std::env::var("AWS_SNS_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        })
    }
}

/// AWS SNS SMS service implementation
pub struct AwsSnsSmsService {
    client: SnsClient,
    config: AwsSnsConfig,
}

impl AwsSnsSmsService {
    /// Create a new AWS SNS SMS service
    pub async fn new(config: AwsSnsConfig) -> Result<Self, InfrastructureError> {
        // Create AWS credentials provider
        let credentials_provider = aws_credential_types::Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "aws_sns_sms_service",
        );
        
        // Create AWS configuration
        let region = Region::new(config.region.clone());
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(credentials_provider)
            .load()
            .await;
        
        // Create SNS client
        let client = SnsClient::new(&aws_config);
        
        info!(
            "AWS SNS SMS service initialized for region: {}",
            config.region
        );
        
        if let Some(ref sender_id) = config.sender_id {
            info!("Using sender ID: {}", sender_id);
        }
        
        Ok(Self { client, config })
    }
    
    /// Create from environment variables
    pub async fn from_env() -> Result<Self, InfrastructureError> {
        let config = AwsSnsConfig::from_env()?;
        Self::new(config).await
    }
    
    /// Validate and normalize phone number to E.164 format
    #[cfg(not(test))]
    fn validate_phone_number(&self, phone: &str) -> Result<String, InfrastructureError> {
        self.validate_phone_number_internal(phone)
    }
    
    /// Validate and normalize phone number to E.164 format (public for testing)
    #[cfg(test)]
    pub fn validate_phone_number(&self, phone: &str) -> Result<String, InfrastructureError> {
        self.validate_phone_number_internal(phone)
    }
    
    fn validate_phone_number_internal(&self, phone: &str) -> Result<String, InfrastructureError> {
        // If already in E.164 format, validate it
        if phone.starts_with('+') {
            // Try to parse with phonenumber crate for validation
            match phone.parse::<PhoneNumber>() {
                Ok(parsed) => {
                    let formatted = parsed.format().mode(Mode::E164).to_string();
                    debug!("Validated phone number: {}", mask_phone_number(&formatted));
                    Ok(formatted)
                }
                Err(e) => {
                    error!("Invalid phone number format: {}", e);
                    Err(InfrastructureError::Sms(format!(
                        "Invalid phone number format: {}",
                        e
                    )))
                }
            }
        } else {
            // Try to parse with a default country code (US)
            let with_country = format!("+1{}", phone);
            match with_country.parse::<PhoneNumber>() {
                Ok(parsed) => {
                    let formatted = parsed.format().mode(Mode::E164).to_string();
                    warn!(
                        "Phone number missing country code, assumed US: {}",
                        mask_phone_number(&formatted)
                    );
                    Ok(formatted)
                }
                Err(_) => {
                    Err(InfrastructureError::Sms(
                        "Phone number must be in E.164 format (e.g., +1234567890)".to_string()
                    ))
                }
            }
        }
    }
    
    /// Create SMS attributes for AWS SNS
    fn create_sms_attributes(&self) -> HashMap<String, MessageAttributeValue> {
        let mut attributes = HashMap::new();
        
        // Set SMS type (Transactional or Promotional)
        attributes.insert(
            "AWS.SNS.SMS.SMSType".to_string(),
            MessageAttributeValue::builder()
                .data_type("String".to_string())
                .string_value(&self.config.sms_type)
                .build()
                .unwrap(),
        );
        
        // Set sender ID if configured (not supported in all regions)
        if let Some(ref sender_id) = self.config.sender_id {
            attributes.insert(
                "AWS.SNS.SMS.SenderID".to_string(),
                MessageAttributeValue::builder()
                    .data_type("String".to_string())
                    .string_value(sender_id)
                    .build()
                    .unwrap(),
            );
        }
        
        attributes
    }
    
    /// Send SMS with retry logic
    async fn send_with_retry(
        &self,
        to: &str,
        message: &str,
    ) -> Result<String, InfrastructureError> {
        let mut attempts = 0;
        let mut delay = Duration::from_millis(self.config.retry_delay_ms);
        
        loop {
            attempts += 1;
            
            debug!(
                "Sending SMS attempt {}/{} to {} via AWS SNS",
                attempts,
                self.config.max_retries,
                mask_phone_number(to)
            );
            
            // Create the SMS attributes
            let attributes = self.create_sms_attributes();
            
            // Try to send the message
            let result = self.client
                .publish()
                .phone_number(to)
                .message(message)
                .set_message_attributes(Some(attributes))
                .send()
                .await;
            
            match result {
                Ok(response) => {
                    let message_id = response.message_id()
                        .unwrap_or("unknown")
                        .to_string();
                    
                    info!(
                        "SMS sent successfully to {} via AWS SNS with message ID: {}",
                        mask_phone_number(to),
                        message_id
                    );
                    
                    return Ok(message_id);
                }
                Err(e) => {
                    error!(
                        "Failed to send SMS via AWS SNS (attempt {}/{}): {}",
                        attempts, self.config.max_retries, e
                    );
                    
                    // Check if we should retry
                    if attempts >= self.config.max_retries {
                        return Err(InfrastructureError::Sms(format!(
                            "Failed to send SMS via AWS SNS after {} attempts: {}",
                            self.config.max_retries, e
                        )));
                    }
                    
                    // Check if the error is retryable
                    let error_msg = e.to_string();
                    
                    // AWS SNS specific error handling
                    if error_msg.contains("Throttling") || error_msg.contains("Rate exceeded") {
                        warn!("Rate limit detected, backing off for {:?}", delay);
                    } else if error_msg.contains("ServiceUnavailable") || 
                              error_msg.contains("InternalError") ||
                              error_msg.contains("RequestTimeout") {
                        warn!("Service error detected, retrying after {:?}", delay);
                    } else if error_msg.contains("InvalidParameter") || 
                              error_msg.contains("InvalidPhoneNumber") ||
                              error_msg.contains("ValidationError") {
                        // Don't retry on validation errors
                        return Err(InfrastructureError::Sms(format!(
                            "Invalid request to AWS SNS: {}",
                            e
                        )));
                    }
                    
                    // Wait before retrying with exponential backoff
                    tokio::time::sleep(delay).await;
                    delay = delay * 2; // Exponential backoff
                }
            }
        }
    }
}

#[async_trait]
impl SmsService for AwsSnsSmsService {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<String, InfrastructureError> {
        // Validate and normalize the phone number
        let normalized_phone = self.validate_phone_number(phone_number)?;
        
        // Log the message being sent (without sensitive data)
        info!(
            "Sending SMS to {} via AWS SNS (message length: {} chars)",
            mask_phone_number(&normalized_phone),
            message.len()
        );
        
        // Check message length (AWS SNS limit is 1600 characters for SMS)
        if message.len() > 1600 {
            return Err(InfrastructureError::Sms(
                "Message exceeds maximum length of 1600 characters".to_string()
            ));
        }
        
        // Send the message with retry logic
        self.send_with_retry(&normalized_phone, message).await
    }
    
    fn provider_name(&self) -> &str {
        "AWS SNS"
    }
    
    async fn is_available(&self) -> bool {
        // Perform a simple health check by attempting to list SMS attributes
        // This is a lightweight operation that verifies AWS credentials and connectivity
        match self.client
            .get_sms_attributes()
            .send()
            .await
        {
            Ok(_) => {
                debug!("AWS SNS health check passed");
                true
            }
            Err(e) => {
                warn!("AWS SNS health check failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phone_validation() {
        let config = AwsSnsConfig {
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
            region: "us-east-1".to_string(),
            sender_id: None,
            sms_type: "Transactional".to_string(),
            max_retries: 3,
            retry_delay_ms: 1000,
            request_timeout_secs: 30,
        };
        
        // Create a mock service for testing (we can't create real client in tests)
        // We'll test the validation logic directly
        let service = AwsSnsSmsService {
            client: unsafe { std::mem::zeroed() }, // This is just for testing validation
            config,
        };
        
        // Test valid E.164 format
        assert!(service.validate_phone_number("+14155552671").is_ok());
        
        // Test US number without country code - should add +1
        let result = service.validate_phone_number("4155552671");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "+14155552671");
        
        // Test invalid formats
        let result = service.validate_phone_number("12");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_config_from_env() {
        // Clean up any existing env vars first
        std::env::remove_var("AWS_SNS_MAX_RETRIES");
        std::env::remove_var("AWS_SNS_RETRY_DELAY_MS");
        std::env::remove_var("AWS_SNS_REQUEST_TIMEOUT_SECS");
        std::env::remove_var("AWS_SNS_SMS_TYPE");
        
        // Set environment variables
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
        std::env::set_var("AWS_REGION", "us-west-2");
        std::env::set_var("AWS_SNS_SENDER_ID", "RenovEasy");
        
        let config = AwsSnsConfig::from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.access_key_id, "test_key");
        assert_eq!(config.secret_access_key, "test_secret");
        assert_eq!(config.region, "us-west-2");
        assert_eq!(config.sender_id, Some("RenovEasy".to_string()));
        assert_eq!(config.sms_type, "Transactional");
        // These use default values since we didn't set env vars
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        
        // Clean up
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("AWS_SNS_SENDER_ID");
    }
    
    #[test]
    fn test_config_validation() {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
        std::env::set_var("AWS_SNS_SMS_TYPE", "InvalidType");
        
        let config = AwsSnsConfig::from_env();
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("'Transactional' or 'Promotional'"));
        
        // Clean up
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_SNS_SMS_TYPE");
    }
}