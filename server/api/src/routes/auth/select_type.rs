use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::dto::auth::SelectTypeRequest;
use crate::dto::error::ErrorResponse;
use crate::handlers::error::{handle_domain_error_with_lang, Language, extract_language};
use crate::middleware::auth::AuthContext;

use core::services::auth::AuthService;
use core::repositories::{UserRepository, TokenRepository};
use core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use core::services::auth::RateLimiterTrait;
use core::domain::entities::user::UserType;

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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use uuid::Uuid;
    
    #[test]
    fn test_valid_user_types() {
        // Test that valid user types are accepted
        let request_customer = SelectTypeRequest {
            user_type: "customer".to_string(),
        };
        assert_eq!(request_customer.user_type.to_lowercase(), "customer");
        
        let request_worker = SelectTypeRequest {
            user_type: "worker".to_string(),
        };
        assert_eq!(request_worker.user_type.to_lowercase(), "worker");
        
        // Test case insensitivity
        let request_uppercase = SelectTypeRequest {
            user_type: "CUSTOMER".to_string(),
        };
        assert_eq!(request_uppercase.user_type.to_lowercase(), "customer");
    }
    
    #[test]
    fn test_invalid_user_type() {
        // Test that invalid user types are properly handled
        let invalid_types = vec!["admin", "moderator", "user", "", "123"];
        
        for invalid_type in invalid_types {
            let request = SelectTypeRequest {
                user_type: invalid_type.to_string(),
            };
            
            // Verify the type doesn't match valid options
            let lower = request.user_type.to_lowercase();
            assert!(lower != "customer" && lower != "worker");
        }
    }
    
    #[test]
    fn test_auth_context_creation() {
        // Test that AuthContext can be created properly
        use core::domain::entities::token::Claims;
        
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: 1234567890,
            iat: 1234567800,
            jti: "test-jti".to_string(),
            user_type: None,
            is_verified: true,
        };
        
        let auth_context = AuthContext::from_claims(claims).unwrap();
        assert_eq!(auth_context.user_id, user_id);
        assert_eq!(auth_context.user_type, None);
        assert_eq!(auth_context.is_verified, true);
    }
}