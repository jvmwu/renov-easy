use actix_web::{web, HttpRequest, HttpResponse};

use crate::dto::auth::LogoutResponse;
use crate::handlers::error::{handle_domain_error_with_lang, Language, extract_language};
use crate::middleware::auth::AuthContext;

use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;

use super::AppState;

/// Handler for POST /api/v1/auth/logout
///
/// Logs out a user by revoking all their access and refresh tokens.
/// Requires authentication via Bearer token in Authorization header.
///
/// # Headers
/// 
/// ```
/// Authorization: Bearer {access_token}
/// ```
///
/// # Response
/// 
/// ## Success (200 OK)
/// ```json
/// {
///     "message": "Logged out successfully"
/// }
/// ```
///
/// ## Errors
/// - 401 Unauthorized: Missing or invalid access token
/// - 500 Internal Server Error: Token revocation failure
pub async fn logout<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<AppState<U, S, C, R, T>>,
    auth: AuthContext,
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
    
    // Extract access token from Authorization header for blacklisting
    let access_token = req.headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        });
    
    // Call the auth service to logout the user
    match state.auth_service.logout(auth.user_id, access_token, Some(client_ip), user_agent, None).await {
        Ok(()) => {
            let message = match lang {
                Language::English => "Logged out successfully",
                Language::Chinese => "登出成功",
            };
            
            let response = LogoutResponse {
                message: message.to_string(),
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

