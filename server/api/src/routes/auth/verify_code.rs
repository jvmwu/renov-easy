use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::auth::{VerifyCodeRequest, AuthResponse};
use crate::handlers::error_standard::{to_standard_response, extract_language};
use crate::middleware::error_handler::ErrorHandlingExt;

use re_core::services::auth::AuthService;
use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::{RateLimiterTrait, mask_phone};
use re_core::errors::DomainError;
use re_shared::types::response::{DetailedResponse, ResponseStatus, ResponseMeta};
use chrono::Utc;
use std::collections::HashMap;

/// Handler for POST /api/v1/auth/verify-code with standardized error responses
///
/// Verifies the OTP code sent to the phone number and returns authentication tokens.
///
/// # Request Body
/// 
/// ```json
/// {
///     "phone": "+1234567890",
///     "code": "123456"
/// }
/// ```
///
/// # Response
/// 
/// ## Success (200 OK)
/// ```json
/// {
///     "status": "success",
///     "data": {
///         "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
///         "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
///         "token_type": "Bearer",
///         "expires_in": 900,
///         "user": {
///             "id": "550e8400-e29b-41d4-a716-446655440000",
///             "phone": "+1234567890",
///             "user_type": "customer",
///             "is_verified": true
///         }
///     },
///     "meta": {
///         "timestamp": "2025-08-14T10:00:00Z",
///         "version": "v1",
///         "request_id": "550e8400-e29b-41d4-a716-446655440000"
///     }
/// }
/// ```
pub async fn verify_code<U, S, C, R, T>(
    req: HttpRequest,
    state: web::Data<crate::routes::auth::AppState<U, S, C, R, T>>,
    request: web::Json<VerifyCodeRequest>,
) -> HttpResponse
where
    U: UserRepository + 'static,
    S: SmsServiceTrait + 'static,
    C: CacheServiceTrait + 'static,
    R: RateLimiterTrait + 'static,
    T: TokenRepository + 'static,
{
    // Extract or generate request ID
    let request_id = req
        .get_request_id()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Extract language preference
    let lang = req.get_language();
    
    // Extract client IP for security logging
    let client_ip = extract_client_ip(&req);
    
    // Extract user agent for audit logging
    let user_agent = extract_user_agent(&req);
    
    // Extract device info from headers
    let device_info = extract_device_info(&req);
    
    // Start timing for response metrics
    let start_time = std::time::Instant::now();
    
    // Log request with trace ID
    log::info!(
        "[{}] Processing verify_code request for phone: {}, ip: {}",
        request_id,
        mask_phone(&request.phone),
        client_ip
    );
    
    // Validate request data
    if let Err(validation_errors) = request.0.validate() {
        let mut field_errors = HashMap::new();
        
        for (field, errors) in validation_errors.field_errors() {
            let error_messages: Vec<String> = errors.iter()
                .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
                .collect();
            field_errors.insert(field.to_string(), error_messages);
        }
        
        log::warn!(
            "[{}] Validation failed for verify_code request: {:?}",
            request_id,
            field_errors
        );
        
        let response = DetailedResponse {
            status: ResponseStatus::Error,
            data: None::<()>,
            meta: ResponseMeta {
                timestamp: Utc::now(),
                version: "v1".to_string(),
                request_id: Some(request_id),
                response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                extra: HashMap::new(),
            },
            error: Some(re_shared::types::response::ErrorDetail {
                code: "VALIDATION_ERROR".to_string(),
                message: match lang {
                    crate::i18n::Language::English => "Invalid request data. Please check your input.".to_string(),
                    crate::i18n::Language::Chinese => "请求数据无效。请检查您的输入。".to_string(),
                },
                fields: Some(field_errors),
                trace: None,
                context: Some({
                    let mut ctx = HashMap::new();
                    ctx.insert("path".to_string(), serde_json::json!(req.path()));
                    ctx.insert("method".to_string(), serde_json::json!(req.method().to_string()));
                    ctx
                }),
            }),
        };
        
        return HttpResponse::BadRequest().json(response);
    }

    // Format phone number with country code if needed
    let phone = if request.phone.starts_with('+') {
        request.phone.clone()
    } else if request.phone.starts_with("+86") || request.phone.starts_with("+61") {
        request.phone.clone()
    } else {
        request.phone.clone()
    };

    // Log verification attempt for security audit
    log::info!(
        "[{}] Verifying code for phone: {}, ip: {}, device: {:?}",
        request_id,
        mask_phone(&phone),
        client_ip,
        device_info
    );

    // Call the auth service to verify the code
    match state.auth_service.verify_code(&phone, &request.code, Some(client_ip.clone()), user_agent, device_info.clone()).await {
        Ok(auth_result) => {
            // Log successful verification
            log::info!(
                "[{}] Code verified successfully for phone: {}, user_id: {}",
                request_id,
                mask_phone(&phone),
                auth_result.user_type.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "pending".to_string())
            );
            
            // Build standardized success response
            let response = DetailedResponse {
                status: ResponseStatus::Success,
                data: Some(AuthResponse {
                    access_token: auth_result.access_token,
                    refresh_token: auth_result.refresh_token,
                    expires_in: auth_result.expires_in,
                    user_type: auth_result.user_type,
                    requires_type_selection: auth_result.requires_type_selection,
                }),
                meta: ResponseMeta {
                    timestamp: Utc::now(),
                    version: "v1".to_string(),
                    request_id: Some(request_id),
                    response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    extra: {
                        let mut extra = HashMap::new();
                        extra.insert("authentication_method".to_string(), serde_json::json!("sms_otp"));
                        extra
                    },
                },
                error: None,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(error) => {
            // Log error with appropriate level based on error type
            match &error {
                DomainError::Auth(re_core::errors::AuthError::InvalidVerificationCode) |
                DomainError::Auth(re_core::errors::AuthError::VerificationCodeExpired) => {
                    log::warn!(
                        "[{}] Invalid or expired code for phone: {}, ip: {}",
                        request_id,
                        mask_phone(&phone),
                        client_ip
                    );
                }
                DomainError::Auth(re_core::errors::AuthError::MaxAttemptsExceeded) => {
                    log::warn!(
                        "[{}] Max attempts exceeded for phone: {}, ip: {} - possible brute force attempt",
                        request_id,
                        mask_phone(&phone),
                        client_ip
                    );
                }
                _ => {
                    log::error!(
                        "[{}] Failed to verify code for phone: {}, ip: {}, error: {:?}",
                        request_id,
                        mask_phone(&phone),
                        client_ip,
                        error
                    );
                }
            }
            
            // Use standardized error response
            to_standard_response(&error, &req)
        }
    }
}

/// Extract client IP address from request
fn extract_client_ip(req: &HttpRequest) -> String {
    // Try to get IP from X-Forwarded-For header (for reverse proxy scenarios)
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
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

/// Extract device information from request headers
fn extract_device_info(req: &HttpRequest) -> Option<String> {
    // Try to get device info from custom header first
    if let Some(device_header) = req.headers().get("X-Device-Info") {
        if let Ok(device_str) = device_header.to_str() {
            return Some(device_str.to_string());
        }
    }
    
    // Fall back to User-Agent if no specific device info
    extract_user_agent(req)
}