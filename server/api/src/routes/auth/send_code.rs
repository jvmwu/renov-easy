use actix_web::{web, HttpResponse};
use validator::Validate;
use std::sync::Arc;

use crate::dto::auth_dto::{SendCodeRequest, SendCodeResponse};
use crate::dto::error_dto::ErrorResponse;
use crate::handlers::error_handler::handle_domain_error;

use core::services::auth::AuthService;
use core::repositories::{UserRepository, TokenRepository};
use core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use core::services::auth::RateLimiterTrait;

/// Application state that holds shared services
pub struct AppState<U, S, C, R, T> 
where
    U: UserRepository,
    S: SmsServiceTrait,
    C: CacheServiceTrait,
    R: RateLimiterTrait,
    T: TokenRepository,
{
    pub auth_service: Arc<AuthService<U, S, C, R, T>>,
}

/// Handler for POST /api/v1/auth/send-code
///
/// Sends a verification code to the specified phone number.
///
/// # Request Body
/// 
/// ```json
/// {
///     "phone": "+1234567890",
///     "country_code": "+1"
/// }
/// ```
///
/// # Response
/// 
/// ## Success (200 OK)
/// ```json
/// {
///     "message": "Verification code sent successfully",
///     "resend_after": 60
/// }
/// ```
///
/// ## Errors
/// - 400 Bad Request: Invalid phone number format or validation errors
/// - 429 Too Many Requests: Rate limit exceeded
/// - 500 Internal Server Error: SMS service failure or other internal errors
pub async fn send_code<U, S, C, R, T>(
    state: web::Data<AppState<U, S, C, R, T>>,
    request: web::Json<SendCodeRequest>,
) -> HttpResponse
where
    U: UserRepository + 'static,
    S: SmsServiceTrait + 'static,
    C: CacheServiceTrait + 'static,
    R: RateLimiterTrait + 'static,
    T: TokenRepository + 'static,
{
    // Validate request data
    if let Err(errors) = request.validate() {
        let mut details = std::collections::HashMap::new();
        details.insert("validation_errors".to_string(), serde_json::json!(errors));
        
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation_error".to_string(),
            message: "Invalid request data".to_string(),
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

    // Call the auth service to send verification code
    match state.auth_service.send_verification_code(&phone).await {
        Ok(result) => {
            // Calculate seconds until next resend is allowed
            let now = chrono::Utc::now();
            let duration = result.next_resend_at.signed_duration_since(now);
            let resend_after = duration.num_seconds().max(0);
            
            HttpResponse::Ok().json(SendCodeResponse {
                message: "Verification code sent successfully".to_string(),
                resend_after,
            })
        }
        Err(error) => handle_domain_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    
    // Note: Comprehensive tests would require mock implementations of all the traits
    // This is a basic structure test to ensure the handler compiles correctly
    
    #[actix_rt::test]
    async fn test_send_code_invalid_request() {
        // This test verifies that validation works for invalid requests
        // Full implementation would require mocks for all dependencies
        
        let request = SendCodeRequest {
            phone: "123".to_string(), // Too short
            country_code: "+1".to_string(),
        };
        
        // Validate that the request fails validation
        assert!(request.validate().is_err());
    }
    
    #[actix_rt::test]
    async fn test_send_code_valid_request() {
        // This test verifies that validation passes for valid requests
        
        let request = SendCodeRequest {
            phone: "1234567890".to_string(),
            country_code: "+1".to_string(),
        };
        
        // Validate that the request passes validation
        assert!(request.validate().is_ok());
    }
}