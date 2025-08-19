//! Type definitions module with domain-specific sub-modules
//!
//! This module organizes types into logical categories:
//! - `common` - Common types like Id, Status, Priority, Coordinates
//! - `language` - Internationalization and language types
//! - `pagination` - Pagination for list endpoints
//! - `response` - API response wrappers and health checks

pub mod common;
pub mod language;
pub mod pagination;
pub mod response;

// Re-export commonly used types at module level
pub use common::{
    Coordinate, DateRange, FileInfo, Id, KeyValue, Priority, SortOrder, SortParams, Status,
    Timestamp, Uuid,
};
pub use language::{Language, LanguagePreference};
pub use pagination::{
    CursorPaginatedResponse, CursorPagination, PaginatedResponse, Pagination,
    PaginationDirection,
};
pub use response::{
    ApiResponse, BatchResponse, BatchSummary, DetailedResponse, ErrorDetail, HealthResponse,
    HealthStatus, ResponseMeta, ResponseStatus, ServiceHealth,
};