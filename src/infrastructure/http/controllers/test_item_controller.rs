use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::application::dtos::{CreateTestItemRequest, UpdateTestItemRequest};
use crate::application::services::TestItemService;
use crate::errors::{ApiError, ApiResult};
use crate::infrastructure::http::authentication::AuthUser;
use crate::shared::ValidatedJson;

pub struct TestItemController;

impl TestItemController {
    /// Create new test item
    pub async fn create(
        service: web::Data<Arc<TestItemService>>,
        req: ValidatedJson<CreateTestItemRequest>,
    ) -> ApiResult<HttpResponse> {
        let item = service.create(req.0).await?;
        Ok(HttpResponse::Created().json(item))
    }

    /// Get all test items with pagination
    pub async fn get_all(
        service: web::Data<Arc<TestItemService>>,
        query: web::Query<PaginationQuery>,
    ) -> ApiResult<HttpResponse> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100);

        let response = service.get_all(page, per_page).await?;
        Ok(HttpResponse::Ok().json(response))
    }

    /// Get test item by ID
    pub async fn get_by_id(
        service: web::Data<Arc<TestItemService>>,
        id: web::Path<String>,
    ) -> ApiResult<HttpResponse> {
        let item = service.get_by_id(&id.into_inner()).await?
            .ok_or_else(|| ApiError::NotFound("Test item not found".to_string()))?;

        Ok(HttpResponse::Ok().json(item))
    }

    /// Update test item
    pub async fn update(
        service: web::Data<Arc<TestItemService>>,
        id: web::Path<String>,
        req: ValidatedJson<UpdateTestItemRequest>,
    ) -> ApiResult<HttpResponse> {
        let item = service.update(&id.into_inner(), req.0).await?;
        Ok(HttpResponse::Ok().json(item))
    }

    /// Delete test item
    pub async fn delete(
        service: web::Data<Arc<TestItemService>>,
        id: web::Path<String>,
    ) -> ApiResult<HttpResponse> {
        service.delete(&id.into_inner()).await?;
        Ok(HttpResponse::NoContent().finish())
    }
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}