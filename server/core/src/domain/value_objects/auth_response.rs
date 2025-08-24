//! Authentication response value object for API responses.

use serde::{Deserialize, Serialize};

/// Authentication response containing tokens and user metadata
///
/// This response is returned after successful authentication and contains:
/// - JWT access and refresh tokens
/// - Token expiration times
/// - User type (if selected)
/// - Flag indicating if user type selection is required
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthResponse {
    /// JWT access token for API authentication
    pub access_token: String,
    
    /// JWT refresh token for obtaining new access tokens
    pub refresh_token: String,
    
    /// Access token expiration time in seconds
    pub expires_in: i64,
    
    /// User type ("customer" or "worker"), None if not yet selected
    pub user_type: Option<String>,
    
    /// Whether the user needs to select their type
    pub requires_type_selection: bool,
}

impl AuthResponse {
    /// Creates a new authentication response
    ///
    /// # Arguments
    ///
    /// * `access_token` - JWT access token
    /// * `refresh_token` - JWT refresh token
    /// * `expires_in` - Access token expiration in seconds
    /// * `user_type` - User type if selected
    /// * `requires_type_selection` - Whether type selection is needed
    ///
    /// # Returns
    ///
    /// A new `AuthResponse` instance
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user_type: Option<String>,
        requires_type_selection: bool,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            user_type,
            requires_type_selection,
        }
    }

    /// Creates an authentication response from a token pair and user information
    ///
    /// # Arguments
    ///
    /// * `token_pair` - The generated token pair
    /// * `user_type` - The user's type (if selected)
    ///
    /// # Returns
    ///
    /// A new `AuthResponse` instance
    pub fn from_token_pair(
        token_pair: crate::domain::entities::token::TokenPair,
        user_type: Option<crate::domain::entities::user::UserType>,
    ) -> Self {
        let user_type_str = user_type.map(|t| match t {
            crate::domain::entities::user::UserType::Customer => "customer".to_string(),
            crate::domain::entities::user::UserType::Worker => "worker".to_string(),
        });

        let requires_type_selection = user_type.is_none();

        Self {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            expires_in: token_pair.access_expires_in,
            user_type: user_type_str,
            requires_type_selection,
        }
    }
}