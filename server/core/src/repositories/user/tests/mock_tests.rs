//! Unit tests for mock user repository

use uuid::Uuid;

use crate::domain::entities::user::{User, UserType};
use crate::errors::DomainError;
use crate::repositories::user::{UserRepository, MockUserRepository};

#[tokio::test]
async fn test_mock_repository_create_and_find() {
    let repo = MockUserRepository::new();
    
    let user = User::new(
        "test_hash".to_string(),
        "+61".to_string(),
    );
    
    let created = repo.create(user.clone()).await.unwrap();
    assert_eq!(created.id, user.id);
    
    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);
}

#[tokio::test]
async fn test_mock_repository_find_by_phone() {
    let repo = MockUserRepository::new();
    
    let user = User::new(
        "phone_hash_123".to_string(),
        "+86".to_string(),
    );
    
    repo.create(user.clone()).await.unwrap();
    
    let found = repo
        .find_by_phone("phone_hash_123", "+86")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);
}

#[tokio::test]
async fn test_mock_repository_duplicate_phone() {
    let repo = MockUserRepository::new();
    
    let user1 = User::new(
        "same_hash".to_string(),
        "+61".to_string(),
    );
    
    let user2 = User::new(
        "same_hash".to_string(),
        "+61".to_string(),
    );
    
    repo.create(user1).await.unwrap();
    let result = repo.create(user2).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::Validation { .. }));
}

#[tokio::test]
async fn test_mock_repository_update() {
    let repo = MockUserRepository::new();
    
    let mut user = User::new(
        "test_hash".to_string(),
        "+61".to_string(),
    );
    
    repo.create(user.clone()).await.unwrap();
    
    user.set_user_type(UserType::Customer);
    user.verify();
    
    let updated = repo.update(user.clone()).await.unwrap();
    assert_eq!(updated.user_type, Some(UserType::Customer));
    assert!(updated.is_verified);
}

#[tokio::test]
async fn test_mock_repository_delete() {
    let repo = MockUserRepository::new();
    
    let user = User::new(
        "test_hash".to_string(),
        "+61".to_string(),
    );
    
    repo.create(user.clone()).await.unwrap();
    
    let deleted = repo.delete(user.id).await.unwrap();
    assert!(deleted);
    
    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_mock_repository_count_by_type() {
    let repo = MockUserRepository::new();
    
    let mut customer = User::new("customer_hash".to_string(), "+61".to_string());
    customer.set_user_type(UserType::Customer);
    
    let mut worker = User::new("worker_hash".to_string(), "+86".to_string());
    worker.set_user_type(UserType::Worker);
    
    let untyped = User::new("untyped_hash".to_string(), "+1".to_string());
    
    repo.create(customer).await.unwrap();
    repo.create(worker).await.unwrap();
    repo.create(untyped).await.unwrap();
    
    assert_eq!(repo.count_by_type(None).await.unwrap(), 3);
    assert_eq!(
        repo.count_by_type(Some(UserType::Customer)).await.unwrap(),
        1
    );
    assert_eq!(
        repo.count_by_type(Some(UserType::Worker)).await.unwrap(),
        1
    );
}