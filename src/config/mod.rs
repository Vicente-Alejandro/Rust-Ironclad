//! # Application Configuration Module
//!
//! This module manages all application configuration settings, including server,
//! database, authentication, and security configurations. It supports loading
//! configuration from environment variables with sensible defaults.
//!
//! ## Configuration Sources
//! - Environment variables from `.env` file
//! - Default values for optional configurations
//! - Environment-specific defaults (development, staging, production)
//!
//! ## Example
//! ```rust
//! use config::AppConfig;
//! let config = AppConfig::from_env()?;
//! ```

pub mod validators;
pub use validators::validate_security_config;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

// ============================================================================
// PRIMARY APPLICATION CONFIGURATION
// ============================================================================

/// Main application configuration structure.
///
/// This serves as the root configuration container that aggregates all
/// sub-configurations for different aspects of the application including
/// server settings, database connections, and security parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration (host, port, environment)
    pub server: ServerConfig,
    /// PostgreSQL primary database configuration
    pub db_postgres: PostgresConfig,
    /// Optional MySQL database configuration
    pub db_mysql: Option<MySqlConfig>,
    /// Optional MongoDB database configuration
    pub mongodb: Option<MongoDBConfig>,
    /// JWT authentication configuration
    pub jwt: JwtConfig,
    /// Bcrypt password hashing configuration
    pub bcrypt: BcryptConfig,
}

// ============================================================================
// SERVER CONFIGURATION
// ============================================================================

/// Server runtime configuration.
///
/// Manages HTTP server settings including network binding and environment
/// identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address (default: "127.0.0.1")
    pub host: String,
    /// Server port number (default: 8080)
    pub port: u16,
    /// Environment identifier (development, staging, production)
    pub env: String,
}

// ============================================================================
// DATABASE CONFIGURATIONS
// ============================================================================

/// PostgreSQL database configuration.
///
/// Contains connection pool settings and database URL for PostgreSQL.
/// This is the primary database configuration and is always required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// PostgreSQL connection URL
    pub postgres_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection acquisition timeout in seconds
    pub acquire_timeout: u64,
    /// Connection idle timeout in seconds before closing
    pub idle_timeout: u64,
}

/// MySQL database configuration.
///
/// Optional secondary database configuration for MySQL support.
/// Enabled only when `MYSQL_URL` environment variable is provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlConfig {
    /// MySQL connection URL
    pub mysql_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection acquisition timeout in seconds
    pub acquire_timeout: u64,
    /// Connection idle timeout in seconds before closing
    pub idle_timeout: u64,
}

/// MongoDB database configuration.
///
/// Optional NoSQL database configuration for MongoDB support.
/// Enabled only when `MONGODB_URL` environment variable is provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    /// MongoDB connection URL
    pub mongo_url: String,
    /// Target database name
    pub database_name: String,
}

// ============================================================================
// AUTHENTICATION & SECURITY CONFIGURATIONS
// ============================================================================

/// JWT (JSON Web Token) configuration.
///
/// Manages JWT token generation and validation settings for API authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key for signing and verifying tokens
    pub secret: String,
    /// Token expiration time in seconds (default: 86400 = 24 hours)
    pub expiration: i64,
}

/// Bcrypt password hashing configuration.
///
/// Controls the computational cost of password hashing operations.
/// Higher values increase security but also increase hashing time.
/// Environment-specific defaults: production=12, staging=10, development=8
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcryptConfig {
    /// Bcrypt cost parameter (4-31, recommended: 10-12)
    pub cost: u32,
}

// ============================================================================
// CONFIGURATION INITIALIZATION
// ============================================================================

impl AppConfig {
    /// Constructs an `AppConfig` instance from environment variables.
    ///
    /// Loads configuration from `.env` file and environment variables with
    /// intelligent fallbacks. Optional database configurations are only
    /// enabled if their respective environment variables are provided.
    ///
    /// # Returns
    /// - `Ok(AppConfig)` - Successfully initialized configuration
    /// - `Err(AnyhowError)` - Failed to parse or load configuration
    ///
    /// # Panics
    /// Does not panic; all errors are returned as `Result`.
    ///
    /// # Example
    /// ```rust
    /// let config = AppConfig::from_env()?;
    /// println!("Server running on {}:{}", config.server.host, config.server.port);
    /// ```
    pub fn from_env() -> Result<Self> {
        // Load environment variables from .env file if present
        dotenv::dotenv().ok();

        let config = AppConfig {
            // --- Server Configuration ---
            server: ServerConfig {
                host: env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                env: env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string()),
            },

            // --- PostgreSQL Configuration (Primary Database) ---
            db_postgres: PostgresConfig {
                postgres_url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| {
                        "postgresql://user:password@localhost/template_db".to_string()
                    }),
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()?,
                acquire_timeout: env::var("DB_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                idle_timeout: env::var("DB_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()?,
            },

            // --- MySQL Configuration (Optional Secondary Database) ---
            db_mysql: if let Ok(mysql_url) = env::var("MYSQL_URL") {
                Some(MySqlConfig {
                    mysql_url,
                    max_connections: env::var("MYSQL_MAX_CONNECTIONS")
                        .unwrap_or_else(|_| "5".to_string())
                        .parse()?,
                    min_connections: env::var("MYSQL_MIN_CONNECTIONS")
                        .unwrap_or_else(|_| "1".to_string())
                        .parse()?,
                    acquire_timeout: env::var("MYSQL_ACQUIRE_TIMEOUT")
                        .unwrap_or_else(|_| "5".to_string())
                        .parse()?,
                    idle_timeout: env::var("MYSQL_IDLE_TIMEOUT")
                        .unwrap_or_else(|_| "300".to_string())
                        .parse()?,
                })
            } else {
                None
            },

            // --- MongoDB Configuration (Optional NoSQL Database) ---
            mongodb: if let Ok(mongo_url) = env::var("MONGODB_URL") {
                Some(MongoDBConfig {
                    mongo_url,
                    database_name: env::var("MONGODB_NAME")
                        .unwrap_or_else(|_| "template_db".to_string()),
                })
            } else {
                None
            },

            // --- JWT Authentication Configuration ---
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| {
                        "your-secret-key-change-in-production".to_string()
                    }),
                expiration: env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()?,
            },

            // --- Bcrypt Security Configuration ---
            // Bcrypt cost is determined by environment for optimal security/performance balance:
            // - Production: cost 12 (highest security, slower)
            // - Staging: cost 10 (balanced)
            // - Development: cost 8 (faster for development)
            // Can be overridden with BCRYPT_COST environment variable
            bcrypt: BcryptConfig {
                cost: env::var("BCRYPT_COST")
                    .unwrap_or_else(|_| {
                        match env::var("ENVIRONMENT").as_deref() {
                            Ok("production") => "12",
                            Ok("staging") => "10",
                            _ => "8", // development default
                        }
                        .to_string()
                    })
                    .parse()
                    .unwrap_or(10), // Safe fallback value
            },
        };

        Ok(config)
    }
}