//! API response types and wrappers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,

    /// Response data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error message (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Create an error response
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Add request ID for tracing
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the response is an error
    pub fn is_error(&self) -> bool {
        !self.success
    }

    /// Extract the data, consuming the response
    pub fn into_data(self) -> Option<T> {
        self.data
    }

    /// Map the data to a different type
    pub fn map<U, F>(self, f: F) -> ApiResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        ApiResponse {
            success: self.success,
            data: self.data.map(f),
            error: self.error,
            timestamp: self.timestamp,
            request_id: self.request_id,
        }
    }
}

/// Detailed API response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedResponse<T> {
    /// Response status
    pub status: ResponseStatus,

    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Response metadata
    pub meta: ResponseMeta,

    /// Error details if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

/// Response status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
    Partial,  // Partial success (some operations failed)
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// API version
    pub version: String,

    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            version: String::from("v1"),
            request_id: None,
            response_time_ms: None,
            extra: HashMap::new(),
        }
    }
}

/// Detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error code for programmatic handling
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Field-specific errors (for validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, Vec<String>>>,

    /// Stack trace (development only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,

    /// Additional error context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T> {
    /// Successfully processed items
    pub successful: Vec<BatchItem<T>>,

    /// Failed items with error details
    pub failed: Vec<BatchError>,

    /// Summary statistics
    pub summary: BatchSummary,
}

/// Individual batch item result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem<T> {
    /// Item identifier
    pub id: String,

    /// Processing result
    pub result: T,
}

/// Batch processing error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Item identifier
    pub id: String,

    /// Error details
    pub error: ErrorDetail,
}

/// Batch operation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total items processed
    pub total: usize,

    /// Number of successful items
    pub successful: usize,

    /// Number of failed items
    pub failed: usize,

    /// Processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,

    /// Individual service health checks
    pub services: HashMap<String, ServiceHealth>,

    /// Server timestamp
    pub timestamp: DateTime<Utc>,

    /// Server version
    pub version: String,
}

/// Standardized error response structure for domain errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for client-side handling
    pub error: String,

    /// Human-readable error message
    pub message: String,

    /// Optional additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,

    /// Timestamp of when the error occurred
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: String, message: String) -> Self {
        Self {
            error,
            message,
            details: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response with additional details
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual service health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service status
    pub status: HealthStatus,

    /// Health check message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}
