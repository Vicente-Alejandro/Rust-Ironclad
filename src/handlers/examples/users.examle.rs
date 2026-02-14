use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::db::postgres::{
    delete_user, get_all_users_with_no_auth, get_user_by_id, update_user_profile,
};
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthUser;
use crate::middleware::role::AdminUser;
use crate::utils::auth::hash_password;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

pub async fn get_user_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    user_id: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let requested_id = user_id.into_inner();

    if auth.0.sub != requested_id && auth.0.role != "admin" {
        return Err(ApiError::Conflict(
            "You don't have permission to view this user".to_string(),
        ));
    }

    let user = get_user_by_id(&pool, &requested_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(user.to_response()))
}

pub async fn get_profile(pool: web::Data<PgPool>, auth: AuthUser) -> ApiResult<HttpResponse> {
    let user = get_user_by_id(&pool, &auth.0.sub)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(user.to_response()))
}

pub async fn get_all_profiles(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
) -> ApiResult<HttpResponse> {
    let users = get_all_users_with_no_auth(&pool).await?;

    let response: Vec<_> = users.iter().map(|u| u.to_response()).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_all_profile_with_no_auth(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let users = get_all_users_with_no_auth(&pool).await?;

    let response: Vec<_> = users.iter().map(|u| u.to_response()).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn update_user_by_admin(
    pool: web::Data<PgPool>,
    _admin: AdminUser,
    user_id: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
) -> ApiResult<HttpResponse> {
    let requested_id = user_id.into_inner();

    let mut user = get_user_by_id(&pool, &requested_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    if let Some(username) = &req.username {
        user.username = username.clone();
    }
    if let Some(email) = &req.email {
        user.email = email.clone();
    }
    if let Some(password) = &req.password {
        user.password_hash = hash_password(password)?;
    }
    if let Some(role) = &req.role {
        if !["admin", "user", "moderator"].contains(&role.as_str()) {
            return Err(ApiError::BadRequest("Invalid role".to_string()));
        }
        user.role = role.clone();
    }

    update_user_profile(&pool, &user).await?;

    Ok(HttpResponse::Ok().json(user.to_response()))
}

pub async fn delete_user_by_id(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    user_id: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let requested_id = user_id.into_inner();
    
    if auth.0.sub != requested_id && auth.0.role != "admin" {
        return Err(ApiError::Conflict("You don't have permission to delete this user".to_string()));
    }

    let deleted = delete_user(&pool, &requested_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}