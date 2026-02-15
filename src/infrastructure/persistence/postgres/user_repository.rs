use sqlx::PgPool;
use async_trait::async_trait;
use chrono::Utc;

use crate::domain::entities::User;
use crate::domain::value_objects::Role;
use crate::errors::ApiError;
use crate::interfaces::repositories::UserRepository;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<User, ApiError> {
        let query = r#"
            INSERT INTO users (id, email, username, password_hash, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;
        
        let created_user = sqlx::query_as::<_, User>(query)
            .bind(&user.id)
            .bind(&user.email)
            .bind(&user.username)
            .bind(&user.password_hash)
            .bind(user.role.as_str())
            .bind(user.is_active)
            .bind(user.created_at)
            .bind(user.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(created_user)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<User>, ApiError> {
        let query = "SELECT * FROM users WHERE id = $1";

        // ✅ Usar query_as
        let user = sqlx::query_as::<_, User>(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let query = "SELECT * FROM users WHERE email = $1";

        // ✅ Usar query_as
        let user = sqlx::query_as::<_, User>(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    async fn get_all(&self) -> Result<Vec<User>, ApiError> {
        let query = "SELECT * FROM users ORDER BY created_at DESC";

        // ✅ Usar query_as
        let users = sqlx::query_as::<_, User>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(users)
    }

    async fn get_paginated(&self, page: i32, per_page: i32) -> Result<(Vec<User>, i32), ApiError> {
        let offset = (page - 1) * per_page;

        let query = r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
        "#;

        // ✅ Usar query_as
        let users = sqlx::query_as::<_, User>(query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let total = self.count().await?;

        Ok((users, total))
    }

    async fn update(&self, user: &User) -> Result<(), ApiError> {
        let query = r#"
            UPDATE users
            SET email = $1, username = $2, password_hash = $3, role = $4, 
                is_active = $5, updated_at = $6
            WHERE id = $7
        "#;

        sqlx::query(query)
            .bind(&user.email)
            .bind(&user.username)
            .bind(&user.password_hash)
            .bind(user.role.as_str())
            .bind(user.is_active)
            .bind(Utc::now())
            .bind(&user.id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool, ApiError> {
        let query = "DELETE FROM users WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> Result<i32, ApiError> {
        let query = "SELECT COUNT(*) as count FROM users";

        let row: (i64,) = sqlx::query_as(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.0 as i32)
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, ApiError> {
        let query = "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)";

        let row: (bool,) = sqlx::query_as(query)
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.0)
    }
}