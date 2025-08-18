//! Authentication route handlers
//!
//! This module contains all authentication-related endpoints including:
//! - Phone verification (sending and verifying codes)
//! - User type selection
//! - Token refresh
//! - Logout

pub mod send_code;
pub mod verify_code;
pub mod select_type;
pub mod refresh;
pub mod logout;

pub use send_code::{send_code, AppState};
pub use verify_code::verify_code;
pub use select_type::select_type;
pub use refresh::refresh_token;
pub use logout::logout;