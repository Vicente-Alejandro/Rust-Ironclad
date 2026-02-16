use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO for creating a test item
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTestItemRequest {
    #[validate(length(min = 1, max = 255, message = "Subject must be between 1 and 255 characters"))]
    pub subject: String,
    
    #[validate(length(max = 1000, message = "Optional field cannot exceed 1000 characters"))]
    pub optional_field: Option<String>,
}

/// DTO for updating a test item
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTestItemRequest {
    #[validate(length(min = 1, max = 255, message = "Subject must be between 1 and 255 characters"))]
    pub subject: Option<String>,
    
    #[validate(length(max = 1000, message = "Optional field cannot exceed 1000 characters"))]
    pub optional_field: Option<String>,
}

/// DTO for test item response
#[derive(Debug, Serialize, Deserialize)]
pub struct TestItemResponse {
    pub id: String,
    pub subject: String,
    pub optional_field: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// DTO for paginated response
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedTestItemsResponse {
    pub data: Vec<TestItemResponse>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl PaginatedTestItemsResponse {
    pub fn new(data: Vec<TestItemResponse>, total: i32, page: i32, per_page: i32) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}