use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::dto::auth::{VerifyCodeRequest, AuthResponse};
use crate::handlers::error::{handle_domain_error_with_lang, extract_language, Language};

use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;
use re_shared::types::response::ErrorResponse;

use super::AppState;

/// Handler for POST /api/v1/auth/verify-code
///
/// Verifies the SMS code sent to a phone number and authenticates the user.
/// Handles both new user registration and existing user login.
///
/// # Request Body
///
/// ```json
/// {
///     "phone": "1234567890",
///     "country_code": "+1",
///     "code": "123456"
/// }
/// ```
///
/// # Response
///
/// ## Success (200 OK)
/// ```json
/// {
///     "access_token": "eyJhbGciOiJIUzI1NiIs...",
///     "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
///     "expires_in": 3600,
///     "user_type": "customer" | "worker" | null,
///     "requires_type_selection": true | false
/// }
/// ```
///
/// ## Errors
/// - 400 Bad Request: Invalid request data, invalid code, or expired code
/// - 403 Forbidden: User blocked
/// - 429 Too Many Requests: Max verification attempts exceeded
/// - 500 Internal Server Error: Database or token generation failure
pub async fn verify_code<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<AppState<U, S, C, R, T>>,
    request: web::Json<VerifyCodeRequest>,
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
    
    // Extract client IP and user agent for audit logging
    let client_ip = extract_client_ip(&req);
    let user_agent = extract_user_agent(&req);

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

    // Log the verification attempt
    log::info!(
        "OTP verification attempt - phone: {}, client_ip: {}, user_agent: {}",
        re_core::services::auth::mask_phone(&phone),
        client_ip,
        user_agent
    );

    // Call the auth service to verify the code with IP for rate limiting
    match state.auth_service.verify_code(&phone, &request.code, Some(client_ip.clone())).await {
        Ok(auth_response) => {
            // Log successful verification
            log::info!(
                "OTP verification successful - phone: {}, user_type: {:?}, requires_type_selection: {}, client_ip: {}",
                re_core::services::auth::mask_phone(&phone),
                auth_response.user_type,
                auth_response.requires_type_selection,
                client_ip
            );
            
            // Convert domain AuthResponse to API AuthResponse
            HttpResponse::Ok().json(AuthResponse {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_in: auth_response.expires_in,
                user_type: auth_response.user_type,
                requires_type_selection: auth_response.requires_type_selection,
            })
        }
        Err(error) => {
            // Log failed verification
            log::warn!(
                "OTP verification failed - phone: {}, error: {}, client_ip: {}",
                re_core::services::auth::mask_phone(&phone),
                error,
                client_ip
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
fn extract_user_agent(req: &HttpRequest) -> String {
    req.headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

