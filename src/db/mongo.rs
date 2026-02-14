use mongodb::{Client, Database};
use crate::config::MongoDBConfig;
use crate::errors::ApiError;

// Not tested yet, just a placeholder for MongoDB integration
pub async fn init_mongodb(config: &MongoDBConfig) -> Result<Database, ApiError> {
    let client = Client::with_uri_str(&config.mongo_url)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(client.database(&config.database_name))
}
