use actix_web::{web, HttpRequest, HttpResponse};
use validator::Validate;
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::auth::{SendCodeRequest, SendCodeResponse};
use crate::handlers::error_standard::{StandardApiError, to_standard_response, extract_language};
use crate::middleware::error_handler::ErrorHandlingExt;

use re_core::services::auth::AuthService;
use re_core::repositories::{UserRepository, TokenRepository};
use re_core::services::verification::{SmsServiceTrait, CacheServiceTrait};
use re_core::services::auth::{RateLimiterTrait, mask_phone};
use re_core::errors::ValidationError as DomainValidationError;
use re_core::errors::DomainError;
use re_shared::types::response::{DetailedResponse, ResponseStatus, ResponseMeta, ErrorDetail};
use chrono::Utc;
use std::collections::HashMap;

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

/// Handler for POST /api/v1/auth/send-code with standardized error responses
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
///     "status": "success",
///     "data": {
///         "message": "Verification code sent successfully",
///         "resend_after": 60
///     },
///     "meta": {
///         "timestamp": "2025-08-14T10:00:00Z",
///         "version": "v1",
///         "request_id": "550e8400-e29b-41d4-a716-446655440000"
///     }
/// }
/// ```
///
/// ## Errors
/// Standardized error responses with appropriate HTTP status codes
pub async fn send_code<U, S, C, R, T>(
    req: HttpRequest,
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
    // Extract or generate request ID
    let request_id = req
        .get_request_id()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Extract language preference
    let lang = req.get_language();
    
    // Extract client IP for rate limiting
    let client_ip = extract_client_ip(&req);
    
    // Extract user agent for audit logging
    let user_agent = extract_user_agent(&req);
    
    // Start timing for response metrics
    let start_time = std::time::Instant::now();
    
    // Log request with trace ID
    log::info!(
        "[{}] Processing send_code request for phone: {}, ip: {}",
        request_id,
        mask_phone(&request.phone),
        client_ip
    );
    
    // Validate request data
    if let Err(validation_errors) = request.0.validate() {
        let mut field_errors = HashMap::new();
        
        // Convert validation errors to field-specific errors
        for (field, errors) in validation_errors.field_errors() {
            let error_messages: Vec<String> = errors.iter()
                .map(|e| e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()))
                .collect();
            field_errors.insert(field.to_string(), error_messages);
        }
        
        let _error = DomainError::ValidationErr(DomainValidationError::InvalidFormat {
            field: "phone".to_string(),
        });
        
        log::warn!(
            "[{}] Validation failed for send_code request: {:?}",
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
            error: Some(ErrorDetail {
                code: "VALIDATION_ERROR".to_string(),
                message: match lang {
                    crate::i18n::Language::English => "Invalid request data. Please check phone number format.".to_string(),
                    crate::i18n::Language::Chinese => "请求数据无效。请检查电话号码格式。".to_string(),
                },
                fields: Some(field_errors.into_iter().map(|(k, v)| (k, v)).collect()),
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

    // Format phone number with country code
    let phone = if request.phone.starts_with('+') {
        request.phone.clone()
    } else {
        format!("{}{}", request.country_code, request.phone)
    };

    // Validate E.164 format
    if !phone.starts_with('+') || phone.len() < 8 || phone.len() > 16 {
        let error = DomainError::ValidationErr(DomainValidationError::InvalidFormat {
            field: "phone".to_string(),
        });
        
        log::warn!(
            "[{}] Invalid phone format: {}",
            request_id,
            mask_phone(&phone)
        );
        
        return to_standard_response(&error, &req);
    }

    // Log the attempt for audit purposes
    log::info!(
        "[{}] Sending verification code to phone: {}, ip: {}",
        request_id,
        mask_phone(&phone),
        client_ip
    );

    // Call the auth service
    match state.auth_service.send_verification_code(&phone, Some(client_ip.clone()), user_agent.clone()).await {
        Ok(result) => {
            // Calculate seconds until next resend is allowed
            let now = chrono::Utc::now();
            let duration = result.next_resend_at.signed_duration_since(now);
            let resend_after = duration.num_seconds().max(0);
            
            let message = match lang {
                crate::i18n::Language::English => "Verification code sent successfully. Please check your SMS.",
                crate::i18n::Language::Chinese => "验证码发送成功。请查看您的短信。",
            };
            
            // Log successful send
            log::info!(
                "[{}] Verification code sent successfully to: {}, message_id: {}",
                request_id,
                mask_phone(&phone),
                result.message_id
            );
            
            // Build standardized success response
            let response = DetailedResponse {
                status: ResponseStatus::Success,
                data: Some(SendCodeResponse {
                    message: message.to_string(),
                    resend_after,
                }),
                meta: ResponseMeta {
                    timestamp: Utc::now(),
                    version: "v1".to_string(),
                    request_id: Some(request_id),
                    response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    extra: {
                        let mut extra = HashMap::new();
                        extra.insert("message_id".to_string(), serde_json::json!(result.message_id));
                        extra
                    },
                },
                error: None,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(error) => {
            // Log error with trace ID
            log::error!(
                "[{}] Failed to send verification code to: {}, ip: {}, error: {:?}",
                request_id,
                mask_phone(&phone),
                client_ip,
                error
            );
            
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