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

// Middleware
mod middleware;

// Routes configuration
mod routes;

use middleware::MaintenanceMode;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;
use tracing_actix_web::TracingLogger;
use std::sync::Arc;

use config::AppConfig;
use infrastructure::{PostgresUserRepository, PostgresTestItemRepository};
use application::{AuthService, UserService, TestItemService};
use interfaces::{UserRepository, TestItemRepository};

// use infrastructure::{*};
// use application::{*};
// use interfaces::{*};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ============================================
    // Initial Configuration
    // ============================================
    let app_config = AppConfig::from_env().expect("Failed to load config");

    // Configure logging - ensure visibility
    // tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG)
        // .with_target(true)
        // .with_thread_ids(true)
        // .with_file(true)
        // .with_line_number(true)
        // .with_env_filter(
        //      tracing_subscriber::EnvFilter::from_default_env()
        //          .add_directive("template_project=debug".parse().unwrap()),
        //  )
        // .pretty()
        // .init();

    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         tracing_subscriber::EnvFilter::from_default_env()
    //             .add_directive("template_project=debug".parse().unwrap()),
    //     )
    //     .init();

    // Configure logging - Clean, minimal output with bulletproof visibility
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(
            // SOLUTION: "info" as the global baseline, "warn" to silence Actix startup noise,
            // and "debug" exclusively for your project's submodules.
            tracing_subscriber::EnvFilter::new("info,actix_server=warn,template_project=debug")
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
    
    let user_service = Arc::new(UserService::new(
        user_repository.clone(),
        Arc::new(app_config.clone()),  
    ));

    let test_item_repository: Arc<dyn TestItemRepository> =
        Arc::new(PostgresTestItemRepository::new(pg_pool.clone()));
    let test_item_service = Arc::new(TestItemService::new(test_item_repository));

    // Prepare data for Actix
    let config_data = web::Data::new(app_config.clone());
    let auth_service_data = web::Data::new(auth_service);
    let user_service_data = web::Data::new(user_service);
    let test_item_service_data = web::Data::new(test_item_service);
    let pool_data = web::Data::new(pg_pool);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    tracing::info!("🌐 Listening on http://{}", address);
    tracing::info!("🔗 Documentation: http://{}:{}/api/docs", app_config.server.host, app_config.server.port);
    tracing::info!("");

    validate_security_config(&app_config);

    // ============================================
    // Start HTTP Server
    // ============================================
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(auth_service_data.clone())
            .app_data(user_service_data.clone())
            .app_data(test_item_service_data.clone())
            .wrap(MaintenanceMode)
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .wrap(TracingLogger::default())
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

fn validate_security_config(config: &AppConfig) {
    if config.bcrypt.cost < 4 {
        tracing::error!(
            "🔴 BCRYPT_COST={} is CRITICALLY LOW for security (minimum: 4)", 
            config.bcrypt.cost
        );
    } else if config.bcrypt.cost < 8 {
        tracing::warn!(
            "⚠️  BCRYPT_COST={} is LOW for security (recommended: 8+)", 
            config.bcrypt.cost
        );
    }

    match config.server.env.as_str() {
        "production" => {
            if config.bcrypt.cost < 10 {
                tracing::error!(
                    "🔴 BCRYPT_COST={} is TOO LOW for production (minimum: 10, recommended: 12)", 
                    config.bcrypt.cost
                );
                tracing::error!("   Production deployment blocked for security reasons");
                std::process::exit(1);  
            } else if config.bcrypt.cost >= 12 {
                tracing::info!(
                    "✅ BCRYPT_COST={} is IDEAL for production", 
                    config.bcrypt.cost
                );
            } else {
                tracing::warn!(
                    "⚠️  BCRYPT_COST={} is acceptable for production (recommended: 12)", 
                    config.bcrypt.cost
                );
            }

            if config.jwt.secret.len() < 32 {
                tracing::error!("🔴 JWT_SECRET is too short for production (minimum: 32 characters)");
                std::process::exit(1);
            }
            if config.jwt.secret.contains("change") || config.jwt.secret.contains("secret") {
                tracing::error!("🔴 JWT_SECRET appears to be a default value - change it!");
                std::process::exit(1);
            }
        }
        "staging" => {
            if config.bcrypt.cost < 8 {
                tracing::warn!(
                    "⚠️  BCRYPT_COST={} is too low for staging (minimum: 8)", 
                    config.bcrypt.cost
                );
            } else if config.bcrypt.cost >= 10 {
                tracing::info!("✅ BCRYPT_COST={} is good for staging", config.bcrypt.cost);
            }
        }
        "development" => {
            if config.bcrypt.cost < 6 {
                tracing::warn!(
                    "⚠️  BCRYPT_COST={} may be too low even for development", 
                    config.bcrypt.cost
                );
            } else {
                tracing::info!(
                    "ℹ️  BCRYPT_COST={} (development mode - faster hashing)", 
                    config.bcrypt.cost
                );
            }
        }
        _ => {
            tracing::warn!("⚠️  Unknown environment: {}", config.server.env);
        }
    }
}
