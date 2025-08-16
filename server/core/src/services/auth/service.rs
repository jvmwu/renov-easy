//! Main authentication service implementation

use std::sync::Arc;
use crate::domain::entities::user::User;
use crate::domain::value_objects::AuthResponse;
use crate::errors::{AuthError, DomainError, DomainResult, ValidationError};
use crate::repositories::{UserRepository, TokenRepository};
use crate::services::verification::{
    VerificationService, SmsServiceTrait, CacheServiceTrait, SendCodeResult,
};
use crate::services::token::TokenService;

use super::config::AuthServiceConfig;
use super::phone_utils::{is_valid_phone_format, mask_phone, hash_phone, extract_country_code};
use super::rate_limiter::RateLimiterTrait;

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
        if !is_valid_phone_format(phone) {
            return Err(DomainError::Auth(AuthError::InvalidPhoneFormat {
                phone: mask_phone(phone),
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
        if !is_valid_phone_format(phone) {
            return Err(DomainError::Auth(AuthError::InvalidPhoneFormat {
                phone: mask_phone(phone),
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
            let (country_code, phone_without_code) = extract_country_code(phone);
            
            // Hash the phone number for storage
            let phone_hash = hash_phone(&phone_without_code);
            
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
}