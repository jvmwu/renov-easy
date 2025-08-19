//! Language and internationalization types

use serde::{Deserialize, Serialize};

/// Language preference for internationalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh")]
    Chinese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Language {
    /// Extract language from Accept-Language header
    pub fn from_accept_language(header: &str) -> Self {
        let header_lower = header.to_lowercase();
        if header_lower.contains("zh") {
            Language::Chinese
        } else {
            Language::English
        }
    }
    
    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }
    
    /// Get language name in English
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "Chinese",
        }
    }
    
    /// Get native language name
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
    
    /// Get locale code
    pub fn locale(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Chinese => "zh-CN",
        }
    }
    
    /// Check if language uses right-to-left script
    pub fn is_rtl(&self) -> bool {
        false  // Neither English nor Chinese is RTL
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl std::str::FromStr for Language {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "eng" | "english" => Ok(Language::English),
            "zh" | "chi" | "chinese" | "中文" => Ok(Language::Chinese),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

/// Language preference with fallback support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePreference {
    /// Primary language
    pub primary: Language,
    
    /// Fallback language if translation not available
    #[serde(default)]
    pub fallback: Option<Language>,
}

impl Default for LanguagePreference {
    fn default() -> Self {
        Self {
            primary: Language::English,
            fallback: None,
        }
    }
}

impl LanguagePreference {
    /// Create a new language preference
    pub fn new(primary: Language) -> Self {
        Self {
            primary,
            fallback: if primary != Language::English {
                Some(Language::English)
            } else {
                None
            },
        }
    }
    
    /// Get the effective language (primary or fallback)
    pub fn effective(&self) -> Language {
        self.primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_from_header() {
        assert_eq!(Language::from_accept_language("en-US,en;q=0.9"), Language::English);
        assert_eq!(Language::from_accept_language("zh-CN,zh;q=0.9"), Language::Chinese);
        assert_eq!(Language::from_accept_language("fr-FR"), Language::English);
        assert_eq!(Language::from_accept_language("ZH-TW"), Language::Chinese);
    }
    
    #[test]
    fn test_language_properties() {
        let en = Language::English;
        assert_eq!(en.code(), "en");
        assert_eq!(en.name(), "English");
        assert_eq!(en.locale(), "en-US");
        assert!(!en.is_rtl());
        
        let zh = Language::Chinese;
        assert_eq!(zh.code(), "zh");
        assert_eq!(zh.native_name(), "中文");
        assert_eq!(zh.locale(), "zh-CN");
    }
    
    #[test]
    fn test_language_from_str() {
        assert_eq!("en".parse::<Language>().unwrap(), Language::English);
        assert_eq!("zh".parse::<Language>().unwrap(), Language::Chinese);
        assert_eq!("english".parse::<Language>().unwrap(), Language::English);
        assert!("invalid".parse::<Language>().is_err());
    }
    
    #[test]
    fn test_language_preference() {
        let pref = LanguagePreference::new(Language::Chinese);
        assert_eq!(pref.primary, Language::Chinese);
        assert_eq!(pref.fallback, Some(Language::English));
        
        let pref = LanguagePreference::new(Language::English);
        assert_eq!(pref.primary, Language::English);
        assert_eq!(pref.fallback, None);
    }
}