use crate::dto::ErrorResponse;
use actix_web::{http::StatusCode, HttpResponse};

pub fn handle_error(error: anyhow::Error) -> HttpResponse {
    // Log the error
    log::error!("API Error: {:?}", error);

    // Create error response
    let error_response = ErrorResponse::new(
        "internal_error".to_string(),
        "An internal error occurred".to_string(),
    );

    error_response.to_response(StatusCode::INTERNAL_SERVER_ERROR)
}