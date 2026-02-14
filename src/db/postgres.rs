use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use crate::config::DatabaseConfig;
use crate::errors::ApiError;
use crate::models::user::User;
use chrono::Utc;

pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool, ApiError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.postgres_url)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}

pub async fn create_user_without_role(pool: &PgPool, user: &User) -> Result<User, ApiError> {
    let query = r#"
        INSERT INTO users (id, email, username, password_hash, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'user', true, $5, $6)
        RETURNING id, email, username, password_hash, role, is_active, created_at, updated_at
    "#;

    sqlx::query_as::<_, User>(query)
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, ApiError> {
    let query = r#"
        SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
        FROM users
        WHERE email = $1
    "#;

    sqlx::query_as::<_, User>(query)
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}

pub async fn get_user_by_id(pool: &PgPool, id: &str) -> Result<Option<User>, ApiError> {
    let query = r#"
        SELECT id, email, username, password_hash, role, is_active, created_at, updated_at
        FROM users
        WHERE id = $1
    "#;

    sqlx::query_as::<_, User>(query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}

pub async fn update_user_profile(pool: &PgPool, user: &User) -> Result<(), ApiError> {
    let query = r#"
        UPDATE users
        SET email = $1, username = $2, password_hash = $3, role = $4, is_active = $5, updated_at = $6
        WHERE id = $7
    "#;

    sqlx::query(query)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.is_active)
        .bind(Utc::now())
        .bind(&user.id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn delete_user(pool: &PgPool, id: &str) -> Result<bool, ApiError> {
    let query = "DELETE FROM users WHERE id = $1";

    let result = sqlx::query(query)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() > 0)
}
