// MULTIPLE DB CONFIG, FOR TESTING PURPOSES ONLY YET DONT USE THIS IN PRODUCTION
use crate::db::DbConfig;
use crate::errors::ApiError;
pub async fn init_db(config: &DbConfig) -> Result<(), ApiError> {
    match config {
        DbConfig::Postgres(pg_config) => {
            let pool = crate::db::postgres::init_pool(pg_config).await?;
            // You can store the pool in a global state or return it as needed
            println!("PostgreSQL connection pool initialized successfully.");
        },
        DbConfig::MySql(mysql_config) => {
            let pool = crate::db::mysql::init_pool(mysql_config).await?;
            println!("MySQL connection pool initialized successfully.");
        },
        DbConfig::MongoDB(mongo_config) => {
            let db = crate::db::mongo::init_mongodb(mongo_config).await?;
            println!("MongoDB connection initialized successfully.");
        },
    }
    Ok(())
} 

pub async fn init_all_sql_db(configs: &[DbConfig]) -> Result<(), ApiError> {
    for config in configs {
        init_db(config).await?;
    }
    Ok(())
}

pub async fn init_all_mongo_db(configs: &[DbConfig]) -> Result<(), ApiError> {
    for config in configs {
        if let DbConfig::MongoDB(mongo_config) = config {
            init_db(config).await?;
        }
    }
    Ok(())
}

pub async fn init_all_sql_db_with_test(configs: &[DbConfig]) -> Result<(), ApiError> {
    for config in configs {
        init_db(config).await?;
        // Optionally, you can add test connection logic here for each DB type
    }
    Ok(())
}