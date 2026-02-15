use crate::domain::entities::User;
use crate::errors::ApiError;
use async_trait::async_trait;

/// User Repository - Data access contract
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create new user
    async fn create(&self, user: &User) -> Result<User, ApiError>;

    /// Get user by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<User>, ApiError>;

    /// Get user by email (String, no Value Object)
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, ApiError>;

    /// Get all users
    async fn get_all(&self) -> Result<Vec<User>, ApiError>;

    /// Get users with pagination
    async fn get_paginated(&self, page: i32, per_page: i32) -> Result<(Vec<User>, i32), ApiError>;

    /// Update user
    async fn update(&self, user: &User) -> Result<(), ApiError>;

    /// Delete user by ID
    async fn delete(&self, id: &str) -> Result<bool, ApiError>;

    /// Count total users
    async fn count(&self) -> Result<i32, ApiError>;

    /// Check if user exists with this email (String)
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApiError>;
}