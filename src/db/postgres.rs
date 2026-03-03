use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration; 
use crate::config::PostgresConfig;
use crate::errors::ApiError;

/// Initialize PostgreSQL connection pool
pub async fn init_pool(config: &PostgresConfig) -> Result<PgPool, ApiError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        // THREADSAFE: You must ensure that the database can handle the minimum number of connections configured here, otherwise the application may fail to start.
        .min_connections(config.min_connections) 
        // THREADSAFE: Setting this too low may cause transient failures under load, setting it too high may cause resource exhaustion.
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        // THREADSAFE: Setting this too low may cause connections to be closed prematurely, setting it too high may delay error detection.
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .connect(&config.postgres_url)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))
}