use sqlx::PgPool;
use sqlx::Row;
use async_trait::async_trait;
use chrono::Utc;

use crate::domain::entities::User;
use crate::domain::value_objects::Role;  // ✅ Solo Role, NO Email
use crate::errors::ApiError;
use crate::interfaces::repositories::UserRepository;

/// Concrete implementation of the repository for PostgreSQL
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
            RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at
        "#;

        let row = sqlx::query(query)
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

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            role: Role::from_str(&row.get::<String, _>("role")),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<User>, ApiError> {
        let query = r#"
            SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            role: Role::from_str(&r.get::<String, _>("role")),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let query = r#"
            SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
            FROM users
            WHERE email = $1
        "#;

        let row = sqlx::query(query)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            role: Role::from_str(&r.get::<String, _>("role")),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_all(&self) -> Result<Vec<User>, ApiError> {
        let query = r#"
            SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                email: r.get("email"),
                username: r.get("username"),
                password_hash: r.get("password_hash"),
                role: Role::from_str(&r.get::<String, _>("role")),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    async fn get_paginated(&self, page: i32, per_page: i32) -> Result<(Vec<User>, i32), ApiError> {
        let offset = (page - 1) * per_page;

        let query = r#"
            SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
        "#;

        let rows = sqlx::query(query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let total = self.count().await?;

        let users = rows
            .iter()
            .map(|r| User {
                id: r.get("id"),
                email: r.get("email"),
                username: r.get("username"),
                password_hash: r.get("password_hash"),
                role: Role::from_str(&r.get::<String, _>("role")),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect();

        Ok((users, total))
    }

    async fn update(&self, user: &User) -> Result<(), ApiError> {
        let query = r#"
            UPDATE users
            SET email = $1, username = $2, password_hash = $3, role = $4, is_active = $5, updated_at = $6
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

        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.get::<i32, _>("count"))
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, ApiError> {
        let query = "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists";

        let row = sqlx::query(query)
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.get::<bool, _>("exists"))
    }
}