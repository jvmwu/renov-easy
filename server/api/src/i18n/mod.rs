use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub en: String,
    pub zh: String,
    pub code: String,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessages {
    pub auth: HashMap<String, ErrorMessage>,
    pub validation: HashMap<String, ErrorMessage>,
    pub token: HashMap<String, ErrorMessage>,
    pub general: HashMap<String, ErrorMessage>,
}

pub static ERROR_MESSAGES: Lazy<ErrorMessages> = Lazy::new(|| {
    load_error_messages().expect("Failed to load error messages")
});

fn load_error_messages() -> Result<ErrorMessages, Box<dyn std::error::Error>> {
    // Try to load from file first, fallback to embedded defaults
    let config_path = Path::new("i18n/error_messages.toml");
    
    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let messages: ErrorMessages = toml::from_str(&content)?;
        Ok(messages)
    } else {
        // Fallback to embedded configuration
        load_default_messages()
    }
}

fn load_default_messages() -> Result<ErrorMessages, Box<dyn std::error::Error>> {
    // Include the default configuration at compile time
    let default_config = include_str!("../../i18n/error_messages.toml");
    let messages: ErrorMessages = toml::from_str(default_config)?;
    Ok(messages)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn from_header(header: Option<&str>) -> Self {
        match header {
            Some(lang) if lang.starts_with("zh") => Language::Chinese,
            _ => Language::English,
        }
    }
}

pub fn get_error_message(category: &str, key: &str, lang: Language) -> Option<(String, String, u16)> {
    let messages = &*ERROR_MESSAGES;
    
    let category_map = match category {
        "auth" => &messages.auth,
        "validation" => &messages.validation,
        "token" => &messages.token,
        "general" => &messages.general,
        _ => return None,
    };
    
    category_map.get(key).map(|msg| {
        let text = match lang {
            Language::English => msg.en.clone(),
            Language::Chinese => msg.zh.clone(),
        };
        (msg.code.clone(), text, msg.http_status)
    })
}

pub fn format_message(template: &str, params: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_error_message() {
        let result = get_error_message("auth", "user_not_found", Language::English);
        assert!(result.is_some());
        
        if let Some((code, message, status)) = result {
            assert_eq!(code, "user_not_found");
            assert_eq!(message, "User not found");
            assert_eq!(status, 404);
        }
    }

    #[test]
    fn test_format_message() {
        let mut params = HashMap::new();
        params.insert("field", "email".to_string());
        
        let template = "Required field: {field}";
        let result = format_message(template, &params);
        assert_eq!(result, "Required field: email");
    }

    #[test]
    fn test_language_from_header() {
        assert_eq!(Language::from_header(Some("zh-CN")), Language::Chinese);
        assert_eq!(Language::from_header(Some("en-US")), Language::English);
        assert_eq!(Language::from_header(None), Language::English);
    }
}