use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

use crate::handlers::error_standard::{StandardApiError, extract_language};

/// Middleware for standardizing error responses
pub struct ErrorHandlerMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ErrorHandlerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorHandlerMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorHandlerMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct ErrorHandlerMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        
        // Generate or extract request ID
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Store request ID in extensions for later use
        req.extensions_mut().insert(request_id.clone());
        
        // Extract language preference
        let language = extract_language(req.request());
        req.extensions_mut().insert(language);
        
        Box::pin(async move {
            let res = service.call(req).await;
            
            // Handle errors and convert them to standardized format
            match res {
                Ok(response) => Ok(response),
                Err(err) => {
                    // The error is already converted to standardized format
                    // by StandardApiError implementation
                    Err(err)
                }
            }
        })
    }
}

/// Extension trait to add error handling capabilities to ServiceRequest
pub trait ErrorHandlingExt {
    fn get_request_id(&self) -> Option<String>;
    fn get_language(&self) -> crate::i18n::Language;
}

impl ErrorHandlingExt for ServiceRequest {
    fn get_request_id(&self) -> Option<String> {
        self.extensions()
            .get::<String>()
            .cloned()
    }
    
    fn get_language(&self) -> crate::i18n::Language {
        self.extensions()
            .get::<crate::i18n::Language>()
            .copied()
            .unwrap_or(crate::i18n::Language::English)
    }
}

impl ErrorHandlingExt for actix_web::HttpRequest {
    fn get_request_id(&self) -> Option<String> {
        self.extensions()
            .get::<String>()
            .cloned()
    }
    
    fn get_language(&self) -> crate::i18n::Language {
        self.extensions()
            .get::<crate::i18n::Language>()
            .copied()
            .unwrap_or(crate::i18n::Language::English)
    }
}