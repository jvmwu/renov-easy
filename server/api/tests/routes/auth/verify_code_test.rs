use actix_web::{test, web, App, http::StatusCode};
use re_api::routes::auth::verify_code::verify_code;
use re_api::routes::auth::send_code::AppState;
use re_api::dto::auth::{VerifyCodeRequest, AuthResponse};
use re_core::services::auth::{AuthService, InMemoryRateLimiter};
use re_core::services::verification::VerificationService;
use re_core::services::token::TokenService;
use re_core::repositories::mock::{MockUserRepository, MockTokenRepository};
use re_core::services::verification::mock::{MockSmsService, MockCacheService};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test application state
    fn create_test_app_state() -> AppState<
        MockUserRepository,
        MockSmsService,
        MockCacheService,
        InMemoryRateLimiter,
        MockTokenRepository,
    > {
        let user_repo = Arc::new(MockUserRepository::new());
        let sms_service = Arc::new(MockSmsService::new());
        let cache_service = Arc::new(MockCacheService::new());
        let rate_limiter = Arc::new(InMemoryRateLimiter::new());
        let token_repo = Arc::new(MockTokenRepository::new());
        
        let verification_service = Arc::new(VerificationService::new(
            sms_service.clone(),
            cache_service.clone(),
        ));
        
        let token_service = Arc::new(TokenService::new(
            token_repo.clone(),
            "test_secret".to_string(),
            900,    // 15 min access token
            2592000, // 30 day refresh token
        ));
        
        let auth_service = Arc::new(AuthService::new(
            user_repo.clone(),
            verification_service,
            rate_limiter.clone(),
            token_service,
            Default::default(),
        ));
        
        AppState { auth_service }
    }

    #[actix_web::test]
    async fn test_verify_code_invalid_request_data() {
        let app_state = web::Data::new(create_test_app_state());
        
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/verify-code", web::post().to(verify_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        // Test with invalid code length (less than 6 digits)
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .set_json(&VerifyCodeRequest {
                phone: "1234567890".to_string(),
                country_code: "+1".to_string(),
                code: "123".to_string(), // Invalid: too short
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        
        // Test with invalid code length (more than 6 digits)
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .set_json(&VerifyCodeRequest {
                phone: "1234567890".to_string(),
                country_code: "+1".to_string(),
                code: "1234567".to_string(), // Invalid: too long
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    
    #[actix_web::test]
    async fn test_verify_code_with_invalid_phone() {
        let app_state = web::Data::new(create_test_app_state());
        
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/verify-code", web::post().to(verify_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        // Test with empty phone number
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .set_json(&VerifyCodeRequest {
                phone: "".to_string(),
                country_code: "+1".to_string(),
                code: "123456".to_string(),
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        
        // Test with phone number that's too long
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .set_json(&VerifyCodeRequest {
                phone: "12345678901234567890".to_string(), // Too long
                country_code: "+1".to_string(),
                code: "123456".to_string(),
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    
    #[actix_web::test]
    async fn test_verify_code_language_support() {
        let app_state = web::Data::new(create_test_app_state());
        
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/verify-code", web::post().to(verify_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        // Test with Chinese language preference
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .insert_header(("Accept-Language", "zh-CN"))
            .set_json(&VerifyCodeRequest {
                phone: "".to_string(), // Invalid to trigger error
                country_code: "+86".to_string(),
                code: "123456".to_string(),
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["message"].as_str().unwrap().contains("请求数据无效"));
        
        // Test with English language preference
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .insert_header(("Accept-Language", "en-US"))
            .set_json(&VerifyCodeRequest {
                phone: "".to_string(), // Invalid to trigger error
                country_code: "+1".to_string(),
                code: "123456".to_string(),
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["message"].as_str().unwrap().contains("Invalid request data"));
    }
    
    #[actix_web::test]
    async fn test_verify_code_phone_formatting() {
        let app_state = web::Data::new(create_test_app_state());
        
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/verify-code", web::post().to(verify_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        // Store a verification code in the mock cache for testing
        let cache_service = &app_state.auth_service.verification_service.cache_service;
        cache_service.set("+11234567890_code", "123456", 300).await.unwrap();
        
        // Test with phone number that already includes country code
        let req = test::TestRequest::post()
            .uri("/verify-code")
            .set_json(&VerifyCodeRequest {
                phone: "+11234567890".to_string(), // Already includes country code
                country_code: "+1".to_string(),
                code: "123456".to_string(),
            })
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        // This should work correctly - the handler should detect the phone already has country code
        assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}