use crate::dto::ErrorResponse;
use actix_web::{http::StatusCode, HttpResponse};
use core::errors::{DomainError, AuthError, TokenError, ValidationError};

pub fn handle_error(error: anyhow::Error) -> HttpResponse {
    // Log the error
    log::error!("API Error: {:?}", error);

    // Create error response
    let error_response = ErrorResponse::new(
        "internal_error".to_string(),
        "An internal error occurred".to_string(),
    );

    error_response.to_response(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handle domain errors and convert them to appropriate HTTP responses
pub fn handle_domain_error(error: DomainError) -> HttpResponse {
    log::error!("Domain Error: {:?}", error);
    
    match error {
        DomainError::Auth(auth_error) => match auth_error {
            AuthError::InvalidPhoneFormat { phone } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_phone_format".to_string(),
                    format!("Invalid phone number format: {}", phone),
                ))
            }
            AuthError::RateLimitExceeded { minutes } => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "rate_limit_exceeded".to_string(),
                    format!("Too many requests. Please try again in {} minutes", minutes),
                ))
            }
            AuthError::SmsServiceFailure => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse::new(
                    "sms_service_failure".to_string(),
                    "SMS service is temporarily unavailable. Please try again later".to_string(),
                ))
            }
            AuthError::InvalidVerificationCode => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_verification_code".to_string(),
                    "Invalid or expired verification code".to_string(),
                ))
            }
            AuthError::VerificationCodeExpired => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "verification_code_expired".to_string(),
                    "Verification code has expired".to_string(),
                ))
            }
            AuthError::MaxAttemptsExceeded => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "max_attempts_exceeded".to_string(),
                    "Maximum verification attempts exceeded. Please request a new code".to_string(),
                ))
            }
            AuthError::UserNotFound => {
                HttpResponse::NotFound().json(ErrorResponse::new(
                    "user_not_found".to_string(),
                    "User not found".to_string(),
                ))
            }
            AuthError::UserAlreadyExists => {
                HttpResponse::Conflict().json(ErrorResponse::new(
                    "user_already_exists".to_string(),
                    "User already exists".to_string(),
                ))
            }
            AuthError::AuthenticationFailed => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "authentication_failed".to_string(),
                    "Authentication failed".to_string(),
                ))
            }
            AuthError::InsufficientPermissions => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "insufficient_permissions".to_string(),
                    "Insufficient permissions".to_string(),
                ))
            }
            AuthError::AccountSuspended => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "account_suspended".to_string(),
                    "Account has been suspended".to_string(),
                ))
            }
            AuthError::SessionExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "session_expired".to_string(),
                    "Session has expired. Please login again".to_string(),
                ))
            }
            AuthError::RegistrationDisabled => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse::new(
                    "registration_disabled".to_string(),
                    "Registration is currently disabled".to_string(),
                ))
            }
            AuthError::UserBlocked => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "user_blocked".to_string(),
                    "User account has been blocked".to_string(),
                ))
            }
        },
        DomainError::ValidationErr(validation_error) => match validation_error {
            ValidationError::RateLimitExceeded { message_en, .. } => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "rate_limit_exceeded".to_string(),
                    message_en,
                ))
            }
            ValidationError::RequiredField { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "required_field".to_string(),
                    format!("Required field: {}", field),
                ))
            }
            ValidationError::InvalidFormat { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_format".to_string(),
                    format!("Invalid format for field: {}", field),
                ))
            }
            ValidationError::OutOfRange { field, min, max } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "out_of_range".to_string(),
                    format!("Field {} out of range (min: {}, max: {})", field, min, max),
                ))
            }
            ValidationError::InvalidLength { field, expected, actual } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_length".to_string(),
                    format!("Invalid length for field {} (expected: {}, actual: {})", field, expected, actual),
                ))
            }
            ValidationError::PatternMismatch { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "pattern_mismatch".to_string(),
                    format!("Pattern mismatch for field: {}", field),
                ))
            }
            ValidationError::InvalidEmail => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_email".to_string(),
                    "Invalid email format".to_string(),
                ))
            }
            ValidationError::InvalidUrl => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_url".to_string(),
                    "Invalid URL format".to_string(),
                ))
            }
            ValidationError::InvalidDate => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_date".to_string(),
                    "Invalid date format".to_string(),
                ))
            }
            ValidationError::DuplicateValue { field } => {
                HttpResponse::Conflict().json(ErrorResponse::new(
                    "duplicate_value".to_string(),
                    format!("Duplicate value for field: {}", field),
                ))
            }
            ValidationError::BusinessRuleViolation { rule } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "business_rule_violation".to_string(),
                    format!("Business rule violation: {}", rule),
                ))
            }
        },
        DomainError::Token(token_error) => match token_error {
            TokenError::TokenExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_expired".to_string(),
                    "Token has expired".to_string(),
                ))
            }
            TokenError::InvalidTokenFormat => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_token_format".to_string(),
                    "Invalid token format".to_string(),
                ))
            }
            TokenError::InvalidSignature => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_signature".to_string(),
                    "Invalid token signature".to_string(),
                ))
            }
            TokenError::TokenNotYetValid => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_not_yet_valid".to_string(),
                    "Token is not yet valid".to_string(),
                ))
            }
            TokenError::InvalidClaims => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_claims".to_string(),
                    "Invalid token claims".to_string(),
                ))
            }
            TokenError::TokenRevoked => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_revoked".to_string(),
                    "Token has been revoked".to_string(),
                ))
            }
            TokenError::RefreshTokenExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "refresh_token_expired".to_string(),
                    "Refresh token has expired".to_string(),
                ))
            }
            TokenError::InvalidRefreshToken => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_refresh_token".to_string(),
                    "Invalid refresh token".to_string(),
                ))
            }
            TokenError::TokenGenerationFailed => {
                HttpResponse::InternalServerError().json(ErrorResponse::new(
                    "token_generation_failed".to_string(),
                    "Failed to generate token".to_string(),
                ))
            }
            TokenError::MissingClaim { claim } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "missing_claim".to_string(),
                    format!("Missing required claim: {}", claim),
                ))
            }
        },
        DomainError::Validation { message } => {
            HttpResponse::BadRequest().json(ErrorResponse::new(
                "validation_error".to_string(),
                message,
            ))
        }
        DomainError::BusinessRule { message } => {
            HttpResponse::BadRequest().json(ErrorResponse::new(
                "business_rule_violation".to_string(),
                message,
            ))
        }
        DomainError::NotFound { resource } => {
            HttpResponse::NotFound().json(ErrorResponse::new(
                "not_found".to_string(),
                format!("{} not found", resource),
            ))
        }
        DomainError::Unauthorized => {
            HttpResponse::Unauthorized().json(ErrorResponse::new(
                "unauthorized".to_string(),
                "Unauthorized access".to_string(),
            ))
        }
        DomainError::Internal { message } => {
            log::error!("Internal error: {}", message);
            HttpResponse::InternalServerError().json(ErrorResponse::new(
                "internal_error".to_string(),
                "An internal server error occurred".to_string(),
            ))
        }
    }
}