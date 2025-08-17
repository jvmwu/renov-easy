//! Integration tests for infrastructure initialization

use renov_infrastructure::{initialize, config};

#[tokio::test]
#[ignore] // Requires actual infrastructure services
async fn test_infrastructure_initialization() {
    let services = initialize().await;
    assert!(services.is_ok());
}

#[test]
fn test_default_config() {
    let config = config::InfrastructureConfig::default();
    assert!(!config.database.url.is_empty());
    assert!(config.database.max_connections > 0);
    assert!(!config.cache.url.is_empty());
}