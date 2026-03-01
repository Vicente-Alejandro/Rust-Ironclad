pub mod validators;
pub use validators::validate_security_config;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub mongodb: Option<MongoDBConfig>,
    pub jwt: JwtConfig,
    pub bcrypt: BcryptConfig,  
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    pub mongo_url: String,
    pub database_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
}

// 🆕 Nueva estructura para Bcrypt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcryptConfig {
    pub cost: u32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let config = AppConfig {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                env: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            },
            database: DatabaseConfig {
                postgres_url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://user:password@localhost/template_db".to_string()),
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
            },
            mongodb: if let Ok(mongo_url) = env::var("MONGODB_URL") {
                Some(MongoDBConfig {
                    mongo_url,
                    database_name: env::var("MONGODB_NAME")
                        .unwrap_or_else(|_| "template_db".to_string()),
                })
            } else {
                None
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
                expiration: env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()?,
            },
            // Read BCRYPT_COST from .env with fallback based on ENVIRONMENT
            bcrypt: BcryptConfig {
                cost: env::var("BCRYPT_COST")
                    .unwrap_or_else(|_| {
                        match env::var("ENVIRONMENT").as_deref() {
                            Ok("production") => "12",
                            Ok("staging") => "10",
                            _ => "8",
                        }
                        .to_string()
                    })
                    .parse()
                    .unwrap_or(10),  
            },
        };

        Ok(config)
    }
}