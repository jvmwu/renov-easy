//! Common validation utilities

use serde::Serialize;
use std::collections::HashMap;

/// Validation error with field-level details
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

/// Collection of validation errors
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) {
        self.add(ValidationError::new(field, message, code));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn to_field_errors(&self) -> HashMap<String, Vec<String>> {
        let mut field_errors: HashMap<String, Vec<String>> = HashMap::new();
        for error in &self.errors {
            field_errors
                .entry(error.field.clone())
                .or_default()
                .push(error.message.clone());
        }
        field_errors
    }
}

/// Trait for types that can be validated
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Common validation functions
pub mod validators {
    /// Check if a string is not empty
    pub fn not_empty(value: &str) -> bool {
        !value.trim().is_empty()
    }

    /// Check if a string length is within bounds
    pub fn length_between(value: &str, min: usize, max: usize) -> bool {
        let len = value.len();
        len >= min && len <= max
    }

    /// Check if a string matches a pattern
    pub fn matches_pattern(value: &str, pattern: &regex::Regex) -> bool {
        pattern.is_match(value)
    }

    /// Check if an email address is valid (basic check)
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() >= 5
    }

    /// Check if a URL is valid (basic check)
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}
