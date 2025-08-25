use actix_web::{web, HttpRequest, HttpResponse};

use crate::dto::auth::{RefreshTokenRequest, AuthResponse as DtoAuthResponse};
use crate::handlers::error::{handle_domain_error_with_lang, extract_language};

use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;

use super::AppState;

/// Handler for POST /api/v1/auth/refresh
///
/// Refreshes an access token using a valid refresh token.
///
/// # Request Body
/// 
/// ```json
/// {
///     "refresh_token": "string"
/// }
/// ```
///
/// # Response
/// 
/// ## Success (200 OK)
/// ```json
/// {
///     "access_token": "eyJ...",
///     "refresh_token": "new_refresh_token_string",
///     "expires_in": 900,
///     "user_type": "customer",
///     "requires_type_selection": false
/// }
/// ```
///
/// ## Errors
/// - 401 Unauthorized: Invalid or expired refresh token
/// - 403 Forbidden: Token has been revoked or user is blocked
/// - 500 Internal Server Error: Token generation failure or other internal errors
pub async fn refresh_token<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<AppState<U, S, C, R, T>>,
    request: web::Json<RefreshTokenRequest>,
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
    
    // Call the auth service to refresh the token
    match state.auth_service.refresh_token(&request.refresh_token, Some(client_ip), Some(user_agent), None).await {
        Ok(auth_response) => {
            // Convert the domain AuthResponse to DTO AuthResponse
            let response = DtoAuthResponse {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_in: auth_response.expires_in,
                user_type: auth_response.user_type.map(|ut| ut.to_string()),
                requires_type_selection: auth_response.requires_type_selection,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(error) => handle_domain_error_with_lang(&error, lang),
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

