use actix_web::{web, HttpRequest, HttpResponse};

use crate::dto::auth::SelectTypeRequest;
use crate::dto::error::ErrorResponse;
use crate::handlers::error::{handle_domain_error_with_lang, Language, extract_language};
use crate::middleware::auth::AuthContext;

use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::RateLimiterTrait;
use re_core::domain::entities::user::UserType;

use super::AppState;

/// Handler for POST /api/v1/auth/select-type
///
/// Allows authenticated users to select their user type (customer or worker).
/// This endpoint can only be called once - after initial registration.
///
/// # Request Headers
///
/// ```
/// Authorization: Bearer {access_token}
/// ```
///
/// # Request Body
///
/// ```json
/// {
///     "user_type": "customer" | "worker"
/// }
/// ```
///
/// # Response
///
/// ## Success (200 OK)
/// ```json
/// {
///     "message": "User type successfully selected",
///     "user_type": "customer" | "worker"
/// }
/// ```
///
/// ## Errors
/// - 400 Bad Request: Invalid user type provided
/// - 401 Unauthorized: Missing or invalid authentication token
/// - 403 Forbidden: User type already selected (cannot be changed)
/// - 404 Not Found: User not found
/// - 500 Internal Server Error: Database update failure
pub async fn select_type<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<AppState<U, S, C, R, T>>,
    auth: AuthContext,
    request: web::Json<SelectTypeRequest>,
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

    // Parse user type from request
    let user_type = match request.user_type.to_lowercase().as_str() {
        "customer" => UserType::Customer,
        "worker" => UserType::Worker,
        _ => {
            let mut details = std::collections::HashMap::new();
            details.insert(
                "invalid_value".to_string(),
                serde_json::json!(request.user_type)
            );
            details.insert(
                "valid_values".to_string(),
                serde_json::json!(["customer", "worker"])
            );

            let message = match lang {
                Language::English => "Invalid user type. Must be 'customer' or 'worker'",
                Language::Chinese => "无效的用户类型。必须是 'customer' 或 'worker'",
            };

            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "validation_error".to_string(),
                message: message.to_string(),
                details: Some(details),
                timestamp: chrono::Utc::now(),
            });
        }
    };

    // Call the auth service to update user type
    match state.auth_service.select_user_type(auth.user_id, user_type).await {
        Ok(()) => {
            // Success response with localized message
            let message = match lang {
                Language::English => "User type successfully selected",
                Language::Chinese => "用户类型选择成功",
            };

            HttpResponse::Ok().json(serde_json::json!({
                "message": message,
                "user_type": request.user_type.to_lowercase()
            }))
        }
        Err(error) => handle_domain_error_with_lang(&error, lang),
    }
}
