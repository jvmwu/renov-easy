use actix_web::{HttpResponse, ResponseError};
use renov_core::errors::{AuthError, DomainError, TokenError, ValidationError};
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::i18n::{format_message, get_error_message};

// Re-export Language for use in other modules
pub use crate::i18n::Language;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: String, message: String) -> Self {
        Self {
            error,
            message,
            details: None,
        }
    }

    pub fn with_details(error: String, message: String, details: serde_json::Value) -> Self {
        Self {
            error,
            message,
            details: Some(details),
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

// Wrapper type to implement ResponseError for DomainError
#[derive(Debug)]
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        ApiError(error)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        handle_domain_error(&self.0)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Get localized message based on language preference
fn get_localized_message(lang: Language, en_msg: &str, zh_msg: &str) -> String {
    match lang {
        Language::English => en_msg.to_string(),
        Language::Chinese => zh_msg.to_string(),
    }
}

/// Extract language preference from request headers
pub fn extract_language(req: &actix_web::HttpRequest) -> Language {
    req.headers()
        .get("Accept-Language")
        .and_then(|v| v.to_str().ok())
        .map(|v| Language::from_header(Some(v)))
        .unwrap_or(Language::English)
}

/// Handle domain errors and return appropriate HTTP responses
pub fn handle_domain_error(error: &DomainError) -> HttpResponse {
    handle_domain_error_with_lang(error, Language::English)
}

/// Handle domain errors with language support
pub fn handle_domain_error_with_lang(error: &DomainError, lang: Language) -> HttpResponse {
    log::error!("Domain Error: {:?}", error);
    
    match error {
        DomainError::Auth(auth_error) => handle_auth_error(auth_error, lang),
        DomainError::ValidationErr(validation_error) => handle_validation_error(validation_error, lang),
        DomainError::Token(token_error) => handle_token_error(token_error, lang),
        DomainError::Validation { message } => handle_general_error("validation_error", Some(message.clone()), lang),
        DomainError::BusinessRule { message } => handle_general_error("business_rule_violation", Some(message.clone()), lang),
        DomainError::NotFound { resource } => {
            let mut params = HashMap::new();
            params.insert("resource", resource.clone());
            handle_general_error_with_params("not_found", params, lang)
        }
        DomainError::Unauthorized => handle_general_error("unauthorized", None, lang),
        DomainError::Internal { message } => {
            log::error!("Internal error: {}", message);
            handle_general_error("internal_error", None, lang)
        }
    }
}

fn handle_auth_error(auth_error: &AuthError, lang: Language) -> HttpResponse {
    let (error_key, params) = match auth_error {
        AuthError::InvalidPhoneFormat { phone } => {
            let mut params = HashMap::new();
            params.insert("phone", phone.clone());
            ("invalid_phone_format", params)
        }
        AuthError::RateLimitExceeded { minutes } => {
            let mut params = HashMap::new();
            params.insert("minutes", minutes.to_string());
            ("rate_limit_exceeded", params)
        }
        AuthError::SmsServiceFailure => ("sms_service_failure", HashMap::new()),
        AuthError::InvalidVerificationCode => ("invalid_verification_code", HashMap::new()),
        AuthError::VerificationCodeExpired => ("verification_code_expired", HashMap::new()),
        AuthError::MaxAttemptsExceeded => ("max_attempts_exceeded", HashMap::new()),
        AuthError::UserNotFound => ("user_not_found", HashMap::new()),
        AuthError::UserAlreadyExists => ("user_already_exists", HashMap::new()),
        AuthError::AuthenticationFailed => ("authentication_failed", HashMap::new()),
        AuthError::InsufficientPermissions => ("insufficient_permissions", HashMap::new()),
        AuthError::AccountSuspended => ("account_suspended", HashMap::new()),
        AuthError::SessionExpired => ("session_expired", HashMap::new()),
        AuthError::RegistrationDisabled => ("registration_disabled", HashMap::new()),
        AuthError::UserBlocked => ("user_blocked", HashMap::new()),
    };

    create_error_response("auth", error_key, params, lang)
}

fn handle_validation_error(validation_error: &ValidationError, lang: Language) -> HttpResponse {
    let (error_key, params) = match validation_error {
        ValidationError::RateLimitExceeded { message_en, message_zh, .. } => {
            // Special case for rate limit with custom messages
            let message = get_localized_message(lang, &message_en, &message_zh);
            return HttpResponse::TooManyRequests().json(ErrorResponse::new(
                "rate_limit_exceeded".to_string(),
                message,
            ));
        }
        ValidationError::RequiredField { field } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            ("required_field", params)
        }
        ValidationError::InvalidFormat { field } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            ("invalid_format", params)
        }
        ValidationError::OutOfRange { field, min, max } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            params.insert("min", min.clone());
            params.insert("max", max.clone());
            ("out_of_range", params)
        }
        ValidationError::InvalidLength { field, expected, actual } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            params.insert("expected", expected.to_string());
            params.insert("actual", actual.to_string());
            ("invalid_length", params)
        }
        ValidationError::PatternMismatch { field } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            ("pattern_mismatch", params)
        }
        ValidationError::InvalidEmail => ("invalid_email", HashMap::new()),
        ValidationError::InvalidUrl => ("invalid_url", HashMap::new()),
        ValidationError::InvalidDate => ("invalid_date", HashMap::new()),
        ValidationError::DuplicateValue { field } => {
            let mut params = HashMap::new();
            params.insert("field", field.clone());
            ("duplicate_value", params)
        }
        ValidationError::BusinessRuleViolation { rule } => {
            let mut params = HashMap::new();
            params.insert("rule", rule.clone());
            ("business_rule_violation", params)
        }
    };

    create_error_response("validation", error_key, params, lang)
}

fn handle_token_error(token_error: &TokenError, lang: Language) -> HttpResponse {
    let (error_key, params) = match token_error {
        TokenError::TokenExpired => ("token_expired", HashMap::new()),
        TokenError::InvalidTokenFormat => ("invalid_token_format", HashMap::new()),
        TokenError::InvalidSignature => ("invalid_signature", HashMap::new()),
        TokenError::TokenNotYetValid => ("token_not_yet_valid", HashMap::new()),
        TokenError::InvalidClaims => ("invalid_claims", HashMap::new()),
        TokenError::TokenRevoked => ("token_revoked", HashMap::new()),
        TokenError::RefreshTokenExpired => ("refresh_token_expired", HashMap::new()),
        TokenError::InvalidRefreshToken => ("invalid_refresh_token", HashMap::new()),
        TokenError::TokenGenerationFailed => ("token_generation_failed", HashMap::new()),
        TokenError::MissingClaim { claim } => {
            let mut params = HashMap::new();
            params.insert("claim", claim.clone());
            ("missing_claim", params)
        }
    };

    create_error_response("token", error_key, params, lang)
}

fn handle_general_error(error_key: &str, message: Option<String>, lang: Language) -> HttpResponse {
    let params = if let Some(msg) = message {
        let mut p = HashMap::new();
        p.insert("message", msg);
        p
    } else {
        HashMap::new()
    };
    
    create_error_response("general", error_key, params, lang)
}

fn handle_general_error_with_params(error_key: &str, params: HashMap<&str, String>, lang: Language) -> HttpResponse {
    create_error_response("general", error_key, params, lang)
}

fn create_error_response(
    category: &str,
    error_key: &str,
    params: HashMap<&str, String>,
    lang: Language
) -> HttpResponse {
    if let Some((code, message_template, http_status)) = get_error_message(category, error_key, lang) {
        let message = format_message(&message_template, &params);
        
        let response = HttpResponse::build(
            actix_web::http::StatusCode::from_u16(http_status)
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        )
        .json(ErrorResponse::new(code, message));
        
        response
    } else {
        // Fallback for unknown errors
        HttpResponse::InternalServerError().json(ErrorResponse::new(
            "unknown_error".to_string(),
            "An unknown error occurred".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new(
            "test_error".to_string(),
            "Test error message".to_string(),
        );
        
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("test_error"));
        assert!(json.contains("Test error message"));
    }

    #[test]
    fn test_auth_error_handling() {
        let error = DomainError::Auth(AuthError::UserNotFound);
        let response = handle_domain_error_with_lang(&error, Language::English);
        
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_validation_error_handling() {
        let error = DomainError::ValidationErr(ValidationError::InvalidEmail);
        let response = handle_domain_error_with_lang(&error, Language::Chinese);
        
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_token_error_handling() {
        let error = DomainError::Token(TokenError::TokenExpired);
        let response = handle_domain_error_with_lang(&error, Language::English);
        
        assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_language_extraction() {
        let req = test::TestRequest::default()
            .insert_header(("Accept-Language", "zh-CN"))
            .to_http_request();
        
        let lang = extract_language(&req);
        assert_eq!(lang, Language::Chinese);
    }
}