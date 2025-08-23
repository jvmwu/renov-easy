//! Security middleware for enforcing HTTPS and other security policies.
//!
//! This middleware ensures that all requests meet security requirements including:
//! - HTTPS enforcement in production environments
//! - Security headers (HSTS, CSP, etc.)
//! - Request origin validation
//! - XSS and clickjacking protection

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorBadRequest, ErrorForbidden},
    http::header::{self, HeaderValue},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    env,
    future::{ready, Ready},
    rc::Rc,
    task::{Context, Poll},
};

/// Security middleware factory for enforcing HTTPS and security policies
pub struct SecurityMiddleware {
    /// Whether to enforce HTTPS (disabled in development)
    enforce_https: bool,
    /// Whether to add security headers
    add_security_headers: bool,
    /// List of trusted proxies for X-Forwarded-* headers
    trusted_proxies: Vec<String>,
}

impl SecurityMiddleware {
    /// Creates a new security middleware with environment-based configuration
    pub fn new() -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let enforce_https = environment == "production";
        let add_security_headers = environment == "production";

        let trusted_proxies = env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        log::info!(
            "Security middleware configured: enforce_https={}, add_headers={}, trusted_proxies={:?}",
            enforce_https, add_security_headers, trusted_proxies
        );

        Self {
            enforce_https,
            add_security_headers,
            trusted_proxies,
        }
    }

    /// Creates a security middleware for development (no HTTPS enforcement)
    pub fn development() -> Self {
        Self {
            enforce_https: false,
            add_security_headers: false,
            trusted_proxies: vec!["127.0.0.1".to_string(), "::1".to_string()],
        }
    }

    /// Creates a security middleware for production (full security)
    pub fn production() -> Self {
        Self {
            enforce_https: true,
            add_security_headers: true,
            trusted_proxies: vec![],
        }
    }

    /// Adds a trusted proxy to the whitelist
    pub fn with_trusted_proxy(mut self, proxy: String) -> Self {
        self.trusted_proxies.push(proxy);
        self
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityMiddlewareService {
            service: Rc::new(service),
            enforce_https: self.enforce_https,
            add_security_headers: self.add_security_headers,
            trusted_proxies: self.trusted_proxies.clone(),
        }))
    }
}

/// Security middleware service implementation
pub struct SecurityMiddlewareService<S> {
    service: Rc<S>,
    enforce_https: bool,
    add_security_headers: bool,
    trusted_proxies: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for SecurityMiddlewareService<S>
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
        let enforce_https = self.enforce_https;
        let add_security_headers = self.add_security_headers;
        let trusted_proxies = self.trusted_proxies.clone();

        Box::pin(async move {
            // Check HTTPS enforcement
            if enforce_https && !is_secure_request(&req, &trusted_proxies) {
                log::warn!(
                    "Insecure request blocked: {} {}",
                    req.method(),
                    req.path()
                );
                return Err(ErrorForbidden("HTTPS required"));
            }

            // Validate request origin if present
            if let Some(origin) = req.headers().get(header::ORIGIN) {
                if !is_valid_origin(&req, origin) {
                    log::warn!(
                        "Invalid origin blocked: {:?} for {} {}",
                        origin,
                        req.method(),
                        req.path()
                    );
                    return Err(ErrorBadRequest("Invalid request origin"));
                }
            }

            // Process the request
            let mut response = service.call(req).await?;

            // Add security headers to response
            if add_security_headers {
                add_security_response_headers(&mut response);
            }

            Ok(response)
        })
    }
}

/// Checks if the request is secure (HTTPS or from trusted source)
fn is_secure_request(req: &ServiceRequest, trusted_proxies: &[String]) -> bool {
    // Check if connection is already secure
    let conn_info = req.connection_info();
    if conn_info.scheme() == "https" {
        return true;
    }

    // Check X-Forwarded-Proto header from trusted proxies
    if let Some(forwarded_proto) = req.headers().get("x-forwarded-proto") {
        if let Ok(proto) = forwarded_proto.to_str() {
            // Only trust the header if request is from a trusted proxy
            let peer_addr = conn_info.peer_addr().unwrap_or("");
            if is_trusted_proxy(peer_addr, trusted_proxies) && proto == "https" {
                return true;
            }
        }
    }

    // Allow localhost in development (but this should be controlled by enforce_https flag)
    let host = conn_info.host();
    if host == "localhost" || host.starts_with("127.0.0.1") || host.starts_with("[::1]") {
        return true;
    }

    false
}

/// Checks if the given IP address is in the trusted proxy list
fn is_trusted_proxy(peer_addr: &str, trusted_proxies: &[String]) -> bool {
    // Extract IP from peer address (might be in format "ip:port")
    let ip = peer_addr.split(':').next().unwrap_or(peer_addr);

    trusted_proxies.iter().any(|trusted| {
        trusted == ip || trusted == peer_addr
    })
}

/// Validates the request origin header
fn is_valid_origin(_req: &ServiceRequest, origin: &HeaderValue) -> bool {
    // In this implementation, we delegate origin validation to CORS middleware
    // This function can be extended to add additional origin validation logic
    // For now, we just ensure the origin header is well-formed

    if let Ok(origin_str) = origin.to_str() {
        // Basic validation: origin should be a valid URL
        if origin_str.starts_with("http://") ||
           origin_str.starts_with("https://") ||
           origin_str.starts_with("capacitor://") ||
           origin_str.starts_with("ionic://") ||
           origin_str.starts_with("arkui://") ||
           origin_str.starts_with("harmony://") {
            return true;
        }
    }

    false
}

/// Adds security headers to the response
fn add_security_response_headers<B>(response: &mut ServiceResponse<B>) {
    let headers = response.headers_mut();

    // Strict Transport Security (HSTS)
    // Enforce HTTPS for 1 year, including subdomains
    headers.insert(
        header::HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // X-Content-Type-Options
    // Prevent MIME type sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options
    // Prevent clickjacking attacks
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // X-XSS-Protection
    // Enable XSS filtering (for older browsers)
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    // Control referrer information
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content-Security-Policy
    // Basic CSP for API responses (can be customized based on needs)
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none';"),
    );

    // Permissions-Policy (formerly Feature-Policy)
    // Disable unnecessary browser features
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
    );
}

/// Creates a security middleware that only adds security headers without HTTPS enforcement.
///
/// Useful for specific endpoints that need security headers but not HTTPS enforcement.
pub fn security_headers_only() -> SecurityMiddleware {
    SecurityMiddleware {
        enforce_https: false,
        add_security_headers: true,
        trusted_proxies: vec![],
    }
}
