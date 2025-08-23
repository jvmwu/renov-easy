use actix_web::{web, HttpRequest, HttpResponse};

use crate::dto::auth::{RefreshTokenRequest, AuthResponse as DtoAuthResponse};
use crate::handlers::error::{handle_domain_error_with_lang, Language, extract_language};

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
    
    // Call the auth service to refresh the token
    match state.auth_service.refresh_token(&request.refresh_token).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::auth::RefreshTokenRequest;

    #[test]
    fn test_refresh_token_request_structure() {
        // Test that the RefreshTokenRequest structure is valid
        let request = RefreshTokenRequest {
            refresh_token: "test_token_123".to_string(),
        };
        
        assert_eq!(request.refresh_token, "test_token_123");
    }
}