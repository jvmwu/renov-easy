use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendCodeRequest {
    #[validate(length(min = 10, max = 15))]
    pub phone: String,
    #[validate(length(min = 1, max = 10))]
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyCodeRequest {
    #[validate(length(min = 10, max = 15))]
    pub phone: String,
    #[validate(length(min = 1, max = 10))]
    pub country_code: String,
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectTypeRequest {
    pub user_type: String, // "customer" or "worker"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user_type: Option<String>,
    pub requires_type_selection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendCodeResponse {
    pub message: String,
    pub resend_after: i64, // seconds until can resend
}