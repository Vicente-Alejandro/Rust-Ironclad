use std::sync::Arc;
use crate::application::dtos::{
    PaginatedResponse, 
    UpdateProfileRequest, 
    UpdateRoleRequest, 
    UserResponse
};
use crate::domain::entities::User;
use crate::domain::value_objects::Role;
use crate::errors::ApiError;
use crate::interfaces::UserRepository;
use crate::utils::auth::hash_password;
use crate::config::AppConfig;
use crate::domain::value_objects::{EmailAddress, Username};

pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
    config: Arc<AppConfig>,  
}

impl UserService {
    pub fn new(user_repository: Arc<dyn UserRepository>, config: Arc<AppConfig>) -> Self {
        Self { 
            user_repository,
            config,  
        }
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, ApiError> {
        let user = self.user_repository.get_by_id(user_id).await?;
        Ok(user.map(|u| u.to_response()))
    }

    /// Get all users with pagination
    pub async fn get_all_users(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<UserResponse>, ApiError> {
        // Validate pagination parameters
        if page < 1 {
            return Err(ApiError::ValidationError(
                "Page must be greater than 0".to_string()
            ));
        }
        if per_page < 1 || per_page > 100 {
            return Err(ApiError::ValidationError(
                "Per page must be between 1 and 100".to_string()
            ));
        }

        let (users, total) = self
            .user_repository
            .get_paginated(page, per_page)
            .await?;

        let user_responses: Vec<UserResponse> = users
            .into_iter()
            .map(|u| u.to_response())
            .collect();

        Ok(PaginatedResponse::new(user_responses, total, page, per_page))
    }

    pub async fn get_all_users_no_pagination(&self) -> Result<Vec<UserResponse>, ApiError> {
        let users = self.user_repository.get_all().await?;
        Ok(users.into_iter().map(|u| u.to_response()).collect())
    }

pub async fn update_profile(
        &self,
        user_id: &str,
        request: UpdateProfileRequest,
    ) -> Result<UserResponse, ApiError> {
        // Buscar usuario
        let mut user = self
            .user_repository
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        // Update username if present
        if let Some(username_str) = request.username {
            // ✅ Build the Value Object. If invalid, returns error automatically.
            let username_vo = Username::new(username_str)?; 
            user.update_username(username_vo);
        }

        // Update email if present
        if let Some(email_str) = request.email {
            // Validar si el email ya existe en BD para otro usuario
            if let Some(existing_user) = self.user_repository.get_by_email(&email_str).await? {
                if existing_user.id != user_id {
                    return Err(ApiError::Conflict(
                        "Email is already in use by another user".to_string()
                    ));
                }
            }
            // ✅ Construimos el Value Object.
            let email_vo = EmailAddress::new(email_str)?;
            user.update_email(email_vo);
        }

        // Update password if present
        if let Some(password) = request.password {
            // ✅ You should add strong password validation here if applicable
            crate::shared::validator::validate_strong_password(&password)
                .map_err(|_| ApiError::ValidationError("Weak password".to_string()))?;

            let password_hash = hash_password(&password, &self.config)?;
            user.update_password_hash(password_hash);
        }

        // Persistir cambios
        self.user_repository.update(&user).await?;

        Ok(user.to_response())
    }


    /// Deactivate user (admin only)
    pub async fn deactivate_user(&self, user_id: &str) -> Result<(), ApiError> {
        let mut user = self
            .user_repository
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        if !user.is_active() {
            return Err(ApiError::ValidationError(
                "User is already inactive".to_string()
            ));
        }

        user.deactivate();
        self.user_repository.update(&user).await?;

        Ok(())
    }

    /// Activate user (admin only)
    pub async fn activate_user(&self, user_id: &str) -> Result<(), ApiError> {
        let mut user = self
            .user_repository
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        if user.is_active() {
            return Err(ApiError::ValidationError(
                "User is already active".to_string()
            ));
        }

        user.activate();
        self.user_repository.update(&user).await?;

        Ok(())
    }

    /// Delete user
    pub async fn delete_user(&self, user_id: &str) -> Result<(), ApiError> {
        let deleted = self.user_repository.delete(user_id).await?;

        if !deleted {
            return Err(ApiError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// Get total user count
    pub async fn get_user_count(&self) -> Result<i32, ApiError> {
        self.user_repository.count().await
    }
}
