//! Integration tests for CORS and Security middleware

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse};
    use std::env;

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ok"
        }))
    }

    #[actix_web::test]
    async fn test_cors_middleware_development() {
        // Set development environment
        env::set_var("ENVIRONMENT", "development");

        // Import the middleware after setting environment
        let cors = api::middleware::cors::create_cors();
        let security = api::middleware::security::SecurityMiddleware::new();

        let app = test::init_service(
            App::new()
                .wrap(security)
                .wrap(cors)
                .route("/test", web::get().to(test_handler))
        ).await;

        // Test that CORS headers are added in development
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Origin", "http://localhost:3000"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Clean up
        env::remove_var("ENVIRONMENT");
    }

    #[actix_web::test]
    async fn test_security_middleware_development() {
        // Set development environment
        env::set_var("ENVIRONMENT", "development");

        // Import the middleware after setting environment
        let security = api::middleware::security::SecurityMiddleware::new();

        let app = test::init_service(
            App::new()
                .wrap(security)
                .route("/test", web::get().to(test_handler))
        ).await;

        // Test that HTTP requests are allowed in development
        let req = test::TestRequest::get()
            .uri("/test")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Clean up
        env::remove_var("ENVIRONMENT");
    }

    #[actix_web::test]
    async fn test_cors_allows_mobile_origins() {
        env::set_var("ENVIRONMENT", "production");
        env::set_var("ALLOWED_ORIGINS", "https://api.example.com");

        let cors = api::middleware::cors::create_cors();

        let app = test::init_service(
            App::new()
                .wrap(cors)
                .route("/test", web::get().to(test_handler))
        ).await;

        // Test iOS Capacitor origin
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Origin", "capacitor://localhost"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Clean up
        env::remove_var("ENVIRONMENT");
        env::remove_var("ALLOWED_ORIGINS");
    }
}