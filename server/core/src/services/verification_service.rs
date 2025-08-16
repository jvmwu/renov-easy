//! Verification service for SMS-based authentication
//!
//! This service integrates SMS sending and code caching to provide
//! a complete verification code workflow for phone number authentication.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::sync::Arc;

use crate::domain::entities::verification_code::{
    VerificationCode, CODE_LENGTH, DEFAULT_EXPIRATION_MINUTES, MAX_ATTEMPTS,
};
use crate::errors::{DomainError, DomainResult, ValidationError};

/// Configuration for the verification service
#[derive(Debug, Clone)]
pub struct VerificationServiceConfig {
    /// Number of minutes before a verification code expires
    pub code_expiration_minutes: i64,
    /// Maximum number of verification attempts allowed
    pub max_attempts: i32,
    /// Whether to use mock SMS service (for development)
    pub use_mock_sms: bool,
    /// Minimum seconds between code resend requests
    pub resend_cooldown_seconds: i64,
}

impl Default for VerificationServiceConfig {
    fn default() -> Self {
        Self {
            code_expiration_minutes: DEFAULT_EXPIRATION_MINUTES,
            max_attempts: MAX_ATTEMPTS,
            use_mock_sms: false,
            resend_cooldown_seconds: 60,
        }
    }
}

/// Trait for SMS service integration
#[async_trait]
pub trait SmsServiceTrait: Send + Sync {
    /// Send a verification code via SMS
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String>;
    /// Check if the phone number format is valid
    fn is_valid_phone_number(&self, phone: &str) -> bool;
}

/// Trait for cache service integration
#[async_trait]
pub trait CacheServiceTrait: Send + Sync {
    /// Store a verification code with expiration
    async fn store_code(&self, phone: &str, code: &str) -> Result<(), String>;
    /// Verify a code and track attempts
    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String>;
    /// Get remaining verification attempts
    async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String>;
    /// Check if a code exists for a phone number
    async fn code_exists(&self, phone: &str) -> Result<bool, String>;
    /// Get time-to-live for a verification code in seconds
    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String>;
    /// Clear verification data for a phone number
    async fn clear_verification(&self, phone: &str) -> Result<(), String>;
}

/// Result of sending a verification code
#[derive(Debug, Clone)]
pub struct SendCodeResult {
    /// The verification code entity that was created
    pub verification_code: VerificationCode,
    /// The SMS message ID from the provider
    pub message_id: String,
    /// When the user can request another code
    pub next_resend_at: DateTime<Utc>,
}

/// Result of verifying a code
#[derive(Debug, Clone)]
pub struct VerifyCodeResult {
    /// Whether the verification was successful
    pub success: bool,
    /// Number of remaining attempts (if verification failed)
    pub remaining_attempts: Option<i32>,
    /// Error message if verification failed
    pub error_message: Option<String>,
}

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

    /// Get the time-to-live for a verification code in seconds
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to check
    ///
    /// # Returns
    ///
    /// * `Ok(Option<i64>)` - TTL in seconds, None if code doesn't exist
    /// * `Err(DomainError)` - If the check fails
    pub async fn get_code_ttl(&self, phone: &str) -> DomainResult<Option<i64>> {
        self.cache_service
            .get_code_ttl(phone)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get code TTL: {}", e),
            })
    }

    /// Clear verification data for a phone number
    ///
    /// Useful for cleanup or when a user completes authentication.
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to clear
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If clearing succeeds
    /// * `Err(DomainError)` - If clearing fails
    pub async fn clear_verification(&self, phone: &str) -> DomainResult<()> {
        self.cache_service
            .clear_verification(phone)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to clear verification: {}", e),
            })
    }

    /// Generate a new 6-digit verification code
    ///
    /// This is a utility method that can be used independently if needed.
    ///
    /// # Returns
    ///
    /// A string containing a 6-digit verification code
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..1_000_000);
        format!("{:06}", code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock SMS service for testing
    struct MockSmsService {
        sent_messages: Arc<Mutex<HashMap<String, String>>>,
        should_fail: bool,
    }

    impl MockSmsService {
        fn new(should_fail: bool) -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(HashMap::new())),
                should_fail,
            }
        }

        fn get_sent_code(&self, phone: &str) -> Option<String> {
            self.sent_messages.lock().unwrap().get(phone).cloned()
        }
    }

    #[async_trait]
    impl SmsServiceTrait for MockSmsService {
        async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
            if self.should_fail {
                return Err("SMS service error".to_string());
            }
            self.sent_messages
                .lock()
                .unwrap()
                .insert(phone.to_string(), code.to_string());
            Ok(format!("mock-msg-{}", uuid::Uuid::new_v4()))
        }

        fn is_valid_phone_number(&self, phone: &str) -> bool {
            phone.starts_with('+') && phone.len() >= 10
        }
    }

    // Mock cache service for testing
    struct MockCacheService {
        codes: Arc<Mutex<HashMap<String, (String, i32)>>>, // phone -> (code, attempts)
        should_fail: bool,
    }

    impl MockCacheService {
        fn new(should_fail: bool) -> Self {
            Self {
                codes: Arc::new(Mutex::new(HashMap::new())),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl CacheServiceTrait for MockCacheService {
        async fn store_code(&self, phone: &str, code: &str) -> Result<(), String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            self.codes
                .lock()
                .unwrap()
                .insert(phone.to_string(), (code.to_string(), 0));
            Ok(())
        }

        async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            
            let mut codes = self.codes.lock().unwrap();
            if let Some((stored_code, attempts)) = codes.get_mut(phone) {
                *attempts += 1;
                if *attempts > MAX_ATTEMPTS {
                    return Ok(false);
                }
                if stored_code == code {
                    codes.remove(phone);
                    return Ok(true);
                }
            }
            Ok(false)
        }

        async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            let codes = self.codes.lock().unwrap();
            if let Some((_, attempts)) = codes.get(phone) {
                Ok((MAX_ATTEMPTS - attempts).max(0) as i64)
            } else {
                Ok(MAX_ATTEMPTS as i64)
            }
        }

        async fn code_exists(&self, phone: &str) -> Result<bool, String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            Ok(self.codes.lock().unwrap().contains_key(phone))
        }

        async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            if self.codes.lock().unwrap().contains_key(phone) {
                Ok(Some(300)) // Mock 5 minutes
            } else {
                Ok(None)
            }
        }

        async fn clear_verification(&self, phone: &str) -> Result<(), String> {
            if self.should_fail {
                return Err("Cache service error".to_string());
            }
            self.codes.lock().unwrap().remove(phone);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_send_verification_code_success() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let config = VerificationServiceConfig::default();
        
        let service = VerificationService::new(sms_service.clone(), cache_service.clone(), config);
        
        let result = service.send_verification_code("+1234567890").await;
        assert!(result.is_ok());
        
        let send_result = result.unwrap();
        assert_eq!(send_result.verification_code.phone, "+1234567890");
        assert_eq!(send_result.verification_code.code.len(), CODE_LENGTH);
        assert!(send_result.message_id.starts_with("mock-msg-"));
        
        // Verify code was sent via SMS
        let sent_code = sms_service.get_sent_code("+1234567890");
        assert_eq!(sent_code, Some(send_result.verification_code.code.clone()));
        
        // Verify code exists in cache
        assert!(cache_service.code_exists("+1234567890").await.unwrap());
    }

    #[tokio::test]
    async fn test_send_verification_code_invalid_phone() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let config = VerificationServiceConfig::default();
        
        let service = VerificationService::new(sms_service, cache_service, config);
        
        let result = service.send_verification_code("1234567890").await; // Missing +
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DomainError::Validation { message } => {
                assert!(message.contains("Invalid phone number format"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_verify_code_success() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let config = VerificationServiceConfig::default();
        
        let service = VerificationService::new(sms_service, cache_service.clone(), config);
        
        // Send a code first
        let send_result = service.send_verification_code("+1234567890").await.unwrap();
        let code = send_result.verification_code.code;
        
        // Verify the correct code
        let verify_result = service.verify_code("+1234567890", &code).await.unwrap();
        assert!(verify_result.success);
        assert!(verify_result.error_message.is_none());
        assert!(verify_result.remaining_attempts.is_none());
        
        // Code should be cleared after successful verification
        assert!(!cache_service.code_exists("+1234567890").await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_code_wrong_code() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let config = VerificationServiceConfig::default();
        
        let service = VerificationService::new(sms_service, cache_service.clone(), config);
        
        // Send a code first
        service.send_verification_code("+1234567890").await.unwrap();
        
        // Verify with wrong code
        let verify_result = service.verify_code("+1234567890", "000000").await.unwrap();
        assert!(!verify_result.success);
        assert!(verify_result.error_message.is_some());
        assert_eq!(verify_result.remaining_attempts, Some(2)); // MAX_ATTEMPTS - 1
        
        // Code should still exist after failed verification
        assert!(cache_service.code_exists("+1234567890").await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_code_invalid_format() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let config = VerificationServiceConfig::default();
        
        let service = VerificationService::new(sms_service, cache_service, config);
        
        // Try to verify with invalid format codes
        let result1 = service.verify_code("+1234567890", "12345").await.unwrap(); // Too short
        assert!(!result1.success);
        assert!(result1.error_message.unwrap().contains("Invalid verification code format"));
        
        let result2 = service.verify_code("+1234567890", "12345a").await.unwrap(); // Contains letter
        assert!(!result2.success);
        assert!(result2.error_message.unwrap().contains("Invalid verification code format"));
    }

    #[tokio::test]
    async fn test_generate_code() {
        // Test code generation multiple times
        for _ in 0..100 {
            let code = VerificationService::<MockSmsService, MockCacheService>::generate_code();
            assert_eq!(code.len(), CODE_LENGTH);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
            
            let num: u32 = code.parse().unwrap();
            assert!(num < 1_000_000);
        }
    }

    #[tokio::test]
    async fn test_cooldown_period() {
        let sms_service = Arc::new(MockSmsService::new(false));
        let cache_service = Arc::new(MockCacheService::new(false));
        let mut config = VerificationServiceConfig::default();
        config.resend_cooldown_seconds = 60;
        
        let service = VerificationService::new(sms_service, cache_service, config);
        
        // Send first code
        let result1 = service.send_verification_code("+1234567890").await;
        assert!(result1.is_ok());
        
        // Try to send another code immediately (should fail due to cooldown)
        let result2 = service.send_verification_code("+1234567890").await;
        assert!(result2.is_err());
        
        match result2.unwrap_err() {
            DomainError::ValidationErr(ValidationError::RateLimitExceeded { .. }) => {}
            _ => panic!("Expected rate limit error"),
        }
    }
}