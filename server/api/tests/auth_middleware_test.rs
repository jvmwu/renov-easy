//! Integration tests for JWT authentication middleware

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse};
    use api::middleware::auth::{AuthContext, JwtAuth};
    
    #[actix_web::test]
    async fn test_middleware_requires_auth_header() {
        // Create test app with auth middleware
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth::new())
                .route("/protected", web::get().to(|| async { 
                    HttpResponse::Ok().body("Protected content")
                }))
        ).await;
        
        // Request without auth header should fail
        let req = test::TestRequest::get()
            .uri("/protected")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
    
    #[actix_web::test] 
    async fn test_middleware_rejects_invalid_token() {
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth::new())
                .route("/protected", web::get().to(|| async {
                    HttpResponse::Ok().body("Protected content")
                }))
        ).await;
        
        // Request with invalid token
        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer invalid-token"))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
    
    #[actix_web::test]
    async fn test_auth_context_extractor() {
        // Test handler that uses AuthContext
        async fn protected_handler(auth: AuthContext) -> HttpResponse {
            HttpResponse::Ok().json(serde_json::json!({
                "user_id": auth.user_id.to_string(),
                "user_type": auth.user_type,
                "is_verified": auth.is_verified
            }))
        }
        
        let app = test::init_service(
            App::new()
                .route("/protected", web::get().to(protected_handler))
        ).await;
        
        // Request without auth context should fail
        let req = test::TestRequest::get()
            .uri("/protected")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
}