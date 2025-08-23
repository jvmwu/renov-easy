use actix_web::http::StatusCode;
pub use re_shared::types::response::ErrorResponse;

// Extension trait for ErrorResponse to add actix-web specific methods
pub trait ErrorResponseExt {
    fn to_response(&self, status: StatusCode) -> actix_web::HttpResponse;
}

impl ErrorResponseExt for ErrorResponse {
    fn to_response(&self, status: StatusCode) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(status).json(self)
    }
}