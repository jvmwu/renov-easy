use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;
use std::sync::Arc;

use crate::dto::auth::{SendCodeRequest, SendCodeResponse};
use crate::dto::error::ErrorResponse;
use crate::handlers::error::{handle_domain_error_with_lang, extract_language, Language};

use re_core::services::auth::AuthService;
use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;

/// Application state that holds shared services
pub struct AppState<U, S, C, R, T> 
where
    U: UserRepository,
    S: SmsServiceTrait,
    C: CacheServiceTrait,
    R: RateLimiterTrait,
    T: TokenRepository,
{
    pub auth_service: Arc<AuthService<U, S, C, R, T>>,
}

/// Handler for POST /api/v1/auth/send-code
///
/// Sends a verification code to the specified phone number.
///
/// # Request Body
/// 
/// ```json
/// {
///     "phone": "+1234567890",
///     "country_code": "+1"
/// }
/// ```
///
/// # Response
/// 
/// ## Success (200 OK)
/// ```json
/// {
///     "message": "Verification code sent successfully",
///     "resend_after": 60
/// }
/// ```
///
/// ## Errors
/// - 400 Bad Request: Invalid phone number format or validation errors
/// - 429 Too Many Requests: Rate limit exceeded
/// - 500 Internal Server Error: SMS service failure or other internal errors
pub async fn send_code<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<AppState<U, S, C, R, T>>,
    request: web::Json<SendCodeRequest>,
) -> HttpResponse
where
    U: UserRepository + 'static,
    S: SmsServiceTrait + 'static,
    C: CacheServiceTrait + 'static,
    R: RateLimiterTrait + 'static,
    T: TokenRepository + 'static,
{
    // Detect language preference from request headers
    let lang = extract_language(&req);
    
    // Validate request data
    if let Err(errors) = request.validate() {
        let mut details = std::collections::HashMap::new();
        details.insert("validation_errors".to_string(), serde_json::json!(errors));
        
        let message = match lang {
            Language::English => "Invalid request data",
            Language::Chinese => "请求数据无效",
        };
        
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation_error".to_string(),
            message: message.to_string(),
            details: Some(details),
            timestamp: chrono::Utc::now(),
        });
    }

    // Format phone number with country code if not already included
    let phone = if request.phone.starts_with('+') {
        request.phone.clone()
    } else {
        format!("{}{}", request.country_code, request.phone)
    };

    // Call the auth service to send verification code
    match state.auth_service.send_verification_code(&phone).await {
        Ok(result) => {
            // Calculate seconds until next resend is allowed
            let now = chrono::Utc::now();
            let duration = result.next_resend_at.signed_duration_since(now);
            let resend_after = duration.num_seconds().max(0);
            
            let message = match lang {
                Language::English => "Verification code sent successfully",
                Language::Chinese => "验证码发送成功",
            };
            
            HttpResponse::Ok().json(SendCodeResponse {
                message: message.to_string(),
                resend_after,
            })
        }
        Err(error) => handle_domain_error_with_lang(&error, lang),
    }
}

