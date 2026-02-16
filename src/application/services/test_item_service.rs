use std::sync::Arc;
use crate::application::dtos::{
    CreateTestItemRequest,
    UpdateTestItemRequest,
    TestItemResponse,
    PaginatedTestItemsResponse,
};
use crate::domain::entities::TestItem;
use crate::errors::ApiError;
use crate::interfaces::TestItemRepository;

pub struct TestItemService {
    repository: Arc<dyn TestItemRepository>,
}

impl TestItemService {
    pub fn new(repository: Arc<dyn TestItemRepository>) -> Self {
        Self { repository }
    }

    /// Create new test item
    pub async fn create(&self, request: CreateTestItemRequest) -> Result<TestItemResponse, ApiError> {
        let item = TestItem::new(request.subject, request.optional_field);
        let created_item = self.repository.create(&item).await?;
        Ok(created_item.to_response())
    }

    /// Get test item by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<TestItemResponse>, ApiError> {
        let item = self.repository.get_by_id(id).await?;
        Ok(item.map(|i| i.to_response()))
    }

    /// Get all test items with pagination
    pub async fn get_all(&self, page: i32, per_page: i32) -> Result<PaginatedTestItemsResponse, ApiError> {
        if page < 1 || per_page < 1 || per_page > 100 {
            return Err(ApiError::ValidationError("Invalid pagination parameters".to_string()));
        }

        let (items, total) = self.repository.get_paginated(page, per_page).await?;
        let item_responses: Vec<TestItemResponse> = items.into_iter().map(|i| i.to_response()).collect();

        Ok(PaginatedTestItemsResponse::new(item_responses, total, page, per_page))
    }

    /// Update test item
    pub async fn update(&self, id: &str, request: UpdateTestItemRequest) -> Result<TestItemResponse, ApiError> {
        let mut item = self.repository.get_by_id(id).await?
            .ok_or_else(|| ApiError::NotFound("Test item not found".to_string()))?;

        if let Some(subject) = request.subject {
            item.update_subject(subject);
        }

        if request.optional_field.is_some() {
            item.update_optional_field(request.optional_field);
        }

        self.repository.update(&item).await?;
        Ok(item.to_response())
    }

    /// Delete test item
    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(ApiError::NotFound("Test item not found".to_string()));
        }
        Ok(())
    }
}