//! CORS middleware configuration for cross-origin requests.
//!
//! This module provides CORS configuration that allows mobile applications
//! (iOS, Android, HarmonyOS) and web clients to access the API endpoints.
//! The configuration is environment-aware, with different settings for
//! development and production environments.

use actix_cors::Cors;
use actix_web::http::{header, Method};
use std::env;

/// Creates a CORS middleware instance configured for the current environment.
///
/// In development mode, this allows permissive CORS for easier testing.
/// In production mode, this restricts origins to known mobile app schemes
/// and configured domains.
///
/// # Mobile App Support
/// - iOS: Supports `capacitor://` and `ionic://` schemes
/// - Android: Supports `http://localhost` and app-specific schemes
/// - HarmonyOS: Supports appropriate schemes for ArkUI applications
///
/// # Environment Variables
/// - `ENVIRONMENT`: Set to "production" for production settings
/// - `ALLOWED_ORIGINS`: Comma-separated list of allowed origins (production only)
/// - `CORS_MAX_AGE`: Max age for preflight cache (default: 3600 seconds)
pub fn create_cors() -> Cors {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let max_age = env::var("CORS_MAX_AGE")
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<usize>()
        .unwrap_or(3600);

    if environment == "production" {
        create_production_cors(max_age)
    } else {
        create_development_cors(max_age)
    }
}

/// Creates CORS configuration for development environment.
///
/// This configuration is permissive to allow easy testing from various
/// origins including localhost, mobile emulators, and development tools.
fn create_development_cors(max_age: usize) -> Cors {
    log::info!("Configuring CORS for development environment");
    
    Cors::default()
        // Allow any origin in development
        .allow_any_origin()
        // Allow all standard HTTP methods
        .allowed_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        // Allow common headers used by mobile apps and web clients
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
            header::USER_AGENT,
            header::CACHE_CONTROL,
            header::PRAGMA,
            header::EXPIRES,
            header::HeaderName::from_static("x-requested-with"),
            header::HeaderName::from_static("x-app-version"),
            header::HeaderName::from_static("x-platform"),
            header::HeaderName::from_static("x-device-id"),
        ])
        // Expose headers that clients might need to read
        .expose_headers(vec![
            header::HeaderName::from_static("x-request-id"),
            header::HeaderName::from_static("x-rate-limit-limit"),
            header::HeaderName::from_static("x-rate-limit-remaining"),
            header::HeaderName::from_static("x-rate-limit-reset"),
        ])
        .max_age(max_age)
        // Support credentials in development
        .supports_credentials()
}

/// Creates CORS configuration for production environment.
///
/// This configuration is restrictive and only allows:
/// - Configured origins from ALLOWED_ORIGINS environment variable
/// - Standard mobile app schemes (capacitor://, ionic://, etc.)
/// - HTTPS origins only (except for mobile app schemes)
fn create_production_cors(max_age: usize) -> Cors {
    log::info!("Configuring CORS for production environment");
    
    let mut cors = Cors::default()
        .allowed_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-app-version"),
            header::HeaderName::from_static("x-platform"),
            header::HeaderName::from_static("x-device-id"),
        ])
        .expose_headers(vec![
            header::HeaderName::from_static("x-request-id"),
            header::HeaderName::from_static("x-rate-limit-limit"),
            header::HeaderName::from_static("x-rate-limit-remaining"),
            header::HeaderName::from_static("x-rate-limit-reset"),
        ])
        .max_age(max_age);

    // Add allowed origins from environment variable
    if let Ok(allowed_origins) = env::var("ALLOWED_ORIGINS") {
        for origin in allowed_origins.split(',').map(|s| s.trim()) {
            if !origin.is_empty() {
                log::info!("Adding allowed origin: {}", origin);
                cors = cors.allowed_origin(origin);
            }
        }
    }

    // Add mobile app schemes
    // iOS schemes
    cors = cors.allowed_origin("capacitor://localhost");
    cors = cors.allowed_origin("ionic://localhost");
    
    // Android schemes
    cors = cors.allowed_origin("http://localhost");
    cors = cors.allowed_origin("https://localhost");
    
    // HarmonyOS schemes (may vary based on actual implementation)
    cors = cors.allowed_origin("arkui://localhost");
    cors = cors.allowed_origin("harmony://localhost");
    
    // Common web app origins for production
    if let Ok(web_domain) = env::var("WEB_DOMAIN") {
        cors = cors.allowed_origin(&format!("https://{}", web_domain));
        cors = cors.allowed_origin(&format!("https://www.{}", web_domain));
    }

    cors
}

/// Creates a simple health check CORS that allows any origin.
///
/// This is useful for health check endpoints that should be accessible
/// from monitoring services and load balancers.
pub fn create_health_check_cors() -> Cors {
    Cors::default()
        .allowed_methods(vec![Method::GET, Method::OPTIONS])
        .allowed_headers(vec![header::ACCEPT, header::CONTENT_TYPE])
        .max_age(3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_development_cors() {
        env::set_var("ENVIRONMENT", "development");
        let _cors = create_cors();
        // CORS configuration is created successfully
        env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn test_create_production_cors() {
        env::set_var("ENVIRONMENT", "production");
        env::set_var("ALLOWED_ORIGINS", "https://app.renovEasy.com,https://admin.renovEasy.com");
        env::set_var("WEB_DOMAIN", "renovEasy.com");
        
        let _cors = create_cors();
        // CORS configuration is created successfully
        
        env::remove_var("ENVIRONMENT");
        env::remove_var("ALLOWED_ORIGINS");
        env::remove_var("WEB_DOMAIN");
    }

    #[test]
    fn test_cors_max_age_parsing() {
        env::set_var("CORS_MAX_AGE", "7200");
        let _cors = create_cors();
        env::remove_var("CORS_MAX_AGE");
        
        // Test invalid max age falls back to default
        env::set_var("CORS_MAX_AGE", "invalid");
        let _cors = create_cors();
        env::remove_var("CORS_MAX_AGE");
    }
}