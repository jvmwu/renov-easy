//! Authentication service for handling user authentication flows
//!
//! This service coordinates the authentication process including:
//! - Phone number verification via SMS
//! - User registration and login
//! - Token generation and refresh
//! - User type selection

use async_trait::async_trait;
use std::sync::Arc;
use crate::domain::entities::user::User;
use crate::domain::value_objects::AuthResponse;
use crate::errors::{AuthError, DomainError, DomainResult, ValidationError};
use crate::repositories::{UserRepository, TokenRepository};
use crate::services::verification_service::{
    VerificationService, SmsServiceTrait, CacheServiceTrait, SendCodeResult,
};
use crate::services::token_service::TokenService;
use sha2::{Sha256, Digest};

/// Configuration for the authentication service
#[derive(Debug, Clone)]
pub struct AuthServiceConfig {
    /// Maximum SMS requests per phone number per hour
    pub max_sms_per_hour: i64,
    /// Hour duration in seconds for rate limiting
    pub rate_limit_window_seconds: i64,
    /// Whether to allow registration of new users
    pub allow_registration: bool,
    /// Whether to require user type selection immediately after registration
    pub require_immediate_user_type: bool,
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            max_sms_per_hour: 3,
            rate_limit_window_seconds: 3600, // 1 hour
            allow_registration: true,
            require_immediate_user_type: false,
        }
    }
}

/// Rate limiting service trait for tracking SMS requests
#[async_trait]
pub trait RateLimiterTrait: Send + Sync {
    /// Check if a phone number has exceeded the rate limit for SMS requests
    async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String>;
    
    /// Increment the SMS request counter for a phone number
    async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String>;
    
    /// Get the remaining time until rate limit resets (in seconds)
    async fn get_rate_limit_reset_time(&self, phone: &str) -> Result<Option<i64>, String>;
}

/// Authentication service for managing the complete authentication flow
pub struct AuthService<U, S, C, R, T> 
where
    U: UserRepository,
    S: SmsServiceTrait,
    C: CacheServiceTrait,
    R: RateLimiterTrait,
    T: TokenRepository,
{
    /// User repository for database operations
    user_repository: Arc<U>,
    /// Verification service for SMS code handling
    verification_service: Arc<VerificationService<S, C>>,
    /// Rate limiter for preventing abuse
    rate_limiter: Arc<R>,
    /// Token service for JWT management
    token_service: Arc<TokenService<T>>,
    /// Service configuration
    config: AuthServiceConfig,
}

impl<U, S, C, R, T> AuthService<U, S, C, R, T>
where
    U: UserRepository,
    S: SmsServiceTrait,
    C: CacheServiceTrait,
    R: RateLimiterTrait,
    T: TokenRepository,
{
    /// Create a new authentication service
    ///
    /// # Arguments
    ///
    /// * `user_repository` - Repository for user data persistence
    /// * `verification_service` - Service for SMS verification
    /// * `rate_limiter` - Service for rate limiting
    /// * `token_service` - Service for JWT token management
    /// * `config` - Service configuration
    pub fn new(
        user_repository: Arc<U>,
        verification_service: Arc<VerificationService<S, C>>,
        rate_limiter: Arc<R>,
        token_service: Arc<TokenService<T>>,
        config: AuthServiceConfig,
    ) -> Self {
        Self {
            user_repository,
            verification_service,
            rate_limiter,
            token_service,
            config,
        }
    }

    /// Send a verification code to a phone number
    ///
    /// This method:
    /// 1. Validates the phone number format
    /// 2. Checks rate limiting (3 requests per hour)
    /// 3. Delegates to verification service for code generation and sending
    /// 4. Increments rate limit counter
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to send the code to (E.164 format)
    ///
    /// # Returns
    ///
    /// * `Ok(SendCodeResult)` - Result containing verification details and next resend time
    /// * `Err(DomainError)` - If validation fails, rate limit exceeded, or sending fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use renov_core::services::auth_service::AuthService;
    /// 
    /// async fn send_code(auth_service: &AuthService) {
    ///     match auth_service.send_verification_code("+1234567890").await {
    ///         Ok(result) => {
    ///             println!("Code sent! Message ID: {}", result.message_id);
    ///             println!("Can resend at: {}", result.next_resend_at);
    ///         }
    ///         Err(e) => eprintln!("Failed to send code: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn send_verification_code(&self, phone: &str) -> DomainResult<SendCodeResult> {
        // Step 1: Validate phone number format
        if !self.is_valid_phone_format(phone) {
            return Err(DomainError::Auth(AuthError::InvalidPhoneFormat {
                phone: Self::mask_phone(phone),
            }));
        }

        // Step 2: Check rate limiting (3 times per hour per phone number)
        let rate_limit_exceeded = self.rate_limiter
            .check_sms_rate_limit(phone)
            .await
            .map_err(|e| {
                DomainError::Internal {
                    message: format!("Failed to check rate limit: {}", e),
                }
            })?;

        if rate_limit_exceeded {
            // Get time until rate limit resets
            let reset_time = self.rate_limiter
                .get_rate_limit_reset_time(phone)
                .await
                .unwrap_or(Some(3600))
                .unwrap_or(3600);
            
            let minutes = (reset_time / 60).max(1) as u32;
            
            return Err(DomainError::Auth(AuthError::RateLimitExceeded { minutes }));
        }

        // Step 3: Delegate to verification service to send the code
        let send_result = self.verification_service
            .send_verification_code(phone)
            .await
            .map_err(|e| {
                match e {
                    DomainError::ValidationErr(ValidationError::RateLimitExceeded { .. }) => {
                        // This is the cooldown period check from verification service
                        // Convert to auth error for consistency
                        DomainError::Auth(AuthError::RateLimitExceeded { minutes: 1 })
                    }
                    DomainError::Internal { message } if message.contains("SMS") => {
                        DomainError::Auth(AuthError::SmsServiceFailure)
                    }
                    _ => e,
                }
            })?;

        // Step 4: Increment rate limit counter after successful send
        let _count = self.rate_limiter
            .increment_sms_counter(phone)
            .await
            .unwrap_or(1);

        Ok(send_result)
    }

    /// Validate phone number format
    ///
    /// Checks if the phone number is in valid E.164 format.
    /// Delegates to the SMS service for validation logic.
    ///
    /// # Arguments
    ///
    /// * `phone` - Phone number to validate
    ///
    /// # Returns
    ///
    /// * `bool` - True if valid, false otherwise
    fn is_valid_phone_format(&self, phone: &str) -> bool {
        // Check basic E.164 format requirements
        if !phone.starts_with('+') {
            return false;
        }

        // Must be between 10 and 15 digits after the +
        let digits = &phone[1..];
        if digits.len() < 10 || digits.len() > 15 {
            return false;
        }

        // All characters after + must be digits
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        true
    }

    /// Verify a verification code for a phone number
    ///
    /// This method:
    /// 1. Validates the phone number format
    /// 2. Delegates to verification service to verify the code
    /// 3. Looks up or creates the user upon successful verification
    /// 4. Updates user login timestamp
    /// 5. Generates JWT tokens for authentication
    /// 6. Returns authentication response with tokens and user type information
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number associated with the code (E.164 format)
    /// * `code` - The verification code to verify
    ///
    /// # Returns
    ///
    /// * `Ok(AuthResponse)` - Authentication response with tokens and user information
    /// * `Err(DomainError)` - If verification fails, user is blocked, or other errors occur
    ///
    /// # Example
    ///
    /// ```no_run
    /// use renov_core::services::auth_service::AuthService;
    /// 
    /// async fn verify_code(auth_service: &AuthService) {
    ///     match auth_service.verify_code("+1234567890", "123456").await {
    ///         Ok(response) => {
    ///             println!("Authentication successful!");
    ///             println!("Access token: {}", response.access_token);
    ///             if response.requires_type_selection {
    ///                 println!("User needs to select their type");
    ///             }
    ///         },
    ///         Err(e) => eprintln!("Verification failed: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn verify_code(&self, phone: &str, code: &str) -> DomainResult<AuthResponse> {
        // Step 1: Validate phone number format
        if !self.is_valid_phone_format(phone) {
            return Err(DomainError::Auth(AuthError::InvalidPhoneFormat {
                phone: Self::mask_phone(phone),
            }));
        }

        // Step 2: Delegate to verification service to verify the code
        let verify_result = self.verification_service
            .verify_code(phone, code)
            .await?;

        // Step 3: Process verification result
        if verify_result.success {
            // Verification successful - proceed with user operations
            
            // Extract country code and phone number parts
            let (country_code, phone_without_code) = Self::extract_country_code(phone);
            
            // Hash the phone number for storage
            let phone_hash = Self::hash_phone(&phone_without_code);
            
            // Step 4: Look up existing user or create new one
            let mut user = match self.user_repository
                .find_by_phone(&phone_hash, &country_code)
                .await
                .map_err(|e| {
                    DomainError::Internal {
                        message: format!("Failed to query user: {}", e),
                    }
                })?
            {
                Some(existing_user) => {
                    // User exists - check if they are blocked
                    if existing_user.is_blocked {
                        return Err(DomainError::Auth(AuthError::UserBlocked));
                    }
                    existing_user
                }
                None => {
                    // New user - check if registration is allowed
                    if !self.config.allow_registration {
                        return Err(DomainError::Auth(AuthError::RegistrationDisabled));
                    }
                    
                    // Create new user
                    let mut new_user = User::new(phone_hash.clone(), country_code.clone());
                    new_user.verify(); // Mark as verified since they completed phone verification
                    
                    // Save the new user to the repository
                    self.user_repository
                        .create(new_user)
                        .await
                        .map_err(|e| {
                            DomainError::Internal {
                                message: format!("Failed to create user: {}", e),
                            }
                        })?
                }
            };
            
            // Step 5: Update user state
            // Mark as verified if not already (for existing users who may not have been verified)
            if !user.is_verified {
                user.verify();
            }
            
            // Update last login timestamp
            user.update_last_login();
            
            // Save the updated user
            let _updated_user = self.user_repository
                .update(user)
                .await
                .map_err(|e| {
                    DomainError::Internal {
                        message: format!("Failed to update user: {}", e),
                    }
                })?;
            
            // Clear the verification code from cache now that it's been used
            let _ = self.verification_service
                .clear_verification(phone)
                .await;
            
            // Step 6: Generate JWT tokens
            let token_pair = self.token_service
                .generate_tokens(
                    _updated_user.id,
                    _updated_user.user_type.clone(),
                    _updated_user.is_verified,
                )
                .await?;
            
            // Step 7: Create and return authentication response
            let auth_response = AuthResponse::from_token_pair(
                token_pair,
                _updated_user.user_type,
            );
            
            Ok(auth_response)
        } else {
            // Verification failed - map to appropriate auth error
            match verify_result.remaining_attempts {
                Some(0) => {
                    // No more attempts remaining
                    Err(DomainError::Auth(AuthError::MaxAttemptsExceeded))
                }
                Some(_remaining) => {
                    // Still have attempts remaining
                    Err(DomainError::Auth(AuthError::InvalidVerificationCode))
                }
                None => {
                    // Code might have expired or doesn't exist
                    if verify_result.error_message.as_ref()
                        .map(|msg| msg.contains("format"))
                        .unwrap_or(false) {
                        Err(DomainError::Auth(AuthError::InvalidVerificationCode))
                    } else {
                        // Assume expired if no specific error
                        Err(DomainError::Auth(AuthError::VerificationCodeExpired))
                    }
                }
            }
        }
    }

    /// Mask phone number for logging (show only last 4 digits)
    ///
    /// # Arguments
    ///
    /// * `phone` - Phone number to mask
    ///
    /// # Returns
    ///
    /// * `String` - Masked phone number
    fn mask_phone(phone: &str) -> String {
        if phone.len() <= 4 {
            return "*".repeat(phone.len());
        }
        format!("***{}", &phone[phone.len() - 4..])
    }
    
    /// Hash a phone number using SHA-256
    ///
    /// # Arguments
    ///
    /// * `phone` - Phone number to hash (without country code)
    ///
    /// # Returns
    ///
    /// * `String` - Hexadecimal representation of SHA-256 hash
    fn hash_phone(phone: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(phone.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    /// Select user type for a user after registration
    ///
    /// This method:
    /// 1. Validates that the user exists
    /// 2. Checks that the user hasn't already selected a type
    /// 3. Updates the user's type (Customer or Worker)
    /// 4. Persists the change to the repository
    ///
    /// # Arguments
    ///
    /// * `user_id` - The UUID of the user
    /// * `user_type` - The type to set (Customer or Worker)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the user type was successfully updated
    /// * `Err(DomainError)` - If user not found, already has a type, or update fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use renov_core::services::auth_service::AuthService;
    /// use renov_core::domain::entities::user::UserType;
    /// use uuid::Uuid;
    /// 
    /// async fn select_type(auth_service: &AuthService, user_id: Uuid) {
    ///     match auth_service.select_user_type(user_id, UserType::Customer).await {
    ///         Ok(()) => println!("User type selected successfully"),
    ///         Err(e) => eprintln!("Failed to select user type: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn select_user_type(
        &self, 
        user_id: uuid::Uuid, 
        user_type: crate::domain::entities::user::UserType
    ) -> DomainResult<()> {
        // Step 1: Fetch the user from the repository
        let mut user = self.user_repository
            .find_by_id(user_id)
            .await
            .map_err(|e| {
                DomainError::Internal {
                    message: format!("Failed to query user: {}", e),
                }
            })?
            .ok_or_else(|| DomainError::Auth(AuthError::UserNotFound))?;
        
        // Step 2: Check if the user already has a type selected
        if user.has_user_type() {
            // User already has a type, cannot change it
            return Err(DomainError::Auth(AuthError::InsufficientPermissions));
        }
        
        // Step 3: Set the user type
        user.set_user_type(user_type);
        
        // Step 4: Update the user in the repository
        self.user_repository
            .update(user)
            .await
            .map_err(|e| {
                DomainError::Internal {
                    message: format!("Failed to update user type: {}", e),
                }
            })?;
        
        Ok(())
    }

    /// Extract country code from a full phone number
    ///
    /// # Arguments
    ///
    /// * `phone` - Full phone number in E.164 format (e.g., +1234567890)
    ///
    /// # Returns
    ///
    /// * `(String, String)` - Tuple of (country_code, phone_without_country_code)
    ///
    /// # Note
    ///
    /// This is a simple implementation that handles common country codes.
    /// For production, consider using a proper phone number parsing library.
    fn extract_country_code(phone: &str) -> (String, String) {
        // Common country codes by length
        // 1 digit: +1 (US/Canada), +7 (Russia)
        // 2 digits: +86 (China), +61 (Australia), +44 (UK), etc.
        // 3 digits: +358 (Finland), +972 (Israel), etc.
        
        // Try common patterns
        if phone.starts_with("+1") && phone.len() == 11 {
            // US/Canada
            ("+1".to_string(), phone[2..].to_string())
        } else if phone.starts_with("+86") {
            // China
            ("+86".to_string(), phone[3..].to_string())
        } else if phone.starts_with("+61") {
            // Australia
            ("+61".to_string(), phone[3..].to_string())
        } else if phone.starts_with("+44") {
            // UK
            ("+44".to_string(), phone[3..].to_string())
        } else if phone.starts_with("+7") && phone.len() == 12 {
            // Russia
            ("+7".to_string(), phone[2..].to_string())
        } else {
            // Default: assume 2-digit country code for now
            // This is a simplification and should be improved with a proper library
            if phone.len() > 3 && phone[1..3].chars().all(|c| c.is_ascii_digit()) {
                (phone[0..3].to_string(), phone[3..].to_string())
            } else {
                // Fallback to single digit
                (phone[0..2].to_string(), phone[2..].to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use crate::services::verification_service::{
        VerificationServiceConfig,
    };
    use crate::services::token_service::{TokenService, TokenServiceConfig};
    use crate::domain::entities::user::{User, UserType};
    use crate::repositories::token_repository::mock::MockTokenRepository;
    use uuid::Uuid;

    // Mock implementations for testing
    struct MockUserRepository {
        users: Arc<Mutex<Vec<User>>>,
    }
    
    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn with_existing_user(user: User) -> Self {
            let repo = Self::new();
            repo.users.lock().unwrap().push(user);
            repo
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_phone(
            &self,
            phone_hash: &str,
            country_code: &str,
        ) -> Result<Option<User>, DomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter()
                .find(|u| u.phone_hash == phone_hash && u.country_code == country_code)
                .cloned())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.id == id).cloned())
        }

        async fn create(&self, user: User) -> Result<User, DomainError> {
            let mut users = self.users.lock().unwrap();
            // Check for duplicate
            if users.iter().any(|u| u.phone_hash == user.phone_hash && u.country_code == user.country_code) {
                return Err(DomainError::Auth(AuthError::UserAlreadyExists));
            }
            users.push(user.clone());
            Ok(user)
        }

        async fn update(&self, user: User) -> Result<User, DomainError> {
            let mut users = self.users.lock().unwrap();
            if let Some(existing) = users.iter_mut().find(|u| u.id == user.id) {
                *existing = user.clone();
                Ok(user)
            } else {
                Err(DomainError::Auth(AuthError::UserNotFound))
            }
        }

        async fn exists_by_phone(
            &self,
            phone_hash: &str,
            country_code: &str,
        ) -> Result<bool, DomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().any(|u| u.phone_hash == phone_hash && u.country_code == country_code))
        }

        async fn count_by_type(&self, user_type: Option<UserType>) -> Result<u64, DomainError> {
            let users = self.users.lock().unwrap();
            let count = match user_type {
                Some(ut) => users.iter().filter(|u| u.user_type == Some(ut)).count(),
                None => users.len(),
            };
            Ok(count as u64)
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
            let mut users = self.users.lock().unwrap();
            if let Some(index) = users.iter().position(|u| u.id == id) {
                users.remove(index);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    struct MockSmsService;

    #[async_trait]
    impl SmsServiceTrait for MockSmsService {
        async fn send_verification_code(&self, _phone: &str, _code: &str) -> Result<String, String> {
            Ok("mock-message-id".to_string())
        }

        fn is_valid_phone_number(&self, phone: &str) -> bool {
            phone.starts_with('+') && phone.len() >= 10
        }
    }

    struct MockCacheService {
        verify_success: bool,
        remaining_attempts: i64,
    }

    impl MockCacheService {
        fn new_success() -> Self {
            Self {
                verify_success: true,
                remaining_attempts: 3,
            }
        }

        fn new_failure(remaining_attempts: i64) -> Self {
            Self {
                verify_success: false,
                remaining_attempts,
            }
        }
    }

    #[async_trait]
    impl CacheServiceTrait for MockCacheService {
        async fn store_code(&self, _phone: &str, _code: &str) -> Result<(), String> {
            Ok(())
        }

        async fn verify_code(&self, _phone: &str, _code: &str) -> Result<bool, String> {
            Ok(self.verify_success)
        }

        async fn get_remaining_attempts(&self, _phone: &str) -> Result<i64, String> {
            Ok(self.remaining_attempts)
        }

        async fn code_exists(&self, _phone: &str) -> Result<bool, String> {
            Ok(false)
        }

        async fn get_code_ttl(&self, _phone: &str) -> Result<Option<i64>, String> {
            Ok(None)
        }

        async fn clear_verification(&self, _phone: &str) -> Result<(), String> {
            Ok(())
        }
    }


    struct MockRateLimiter {
        counters: Arc<Mutex<HashMap<String, i64>>>,
        max_requests: i64,
    }

    impl MockRateLimiter {
        fn new(max_requests: i64) -> Self {
            Self {
                counters: Arc::new(Mutex::new(HashMap::new())),
                max_requests,
            }
        }
    }

    #[async_trait]
    impl RateLimiterTrait for MockRateLimiter {
        async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String> {
            let counters = self.counters.lock().unwrap();
            let count = counters.get(phone).copied().unwrap_or(0);
            Ok(count >= self.max_requests)
        }

        async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String> {
            let mut counters = self.counters.lock().unwrap();
            let count = counters.entry(phone.to_string()).or_insert(0);
            *count += 1;
            Ok(*count)
        }

        async fn get_rate_limit_reset_time(&self, _phone: &str) -> Result<Option<i64>, String> {
            Ok(Some(3600))
        }
    }

    #[tokio::test]
    async fn test_send_verification_code_success() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        let result = auth_service.send_verification_code("+1234567890").await;
        assert!(result.is_ok());

        let send_result = result.unwrap();
        assert!(send_result.message_id.starts_with("mock-message"));
        assert_eq!(send_result.verification_code.phone, "+1234567890");
    }

    #[tokio::test]
    async fn test_send_verification_code_invalid_phone() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Test without + prefix
        let result = auth_service.send_verification_code("1234567890").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::InvalidPhoneFormat { .. }) => {}
            _ => panic!("Expected InvalidPhoneFormat error"),
        }

        // Test too short
        let result = auth_service.send_verification_code("+123").await;
        assert!(result.is_err());

        // Test with letters
        let result = auth_service.send_verification_code("+123abc7890").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_verification_code_rate_limit() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter.clone(),
            token_service,
            config,
        );

        let phone = "+1234567890";

        // Send 3 codes successfully
        for _ in 0..3 {
            let result = auth_service.send_verification_code(phone).await;
            assert!(result.is_ok());
        }

        // 4th attempt should fail due to rate limit
        let result = auth_service.send_verification_code(phone).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::RateLimitExceeded { .. }) => {}
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }

    #[test]
    fn test_mask_phone() {
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::mask_phone("+1234567890"),
            "***7890"
        );
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::mask_phone("+123"),
            "****"
        );
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::mask_phone("1234"),
            "****"
        );
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::mask_phone("123"),
            "***"
        );
    }
    
    #[test]
    fn test_hash_phone() {
        let phone = "1234567890";
        let hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone(phone);
        // SHA-256 hash should be 64 characters long (hex representation)
        assert_eq!(hash.len(), 64);
        // Should be consistent
        let hash2 = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone(phone);
        assert_eq!(hash, hash2);
        // Different input should produce different hash
        let hash3 = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("0987654321");
        assert_ne!(hash, hash3);
    }
    
    #[test]
    fn test_extract_country_code() {
        // US/Canada
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::extract_country_code("+1234567890"),
            ("+1".to_string(), "234567890".to_string())
        );
        // China
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::extract_country_code("+8613812345678"),
            ("+86".to_string(), "13812345678".to_string())
        );
        // Australia
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::extract_country_code("+61412345678"),
            ("+61".to_string(), "412345678".to_string())
        );
        // UK
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::extract_country_code("+447123456789"),
            ("+44".to_string(), "7123456789".to_string())
        );
        // Russia
        assert_eq!(
            AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::extract_country_code("+79123456789"),
            ("+7".to_string(), "9123456789".to_string())
        );
    }

    #[test]
    fn test_is_valid_phone_format() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Valid formats
        assert!(auth_service.is_valid_phone_format("+1234567890"));
        assert!(auth_service.is_valid_phone_format("+861234567890"));
        assert!(auth_service.is_valid_phone_format("+12345678901234"));

        // Invalid formats
        assert!(!auth_service.is_valid_phone_format("1234567890")); // Missing +
        assert!(!auth_service.is_valid_phone_format("+123")); // Too short
        assert!(!auth_service.is_valid_phone_format("+1234567890123456")); // Too long
        assert!(!auth_service.is_valid_phone_format("+123abc7890")); // Contains letters
        assert!(!auth_service.is_valid_phone_format("")); // Empty
    }

    #[tokio::test]
    async fn test_verify_code_success() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_ok());
        
        let auth_response = result.unwrap();
        assert!(!auth_response.access_token.is_empty());
        assert!(!auth_response.refresh_token.is_empty());
        assert_eq!(auth_response.expires_in, 900); // 15 minutes
        assert!(auth_response.requires_type_selection); // New user has no type
        assert_eq!(auth_response.user_type, None);
    }

    #[tokio::test]
    async fn test_verify_code_invalid_phone() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Test invalid phone format
        let result = auth_service.verify_code("1234567890", "123456").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::InvalidPhoneFormat { .. }) => {}
            _ => panic!("Expected InvalidPhoneFormat error"),
        }
    }

    #[tokio::test]
    async fn test_verify_code_invalid_code() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_failure(2)); // 2 attempts remaining
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::InvalidVerificationCode) => {}
            _ => panic!("Expected InvalidVerificationCode error"),
        }
    }

    #[tokio::test]
    async fn test_verify_code_max_attempts_exceeded() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_failure(0)); // No attempts remaining
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::MaxAttemptsExceeded) => {}
            _ => panic!("Expected MaxAttemptsExceeded error"),
        }
    }
    
    #[tokio::test]
    async fn test_verify_code_creates_new_user() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Verify code for a new user
        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_ok());
        
        let auth_response = result.unwrap();
        assert!(auth_response.requires_type_selection); // New user needs to select type
        assert_eq!(auth_response.user_type, None);
        
        // Check that user was created
        assert_eq!(user_repo.count_by_type(None).await.unwrap(), 1);
        
        // Verify user properties
        let phone_hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("234567890");
        let user = user_repo.find_by_phone(&phone_hash, "+1").await.unwrap().unwrap();
        assert!(user.is_verified);
        assert!(user.last_login_at.is_some());
        assert!(!user.is_blocked);
    }
    
    #[tokio::test]
    async fn test_verify_code_existing_user_login() {
        // Create an existing user with a type
        let phone_hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("234567890");
        let mut existing_user = User::new(phone_hash.clone(), "+1".to_string());
        existing_user.verify();
        existing_user.set_user_type(UserType::Customer); // Set user type
        let original_login_time = existing_user.last_login_at;
        
        let user_repo = Arc::new(MockUserRepository::with_existing_user(existing_user.clone()));
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Small delay to ensure time difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Verify code for existing user
        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_ok());
        
        let auth_response = result.unwrap();
        assert!(!auth_response.requires_type_selection); // Existing user with type
        assert_eq!(auth_response.user_type, Some("customer".to_string()));
        
        // Check that no new user was created
        assert_eq!(user_repo.count_by_type(None).await.unwrap(), 1);
        
        // Verify user's last login was updated
        let user = user_repo.find_by_phone(&phone_hash, "+1").await.unwrap().unwrap();
        assert!(user.last_login_at > original_login_time);
    }
    
    #[tokio::test]
    async fn test_verify_code_blocked_user() {
        // Create a blocked user
        let phone_hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("234567890");
        let mut blocked_user = User::new(phone_hash.clone(), "+1".to_string());
        blocked_user.block();
        
        let user_repo = Arc::new(MockUserRepository::with_existing_user(blocked_user));
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Try to verify code for blocked user
        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::UserBlocked) => {}
            _ => panic!("Expected UserBlocked error"),
        }
    }
    
    #[tokio::test]
    async fn test_verify_code_registration_disabled() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let mut config = AuthServiceConfig::default();
        config.allow_registration = false; // Disable registration

        let auth_service = AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Try to verify code for new user when registration is disabled
        let result = auth_service.verify_code("+1234567890", "123456").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::RegistrationDisabled) => {}
            _ => panic!("Expected RegistrationDisabled error"),
        }
        
        // Verify no user was created
        assert_eq!(user_repo.count_by_type(None).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_select_user_type_success() {
        // Create a user without a type
        let phone_hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("234567890");
        let mut user = User::new(phone_hash.clone(), "+1".to_string());
        user.verify();
        let user_id = user.id;
        
        let user_repo = Arc::new(MockUserRepository::with_existing_user(user));
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Select user type as Customer
        let result = auth_service.select_user_type(user_id, UserType::Customer).await;
        assert!(result.is_ok());
        
        // Verify the user now has the Customer type
        let updated_user = user_repo.find_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(updated_user.user_type, Some(UserType::Customer));
        assert!(updated_user.is_customer());
        assert!(!updated_user.is_worker());
    }

    #[tokio::test]
    async fn test_select_user_type_already_selected() {
        // Create a user with a type already set
        let phone_hash = AuthService::<MockUserRepository, MockSmsService, MockCacheService, MockRateLimiter, MockTokenRepository>::hash_phone("234567890");
        let mut user = User::new(phone_hash.clone(), "+1".to_string());
        user.verify();
        user.set_user_type(UserType::Worker); // Already has a type
        let user_id = user.id;
        
        let user_repo = Arc::new(MockUserRepository::with_existing_user(user));
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Try to change user type
        let result = auth_service.select_user_type(user_id, UserType::Customer).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::InsufficientPermissions) => {}
            _ => panic!("Expected InsufficientPermissions error"),
        }
        
        // Verify the user type hasn't changed
        let user = user_repo.find_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.user_type, Some(UserType::Worker));
    }

    #[tokio::test]
    async fn test_select_user_type_user_not_found() {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService);
        let cache_service = Arc::new(MockCacheService::new_success());
        let verification_service = Arc::new(VerificationService::new(
            sms_service,
            cache_service,
            VerificationServiceConfig::default(),
        ));
        let rate_limiter = Arc::new(MockRateLimiter::new(3));
        let token_repo = MockTokenRepository::new();
        let token_service = Arc::new(TokenService::new(
            token_repo,
            TokenServiceConfig::default(),
        ));
        let config = AuthServiceConfig::default();

        let auth_service = AuthService::new(
            user_repo,
            verification_service,
            rate_limiter,
            token_service,
            config,
        );

        // Try to select type for non-existent user
        let non_existent_id = Uuid::new_v4();
        let result = auth_service.select_user_type(non_existent_id, UserType::Customer).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Auth(AuthError::UserNotFound) => {}
            _ => panic!("Expected UserNotFound error"),
        }
    }
}