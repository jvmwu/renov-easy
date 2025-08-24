//! Pagination related types for list endpoints

use serde::{Deserialize, Serialize};

/// Pagination parameters for list endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl Pagination {
    /// Create a new pagination with custom values
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, MAX_PER_PAGE),
        }
    }

    /// Calculate the offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Get the limit for database queries
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Calculate offset as i64 for SQL queries
    pub fn offset_i64(&self) -> i64 {
        self.offset() as i64
    }

    /// Calculate limit as i64 for SQL queries
    pub fn limit_i64(&self) -> i64 {
        self.limit() as i64
    }

    /// Check if this is the first page
    pub fn is_first_page(&self) -> bool {
        self.page == 1
    }

    /// Calculate the page number for a given offset
    pub fn from_offset(offset: u32, per_page: u32) -> Self {
        let page = (offset / per_page) + 1;
        Self::new(page, per_page)
    }

    /// Validate and sanitize pagination parameters
    pub fn validate(mut self) -> Self {
        self.page = self.page.max(1);
        self.per_page = self.per_page.clamp(MIN_PER_PAGE, MAX_PER_PAGE);
        self
    }
}

/// Paginated response wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The actual data items
    pub data: Vec<T>,

    /// Current page number
    pub page: u32,

    /// Items per page
    pub per_page: u32,

    /// Total number of items
    pub total: u64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there's a next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_next: Option<bool>,

    /// Whether there's a previous page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_prev: Option<bool>,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, pagination: Pagination, total: u64) -> Self {
        let total_pages = Self::calculate_total_pages(total, pagination.per_page);
        let has_next = pagination.page < total_pages;
        let has_prev = pagination.page > 1;

        Self {
            data,
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages,
            has_next: Some(has_next),
            has_prev: Some(has_prev),
        }
    }

    /// Create an empty paginated response
    pub fn empty(pagination: Pagination) -> Self {
        Self {
            data: Vec::new(),
            page: pagination.page,
            per_page: pagination.per_page,
            total: 0,
            total_pages: 0,
            has_next: Some(false),
            has_prev: Some(false),
        }
    }

    /// Calculate total pages from total items and items per page
    fn calculate_total_pages(total: u64, per_page: u32) -> u32 {
        if total == 0 {
            return 0;
        }
        ((total as f64) / (per_page as f64)).ceil() as u32
    }

    /// Transform the data items using a function
    pub fn map<U, F>(self, f: F) -> PaginatedResponse<U>
    where
        F: FnMut(T) -> U,
    {
        PaginatedResponse {
            data: self.data.into_iter().map(f).collect(),
            page: self.page,
            per_page: self.per_page,
            total: self.total,
            total_pages: self.total_pages,
            has_next: self.has_next,
            has_prev: self.has_prev,
        }
    }

    /// Check if the response is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the number of items in this page
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

/// Cursor-based pagination for large datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination {
    /// Cursor pointing to the start of the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Number of items to fetch
    #[serde(default = "default_per_page")]
    pub limit: u32,

    /// Direction of pagination
    #[serde(default)]
    pub direction: PaginationDirection,
}

/// Direction for cursor-based pagination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaginationDirection {
    Forward,
    Backward,
}

impl Default for PaginationDirection {
    fn default() -> Self {
        PaginationDirection::Forward
    }
}

/// Response for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPaginatedResponse<T> {
    /// The data items
    pub data: Vec<T>,

    /// Cursor for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Cursor for the previous page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,

    /// Whether there are more items
    pub has_more: bool,
}

// Constants
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PER_PAGE: u32 = 20;
const MIN_PER_PAGE: u32 = 1;
const MAX_PER_PAGE: u32 = 100;

fn default_page() -> u32 {
    DEFAULT_PAGE
}

fn default_per_page() -> u32 {
    DEFAULT_PER_PAGE
}
