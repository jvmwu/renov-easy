use actix_web::web;

use crate::routes::auth::{send_code, verify_code};
use crate::middleware::error_handler::ErrorHandlerMiddleware;

/// Configure standardized error handling for the application
pub fn configure_error_handling(cfg: &mut web::ServiceConfig) {
    // Configure standardized auth routes with standardized errors
    cfg.service(
        web::scope("/api/v1")
            .wrap(ErrorHandlerMiddleware)
            .service(
                web::scope("/auth")
                    .route("/send-code", web::post().to(send_code::send_code::<_, _, _, _, _>))
                    .route("/verify-code", web::post().to(verify_code::verify_code::<_, _, _, _, _>))
            )
    );
}

/// Error response configuration options
#[derive(Debug, Clone)]
pub struct ErrorConfig {
    /// Include stack traces in error responses (development only)
    pub include_trace: bool,
    
    /// Include detailed error context in responses
    pub include_context: bool,
    
    /// Default language for error messages
    pub default_language: crate::i18n::Language,
    
    /// Enable request ID generation
    pub enable_request_id: bool,
    
    /// Enable response time metrics
    pub enable_metrics: bool,
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            include_trace: cfg!(debug_assertions), // Only in debug builds
            include_context: true,
            default_language: crate::i18n::Language::English,
            enable_request_id: true,
            enable_metrics: true,
        }
    }
}

impl ErrorConfig {
    /// Create production configuration
    pub fn production() -> Self {
        Self {
            include_trace: false,
            include_context: true,
            default_language: crate::i18n::Language::English,
            enable_request_id: true,
            enable_metrics: true,
        }
    }
    
    /// Create development configuration
    pub fn development() -> Self {
        Self {
            include_trace: true,
            include_context: true,
            default_language: crate::i18n::Language::English,
            enable_request_id: true,
            enable_metrics: true,
        }
    }
}