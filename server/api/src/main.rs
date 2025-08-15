use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use log::info;
use std::env;

mod config;
mod dto;
mod handlers;
mod middleware;
mod routes;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "renov-easy-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

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
    
    // Create HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            // Add middleware
            .wrap(cors)
            .wrap(Logger::default())
            
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            
            // API v1 routes
            .service(
                web::scope("/api/v1")
                    // Auth routes will be added here
                    .route("/", web::get().to(|| async { 
                        HttpResponse::Ok().json(serde_json::json!({
                            "message": "RenovEasy API v1",
                            "endpoints": {
                                "health": "/health",
                                "auth": {
                                    "send_code": "/api/v1/auth/send-code",
                                    "verify_code": "/api/v1/auth/verify-code",
                                    "select_type": "/api/v1/auth/select-type",
                                    "refresh": "/api/v1/auth/refresh",
                                    "logout": "/api/v1/auth/logout"
                                }
                            }
                        }))
                    }))
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