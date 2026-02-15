// ============================================
// Application Modules
// ============================================

// Domain Layer - Pure business logic
mod domain;

// Application Layer - Use cases and services
mod application;

// Infrastructure Layer - Technical details
mod infrastructure;

// Interfaces / Repositories - Contracts
mod interfaces;

// Shared utilities - Cross-cutting utilities
mod shared;

// Configuration
mod config;

// Error handling
mod errors;

// Utilities
mod utils;

// CLI Tools
mod cli;

// Database (legacy - initialization only)
mod db;

// Routes configuration
mod routes;

use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;
use std::sync::Arc;

use config::AppConfig;
use infrastructure::{AuthController, UserController};
use infrastructure::PostgresUserRepository;
use application::{AuthService, UserService};
use interfaces::UserRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ============================================
    // Initial Configuration
    // ============================================
    let app_config = AppConfig::from_env().expect("Failed to load config");

    // Configure logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("template_project=debug".parse().unwrap()),
        )
        .init();

    tracing::info!("╔════════════════════════════════════════════════════╗");
    tracing::info!("║    🚀 Rust Ironclad Framework (DDD Architecture)  ║");
    tracing::info!("╚════════════════════════════════════════════════════╝");
    tracing::info!("📍 Server: http://{}:{}", app_config.server.host, app_config.server.port);

    // ============================================
    // Initialize Database
    // ============================================
    let pg_pool = db::postgres::init_pool(&app_config.database)
        .await
        .expect("Failed to initialize PostgreSQL pool");
    tracing::info!("✅ PostgreSQL connected");

    // MongoDB (optional)
    if let Some(mongo_config) = &app_config.mongodb {
        match db::mongo::init_mongodb(mongo_config).await {
            Ok(_) => tracing::info!("✅ MongoDB connected"),
            Err(e) => tracing::warn!("⚠️  MongoDB skipped: {}", e),
        }
    }

    // ============================================
    // Dependency Injection (DI Container)
    // ============================================
    
    // Repositories (Interfaces)
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(PostgresUserRepository::new(pg_pool.clone()));

    // Services (Application Layer)
    let auth_service = Arc::new(AuthService::new(
        user_repository.clone(),
        Arc::new(app_config.clone()),
    ));
    
    let user_service = Arc::new(UserService::new(user_repository.clone()));

    // Prepare data for Actix
    let config_data = web::Data::new(app_config.clone());
    let auth_service_data = web::Data::new(auth_service);
    let user_service_data = web::Data::new(user_service);
    let pool_data = web::Data::new(pg_pool);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    tracing::info!("🌐 Listening on http://{}", address);
    tracing::info!("🔗 Documentation: http://{}:{}/api/docs", app_config.server.host, app_config.server.port);
    tracing::info!("");

    // ============================================
    // Start HTTP Server
    // ============================================
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(auth_service_data.clone())
            .app_data(user_service_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .configure(routes::configure)
            .default_service(web::route().to(handle_not_found))
    })
    .bind(&address)?
    .run()
    .await
}

/// Handler for not found routes
async fn handle_not_found() -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotFound().json(serde_json::json!({
        "error": "Not Found",
        "message": "The requested endpoint does not exist",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        // "hint": "Check the API documentation at GET /api/docs"
    }))
}

