use actix_web::{test, web, App, http::StatusCode};
use re_api::routes::auth::send_code::{send_code, AppState};
use re_api::dto::auth::{SendCodeRequest, SendCodeResponse};
use re_core::services::auth::AuthService;
use re_core::services::verification::{SendCodeResult, VerificationService};
use re_core::services::auth::InMemoryRateLimiter;
use re_core::services::token::TokenService;
use re_core::repositories::mock::{MockUserRepository, MockTokenRepository};
use re_core::services::verification::mock::{MockSmsService, MockCacheService};
use std::sync::Arc;
use chrono::Utc;

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
            3600, // 1 hour access token
            86400, // 24 hour refresh token
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
    async fn test_send_code_success_chinese_phone() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .set_json(&SendCodeRequest {
                phone: "13812345678".to_string(),
                country_code: "+86".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body: SendCodeResponse = test::read_body_json(resp).await;
        assert!(body.message.contains("success") || body.message.contains("成功"));
        assert!(body.resend_after >= 0);
    }

    #[actix_web::test]
    async fn test_send_code_success_australian_phone() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .set_json(&SendCodeRequest {
                phone: "412345678".to_string(),
                country_code: "+61".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body: SendCodeResponse = test::read_body_json(resp).await;
        assert!(body.message.contains("success") || body.message.contains("成功"));
        assert!(body.resend_after >= 0);
    }

    #[actix_web::test]
    async fn test_send_code_invalid_phone_format() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .set_json(&SendCodeRequest {
                phone: "123".to_string(), // Too short
                country_code: "+86".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_send_code_with_full_e164_format() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .set_json(&SendCodeRequest {
                phone: "+8613812345678".to_string(), // Full E.164 format
                country_code: "+86".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_send_code_with_chinese_language_header() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .insert_header(("Accept-Language", "zh-CN"))
            .set_json(&SendCodeRequest {
                phone: "13812345678".to_string(),
                country_code: "+86".to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body: SendCodeResponse = test::read_body_json(resp).await;
        assert!(body.message.contains("成功"));
    }

    #[actix_web::test]
    async fn test_send_code_rate_limit() {
        let app_state = create_test_app_state();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/api/v1/auth/send-code", web::post().to(send_code::<
                    MockUserRepository,
                    MockSmsService,
                    MockCacheService,
                    InMemoryRateLimiter,
                    MockTokenRepository,
                >))
        ).await;
        
        let phone = "13812345678";
        let country_code = "+86";
        
        // Send 3 requests (should succeed)
        for _ in 0..3 {
            let req = test::TestRequest::post()
                .uri("/api/v1/auth/send-code")
                .set_json(&SendCodeRequest {
                    phone: phone.to_string(),
                    country_code: country_code.to_string(),
                })
                .to_request();
                
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
        
        // 4th request should fail with rate limit
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/send-code")
            .set_json(&SendCodeRequest {
                phone: phone.to_string(),
                country_code: country_code.to_string(),
            })
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}