use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use crate::config::DatabaseConfig;
use crate::errors::ApiError;

/// Initialize PostgreSQL connection pool
pub async fn init_pool(config: &DatabaseConfig) -> Result<PgPool, ApiError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.postgres_url)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}
