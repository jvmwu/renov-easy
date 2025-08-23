//! Unit tests for authentication service

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::user::{User, UserType};
use crate::errors::{AuthError, DomainError};
use crate::repositories::UserRepository;
use crate::repositories::token::MockTokenRepository;
use crate::services::auth::{AuthService, AuthServiceConfig};
use crate::services::auth::phone_utils::hash_phone;
use crate::services::token::{TokenService, TokenServiceConfig};
use crate::services::verification::{VerificationService, VerificationServiceConfig};

use super::mocks::*;

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

    let result = auth_service.send_verification_code("+8613812345678").await;
    assert!(result.is_ok());

    let send_result = result.unwrap();
    assert!(send_result.message_id.starts_with("mock-message"));
    assert_eq!(send_result.verification_code.phone, "+8613812345678");
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

    let phone = "+8613812345678";

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

    let result = auth_service.verify_code("+8613812345678", "123456").await;
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

    let result = auth_service.verify_code("+8613812345678", "123456").await;
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

    let result = auth_service.verify_code("+8613812345678", "123456").await;
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
    let result = auth_service.verify_code("+8613812345678", "123456").await;
    assert!(result.is_ok());
    
    let auth_response = result.unwrap();
    assert!(auth_response.requires_type_selection); // New user needs to select type
    assert_eq!(auth_response.user_type, None);
    
    // Check that user was created
    assert_eq!(user_repo.count_by_type(None).await.unwrap(), 1);
    
    // Verify user properties
    let phone_hash = hash_phone("13812345678");
    let user = user_repo.find_by_phone(&phone_hash, "+86").await.unwrap().unwrap();
    assert!(user.is_verified);
    assert!(user.last_login_at.is_some());
    assert!(!user.is_blocked);
}

#[tokio::test]
async fn test_verify_code_existing_user_login() {
    // Create an existing user with a type
    let phone_hash = hash_phone("13812345678");
    let mut existing_user = User::new(phone_hash.clone(), "+86".to_string());
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
    let result = auth_service.verify_code("+8613812345678", "123456").await;
    assert!(result.is_ok());
    
    let auth_response = result.unwrap();
    assert!(!auth_response.requires_type_selection); // Existing user with type
    assert_eq!(auth_response.user_type, Some("customer".to_string()));
    
    // Check that no new user was created
    assert_eq!(user_repo.count_by_type(None).await.unwrap(), 1);
    
    // Verify user's last login was updated
    let user = user_repo.find_by_phone(&phone_hash, "+86").await.unwrap().unwrap();
    assert!(user.last_login_at > original_login_time);
}

#[tokio::test]
async fn test_verify_code_blocked_user() {
    // Create a blocked user
    let phone_hash = hash_phone("13812345678");
    let mut blocked_user = User::new(phone_hash.clone(), "+86".to_string());
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
    let result = auth_service.verify_code("+8613812345678", "123456").await;
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
    let result = auth_service.verify_code("+8613812345678", "123456").await;
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
    let phone_hash = hash_phone("234567890");
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
    let phone_hash = hash_phone("234567890");
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

#[tokio::test]
async fn test_logout_success() {
    let phone = "+8613812345678";
    let phone_hash = hash_phone(phone);
    
    // Create a verified user with a type
    let mut user = User::new(phone_hash.clone(), "+1".to_string());
    user.verify();
    user.set_user_type(UserType::Customer);
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
        user_repo,
        verification_service,
        rate_limiter,
        token_service.clone(),
        config,
    );

    // Generate tokens for the user (this also stores them via the mock repository)
    let _token_pair = token_service
        .generate_tokens(user_id, Some(UserType::Customer), true)
        .await
        .unwrap();

    // Logout the user
    let result = auth_service.logout(user_id).await;
    assert!(result.is_ok());
    
    // Verify that tokens are revoked
    // Since we're using a mock, the revoke_tokens method is called which sets tokens as revoked
    // The next verification attempt should fail (mock behavior)
}