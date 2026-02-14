use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::config::AppConfig;
use crate::db::postgres::{create_user_without_role, get_user_by_email};
use crate::errors::{ApiError, ApiResult};
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User};
use crate::middleware::role::AdminUser;
use crate::utils::auth::{hash_password, verify_password};
use crate::utils::jwt::create_token;

pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    req: web::Json<RegisterRequest>,
) -> ApiResult<HttpResponse> {
    // Validate input
    req.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Verify if user already exists
    if get_user_by_email(&pool, &req.email)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict("User already exists".to_string()));
    }
    
    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Create user with default role
    let user = User::new(req.email.clone(), req.username.clone(), password_hash);
    let created_user = create_user_without_role(&pool, &user).await?;

    // Create JWT token including the role (default role is "user")
    let token = create_token(&created_user.id, &created_user.email, &created_user.role, &config)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        user: created_user.to_response(),
        token,
    }))
}

pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<AppConfig>,
    req: web::Json<LoginRequest>,
) -> ApiResult<HttpResponse> {
    // Get user by email
    let user = get_user_by_email(&pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;

    // Verify password
    let password_valid = verify_password(&req.password, &user.password_hash)?;
    if !password_valid {
        return Err(ApiError::Unauthorized);
    }

    // Create JWT token including the role
    let token = create_token(&user.id, &user.email, &user.role, &config)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        user: user.to_response(),
        token,
    }))
}

// Endpoint to ensure the user is  an admin
pub async fn verify_admin(
    _admin: AdminUser,
) -> ApiResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "verified": true,
        "role": "admin"
    })))
}