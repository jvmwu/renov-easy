use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;
use std::sync::Arc;
use std::collections::HashMap;

use crate::dto::auth::{SendCodeRequest, SendCodeResponse};
use crate::handlers::error::{handle_domain_error_with_lang, extract_language, Language};

use re_core::services::auth::AuthService;
use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::{RateLimiterTrait, mask_phone};
use re_shared::types::response::ErrorResponse;

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
    
    // Extract client IP for rate limiting
    let client_ip = extract_client_ip(&req);
    
    // Extract user agent for audit logging
    let user_agent = extract_user_agent(&req);
    
    // Validate request data using the validator
    if let Err(errors) = request.0.validate() {
        let mut details = HashMap::new();
        details.insert("validation_errors".to_string(), serde_json::json!(errors));
        
        let message = match lang {
            Language::English => "Invalid request data. Please check phone number format.",
            Language::Chinese => "请求数据无效。请检查电话号码格式。",
        };
        
        return HttpResponse::BadRequest().json(ErrorResponse::new(
            "validation_error".to_string(),
            message.to_string(),
        ).with_details(details));
    }

    // Format phone number with country code if not already included
    let phone = if request.phone.starts_with('+') {
        request.phone.clone()
    } else {
        format!("{}{}", request.country_code, request.phone)
    };

    // Additional validation for E.164 format and supported countries
    // The AuthService will perform detailed validation, but we can do a quick check here
    if !phone.starts_with('+') {
        let message = match lang {
            Language::English => "Phone number must include country code (e.g., +86 for China, +61 for Australia)",
            Language::Chinese => "电话号码必须包含国家代码（例如：中国 +86，澳大利亚 +61）",
        };
        
        return HttpResponse::BadRequest().json(ErrorResponse::new(
            "invalid_phone_format".to_string(),
            message.to_string(),
        ));
    }

    // Log the attempt for audit purposes
    log::info!(
        "Sending verification code to phone: {}, country_code: {}, ip: {}",
        mask_phone(&phone),
        request.country_code,
        &client_ip
    );

    // Call the auth service to send verification code with IP for rate limiting and user agent for audit
    match state.auth_service.send_verification_code(&phone, Some(client_ip.clone()), user_agent.clone()).await {
        Ok(result) => {
            // Calculate seconds until next resend is allowed
            let now = chrono::Utc::now();
            let duration = result.next_resend_at.signed_duration_since(now);
            let resend_after = duration.num_seconds().max(0);
            
            let message = match lang {
                Language::English => "Verification code sent successfully. Please check your SMS.",
                Language::Chinese => "验证码发送成功。请查看您的短信。",
            };
            
            // Log successful send for monitoring
            log::info!(
                "Verification code sent successfully to: {}, message_id: {}",
                mask_phone(&phone),
                result.message_id
            );
            
            HttpResponse::Ok().json(SendCodeResponse {
                message: message.to_string(),
                resend_after,
            })
        }
        Err(error) => {
            // Log error for monitoring and debugging
            log::error!(
                "Failed to send verification code to: {}, ip: {}, error: {:?}",
                mask_phone(&phone),
                client_ip,
                error
            );
            
            handle_domain_error_with_lang(&error, lang)
        }
    }
}

/// Extract client IP address from request
fn extract_client_ip(req: &HttpRequest) -> String {
    // Try to get IP from X-Forwarded-For header (for reverse proxy scenarios)
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // Take the first IP from the comma-separated list
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try to get IP from X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fall back to connection info
    req.connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string()
}

/// Extract user agent from request headers
fn extract_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

