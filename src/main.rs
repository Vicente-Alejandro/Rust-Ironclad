// ============================================
// Application Modules
// ============================================

mod domain;
mod application;
mod infrastructure;
mod interfaces;
mod shared;
mod config;
mod errors;
mod utils;
mod cli;
mod db;
mod middleware;
mod routes;
mod app_state;  // 🆕

use middleware::MaintenanceMode;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;
use tracing_actix_web::TracingLogger;

use config::AppConfig;
use app_state::AppState;  // 🆕

#[macro_use]
mod macros;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ============================================
    // Initial Configuration
    // ============================================
    let app_config = AppConfig::from_env().expect("Failed to load config");

    // Configure logging
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::new("info,actix_server=warn,ironclads=debug")
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
    // 🎯 Initialize AppState Container
    // ============================================
    let app_state = AppState::new(app_config.clone(), pg_pool);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    tracing::info!("🌐 Listening on http://{}", address);
    tracing::info!("🔗 Documentation: http://{}:{}/api/docs", app_config.server.host, app_config.server.port);
    tracing::info!("");

    validate_security_config(&app_config);

    // ============================================
    // 🎯 Start HTTP Server
    // ============================================
    HttpServer::new(move || {
        let app = App::new();
        
        // Register services and app state macro (You must import them in app_state.rs)
        let app = register_services!(
            app,
            app_state,
            config,
            pool,
            auth_service,
            user_service,
            test_item_service
        );
        
        app.wrap(MaintenanceMode)
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