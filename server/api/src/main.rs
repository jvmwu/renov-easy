use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use log::info;
use std::env;

// Re-export the core crate to avoid naming conflicts  
extern crate core as renov_core;

// mod app; // Will be used when dependencies are wired up
mod config;
mod dto;
mod handlers;
mod i18n;
mod middleware;
mod routes;

// For now, we'll create a simple example showing how to wire up the endpoint
// In production, you would initialize real implementations of all the services

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    info!("Starting RenovEasy API Server");
    
    // Load configuration
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");
    
    let bind_address = format!("{}:{}", server_host, server_port);
    info!("Server will bind to: {}", bind_address);
    
    // Note: In a real implementation, you would:
    // 1. Initialize database connections
    // 2. Create repository implementations
    // 3. Create service implementations (SMS, Cache, etc.)
    // 4. Wire everything together with dependency injection
    //
    // Example structure:
    // ```
    // let db_pool = create_database_pool().await?;
    // let redis_client = create_redis_client().await?;
    // 
    // let user_repo = Arc::new(MySqlUserRepository::new(db_pool.clone()));
    // let token_repo = Arc::new(MySqlTokenRepository::new(db_pool.clone()));
    // 
    // let sms_service = Arc::new(TwilioSmsService::new(config));
    // let cache_service = Arc::new(RedisCacheService::new(redis_client.clone()));
    // let verification_service = Arc::new(VerificationService::new(sms_service, cache_service));
    // 
    // let rate_limiter = Arc::new(RedisRateLimiter::new(redis_client));
    // let token_service = Arc::new(TokenService::new(token_repo.clone(), config));
    // 
    // let auth_service = Arc::new(AuthService::new(
    //     user_repo,
    //     verification_service,
    //     rate_limiter,
    //     token_service,
    //     auth_config,
    // ));
    // ```
    
    // For now, we'll use the simplified version without real implementations
    // This allows the code to compile and demonstrates the structure
    
    HttpServer::new(move || {
        // Use the original simple app for now
        // When implementations are ready, switch to:
        // app::create_app(auth_service.clone())
        
        let cors = middleware::cors::create_cors();
        let security = middleware::security::SecurityMiddleware::new();
        
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(security)
            
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            
            // API v1 routes
            .service(
                web::scope("/api/v1")
                    // Auth routes - the structure is ready, implementations will be added
                    // when the services are wired up
                    .service(
                        web::scope("/auth")
                            // The send-code endpoint is ready to be wired when services are available
                            // .route("/send-code", web::post().to(routes::auth::send_code))
                    )
                    .route("/", web::get().to(api_info))
            )
            
            // Default 404 handler
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "not_found",
                    "message": "The requested resource was not found"
                }))
            }))
    })
    .bind(&bind_address)?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "renov-easy-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn api_info() -> HttpResponse {
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
                    "status": "Coming soon"
                },
                "select_type": {
                    "path": "/api/v1/auth/select-type",
                    "method": "POST",
                    "description": "Select user type (customer/worker)",
                    "status": "Coming soon"
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