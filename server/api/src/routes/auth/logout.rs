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
    
    // Call the auth service to logout the user
    match state.auth_service.logout(auth.user_id).await {
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

