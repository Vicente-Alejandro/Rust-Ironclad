use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::time::Duration;
use crate::config::MySqlConfig;
use crate::errors::ApiError;

/// Initialize MySQL connection pool
pub async fn init_pool(config: &MySqlConfig) -> Result<MySqlPool, ApiError> {
    MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .connect(&config.mysql_url)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("MySQL connection failed: {}", e)))
}

/// Test MySQL connection with a ping
pub async fn test_connection(pool: &MySqlPool) -> Result<(), ApiError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("MySQL ping failed: {}", e)))?;
    
    Ok(())
}