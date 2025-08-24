use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::dto::auth::{VerifyCodeRequest, AuthResponse};
use crate::dto::error::{ErrorResponse, ErrorResponseExt};
use crate::handlers::error::{handle_domain_error_with_lang, extract_language, Language};

use re_core::services::auth::AuthService;
use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;

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

    // Call the auth service to verify the code
    match state.auth_service.verify_code(&phone, &request.code).await {
        Ok(auth_response) => {
            // Convert domain AuthResponse to API AuthResponse
            HttpResponse::Ok().json(AuthResponse {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_in: auth_response.expires_in,
                user_type: auth_response.user_type,
                requires_type_selection: auth_response.requires_type_selection,
            })
        }
        Err(error) => handle_domain_error_with_lang(&error, lang),
    }
}

