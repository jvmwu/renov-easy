// Integration tests for bilingual error handling

extern crate core as renov_core;

use api::handlers::error::{Language, handle_domain_error_with_lang, extract_language};
use renov_core::errors::{DomainError, AuthError, ValidationError, TokenError};
use actix_web::test;

#[test]
fn test_auth_error_invalid_verification_code() {
    // Test English version
    let error_en = DomainError::Auth(AuthError::InvalidVerificationCode);
    let response_en = handle_domain_error_with_lang(&error_en, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::BAD_REQUEST);
    
    // Test Chinese version
    let error_zh = DomainError::Auth(AuthError::InvalidVerificationCode);
    let response_zh = handle_domain_error_with_lang(&error_zh, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[test]
fn test_auth_error_rate_limit() {
    let error = DomainError::Auth(AuthError::RateLimitExceeded { minutes: 5 });
    
    // English
    let response_en = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
    
    // Chinese
    let response_zh = handle_domain_error_with_lang(&error, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_auth_error_user_not_found() {
    let error = DomainError::Auth(AuthError::UserNotFound);
    
    let response_en = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::NOT_FOUND);
    
    let response_zh = handle_domain_error_with_lang(&error, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[test]
fn test_validation_error_invalid_email() {
    let error = DomainError::ValidationErr(ValidationError::InvalidEmail);
    
    let response_en = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::BAD_REQUEST);
    
    let response_zh = handle_domain_error_with_lang(&error, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[test]
fn test_validation_error_required_field() {
    let error = DomainError::ValidationErr(ValidationError::RequiredField { 
        field: "username".to_string() 
    });
    
    let response = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[test]
fn test_token_error_expired() {
    let error = DomainError::Token(TokenError::TokenExpired);
    
    let response_en = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    
    let response_zh = handle_domain_error_with_lang(&error, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_token_error_invalid_signature() {
    let error = DomainError::Token(TokenError::InvalidSignature);
    
    let response = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_general_not_found_error() {
    let error = DomainError::NotFound { 
        resource: "Order".to_string() 
    };
    
    let response_en = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response_en.status(), actix_web::http::StatusCode::NOT_FOUND);
    
    let response_zh = handle_domain_error_with_lang(&error, Language::Chinese);
    assert_eq!(response_zh.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[test]
fn test_unauthorized_error() {
    let error = DomainError::Unauthorized;
    
    let response = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_internal_error() {
    let error = DomainError::Internal { 
        message: "Database connection failed".to_string() 
    };
    
    let response = handle_domain_error_with_lang(&error, Language::English);
    assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_language_extraction_from_request() {
    // Test Chinese language detection
    let req = test::TestRequest::default()
        .insert_header(("Accept-Language", "zh-CN,zh;q=0.9"))
        .to_http_request();
    assert_eq!(extract_language(&req), Language::Chinese);
    
    // Test English language detection
    let req = test::TestRequest::default()
        .insert_header(("Accept-Language", "en-US,en;q=0.9"))
        .to_http_request();
    assert_eq!(extract_language(&req), Language::English);
    
    // Test default to English when no header
    let req = test::TestRequest::default()
        .to_http_request();
    assert_eq!(extract_language(&req), Language::English);
    
    // Test default to English for unsupported language
    let req = test::TestRequest::default()
        .insert_header(("Accept-Language", "fr-FR"))
        .to_http_request();
    assert_eq!(extract_language(&req), Language::English);
}

#[test]
fn test_auth_error_all_variants() {
    // Test all auth error variants have proper status codes
    let test_cases = vec![
        (AuthError::InvalidPhoneFormat { phone: "123".to_string() }, 400),
        (AuthError::InvalidVerificationCode, 400),
        (AuthError::VerificationCodeExpired, 400),
        (AuthError::MaxAttemptsExceeded, 429),
        (AuthError::RateLimitExceeded { minutes: 5 }, 429),
        (AuthError::SmsServiceFailure, 503),
        (AuthError::UserNotFound, 404),
        (AuthError::UserAlreadyExists, 409),
        (AuthError::AuthenticationFailed, 401),
        (AuthError::InsufficientPermissions, 403),
        (AuthError::AccountSuspended, 403),
        (AuthError::SessionExpired, 401),
        (AuthError::RegistrationDisabled, 503),
        (AuthError::UserBlocked, 403),
    ];
    
    for (auth_error, expected_status) in test_cases {
        let error = DomainError::Auth(auth_error);
        let response = handle_domain_error_with_lang(&error, Language::English);
        assert_eq!(
            response.status().as_u16(), 
            expected_status,
            "Failed for auth error variant"
        );
    }
}