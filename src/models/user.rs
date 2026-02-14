use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,  // "admin", "user", "moderator", etc - definible por backend
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl User {
    pub fn new(email: String, username: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            role: "user".to_string(), // Default role
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn new_with_role(email: String, username: String, password_hash: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            role,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id.clone(),
            email: self.email.clone(),
            username: self.username.clone(),
            role: self.role.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
        }
    }
}