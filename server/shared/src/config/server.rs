//! Server configuration module

use serde::{Deserialize, Serialize};

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Worker threads (0 = number of CPU cores)
    #[serde(default)]
    pub workers: usize,
    
    /// Keep-alive timeout in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,
    
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
    
    /// Maximum payload size in bytes
    #[serde(default = "default_max_payload_size")]
    pub max_payload_size: usize,
    
    /// Enable HTTP/2
    #[serde(default)]
    pub enable_http2: bool,
    
    /// Enable compression
    #[serde(default = "default_enable_compression")]
    pub enable_compression: bool,
    
    /// TLS configuration
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: String::from("0.0.0.0"),
            port: 8080,
            workers: 0,  // Use all CPU cores
            keep_alive: default_keep_alive(),
            request_timeout: default_request_timeout(),
            max_payload_size: default_max_payload_size(),
            enable_http2: false,
            enable_compression: default_enable_compression(),
            tls: None,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }
    
    /// Enable TLS/HTTPS
    pub fn with_tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.tls = Some(TlsConfig {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            ..Default::default()
        });
        self
    }
    
    /// Get the bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    
    /// Check if running on default HTTP port
    pub fn is_default_port(&self) -> bool {
        self.port == 80 || self.port == 8080
    }
    
    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls.is_some()
    }
}

/// TLS/SSL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: String,
    
    /// Path to private key file
    pub key_path: String,
    
    /// Path to CA certificate (for client verification)
    #[serde(default)]
    pub ca_path: Option<String>,
    
    /// Enable client certificate verification
    #[serde(default)]
    pub verify_client: bool,
    
    /// Minimum TLS version (e.g., "1.2", "1.3")
    #[serde(default = "default_min_tls_version")]
    pub min_version: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_path: String::from("certs/server.crt"),
            key_path: String::from("certs/server.key"),
            ca_path: None,
            verify_client: false,
            min_version: default_min_tls_version(),
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    /// Enable CORS
    #[serde(default = "default_cors_enabled")]
    pub enabled: bool,
    
    /// Allowed origins
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    
    /// Allowed methods
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,
    
    /// Allowed headers
    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,
    
    /// Exposed headers
    #[serde(default)]
    pub exposed_headers: Vec<String>,
    
    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,
    
    /// Max age for preflight cache in seconds
    #[serde(default = "default_max_age")]
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: default_cors_enabled(),
            allowed_origins: vec![],
            allowed_methods: default_allowed_methods(),
            allowed_headers: default_allowed_headers(),
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: default_max_age(),
        }
    }
}

impl CorsConfig {
    /// Create a permissive CORS configuration for development
    pub fn development() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["*".to_string()],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            allow_credentials: true,
            max_age: 3600,
        }
    }
}

fn default_keep_alive() -> u64 {
    75  // 75 seconds
}

fn default_request_timeout() -> u64 {
    30  // 30 seconds
}

fn default_max_payload_size() -> usize {
    10 * 1024 * 1024  // 10 MB
}

fn default_enable_compression() -> bool {
    true
}

fn default_min_tls_version() -> String {
    String::from("1.2")
}

fn default_cors_enabled() -> bool {
    true
}

fn default_allowed_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_allowed_headers() -> Vec<String> {
    vec![
        "Content-Type".to_string(),
        "Authorization".to_string(),
        "Accept".to_string(),
        "Accept-Language".to_string(),
    ]
}

fn default_max_age() -> u64 {
    86400  // 24 hours
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 0);
        assert!(config.enable_compression);
        assert!(!config.is_tls_enabled());
    }
    
    #[test]
    fn test_server_config_with_tls() {
        let config = ServerConfig::new("localhost", 443)
            .with_tls("cert.pem", "key.pem");
        
        assert!(config.is_tls_enabled());
        assert_eq!(config.bind_address(), "localhost:443");
    }
    
    #[test]
    fn test_cors_config_development() {
        let config = CorsConfig::development();
        assert!(config.enabled);
        assert_eq!(config.allowed_origins, vec!["*"]);
        assert!(config.allow_credentials);
    }
    
    #[test]
    fn test_is_default_port() {
        assert!(ServerConfig::new("localhost", 80).is_default_port());
        assert!(ServerConfig::new("localhost", 8080).is_default_port());
        assert!(!ServerConfig::new("localhost", 3000).is_default_port());
    }
}