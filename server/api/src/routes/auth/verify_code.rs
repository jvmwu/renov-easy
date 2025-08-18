use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::dto::auth_dto::{VerifyCodeRequest, AuthResponse};
use crate::dto::error_dto::ErrorResponse;
use crate::handlers::error::{handle_domain_error_with_lang, Language};

use core::services::auth::AuthService;
use core::repositories::{UserRepository, TokenRepository};
use core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use core::services::auth::RateLimiterTrait;

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
    let lang = Language::from_request(&req);
    
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
        Err(error) => handle_domain_error_with_lang(error, lang),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    
    // Note: Comprehensive tests would require mock implementations of all the traits
    // This is a basic structure test to ensure the handler compiles correctly
    
    #[actix_rt::test]
    async fn test_verify_code_invalid_request() {
        // Test that validation works for invalid requests
        
        let request = VerifyCodeRequest {
            phone: "123".to_string(), // Too short
            country_code: "+1".to_string(),
            code: "123".to_string(), // Too short (should be 6 digits)
        };
        
        // Validate that the request fails validation
        assert!(request.validate().is_err());
    }
    
    #[actix_rt::test]
    async fn test_verify_code_valid_request() {
        // Test that validation passes for valid requests
        
        let request = VerifyCodeRequest {
            phone: "1234567890".to_string(),
            country_code: "+1".to_string(),
            code: "123456".to_string(),
        };
        
        // Validate that the request passes validation
        assert!(request.validate().is_ok());
    }
    
    #[actix_rt::test]
    async fn test_verify_code_invalid_code_length() {
        // Test that code must be exactly 6 digits
        
        let request_short = VerifyCodeRequest {
            phone: "1234567890".to_string(),
            country_code: "+1".to_string(),
            code: "12345".to_string(), // Too short
        };
        assert!(request_short.validate().is_err());
        
        let request_long = VerifyCodeRequest {
            phone: "1234567890".to_string(),
            country_code: "+1".to_string(),
            code: "1234567".to_string(), // Too long
        };
        assert!(request_long.validate().is_err());
    }
}