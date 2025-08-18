//! Main verification service implementation

use chrono::Utc;
use rand::Rng;
use std::sync::Arc;

use crate::domain::entities::verification_code::{VerificationCode, CODE_LENGTH};
use crate::errors::{DomainError, DomainResult, ValidationError};

use super::config::VerificationServiceConfig;
use super::traits::{SmsServiceTrait, CacheServiceTrait};
use super::types::{SendCodeResult, VerifyCodeResult};

/// Verification service for handling SMS verification codes
pub struct VerificationService<S: SmsServiceTrait, C: CacheServiceTrait> {
    /// SMS service for sending messages
    sms_service: Arc<S>,
    /// Cache service for storing codes
    cache_service: Arc<C>,
    /// Service configuration
    config: VerificationServiceConfig,
}

impl<S: SmsServiceTrait, C: CacheServiceTrait> VerificationService<S, C> {
    /// Create a new verification service
    ///
    /// # Arguments
    ///
    /// * `sms_service` - SMS service implementation
    /// * `cache_service` - Cache service implementation
    /// * `config` - Service configuration
    pub fn new(
        sms_service: Arc<S>,
        cache_service: Arc<C>,
        config: VerificationServiceConfig,
    ) -> Self {
        Self {
            sms_service,
            cache_service,
            config,
        }
    }

    /// Send a verification code to a phone number
    ///
    /// This method:
    /// 1. Validates the phone number format
    /// 2. Checks for existing codes and cooldown periods
    /// 3. Generates a new verification code
    /// 4. Stores the code in cache
    /// 5. Sends the code via SMS
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to send the code to (E.164 format)
    ///
    /// # Returns
    ///
    /// * `Ok(SendCodeResult)` - Result containing the verification code and SMS details
    /// * `Err(DomainError)` - If validation fails or sending fails
    pub async fn send_verification_code(&self, phone: &str) -> DomainResult<SendCodeResult> {
        // Validate phone number format
        if !self.sms_service.is_valid_phone_number(phone) {
            return Err(DomainError::Validation {
                message: format!("Invalid phone number format: {}", phone),
            });
        }

        // Check if a code already exists and is still valid
        if let Ok(true) = self.cache_service.code_exists(phone).await {
            // Check TTL to see if we're still in cooldown
            if let Ok(Some(ttl)) = self.cache_service.get_code_ttl(phone).await {
                let cooldown_remaining = ttl - (self.config.code_expiration_minutes * 60 - self.config.resend_cooldown_seconds);
                if cooldown_remaining > 0 {
                    return Err(DomainError::ValidationErr(ValidationError::RateLimitExceeded {
                        message_en: format!(
                            "Please wait {} seconds before requesting a new code",
                            cooldown_remaining
                        ),
                        message_zh: format!("请等待 {} 秒后再请求新的验证码", cooldown_remaining),
                        limit: 1,
                        window_seconds: self.config.resend_cooldown_seconds as u64,
                    }));
                }
            }
            
            // Clear existing code if cooldown has passed
            let _ = self.cache_service.clear_verification(phone).await;
        }

        // Generate new verification code
        let verification_code = VerificationCode::new_with_expiration(
            phone.to_string(),
            self.config.code_expiration_minutes,
        );

        // Store code in cache
        self.cache_service
            .store_code(phone, &verification_code.code)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to store verification code: {}", e),
            })?;

        // Send SMS
        let message_id = self
            .sms_service
            .send_verification_code(phone, &verification_code.code)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to send SMS: {}", e),
            })?;

        // Calculate next resend time
        let next_resend_at = Utc::now() + chrono::Duration::seconds(self.config.resend_cooldown_seconds);

        Ok(SendCodeResult {
            verification_code,
            message_id,
            next_resend_at,
        })
    }

    /// Verify a verification code
    ///
    /// This method:
    /// 1. Validates the code format
    /// 2. Checks the code against the cached value
    /// 3. Tracks verification attempts
    /// 4. Clears the code on successful verification
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number associated with the code
    /// * `code` - The verification code to verify
    ///
    /// # Returns
    ///
    /// * `Ok(VerifyCodeResult)` - Result containing verification status and details
    /// * `Err(DomainError)` - If verification fails due to system error
    pub async fn verify_code(&self, phone: &str, code: &str) -> DomainResult<VerifyCodeResult> {
        // Validate code format
        if code.len() != CODE_LENGTH || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(VerifyCodeResult {
                success: false,
                remaining_attempts: None,
                error_message: Some("Invalid verification code format".to_string()),
            });
        }

        // Verify code with cache service
        match self.cache_service.verify_code(phone, code).await {
            Ok(true) => {
                // Successful verification
                Ok(VerifyCodeResult {
                    success: true,
                    remaining_attempts: None,
                    error_message: None,
                })
            }
            Ok(false) => {
                // Failed verification - get remaining attempts
                let remaining = self
                    .cache_service
                    .get_remaining_attempts(phone)
                    .await
                    .unwrap_or(0) as i32;

                let error_message = if remaining > 0 {
                    format!("Invalid verification code. {} attempts remaining", remaining)
                } else {
                    "Maximum verification attempts exceeded".to_string()
                };

                Ok(VerifyCodeResult {
                    success: false,
                    remaining_attempts: Some(remaining),
                    error_message: Some(error_message),
                })
            }
            Err(e) => {
                // System error
                Err(DomainError::Internal {
                    message: format!("Failed to verify code: {}", e),
                })
            }
        }
    }

    /// Get the remaining verification attempts for a phone number
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to check
    ///
    /// # Returns
    ///
    /// * `Ok(i32)` - Number of remaining attempts
    /// * `Err(DomainError)` - If the check fails
    pub async fn get_remaining_attempts(&self, phone: &str) -> DomainResult<i32> {
        self.cache_service
            .get_remaining_attempts(phone)
            .await
            .map(|a| a as i32)
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get remaining attempts: {}", e),
            })
    }

    /// Check if a verification code exists for a phone number
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to check
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - True if a code exists and hasn't expired
    /// * `Err(DomainError)` - If the check fails
    pub async fn code_exists(&self, phone: &str) -> DomainResult<bool> {
        self.cache_service
            .code_exists(phone)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to check code existence: {}", e),
            })
    }

    /// Clear verification data for a phone number
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to clear
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If clearing was successful
    /// * `Err(DomainError)` - If clearing fails
    pub async fn clear_verification(&self, phone: &str) -> DomainResult<()> {
        self.cache_service
            .clear_verification(phone)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to clear verification: {}", e),
            })
    }

    /// Generate a random verification code
    ///
    /// # Returns
    ///
    /// A random 6-digit verification code as a string
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100000..999999);
        format!("{:06}", code)
    }
}