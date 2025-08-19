//! Common type definitions and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// UUID v4 type alias for consistency
pub type Uuid = uuid::Uuid;

/// Timestamp type alias
pub type Timestamp = DateTime<Utc>;

/// Generic ID type that can be either numeric or string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Numeric(u64),
    String(String),
    Uuid(Uuid),
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Numeric(n) => write!(f, "{}", n),
            Id::String(s) => write!(f, "{}", s),
            Id::Uuid(u) => write!(f, "{}", u),
        }
    }
}

impl From<u64> for Id {
    fn from(value: u64) -> Self {
        Id::Numeric(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::String(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id::String(value.to_string())
    }
}

impl From<Uuid> for Id {
    fn from(value: Uuid) -> Self {
        Id::Uuid(value)
    }
}

/// Sort order for list queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// Generic sorting parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortParams {
    /// Field to sort by
    pub field: String,
    
    /// Sort order
    #[serde(default)]
    pub order: SortOrder,
}

impl SortParams {
    pub fn new(field: impl Into<String>, order: SortOrder) -> Self {
        Self {
            field: field.into(),
            order,
        }
    }
    
    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortOrder::Asc)
    }
    
    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortOrder::Desc)
    }
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<DateTime<Utc>>,
    
    /// End date (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<DateTime<Utc>>,
}

impl DateRange {
    /// Create a date range
    pub fn new(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }
    
    /// Create a range from a specific date onwards
    pub fn from_date(from: DateTime<Utc>) -> Self {
        Self {
            from: Some(from),
            to: None,
        }
    }
    
    /// Create a range up to a specific date
    pub fn until_date(to: DateTime<Utc>) -> Self {
        Self {
            from: None,
            to: Some(to),
        }
    }
    
    /// Create a range for today
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = now.date_naive().and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        Self {
            from: Some(start),
            to: Some(end),
        }
    }
    
    /// Check if a date is within the range
    pub fn contains(&self, date: &DateTime<Utc>) -> bool {
        let after_start = self.from.map_or(true, |from| date >= &from);
        let before_end = self.to.map_or(true, |to| date <= &to);
        after_start && before_end
    }
}

/// Generic key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> KeyValue<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

/// Coordinate for location-based features
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinate {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }
    
    /// Calculate distance to another coordinate (in meters)
    /// Using Haversine formula
    pub fn distance_to(&self, other: &Coordinate) -> f64 {
        const EARTH_RADIUS_M: f64 = 6_371_000.0;
        
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        EARTH_RADIUS_M * c
    }
}

/// File upload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Original filename
    pub filename: String,
    
    /// MIME type
    pub content_type: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// Storage path or URL
    pub path: String,
    
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    
    /// File checksum (MD5/SHA256)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Status for various entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Suspended,
    Deleted,
}

impl Default for Status {
    fn default() -> Self {
        Status::Active
    }
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Urgent = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_id_conversions() {
        let numeric_id = Id::from(123u64);
        assert_eq!(numeric_id.to_string(), "123");
        
        let string_id = Id::from("abc");
        assert_eq!(string_id.to_string(), "abc");
        
        let uuid = Uuid::new_v4();
        let uuid_id = Id::from(uuid);
        assert_eq!(uuid_id.to_string(), uuid.to_string());
    }
    
    #[test]
    fn test_sort_order() {
        assert_eq!(SortOrder::default(), SortOrder::Asc);
        assert_eq!(SortOrder::Asc.to_string(), "ASC");
        assert_eq!(SortOrder::Desc.to_string(), "DESC");
    }
    
    #[test]
    fn test_date_range() {
        let now = Utc::now();
        let yesterday = now - chrono::Duration::days(1);
        let tomorrow = now + chrono::Duration::days(1);
        
        let range = DateRange::new(Some(yesterday), Some(tomorrow));
        assert!(range.contains(&now));
        
        let future = now + chrono::Duration::days(2);
        assert!(!range.contains(&future));
    }
    
    #[test]
    fn test_coordinate_distance() {
        // San Francisco to Los Angeles (approximately 559 km)
        let sf = Coordinate::new(37.7749, -122.4194);
        let la = Coordinate::new(34.0522, -118.2437);
        
        let distance = sf.distance_to(&la);
        let distance_km = distance / 1000.0;
        
        // Should be approximately 559 km (with some tolerance for calculation)
        assert!((distance_km - 559.0).abs() < 10.0);
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Urgent);
    }
}