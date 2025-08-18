use crate::dto::ErrorResponse;
use actix_web::{http::{header, StatusCode}, HttpRequest, HttpResponse};
use core::errors::{DomainError, AuthError, TokenError, ValidationError};

/// Language preference for error messages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    /// Detect language preference from Accept-Language header
    pub fn from_request(req: &HttpRequest) -> Self {
        if let Some(header_value) = req.headers().get(header::ACCEPT_LANGUAGE) {
            if let Ok(header_str) = header_value.to_str() {
                // Parse the Accept-Language header
                // Example: "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7"
                let languages: Vec<(String, f32)> = header_str
                    .split(',')
                    .filter_map(|lang| {
                        let parts: Vec<&str> = lang.trim().split(';').collect();
                        let language = parts[0].to_lowercase();
                        let quality = if parts.len() > 1 {
                            parts[1]
                                .trim_start_matches("q=")
                                .parse::<f32>()
                                .unwrap_or(1.0)
                        } else {
                            1.0
                        };
                        Some((language, quality))
                    })
                    .collect();

                // Find the highest quality language preference
                let mut preferred_lang = Language::English;
                let mut max_quality = 0.0;

                for (lang, quality) in languages {
                    if lang.starts_with("zh") && quality > max_quality {
                        preferred_lang = Language::Chinese;
                        max_quality = quality;
                    } else if lang.starts_with("en") && quality > max_quality {
                        preferred_lang = Language::English;
                        max_quality = quality;
                    }
                }

                return preferred_lang;
            }
        }
        
        // Default to English
        Language::English
    }
}

/// Helper function to get localized message
fn get_localized_message(lang: Language, en: &str, zh: &str) -> String {
    match lang {
        Language::English => en.to_string(),
        Language::Chinese => zh.to_string(),
    }
}

pub fn handle_error(error: anyhow::Error) -> HttpResponse {
    handle_error_with_lang(error, Language::English)
}

pub fn handle_error_with_lang(error: anyhow::Error, lang: Language) -> HttpResponse {
    // Log the error
    log::error!("API Error: {:?}", error);

    // Create error response with localized message
    let error_response = ErrorResponse::new(
        "internal_error".to_string(),
        get_localized_message(
            lang,
            "An internal error occurred",
            "发生内部错误"
        ),
    );

    error_response.to_response(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handle domain errors and convert them to appropriate HTTP responses
pub fn handle_domain_error(error: DomainError) -> HttpResponse {
    handle_domain_error_with_lang(error, Language::English)
}

/// Handle domain errors with language support
pub fn handle_domain_error_with_lang(error: DomainError, lang: Language) -> HttpResponse {
    log::error!("Domain Error: {:?}", error);
    
    match error {
        DomainError::Auth(auth_error) => match auth_error {
            AuthError::InvalidPhoneFormat { phone } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_phone_format".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Invalid phone number format: {}", phone),
                        &format!("无效的手机号码格式：{}", phone)
                    ),
                ))
            }
            AuthError::RateLimitExceeded { minutes } => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "rate_limit_exceeded".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Too many requests. Please try again in {} minutes", minutes),
                        &format!("请求过于频繁，请在{}分钟后重试", minutes)
                    ),
                ))
            }
            AuthError::SmsServiceFailure => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse::new(
                    "sms_service_failure".to_string(),
                    get_localized_message(
                        lang,
                        "SMS service is temporarily unavailable. Please try again later",
                        "短信服务暂时不可用，请稍后重试"
                    ),
                ))
            }
            AuthError::InvalidVerificationCode => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_verification_code".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid or expired verification code",
                        "验证码无效或已过期"
                    ),
                ))
            }
            AuthError::VerificationCodeExpired => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "verification_code_expired".to_string(),
                    get_localized_message(
                        lang,
                        "Verification code has expired",
                        "验证码已过期"
                    ),
                ))
            }
            AuthError::MaxAttemptsExceeded => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "max_attempts_exceeded".to_string(),
                    get_localized_message(
                        lang,
                        "Maximum verification attempts exceeded. Please request a new code",
                        "验证次数超过上限，请重新获取验证码"
                    ),
                ))
            }
            AuthError::UserNotFound => {
                HttpResponse::NotFound().json(ErrorResponse::new(
                    "user_not_found".to_string(),
                    get_localized_message(
                        lang,
                        "User not found",
                        "用户不存在"
                    ),
                ))
            }
            AuthError::UserAlreadyExists => {
                HttpResponse::Conflict().json(ErrorResponse::new(
                    "user_already_exists".to_string(),
                    get_localized_message(
                        lang,
                        "User already exists",
                        "用户已存在"
                    ),
                ))
            }
            AuthError::AuthenticationFailed => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "authentication_failed".to_string(),
                    get_localized_message(
                        lang,
                        "Authentication failed",
                        "认证失败"
                    ),
                ))
            }
            AuthError::InsufficientPermissions => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "insufficient_permissions".to_string(),
                    get_localized_message(
                        lang,
                        "Insufficient permissions",
                        "权限不足"
                    ),
                ))
            }
            AuthError::AccountSuspended => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "account_suspended".to_string(),
                    get_localized_message(
                        lang,
                        "Account has been suspended",
                        "账户已被暂停"
                    ),
                ))
            }
            AuthError::SessionExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "session_expired".to_string(),
                    get_localized_message(
                        lang,
                        "Session has expired. Please login again",
                        "会话已过期，请重新登录"
                    ),
                ))
            }
            AuthError::RegistrationDisabled => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse::new(
                    "registration_disabled".to_string(),
                    get_localized_message(
                        lang,
                        "Registration is currently disabled",
                        "注册功能暂时关闭"
                    ),
                ))
            }
            AuthError::UserBlocked => {
                HttpResponse::Forbidden().json(ErrorResponse::new(
                    "user_blocked".to_string(),
                    get_localized_message(
                        lang,
                        "User account has been blocked",
                        "用户账户已被封禁"
                    ),
                ))
            }
        },
        DomainError::ValidationErr(validation_error) => match validation_error {
            ValidationError::RateLimitExceeded { message_en, message_zh, .. } => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new(
                    "rate_limit_exceeded".to_string(),
                    get_localized_message(
                        lang,
                        &message_en,
                        &message_zh
                    ),
                ))
            }
            ValidationError::RequiredField { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "required_field".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Required field: {}", field),
                        &format!("必填字段：{}", field)
                    ),
                ))
            }
            ValidationError::InvalidFormat { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_format".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Invalid format for field: {}", field),
                        &format!("字段格式无效：{}", field)
                    ),
                ))
            }
            ValidationError::OutOfRange { field, min, max } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "out_of_range".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Field {} out of range (min: {}, max: {})", field, min, max),
                        &format!("字段{}超出范围（最小值：{}，最大值：{}）", field, min, max)
                    ),
                ))
            }
            ValidationError::InvalidLength { field, expected, actual } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_length".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Invalid length for field {} (expected: {}, actual: {})", field, expected, actual),
                        &format!("字段{}长度无效（期望：{}，实际：{}）", field, expected, actual)
                    ),
                ))
            }
            ValidationError::PatternMismatch { field } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "pattern_mismatch".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Pattern mismatch for field: {}", field),
                        &format!("字段格式不匹配：{}", field)
                    ),
                ))
            }
            ValidationError::InvalidEmail => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_email".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid email format",
                        "邮箱格式无效"
                    ),
                ))
            }
            ValidationError::InvalidUrl => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_url".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid URL format",
                        "URL格式无效"
                    ),
                ))
            }
            ValidationError::InvalidDate => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "invalid_date".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid date format",
                        "日期格式无效"
                    ),
                ))
            }
            ValidationError::DuplicateValue { field } => {
                HttpResponse::Conflict().json(ErrorResponse::new(
                    "duplicate_value".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Duplicate value for field: {}", field),
                        &format!("字段值重复：{}", field)
                    ),
                ))
            }
            ValidationError::BusinessRuleViolation { rule } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "business_rule_violation".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Business rule violation: {}", rule),
                        &format!("违反业务规则：{}", rule)
                    ),
                ))
            }
        },
        DomainError::Token(token_error) => match token_error {
            TokenError::TokenExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_expired".to_string(),
                    get_localized_message(
                        lang,
                        "Token has expired",
                        "令牌已过期"
                    ),
                ))
            }
            TokenError::InvalidTokenFormat => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_token_format".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid token format",
                        "令牌格式无效"
                    ),
                ))
            }
            TokenError::InvalidSignature => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_signature".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid token signature",
                        "令牌签名无效"
                    ),
                ))
            }
            TokenError::TokenNotYetValid => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_not_yet_valid".to_string(),
                    get_localized_message(
                        lang,
                        "Token is not yet valid",
                        "令牌尚未生效"
                    ),
                ))
            }
            TokenError::InvalidClaims => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_claims".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid token claims",
                        "令牌声明无效"
                    ),
                ))
            }
            TokenError::TokenRevoked => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "token_revoked".to_string(),
                    get_localized_message(
                        lang,
                        "Token has been revoked",
                        "令牌已被撤销"
                    ),
                ))
            }
            TokenError::RefreshTokenExpired => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "refresh_token_expired".to_string(),
                    get_localized_message(
                        lang,
                        "Refresh token has expired",
                        "刷新令牌已过期"
                    ),
                ))
            }
            TokenError::InvalidRefreshToken => {
                HttpResponse::Unauthorized().json(ErrorResponse::new(
                    "invalid_refresh_token".to_string(),
                    get_localized_message(
                        lang,
                        "Invalid refresh token",
                        "刷新令牌无效"
                    ),
                ))
            }
            TokenError::TokenGenerationFailed => {
                HttpResponse::InternalServerError().json(ErrorResponse::new(
                    "token_generation_failed".to_string(),
                    get_localized_message(
                        lang,
                        "Failed to generate token",
                        "生成令牌失败"
                    ),
                ))
            }
            TokenError::MissingClaim { claim } => {
                HttpResponse::BadRequest().json(ErrorResponse::new(
                    "missing_claim".to_string(),
                    get_localized_message(
                        lang,
                        &format!("Missing required claim: {}", claim),
                        &format!("缺少必需的声明：{}", claim)
                    ),
                ))
            }
        },
        DomainError::Validation { message } => {
            HttpResponse::BadRequest().json(ErrorResponse::new(
                "validation_error".to_string(),
                get_localized_message(
                    lang,
                    &message,
                    "验证错误"
                ),
            ))
        }
        DomainError::BusinessRule { message } => {
            HttpResponse::BadRequest().json(ErrorResponse::new(
                "business_rule_violation".to_string(),
                get_localized_message(
                    lang,
                    &message,
                    "业务规则违反"
                ),
            ))
        }
        DomainError::NotFound { resource } => {
            HttpResponse::NotFound().json(ErrorResponse::new(
                "not_found".to_string(),
                get_localized_message(
                    lang,
                    &format!("{} not found", resource),
                    &format!("{}不存在", resource)
                ),
            ))
        }
        DomainError::Unauthorized => {
            HttpResponse::Unauthorized().json(ErrorResponse::new(
                "unauthorized".to_string(),
                get_localized_message(
                    lang,
                    "Unauthorized access",
                    "未授权访问"
                ),
            ))
        }
        DomainError::Internal { message } => {
            log::error!("Internal error: {}", message);
            HttpResponse::InternalServerError().json(ErrorResponse::new(
                "internal_error".to_string(),
                get_localized_message(
                    lang,
                    "An internal server error occurred",
                    "发生内部服务器错误"
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_language_detection_chinese() {
        // Test Chinese language detection with high priority
        let req = TestRequest::default()
            .insert_header(("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8"))
            .to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::Chinese);
    }

    #[test]
    fn test_language_detection_english() {
        // Test English language detection with high priority
        let req = TestRequest::default()
            .insert_header(("Accept-Language", "en-US,en;q=0.9,zh-CN;q=0.8"))
            .to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_language_detection_default() {
        // Test default to English when no header
        let req = TestRequest::default().to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_localized_message_english() {
        let message = get_localized_message(Language::English, "Hello", "你好");
        assert_eq!(message, "Hello");
    }

    #[test]
    fn test_localized_message_chinese() {
        let message = get_localized_message(Language::Chinese, "Hello", "你好");
        assert_eq!(message, "你好");
    }
}