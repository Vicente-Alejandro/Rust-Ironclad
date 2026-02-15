use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::application::dtos::{UpdateProfileRequest, UpdateRoleRequest};
use crate::application::services::UserService;
use crate::errors::{ApiError, ApiResult};
use crate::infrastructure::http::authentication::{AdminUser, AuthUser};
use crate::shared::ValidatedJson;  // 🆕

pub struct UserController;

impl UserController {
    /// Get authenticated user's own profile
    pub async fn get_profile(
        service: web::Data<Arc<UserService>>,
        auth: AuthUser,
    ) -> ApiResult<HttpResponse> {
        let user = service
            .get_user(&auth.0.sub)
            .await?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(HttpResponse::Ok().json(user))
    }

    /// Get all users (admin only)
    pub async fn get_all_users(
        service: web::Data<Arc<UserService>>,
        _admin: AdminUser,
        query: web::Query<PaginationQuery>,
    ) -> ApiResult<HttpResponse> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100); // Max 100 per page

        let response = service.get_all_users(page, per_page).await?;
        Ok(HttpResponse::Ok().json(response))
    }

    /// Get user by ID (self or admin)
    pub async fn get_user(
        service: web::Data<Arc<UserService>>,
        auth: AuthUser,
        user_id: web::Path<String>,
    ) -> ApiResult<HttpResponse> {
        let requested_id = user_id.into_inner();

        // Only admin or the same user can view
        if auth.0.sub != requested_id && !auth.0.is_admin() {
            return Err(ApiError::Forbidden(
                "You don't have permission to view this user".to_string(),
            ));
        }

        let user = service
            .get_user(&requested_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(HttpResponse::Ok().json(user))
    }

    /// Get all users without session (public endpoint)
    /// This is just an example and should be used with caution in a real application
    pub async fn get_all_users_no_session(
        service: web::Data<Arc<UserService>>,
    ) -> ApiResult<HttpResponse> {
        let users = service.get_all_users_no_pagination().await?;
        Ok(HttpResponse::Ok().json(users))
    }

    /// Update user profile (self only)
    pub async fn update_profile(
        service: web::Data<Arc<UserService>>,
        auth: AuthUser,
        user_id: web::Path<String>,
        req: ValidatedJson<UpdateProfileRequest>,  // 🆕
    ) -> ApiResult<HttpResponse> {
        let requested_id = user_id.into_inner();

        if auth.0.sub != requested_id {
            return Err(ApiError::Forbidden(
                "You can only update your own profile".to_string(),
            ));
        }

        let user = service
            .update_profile(&requested_id, req.0)  // 🆕 req.0
            .await?;

        Ok(HttpResponse::Ok().json(user))
    }

    /// Update user role (admin only)
    // pub async fn update_user_role(
    //     service: web::Data<Arc<UserService>>,
    //     _admin: AdminUser,
    //     user_id: web::Path<String>,
    //     req: ValidatedJson<UpdateRoleRequest>,  // 🆕
    // ) -> ApiResult<HttpResponse> {
    //     let user = service
    //         .update_user_role(&user_id.into_inner(), req.0)  // 🆕
    //         .await?;

    //     Ok(HttpResponse::Ok().json(user))
    // }

    /// Delete user (self or admin)
    pub async fn delete_user(
        service: web::Data<Arc<UserService>>,
        auth: AuthUser,
        user_id: web::Path<String>,
    ) -> ApiResult<HttpResponse> {
        let requested_id = user_id.into_inner();

        // Only admin or the same user can delete
        if auth.0.sub != requested_id && !auth.0.is_admin() {
            return Err(ApiError::Forbidden(
                "You don't have permission to delete this user".to_string(),
            ));
        }

        service.delete_user(&requested_id).await?;
        Ok(HttpResponse::NoContent().finish())
    }
}

/// Query params for pagination
#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}