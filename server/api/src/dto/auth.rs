use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendCodeRequest {
    /// Phone number without country code, or full E.164 format with country code
    /// Examples: "13812345678" (China), "412345678" (Australia), or "+8613812345678"
    #[validate(length(min = 7, max = 15))]
    pub phone: String,
    
    /// Country code with or without '+' prefix
    /// Examples: "+86", "86" (China), "+61", "61" (Australia)
    #[validate(length(min = 1, max = 5))]
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyCodeRequest {
    /// Phone number without country code, or full E.164 format with country code
    #[validate(length(min = 7, max = 15))]
    pub phone: String,
    
    /// Country code with or without '+' prefix
    #[validate(length(min = 1, max = 5))]
    pub country_code: String,
    
    /// 6-digit verification code
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}