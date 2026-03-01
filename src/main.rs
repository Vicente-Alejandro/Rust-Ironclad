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
mod app_state;

use middleware::MaintenanceMode;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;
use tracing_actix_web::TracingLogger;

use config::{AppConfig, validate_security_config};  
use app_state::AppState;
use infrastructure::http::handle_not_found;  

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
    // Validate Security Configuration
    // ============================================
    validate_security_config(&app_config);  // Valide bcrypt cost and JWT settings

    // ============================================
    // Initialize AppState Container
    // ============================================
    let app_state = AppState::new(app_config.clone(), pg_pool);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    tracing::info!("🌐 Listening on http://{}", address);
    tracing::info!("🔗 Documentation: http://{}:{}/api/docs", app_config.server.host, app_config.server.port);
    tracing::info!("");

    // ============================================
    // Start HTTP Server
    // ============================================
    HttpServer::new(move || {
        let app = App::new();
        
        // Register services and app state macro 
         let app = register_services!(
            app,
            app_state,
            config,
            pool,
            auth_service,
            user_service,
            test_item_service
        );
        
        app.wrap(MaintenanceMode) // Middleware for maintenance mode (NOTE: This should be before CORS to ensure it can handle maintenance responses properly)
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