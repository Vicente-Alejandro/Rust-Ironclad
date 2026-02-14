use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

use crate::db::postgres::{delete_user, get_user_by_id, update_user_profile};
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthUser;
use crate::utils::auth::hash_password;
use crate::utils::auth::{*};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

// Update user profile (both self-update and admin update)
pub async fn update_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    user_id: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
) -> ApiResult<HttpResponse> {
    let requested_id = user_id.into_inner();
    
    // Optional allow if it's the same user OR if it's admin
    verify_self_or_admin(&auth.0, &requested_id)?;

    // Search for the user
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

    update_user_profile(&pool, &user).await?;

    Ok(HttpResponse::Ok().json(user.to_response()))
}


pub async fn delete_profile(
    pool: web::Data<PgPool>,
    auth: AuthUser,
    user_id: web::Path<String>,  
) -> ApiResult<HttpResponse> {
    let requested_id = user_id.into_inner();
    
    // Optional allow if it's the same user OR if it's admin
    verify_self_or_admin(&auth.0, &requested_id)?;

    let deleted = delete_user(&pool, &requested_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("User not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}