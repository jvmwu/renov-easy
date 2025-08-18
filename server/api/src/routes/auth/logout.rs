use actix_web::{web, HttpResponse};

use crate::dto::auth_dto::LogoutResponse;
use crate::handlers::error::handle_domain_error;
use crate::middleware::auth::AuthContext;

use core::repositories::{UserRepository, TokenRepository};
use core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use core::services::auth::RateLimiterTrait;

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
    // Call the auth service to logout the user
    match state.auth_service.logout(auth.user_id).await {
        Ok(()) => {
            let response = LogoutResponse {
                message: "Logged out successfully".to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(error) => handle_domain_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::auth_dto::LogoutResponse;
    use uuid::Uuid;

    #[test]
    fn test_logout_response_structure() {
        // Test that the LogoutResponse structure is valid
        let response = LogoutResponse {
            message: "Logged out successfully".to_string(),
        };
        
        assert_eq!(response.message, "Logged out successfully");
    }

    #[test]
    fn test_auth_context_user_id() {
        // Test that AuthContext properly holds user_id
        let user_id = Uuid::new_v4();
        let auth_context = AuthContext {
            user_id,
            user_type: Some("customer".to_string()),
            is_verified: true,
            jti: "test_jti".to_string(),
        };
        
        assert_eq!(auth_context.user_id, user_id);
    }
}