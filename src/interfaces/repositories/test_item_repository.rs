use crate::domain::entities::TestItem;
use crate::errors::ApiError;
use async_trait::async_trait;

/// TestItem Repository trait
#[async_trait]
pub trait TestItemRepository: Send + Sync {
    /// Create new test item
    async fn create(&self, item: &TestItem) -> Result<TestItem, ApiError>;

    /// Get test item by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<TestItem>, ApiError>;

    /// Get all test items
    async fn get_all(&self) -> Result<Vec<TestItem>, ApiError>;

    /// Get test items with pagination
    async fn get_paginated(&self, page: i32, per_page: i32) -> Result<(Vec<TestItem>, i32), ApiError>;

    /// Update test item
    async fn update(&self, item: &TestItem) -> Result<(), ApiError>;

    /// Delete test item by ID
    async fn delete(&self, id: &str) -> Result<bool, ApiError>;

    /// Count total test items
    async fn count(&self) -> Result<i32, ApiError>;
}