use serde::{Deserialize, Serialize};
use crate::domain::value_objects::Role;
use validator::Validate;  

/// DTO para registro de usuario
#[derive(Debug, Deserialize, Serialize, Validate)]  // 🆕 Agregar Validate
pub struct RegisterUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// DTO para login
#[derive(Debug, Deserialize, Serialize, Validate)]  // 🆕 Agregar Validate
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// DTO para actualizar perfil de usuario
#[derive(Debug, Deserialize, Serialize, Validate)]  // 🆕 Agregar Validate
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    pub username: Option<String>,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: Option<String>,
}

/// DTO para actualizar role (solo admin)
#[derive(Debug, Deserialize, Serialize, Validate)]  // 🆕 Agregar Validate
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
}


/// DTO para respuesta de usuario (sin datos sensibles)
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// DTO for authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

/// DTO para respuesta paginada
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i32, page: i32, per_page: i32) -> Self {
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
