use bcrypt::{hash, verify, DEFAULT_COST};
use crate::errors::ApiError;
use crate::models::user::Claims;

/// Hash password with bcrypt
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| ApiError::InternalServerError(format!("Error hashing password: {}", e)))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    verify(password, hash)
        .map_err(|e| ApiError::InternalServerError(format!("Error verifying password: {}", e)))
}

/// Verifies if the authenticated user can access the requested resource
/// AND
/// Allows access if it's the same user OR if it's an admin, this is different than only use AuthUser, so it is not necesary unless you want to check for admin or self
pub fn verify_self_or_admin(claims: &Claims, target_user_id: &str) -> Result<(), ApiError> {
    if claims.sub == target_user_id || claims.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Conflict("You don't have permission to access this resource".to_string()))
    }
}

pub fn verify_admin(claims: &Claims) -> Result<(), ApiError> {
    if claims.role == "admin" {
        Ok(())
    } else {
        Err(ApiError::Conflict("You don't have permission to access this resource".to_string()))
    }
}