//! Application state and factory
//!
//! This module handles the initialization of the application state
//! and provides the factory for creating the Actix-web application.

use std::sync::Arc;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};

use crate::middleware::{cors::create_cors, security::SecurityMiddleware, auth::JwtAuth};
use crate::routes::auth::{
    send_code::send_code, 
    verify_code::verify_code, 
    select_type::select_type, 
    refresh::refresh as refresh_token, 
    logout::logout,
    AppState
};

use re_core::services::auth::{AuthService, AuthServiceConfig, RateLimiterTrait};
use re_core::services::verification::{VerificationService, SmsServiceTrait, CacheServiceTrait};
use re_core::services::token::TokenService;
use re_core::repositories::{UserRepository, TokenRepository};

/// Create and configure the application with all dependencies
pub fn create_app<U, S, C, R, T>(
    app_state: web::Data<AppState<U, S, C, R, T>>
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = ()
    >
>
where
    U: UserRepository + 'static,
    S: SmsServiceTrait + 'static,
    C: CacheServiceTrait + 'static,
    R: RateLimiterTrait + 'static,
    T: TokenRepository + 'static,
{

    // Configure CORS using our custom middleware
    let cors = create_cors();
    
    // Configure security middleware
    let security = SecurityMiddleware::new();
    
    App::new()
        // Add application state
        .app_data(app_state)
        
        // Add middleware (order matters: security first, then CORS, then logging)
        .wrap(Logger::default())
        .wrap(cors)
        .wrap(security)
        
        // Health check endpoint
        .route("/health", web::get().to(health_check))
        
        // API v1 routes
        .service(
            web::scope("/api/v1")
                // Auth routes
                .service(
                    web::scope("/auth")
                        .route("/send-code", web::post().to(send_code::<U, S, C, R, T>))
                        .route("/verify-code", web::post().to(verify_code::<U, S, C, R, T>))
                        .route("/select-type", 
                            web::post()
                                .to(select_type::<U, S, C, R, T>)
                                .wrap(JwtAuth::new())
                        )
                        .route("/refresh", web::post().to(refresh_token::<U, S, C, R, T>))
                        .route("/logout", 
                            web::post()
                                .to(logout::<U, S, C, R, T>)
                                .wrap(JwtAuth::new())
                        )
                )
                // API documentation endpoint
                .route("/", web::get().to(api_documentation))
        )
        
        // Default 404 handler
        .default_service(web::route().to(not_found))
}

/// Health check endpoint handler
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "renov-easy-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// API documentation endpoint
async fn api_documentation() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "RenovEasy API v1",
        "endpoints": {
            "health": "/health",
            "auth": {
                "send_code": {
                    "path": "/api/v1/auth/send-code",
                    "method": "POST",
                    "description": "Send verification code via SMS",
                    "request_body": {
                        "phone": "string (10-15 chars)",
                        "country_code": "string (1-10 chars)"
                    },
                    "responses": {
                        "200": "Code sent successfully",
                        "400": "Invalid phone format",
                        "429": "Rate limit exceeded",
                        "503": "SMS service unavailable"
                    }
                },
                "verify_code": {
                    "path": "/api/v1/auth/verify-code",
                    "method": "POST",
                    "description": "Verify SMS code and authenticate",
                    "request_body": {
                        "phone": "string (10-15 chars)",
                        "country_code": "string (1-10 chars)",
                        "code": "string (exactly 6 chars)"
                    },
                    "responses": {
                        "200": "Authentication successful, returns tokens",
                        "400": "Invalid code or expired",
                        "403": "User blocked",
                        "429": "Max attempts exceeded"
                    }
                },
                "select_type": {
                    "path": "/api/v1/auth/select-type",
                    "method": "POST",
                    "description": "Select user type (customer/worker)",
                    "requires_auth": true,
                    "request_body": {
                        "user_type": "string ('customer' or 'worker')"
                    },
                    "responses": {
                        "200": "User type successfully selected",
                        "400": "Invalid user type",
                        "401": "Authentication required",
                        "403": "User type already selected"
                    }
                },
                "refresh": {
                    "path": "/api/v1/auth/refresh",
                    "method": "POST",
                    "description": "Refresh access token",
                    "status": "Coming soon"
                },
                "logout": {
                    "path": "/api/v1/auth/logout",
                    "method": "POST",
                    "description": "Logout and invalidate tokens",
                    "status": "Coming soon"
                }
            }
        }
    }))
}

/// Default 404 handler
async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "not_found",
        "message": "The requested resource was not found"
    }))
}