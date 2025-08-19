//! Environment configuration module

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging/test environment
    Staging,
    /// Production environment
    Production,
}

impl Environment {
    /// Check if running in production
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
    
    /// Check if running in development
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
    
    /// Check if running in staging
    pub fn is_staging(&self) -> bool {
        matches!(self, Environment::Staging)
    }
    
    /// Get environment from ENV variable
    pub fn from_env() -> Self {
        env::var("ENVIRONMENT")
            .or_else(|_| env::var("ENV"))
            .or_else(|_| env::var("RUST_ENV"))
            .unwrap_or_else(|_| String::from("development"))
            .parse()
            .unwrap_or(Environment::Development)
    }
    
    /// Get the configuration file name for this environment
    pub fn config_file(&self) -> &str {
        match self {
            Environment::Development => "config.development.toml",
            Environment::Staging => "config.staging.toml",
            Environment::Production => "config.production.toml",
        }
    }
    
    /// Get the .env file name for this environment
    pub fn env_file(&self) -> &str {
        match self {
            Environment::Development => ".env.development",
            Environment::Staging => ".env.staging",
            Environment::Production => ".env.production",
        }
    }
    
    /// Check if debug mode should be enabled
    pub fn is_debug(&self) -> bool {
        match self {
            Environment::Development => true,
            Environment::Staging => true,
            Environment::Production => false,
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" | "test" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Invalid environment: {}", s)),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (json, pretty, compact)
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
    
    /// Enable file logging
    #[serde(default)]
    pub file: Option<FileLoggingConfig>,
    
    /// Enable colored output (terminal only)
    #[serde(default = "default_colored")]
    pub colored: bool,
    
    /// Include timestamp in logs
    #[serde(default = "default_timestamp")]
    pub timestamp: bool,
    
    /// Include source location in logs
    #[serde(default)]
    pub source_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: String::from("info"),
            format: default_log_format(),
            file: None,
            colored: default_colored(),
            timestamp: default_timestamp(),
            source_location: false,
        }
    }
}

impl LoggingConfig {
    /// Create logging config for environment
    pub fn for_environment(env: Environment) -> Self {
        match env {
            Environment::Development => Self {
                level: String::from("debug"),
                format: LogFormat::Pretty,
                file: None,
                colored: true,
                timestamp: true,
                source_location: true,
            },
            Environment::Staging => Self {
                level: String::from("info"),
                format: LogFormat::Json,
                file: Some(FileLoggingConfig::default()),
                colored: false,
                timestamp: true,
                source_location: false,
            },
            Environment::Production => Self {
                level: String::from("warn"),
                format: LogFormat::Json,
                file: Some(FileLoggingConfig::default()),
                colored: false,
                timestamp: true,
                source_location: false,
            },
        }
    }
}

/// Log format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

/// File logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileLoggingConfig {
    /// Log file path
    pub path: PathBuf,
    
    /// Max file size in MB before rotation
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    
    /// Number of rotated files to keep
    #[serde(default = "default_max_files")]
    pub max_files: u32,
    
    /// Compress rotated files
    #[serde(default)]
    pub compress: bool,
}

impl Default for FileLoggingConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("logs/app.log"),
            max_size: default_max_size(),
            max_files: default_max_files(),
            compress: false,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    #[serde(default)]
    pub metrics_enabled: bool,
    
    /// Metrics endpoint path
    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,
    
    /// Enable health check endpoint
    #[serde(default = "default_health_enabled")]
    pub health_enabled: bool,
    
    /// Health check endpoint path
    #[serde(default = "default_health_path")]
    pub health_path: String,
    
    /// Enable distributed tracing
    #[serde(default)]
    pub tracing_enabled: bool,
    
    /// Tracing sample rate (0.0 to 1.0)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: false,
            metrics_path: default_metrics_path(),
            health_enabled: default_health_enabled(),
            health_path: default_health_path(),
            tracing_enabled: false,
            sample_rate: default_sample_rate(),
        }
    }
}

fn default_log_format() -> LogFormat {
    LogFormat::Pretty
}

fn default_colored() -> bool {
    true
}

fn default_timestamp() -> bool {
    true
}

fn default_max_size() -> u64 {
    100  // 100 MB
}

fn default_max_files() -> u32 {
    10
}

fn default_metrics_path() -> String {
    String::from("/metrics")
}

fn default_health_enabled() -> bool {
    true
}

fn default_health_path() -> String {
    String::from("/health")
}

fn default_sample_rate() -> f64 {
    0.1  // 10% sampling
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_from_str() {
        assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
        assert!("invalid".parse::<Environment>().is_err());
    }
    
    #[test]
    fn test_environment_properties() {
        let dev = Environment::Development;
        assert!(dev.is_development());
        assert!(dev.is_debug());
        assert_eq!(dev.config_file(), "config.development.toml");
        
        let prod = Environment::Production;
        assert!(prod.is_production());
        assert!(!prod.is_debug());
        assert_eq!(prod.env_file(), ".env.production");
    }
    
    #[test]
    fn test_logging_config_for_environment() {
        let dev_log = LoggingConfig::for_environment(Environment::Development);
        assert_eq!(dev_log.level, "debug");
        assert!(dev_log.colored);
        assert!(dev_log.source_location);
        
        let prod_log = LoggingConfig::for_environment(Environment::Production);
        assert_eq!(prod_log.level, "warn");
        assert!(!prod_log.colored);
        assert!(!prod_log.source_location);
    }
    
    #[test]
    fn test_monitoring_config() {
        let config = MonitoringConfig::default();
        assert!(config.health_enabled);
        assert_eq!(config.health_path, "/health");
        assert_eq!(config.sample_rate, 0.1);
    }
}