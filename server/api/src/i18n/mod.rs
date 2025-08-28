use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Language-specific error message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedMessage {
    pub message: String,
    pub code: String,
    pub http_status: u16,
}

/// Holds messages for all categories in a single language
#[derive(Debug, Clone, Default)]
pub struct LanguageMessages {
    pub auth: HashMap<String, LocalizedMessage>,
    pub token: HashMap<String, LocalizedMessage>,
    pub validation: HashMap<String, LocalizedMessage>,
    pub general: HashMap<String, LocalizedMessage>,
}

/// Global message storage for all supported languages
pub struct I18nMessages {
    pub en_us: LanguageMessages,
    pub zh_cn: LanguageMessages,
}

/// Lazily loaded i18n messages
pub static MESSAGES: Lazy<I18nMessages> = Lazy::new(|| {
    load_all_messages().expect("Failed to load i18n messages")
});

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    /// Parse language from Accept-Language header
    pub fn from_header(header: Option<&str>) -> Self {
        match header {
            Some(lang) if lang.starts_with("zh") => Language::Chinese,
            _ => Language::English,
        }
    }
    
    /// Get the locale code for the language
    pub fn locale_code(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Chinese => "zh-CN",
        }
    }
}

/// Load all message files for all languages
fn load_all_messages() -> Result<I18nMessages, Box<dyn std::error::Error>> {
    let en_us = load_language_messages("en-US")?;
    let zh_cn = load_language_messages("zh-CN")?;
    
    Ok(I18nMessages { en_us, zh_cn })
}

/// Load all message files for a specific language
fn load_language_messages(locale: &str) -> Result<LanguageMessages, Box<dyn std::error::Error>> {
    let mut messages = LanguageMessages::default();
    
    // Define the base path for locale files
    let base_path = format!("src/i18n/locales/{}", locale);
    let base = Path::new(&base_path);
    
    // Try runtime path first, then fallback to compile-time embedded files
    if base.exists() {
        // Load from filesystem at runtime
        messages.auth = load_category_from_file(&base.join("auth.toml"))?;
        messages.token = load_category_from_file(&base.join("token.toml"))?;
        messages.validation = load_category_from_file(&base.join("validation.toml"))?;
        messages.general = load_category_from_file(&base.join("general.toml"))?;
    } else {
        // Fallback to compile-time embedded files
        if locale == "en-US" {
            messages.auth = load_category_from_str(
                include_str!("locales/en-US/auth.toml")
            )?;
            messages.token = load_category_from_str(
                include_str!("locales/en-US/token.toml")
            )?;
            messages.validation = load_category_from_str(
                include_str!("locales/en-US/validation.toml")
            )?;
            messages.general = load_category_from_str(
                include_str!("locales/en-US/general.toml")
            )?;
        } else if locale == "zh-CN" {
            messages.auth = load_category_from_str(
                include_str!("locales/zh-CN/auth.toml")
            )?;
            messages.token = load_category_from_str(
                include_str!("locales/zh-CN/token.toml")
            )?;
            messages.validation = load_category_from_str(
                include_str!("locales/zh-CN/validation.toml")
            )?;
            messages.general = load_category_from_str(
                include_str!("locales/zh-CN/general.toml")
            )?;
        }
    }
    
    Ok(messages)
}

/// Load a category of messages from a file
fn load_category_from_file(path: &Path) -> Result<HashMap<String, LocalizedMessage>, Box<dyn std::error::Error>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let messages: HashMap<String, LocalizedMessage> = toml::from_str(&content)?;
        Ok(messages)
    } else {
        Ok(HashMap::new())
    }
}

/// Load a category of messages from a string (for embedded files)
fn load_category_from_str(content: &str) -> Result<HashMap<String, LocalizedMessage>, Box<dyn std::error::Error>> {
    let messages: HashMap<String, LocalizedMessage> = toml::from_str(content)?;
    Ok(messages)
}

/// Get an error message for a specific category, key, and language
pub fn get_error_message(category: &str, key: &str, lang: Language) -> Option<(String, String, u16)> {
    let messages = &*MESSAGES;
    
    let lang_messages = match lang {
        Language::English => &messages.en_us,
        Language::Chinese => &messages.zh_cn,
    };
    
    let category_map = match category {
        "auth" => &lang_messages.auth,
        "token" => &lang_messages.token,
        "validation" => &lang_messages.validation,
        "general" => &lang_messages.general,
        _ => return None,
    };
    
    category_map.get(key).map(|msg| {
        (msg.code.clone(), msg.message.clone(), msg.http_status)
    })
}

/// Format a message template with parameters
pub fn format_message(template: &str, params: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Get all messages for a specific language (useful for debugging/testing)
pub fn get_language_messages(lang: Language) -> &'static LanguageMessages {
    match lang {
        Language::English => &MESSAGES.en_us,
        Language::Chinese => &MESSAGES.zh_cn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_header() {
        assert_eq!(Language::from_header(Some("zh-CN")), Language::Chinese);
        assert_eq!(Language::from_header(Some("zh")), Language::Chinese);
        assert_eq!(Language::from_header(Some("en-US")), Language::English);
        assert_eq!(Language::from_header(Some("en")), Language::English);
        assert_eq!(Language::from_header(None), Language::English);
    }

    #[test]
    fn test_format_message() {
        let mut params = HashMap::new();
        params.insert("minutes", "5".to_string());
        
        let result = format_message("Please wait {minutes} minutes", &params);
        assert_eq!(result, "Please wait 5 minutes");
    }
    
    #[test]
    fn test_get_error_message() {
        // Test getting an auth message in English
        let msg = get_error_message("auth", "user_not_found", Language::English);
        assert!(msg.is_some());
        if let Some((code, message, status)) = msg {
            assert_eq!(code, "user_not_found");
            assert_eq!(status, 404);
            assert!(message.contains("User not found"));
        }
        
        // Test getting an auth message in Chinese
        let msg = get_error_message("auth", "user_not_found", Language::Chinese);
        assert!(msg.is_some());
        if let Some((code, message, status)) = msg {
            assert_eq!(code, "user_not_found");
            assert_eq!(status, 404);
            assert!(message.contains("用户不存在"));
        }
    }
}