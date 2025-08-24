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
