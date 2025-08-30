use actix_web::{HttpRequest, HttpResponse, ResponseError};
use chrono::Utc;
use re_core::errors::{AuthError, DomainError, TokenError, ValidationError};
use re_shared::types::response::{ApiResponse, ErrorDetail, ResponseMeta, ResponseStatus, DetailedResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use crate::i18n::{format_message, get_error_message, Language};

/// Standard error response builder with comprehensive error details
pub struct StandardErrorBuilder {
    error_code: String,
    message: String,
    details: Option<HashMap<String, serde_json::Value>>,
    trace_id: Option<String>,
    path: Option<String>,
    method: Option<String>,
    language: Language,
}

impl StandardErrorBuilder {
    pub fn new(error_code: String, message: String) -> Self {
        Self {
            error_code,
            message,
            details: None,
            trace_id: None,
            path: None,
            method: None,
            language: Language::English,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_context(mut self, req: &HttpRequest) -> Self {
        self.path = Some(req.path().to_string());
        self.method = Some(req.method().to_string());
        
        // Extract trace ID from headers or generate new one
        self.trace_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .or_else(|| Some(Uuid::new_v4().to_string()));
        
        // Extract language preference
        self.language = extract_language(req);
        
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Build a standard API response
    pub fn build_api_response(self) -> ApiResponse<()> {
        let mut response = ApiResponse::<()>::error(self.message.clone());
        
        if let Some(trace_id) = self.trace_id {
            response = response.with_request_id(trace_id);
        }
        
        response
    }

    /// Build a detailed response with metadata
    pub fn build_detailed_response(self) -> DetailedResponse<()> {
        let mut context = HashMap::new();
        
        if let Some(path) = self.path {
            context.insert("path".to_string(), serde_json::json!(path));
        }
        
        if let Some(method) = self.method {
            context.insert("method".to_string(), serde_json::json!(method));
        }
        
        let error_detail = ErrorDetail {
            code: self.error_code,
            message: self.message,
            fields: None,
            trace: None,
            context: if context.is_empty() { None } else { Some(context) },
        };
        
        let mut meta = ResponseMeta::default();
        if let Some(trace_id) = self.trace_id {
            meta.request_id = Some(trace_id);
        }
        
        DetailedResponse {
            status: ResponseStatus::Error,
            data: None,
            meta,
            error: Some(error_detail),
        }
    }
}

/// Extract language preference from request
pub fn extract_language(req: &HttpRequest) -> Language {
    req.headers()
        .get("Accept-Language")
        .and_then(|v| v.to_str().ok())
        .map(|v| Language::from_header(Some(v)))
        .unwrap_or(Language::English)
}

/// Convert domain error to standardized HTTP response
pub fn to_standard_response(error: &DomainError, req: &HttpRequest) -> HttpResponse {
    let lang = extract_language(req);
    let trace_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    log::error!("Request {} - Domain Error: {:?}", trace_id, error);
    
    let (error_code, message, status_code) = match error {
        DomainError::Auth(auth_error) => map_auth_error(auth_error, lang),
        DomainError::ValidationErr(validation_error) => map_validation_error(validation_error, lang),
        DomainError::Token(token_error) => map_token_error(token_error, lang),
        DomainError::Validation { message } => {
            let msg = get_localized_message(lang, "validation_error", Some(message.as_str()));
            ("VALIDATION_ERROR".to_string(), msg, 400)
        }
        DomainError::BusinessRule { message } => {
            let msg = get_localized_message(lang, "business_rule_violation", Some(message.as_str()));
            ("BUSINESS_RULE_ERROR".to_string(), msg, 422)
        }
        DomainError::NotFound { resource } => {
            let mut params = HashMap::new();
            params.insert("resource", resource.clone());
            let msg = format_template_message(lang, "not_found", params);
            ("NOT_FOUND".to_string(), msg, 404)
        }
        DomainError::Unauthorized => {
            let msg = get_localized_message(lang, "unauthorized", None);
            ("UNAUTHORIZED".to_string(), msg, 401)
        }
        DomainError::Internal { message } => {
            log::error!("Internal error: {}", message);
            let msg = get_localized_message(lang, "internal_error", None);
            ("INTERNAL_ERROR".to_string(), msg, 500)
        }
    };
    
    let response = DetailedResponse {
        status: ResponseStatus::Error,
        data: None::<()>,
        meta: ResponseMeta {
            timestamp: Utc::now(),
            version: "v1".to_string(),
            request_id: Some(trace_id.clone()),
            response_time_ms: None,
            extra: HashMap::new(),
        },
        error: Some(ErrorDetail {
            code: error_code,
            message: message.clone(),
            fields: None,
            trace: None,
            context: Some({
                let mut ctx = HashMap::new();
                ctx.insert("path".to_string(), serde_json::json!(req.path()));
                ctx.insert("method".to_string(), serde_json::json!(req.method().to_string()));
                ctx
            }),
        }),
    };
    
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    )
    .json(response)
}

/// Map authentication errors to standardized format
fn map_auth_error(auth_error: &AuthError, lang: Language) -> (String, String, u16) {
    let (code, key, params) = match auth_error {
        AuthError::InvalidPhoneFormat { phone } => {
            let mut params = HashMap::new();
            params.insert("phone", phone.clone());
            ("INVALID_PHONE_FORMAT", "invalid_phone_format", params)
        }
        AuthError::RateLimitExceeded { minutes } => {
            let mut params = HashMap::new();
            params.insert("minutes", minutes.to_string());
            ("RATE_LIMIT_EXCEEDED", "rate_limit_exceeded", params)
        }
        AuthError::SmsServiceFailure => {
            ("SMS_SERVICE_FAILURE", "sms_service_failure", HashMap::new())
        }
        AuthError::InvalidVerificationCode => {
            ("INVALID_VERIFICATION_CODE", "invalid_verification_code", HashMap::new())
        }
        AuthError::VerificationCodeExpired => {
            ("VERIFICATION_CODE_EXPIRED", "verification_code_expired", HashMap::new())
        }
        AuthError::MaxAttemptsExceeded => {
            ("MAX_ATTEMPTS_EXCEEDED", "max_attempts_exceeded", HashMap::new())
        }
        AuthError::UserNotFound => {
            ("USER_NOT_FOUND", "user_not_found", HashMap::new())
        }
        AuthError::UserAlreadyExists => {
            ("USER_ALREADY_EXISTS", "user_already_exists", HashMap::new())
        }
        AuthError::AuthenticationFailed => {
            ("AUTHENTICATION_FAILED", "authentication_failed", HashMap::new())
        }
        AuthError::InsufficientPermissions => {
            ("INSUFFICIENT_PERMISSIONS", "insufficient_permissions", HashMap::new())
        }
        AuthError::AccountSuspended => {
            ("ACCOUNT_SUSPENDED", "account_suspended", HashMap::new())
        }
        AuthError::SessionExpired => {
            ("SESSION_EXPIRED", "session_expired", HashMap::new())
        }
        AuthError::RegistrationDisabled => {
            ("REGISTRATION_DISABLED", "registration_disabled", HashMap::new())
        }
        AuthError::UserBlocked => {
            ("USER_BLOCKED", "user_blocked", HashMap::new())
        }
    };
    
    let (_, message, http_status) = get_error_message("auth", key, lang)
        .unwrap_or_else(|| ("unknown_error".to_string(), "An error occurred".to_string(), 500));
    
    let formatted_message = format_message(&message, &params.iter().map(|(k, v)| (*k, v.clone())).collect());
    
    (code.to_string(), formatted_message, http_status)
}

/// Map validation errors to standardized format
fn map_validation_error(validation_error: &ValidationError, lang: Language) -> (String, String, u16) {
    match validation_error {
        ValidationError::RateLimitExceeded { message_en, message_zh, .. } => {
            let message = match lang {
                Language::English => message_en.clone(),
                Language::Chinese => message_zh.clone(),
            };
            ("RATE_LIMIT_EXCEEDED".to_string(), message, 429)
        }
        _ => {
            let (code, key, params) = match validation_error {
                ValidationError::RequiredField { field } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    ("REQUIRED_FIELD", "required_field", params)
                }
                ValidationError::InvalidFormat { field } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    ("INVALID_FORMAT", "invalid_format", params)
                }
                ValidationError::OutOfRange { field, min, max } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    params.insert("min", min.clone());
                    params.insert("max", max.clone());
                    ("OUT_OF_RANGE", "out_of_range", params)
                }
                ValidationError::InvalidLength { field, expected, actual } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    params.insert("expected", expected.to_string());
                    params.insert("actual", actual.to_string());
                    ("INVALID_LENGTH", "invalid_length", params)
                }
                ValidationError::PatternMismatch { field } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    ("PATTERN_MISMATCH", "pattern_mismatch", params)
                }
                ValidationError::InvalidEmail => {
                    ("INVALID_EMAIL", "invalid_email", HashMap::new())
                }
                ValidationError::InvalidUrl => {
                    ("INVALID_URL", "invalid_url", HashMap::new())
                }
                ValidationError::InvalidDate => {
                    ("INVALID_DATE", "invalid_date", HashMap::new())
                }
                ValidationError::DuplicateValue { field } => {
                    let mut params = HashMap::new();
                    params.insert("field", field.clone());
                    ("DUPLICATE_VALUE", "duplicate_value", params)
                }
                ValidationError::BusinessRuleViolation { rule } => {
                    let mut params = HashMap::new();
                    params.insert("rule", rule.clone());
                    ("BUSINESS_RULE_VIOLATION", "business_rule_violation", params)
                }
                _ => unreachable!(),
            };
            
            let (_, message, http_status) = get_error_message("validation", key, lang)
                .unwrap_or_else(|| ("unknown_error".to_string(), "Validation error".to_string(), 400));
            
            let formatted_message = format_message(&message, &params.iter().map(|(k, v)| (*k, v.clone())).collect());
            
            (code.to_string(), formatted_message, http_status)
        }
    }
}

/// Map token errors to standardized format
fn map_token_error(token_error: &TokenError, lang: Language) -> (String, String, u16) {
    let (code, key, params) = match token_error {
        TokenError::TokenExpired => {
            ("TOKEN_EXPIRED", "token_expired", HashMap::new())
        }
        TokenError::InvalidTokenFormat => {
            ("INVALID_TOKEN_FORMAT", "invalid_token_format", HashMap::new())
        }
        TokenError::InvalidSignature => {
            ("INVALID_SIGNATURE", "invalid_signature", HashMap::new())
        }
        TokenError::TokenNotYetValid => {
            ("TOKEN_NOT_YET_VALID", "token_not_yet_valid", HashMap::new())
        }
        TokenError::InvalidClaims => {
            ("INVALID_CLAIMS", "invalid_claims", HashMap::new())
        }
        TokenError::TokenRevoked => {
            ("TOKEN_REVOKED", "token_revoked", HashMap::new())
        }
        TokenError::RefreshTokenExpired => {
            ("REFRESH_TOKEN_EXPIRED", "refresh_token_expired", HashMap::new())
        }
        TokenError::InvalidRefreshToken => {
            ("INVALID_REFRESH_TOKEN", "invalid_refresh_token", HashMap::new())
        }
        TokenError::TokenGenerationFailed => {
            ("TOKEN_GENERATION_FAILED", "token_generation_failed", HashMap::new())
        }
        TokenError::MissingClaim { claim } => {
            let mut params = HashMap::new();
            params.insert("claim", claim.clone());
            ("MISSING_CLAIM", "missing_claim", params)
        }
        TokenError::KeyLoadError { message } => {
            let mut params = HashMap::new();
            params.insert("message", message.clone());
            ("KEY_LOAD_ERROR", "key_load_error", params)
        }
    };
    
    let (_, message, http_status) = get_error_message("token", key, lang)
        .unwrap_or_else(|| ("unknown_error".to_string(), "Token error".to_string(), 401));
    
    let formatted_message = format_message(&message, &params.iter().map(|(k, v)| (*k, v.clone())).collect());
    
    (code.to_string(), formatted_message, http_status)
}

/// Get localized message with fallback
fn get_localized_message(lang: Language, key: &str, custom_msg: Option<&str>) -> String {
    if let Some(msg) = custom_msg {
        return msg.to_string();
    }
    
    get_error_message("general", key, lang)
        .map(|(_, message, _)| message)
        .unwrap_or_else(|| match key {
            "validation_error" => "Validation error occurred".to_string(),
            "business_rule_violation" => "Business rule violation".to_string(),
            "unauthorized" => "Unauthorized access".to_string(),
            "internal_error" => "Internal server error".to_string(),
            _ => "An error occurred".to_string(),
        })
}

/// Format template message with parameters
fn format_template_message(lang: Language, key: &str, params: HashMap<&str, String>) -> String {
    get_error_message("general", key, lang)
        .map(|(_, message, _)| format_message(&message, &params))
        .unwrap_or_else(|| format!("{}: {:?}", key, params))
}

/// Standard API error wrapper for ResponseError trait
#[derive(Debug)]
pub struct StandardApiError {
    pub error: DomainError,
    pub request_context: Option<RequestContext>,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub path: String,
    pub method: String,
    pub trace_id: String,
    pub language: Language,
}

impl StandardApiError {
    pub fn new(error: DomainError) -> Self {
        Self {
            error,
            request_context: None,
        }
    }

    pub fn with_context(error: DomainError, req: &HttpRequest) -> Self {
        let context = RequestContext {
            path: req.path().to_string(),
            method: req.method().to_string(),
            trace_id: req
                .headers()
                .get("X-Request-ID")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            language: extract_language(req),
        };
        
        Self {
            error,
            request_context: Some(context),
        }
    }
}

impl fmt::Display for StandardApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl ResponseError for StandardApiError {
    fn error_response(&self) -> HttpResponse {
        let lang = self.request_context
            .as_ref()
            .map(|ctx| ctx.language)
            .unwrap_or(Language::English);
        
        let trace_id = self.request_context
            .as_ref()
            .map(|ctx| ctx.trace_id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        log::error!("Request {} - Error: {:?}", trace_id, self.error);
        
        let (error_code, message, status_code) = match &self.error {
            DomainError::Auth(auth_error) => map_auth_error(auth_error, lang),
            DomainError::ValidationErr(validation_error) => map_validation_error(validation_error, lang),
            DomainError::Token(token_error) => map_token_error(token_error, lang),
            _ => ("UNKNOWN_ERROR".to_string(), "An error occurred".to_string(), 500),
        };
        
        let mut context = HashMap::new();
        if let Some(ctx) = &self.request_context {
            context.insert("path".to_string(), serde_json::json!(ctx.path));
            context.insert("method".to_string(), serde_json::json!(ctx.method));
        }
        
        let response = DetailedResponse {
            status: ResponseStatus::Error,
            data: None::<()>,
            meta: ResponseMeta {
                timestamp: Utc::now(),
                version: "v1".to_string(),
                request_id: Some(trace_id),
                response_time_ms: None,
                extra: HashMap::new(),
            },
            error: Some(ErrorDetail {
                code: error_code,
                message,
                fields: None,
                trace: None,
                context: if context.is_empty() { None } else { Some(context) },
            }),
        };
        
        HttpResponse::build(
            actix_web::http::StatusCode::from_u16(status_code)
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        )
        .json(response)
    }
}