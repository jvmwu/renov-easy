//! JWT authentication middleware for protecting API endpoints.
//!
//! This middleware extracts JWT tokens from the Authorization header,
//! verifies their validity, and injects user context into requests.
//! 
//! The middleware can work in two modes:
//! 1. Standalone mode: Uses jsonwebtoken directly for simple JWT verification
//! 2. Integrated mode: Uses the core TokenService when available in app data

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::AUTHORIZATION,
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use re_core::{
    domain::entities::token::Claims,
    errors::{DomainError, TokenError},
    services::token::TokenService,
    repositories::TokenRepository,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};
use uuid::Uuid;

/// User authentication context injected into requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// User ID extracted from JWT claims
    pub user_id: Uuid,
    /// User type (customer or worker) if set
    pub user_type: Option<String>,
    /// Whether the user's account is verified
    pub is_verified: bool,
    /// JWT ID for tracking
    pub jti: String,
}

impl AuthContext {
    /// Creates a new authentication context from JWT claims
    pub fn from_claims(claims: Claims) -> Result<Self, DomainError> {
        let user_id = claims.user_id()
            .map_err(|_| DomainError::Token(TokenError::InvalidTokenFormat))?;
        Ok(Self {
            user_id,
            user_type: claims.user_type,
            is_verified: claims.is_verified,
            jti: claims.jti,
        })
    }
}

/// JWT authentication middleware factory
pub struct JwtAuth {
    /// Optional JWT secret for standalone mode
    jwt_secret: Option<String>,
}

impl JwtAuth {
    /// Creates a new JWT authentication middleware
    pub fn new() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").ok(),
        }
    }
    
    /// Creates a new JWT authentication middleware with a specific secret
    pub fn with_secret(secret: String) -> Self {
        Self {
            jwt_secret: Some(secret),
        }
    }
}

impl Default for JwtAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

/// JWT authentication middleware service
pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    jwt_secret: Option<String>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = match extract_bearer_token(&req) {
                Some(token) => token,
                None => {
                    return Err(ErrorUnauthorized("Missing or invalid Authorization header"));
                }
            };

            // Try to get TokenService from app data (if available)
            // This allows for integration with the core layer's TokenService
            let auth_context = if let Some(token_service) = req.app_data::<web::Data<Arc<dyn TokenServiceWrapper>>>() {
                // Use the TokenService from core layer
                match token_service.verify_access_token(&token) {
                    Ok(claims) => {
                        match AuthContext::from_claims(claims) {
                            Ok(context) => context,
                            Err(e) => return Err(ErrorUnauthorized(format!("Invalid token: {}", e))),
                        }
                    }
                    Err(e) => return Err(ErrorUnauthorized(format!("Token verification failed: {}", e))),
                }
            } else if let Some(secret) = jwt_secret {
                // Fallback to standalone verification
                match verify_token_standalone(&token, &secret) {
                    Ok(context) => context,
                    Err(e) => return Err(ErrorUnauthorized(format!("Token verification failed: {}", e))),
                }
            } else {
                return Err(ErrorUnauthorized("JWT verification not configured"));
            };

            // Inject auth context into request extensions
            req.extensions_mut().insert(auth_context);

            // Continue with the request
            service.call(req).await
        })
    }
}

/// Extracts Bearer token from Authorization header
fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

/// Standalone token verification (for when TokenService is not available)
fn verify_token_standalone(token: &str, secret: &str) -> Result<AuthContext, String> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token decode error: {}", e))?;
    
    AuthContext::from_claims(token_data.claims)
        .map_err(|e| format!("Invalid claims: {}", e))
}

/// Trait for wrapping TokenService to allow dynamic dispatch
pub trait TokenServiceWrapper: Send + Sync {
    fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError>;
}

/// Implementation of TokenServiceWrapper for any TokenService
impl<R: TokenRepository> TokenServiceWrapper for TokenService<R> {
    fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError> {
        self.verify_access_token(token)
    }
}

/// Extractor for required authentication
impl FromRequest for AuthContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = req
            .extensions()
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| ErrorUnauthorized("Authentication required"));
        
        ready(result)
    }
}

/// Extractor for optional authentication
pub struct OptionalAuth(pub Option<AuthContext>);

impl FromRequest for OptionalAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let auth = req.extensions().get::<AuthContext>().cloned();
        ready(Ok(OptionalAuth(auth)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        use actix_web::test;
        
        let req = test::TestRequest::default()
            .insert_header((AUTHORIZATION, "Bearer test_token_123"))
            .to_srv_request();
        
        assert_eq!(extract_bearer_token(&req), Some("test_token_123".to_string()));
        
        let req_no_bearer = test::TestRequest::default()
            .insert_header((AUTHORIZATION, "test_token_123"))
            .to_srv_request();
        
        assert_eq!(extract_bearer_token(&req_no_bearer), None);
        
        let req_no_header = test::TestRequest::default().to_srv_request();
        assert_eq!(extract_bearer_token(&req_no_header), None);
    }
}