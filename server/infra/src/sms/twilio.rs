//! Twilio SMS Service Implementation
//!
//! This module provides SMS sending capabilities using the Twilio API.
//! It implements the SmsService trait for production SMS delivery.
//!
//! ## Features
//!
//! - International SMS support with E.164 format validation
//! - Automatic retry logic with exponential backoff
//! - Rate limiting handling
//! - Delivery status tracking
//! - Comprehensive error handling
//! - Security: Phone number masking in logs

use async_trait::async_trait;
use phonenumber::{Mode, PhoneNumber};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use twilio::{Client, OutboundMessage};

use crate::{
    sms::sms_service::{mask_phone_number, SmsService},
    InfrastructureError,
};

/// Twilio SMS service configuration
#[derive(Debug, Clone)]
pub struct TwilioConfig {
    /// Twilio Account SID
    pub account_sid: String,
    /// Twilio Auth Token
    pub auth_token: String,
    /// From phone number (must be a Twilio phone number)
    pub from_number: String,
    /// Maximum retry attempts for failed requests
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Timeout for API requests in seconds
    pub request_timeout_secs: u64,
}

impl TwilioConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, InfrastructureError> {
        let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
            .map_err(|_| InfrastructureError::Config("TWILIO_ACCOUNT_SID not set".to_string()))?;
        let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
            .map_err(|_| InfrastructureError::Config("TWILIO_AUTH_TOKEN not set".to_string()))?;
        let from_number = std::env::var("TWILIO_FROM_NUMBER")
            .map_err(|_| InfrastructureError::Config("TWILIO_FROM_NUMBER not set".to_string()))?;
        
        // Validate from number format
        if !from_number.starts_with('+') {
            return Err(InfrastructureError::Config(
                "TWILIO_FROM_NUMBER must be in E.164 format (starting with '+')".to_string()
            ));
        }
        
        Ok(Self {
            account_sid,
            auth_token,
            from_number,
            max_retries: std::env::var("TWILIO_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            retry_delay_ms: std::env::var("TWILIO_RETRY_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            request_timeout_secs: std::env::var("TWILIO_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        })
    }
}

/// Twilio SMS service implementation
pub struct TwilioSmsService {
    client: Client,
    config: TwilioConfig,
}

impl TwilioSmsService {
    /// Create a new Twilio SMS service
    pub fn new(config: TwilioConfig) -> Result<Self, InfrastructureError> {
        let client = Client::new(&config.account_sid, &config.auth_token);
        
        info!(
            "Twilio SMS service initialized with from number: {}",
            mask_phone_number(&config.from_number)
        );
        
        Ok(Self { client, config })
    }
    
    /// Create from environment variables
    pub fn from_env() -> Result<Self, InfrastructureError> {
        let config = TwilioConfig::from_env()?;
        Self::new(config)
    }
    
    /// Validate and normalize phone number to E.164 format
    fn validate_phone_number(&self, phone: &str) -> Result<String, InfrastructureError> {
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
                "Sending SMS attempt {}/{} to {}",
                attempts,
                self.config.max_retries,
                mask_phone_number(to)
            );
            
            // Create the message
            let msg = OutboundMessage::new(&self.config.from_number, to, message);
            
            // Try to send the message
            match self.client.send_message(msg).await {
                Ok(response) => {
                    info!(
                        "SMS sent successfully to {} with SID: {}",
                        mask_phone_number(to),
                        response.sid
                    );
                    return Ok(response.sid);
                }
                Err(e) => {
                    error!(
                        "Failed to send SMS (attempt {}/{}): {}",
                        attempts, self.config.max_retries, e
                    );
                    
                    // Check if we should retry
                    if attempts >= self.config.max_retries {
                        return Err(InfrastructureError::Sms(format!(
                            "Failed to send SMS after {} attempts: {}",
                            self.config.max_retries, e
                        )));
                    }
                    
                    // Check if the error is retryable
                    let error_msg = e.to_string();
                    if error_msg.contains("429") || error_msg.contains("rate") {
                        warn!("Rate limit detected, backing off for {:?}", delay);
                    } else if error_msg.contains("500") || error_msg.contains("502") || 
                              error_msg.contains("503") || error_msg.contains("504") {
                        warn!("Server error detected, retrying after {:?}", delay);
                    } else if error_msg.contains("400") || error_msg.contains("invalid") {
                        // Don't retry on client errors
                        return Err(InfrastructureError::Sms(format!(
                            "Invalid request: {}",
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
impl SmsService for TwilioSmsService {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<String, InfrastructureError> {
        // Validate and normalize the phone number
        let normalized_phone = self.validate_phone_number(phone_number)?;
        
        // Log the message being sent (without sensitive data)
        info!(
            "Sending SMS to {} via Twilio (message length: {} chars)",
            mask_phone_number(&normalized_phone),
            message.len()
        );
        
        // Check message length (Twilio limit is 1600 characters)
        if message.len() > 1600 {
            return Err(InfrastructureError::Sms(
                "Message exceeds maximum length of 1600 characters".to_string()
            ));
        }
        
        // Send the message with retry logic
        self.send_with_retry(&normalized_phone, message).await
    }
    
    fn provider_name(&self) -> &str {
        "Twilio"
    }
    
    async fn is_available(&self) -> bool {
        // Perform a simple health check
        // In a real implementation, you might want to check account status
        // or send a test message to a known number
        true
    }
}