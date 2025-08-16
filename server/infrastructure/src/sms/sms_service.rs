//! SMS Service Interface
//!
//! Defines the trait for SMS service implementations that handle
//! sending verification codes and other SMS messages.

use async_trait::async_trait;
use crate::InfrastructureError;

/// SMS service trait for sending text messages
///
/// Implementations include:
/// - Twilio SMS API
/// - AWS SNS
/// - Mock implementation for development
#[async_trait]
pub trait SmsService: Send + Sync {
    /// Send an SMS message to a phone number
    ///
    /// # Arguments
    ///
    /// * `phone_number` - The recipient's phone number (E.164 format)
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Ok(message_id)` - Unique identifier for the sent message
    /// * `Err(InfrastructureError)` - If sending fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let service = MockSmsService::new();
    /// let message_id = service.send_sms("+1234567890", "Your code is 123456").await?;
    /// ```
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<String, InfrastructureError>;

    /// Send a verification code via SMS
    ///
    /// This is a convenience method that formats the verification code message
    /// according to the application's standard format.
    ///
    /// # Arguments
    ///
    /// * `phone_number` - The recipient's phone number (E.164 format)
    /// * `code` - The verification code to send
    ///
    /// # Returns
    ///
    /// * `Ok(message_id)` - Unique identifier for the sent message
    /// * `Err(InfrastructureError)` - If sending fails
    async fn send_verification_code(&self, phone_number: &str, code: &str) -> Result<String, InfrastructureError> {
        let message = format!("Your RenovEasy verification code is: {}. This code will expire in 5 minutes.", code);
        self.send_sms(phone_number, &message).await
    }

    /// Get the service provider name
    ///
    /// Returns the name of the SMS service provider (e.g., "Twilio", "AWS SNS", "Mock")
    fn provider_name(&self) -> &str;

    /// Check if the service is available
    ///
    /// Performs a health check on the SMS service.
    /// Default implementation always returns true.
    async fn is_available(&self) -> bool {
        true
    }
}

/// Helper function to mask phone numbers for logging
///
/// Shows only the last 4 digits of the phone number for security.
///
/// # Example
///
/// ```ignore
/// let masked = mask_phone_number("+1234567890");
/// assert_eq!(masked, "+******7890");
/// ```
pub fn mask_phone_number(phone: &str) -> String {
    if phone.len() <= 4 {
        return "*".repeat(phone.len());
    }
    
    let visible_digits = 4;
    let masked_count = phone.len() - visible_digits;
    let last_digits = &phone[phone.len() - visible_digits..];
    
    if phone.starts_with('+') {
        format!("+{}{}", "*".repeat(masked_count - 1), last_digits)
    } else {
        format!("{}{}", "*".repeat(masked_count), last_digits)
    }
}

/// Validate phone number format (E.164)
///
/// Checks if the phone number is in valid E.164 format:
/// - Starts with '+'
/// - Contains only digits after '+'
/// - Length between 10 and 15 digits (excluding '+')
///
/// # Example
///
/// ```ignore
/// assert!(is_valid_phone_number("+1234567890"));
/// assert!(!is_valid_phone_number("1234567890")); // Missing '+'
/// ```
pub fn is_valid_phone_number(phone: &str) -> bool {
    if !phone.starts_with('+') {
        return false;
    }
    
    let digits = &phone[1..];
    if digits.is_empty() || digits.len() < 10 || digits.len() > 15 {
        return false;
    }
    
    digits.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone_number() {
        assert_eq!(mask_phone_number("+1234567890"), "+******7890");
        assert_eq!(mask_phone_number("+12345678901234"), "+**********1234");
        assert_eq!(mask_phone_number("1234567890"), "******7890");
        assert_eq!(mask_phone_number("123"), "***");
        assert_eq!(mask_phone_number("1234"), "****");
    }

    #[test]
    fn test_is_valid_phone_number() {
        // Valid numbers
        assert!(is_valid_phone_number("+1234567890"));
        assert!(is_valid_phone_number("+12345678901"));
        assert!(is_valid_phone_number("+123456789012345"));
        
        // Invalid numbers
        assert!(!is_valid_phone_number("1234567890")); // No plus
        assert!(!is_valid_phone_number("+123")); // Too short
        assert!(!is_valid_phone_number("+1234567890123456")); // Too long
        assert!(!is_valid_phone_number("+123abc4567")); // Contains letters
        assert!(!is_valid_phone_number("+")); // Only plus sign
    }
}