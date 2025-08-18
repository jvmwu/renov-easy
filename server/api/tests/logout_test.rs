//! Integration tests for logout endpoint

use actix_web::{http::header, test, web, App};
use api::app::create_app;
use api::routes::auth::AppState;
use core::repositories::{UserRepository, TokenRepository};
use core::services::{
    auth::{AuthService, AuthServiceConfig, RateLimiterTrait},
    token::TokenService,
    verification::{SmsServiceTrait, CacheServiceTrait, VerificationService},
};
use infra::services::InMemoryRateLimiter;
use std::sync::Arc;
use uuid::Uuid;

// Mock implementations for testing
use infra::repositories::MockUserRepository;
use infra::repositories::MockTokenRepository;
use infra::services::MockSmsService;
use infra::services::MockCacheService;

#[actix_web::test]
async fn test_logout_success() {
    // Setup mocks
    let user_repo = Arc::new(MockUserRepository::new());
    let token_repo = Arc::new(MockTokenRepository::new());
    let sms_service = Arc::new(MockSmsService::new());
    let cache_service = Arc::new(MockCacheService::new());
    let rate_limiter = Arc::new(InMemoryRateLimiter::new());
    
    // Setup services
    let verification_service = Arc::new(VerificationService::new(
        sms_service.clone(),
        cache_service.clone(),
    ));
    
    let token_service = Arc::new(TokenService::new(
        token_repo.clone(),
        "test_secret".to_string(),
    ));
    
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        verification_service.clone(),
        rate_limiter.clone(),
        token_service.clone(),
        AuthServiceConfig::default(),
    ));
    
    // Create app state
    let app_state = web::Data::new(AppState {
        auth_service,
        user_repository: user_repo,
        verification_service,
        rate_limiter,
        token_service: token_service.clone(),
    });
    
    // Create test user and generate token
    let user_id = Uuid::new_v4();
    let token_pair = token_service
        .generate_tokens(user_id, Some("customer".to_string()), true)
        .await
        .unwrap();
    
    // Create app
    let app = test::init_service(create_app(app_state)).await;
    
    // Make logout request with valid token
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token_pair.access_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Verify response
    assert_eq!(resp.status(), 200);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Logged out successfully");
}

#[actix_web::test]
async fn test_logout_without_auth() {
    // Setup mocks
    let user_repo = Arc::new(MockUserRepository::new());
    let token_repo = Arc::new(MockTokenRepository::new());
    let sms_service = Arc::new(MockSmsService::new());
    let cache_service = Arc::new(MockCacheService::new());
    let rate_limiter = Arc::new(InMemoryRateLimiter::new());
    
    // Setup services
    let verification_service = Arc::new(VerificationService::new(
        sms_service.clone(),
        cache_service.clone(),
    ));
    
    let token_service = Arc::new(TokenService::new(
        token_repo.clone(),
        "test_secret".to_string(),
    ));
    
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        verification_service.clone(),
        rate_limiter.clone(),
        token_service.clone(),
        AuthServiceConfig::default(),
    ));
    
    // Create app state
    let app_state = web::Data::new(AppState {
        auth_service,
        user_repository: user_repo,
        verification_service,
        rate_limiter,
        token_service,
    });
    
    // Create app
    let app = test::init_service(create_app(app_state)).await;
    
    // Make logout request without auth token
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Verify unauthorized response
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_logout_with_invalid_token() {
    // Setup mocks
    let user_repo = Arc::new(MockUserRepository::new());
    let token_repo = Arc::new(MockTokenRepository::new());
    let sms_service = Arc::new(MockSmsService::new());
    let cache_service = Arc::new(MockCacheService::new());
    let rate_limiter = Arc::new(InMemoryRateLimiter::new());
    
    // Setup services
    let verification_service = Arc::new(VerificationService::new(
        sms_service.clone(),
        cache_service.clone(),
    ));
    
    let token_service = Arc::new(TokenService::new(
        token_repo.clone(),
        "test_secret".to_string(),
    ));
    
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        verification_service.clone(),
        rate_limiter.clone(),
        token_service.clone(),
        AuthServiceConfig::default(),
    ));
    
    // Create app state
    let app_state = web::Data::new(AppState {
        auth_service,
        user_repository: user_repo,
        verification_service,
        rate_limiter,
        token_service,
    });
    
    // Create app
    let app = test::init_service(create_app(app_state)).await;
    
    // Make logout request with invalid token
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header((header::AUTHORIZATION, "Bearer invalid_token"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Verify unauthorized response
    assert_eq!(resp.status(), 401);
}