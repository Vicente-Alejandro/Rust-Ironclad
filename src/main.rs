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
pub mod route_registry;
mod bootstrap;
mod queue;
mod monitoring;

use middleware::MaintenanceMode;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;
use tracing_actix_web::TracingLogger;
use std::sync::Arc;
use crate::queue::Worker;

use config::{AppConfig, validate_security_config};  
use bootstrap::AppState;
use infrastructure::http::handle_not_found;  

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
    // Initialize Databases
    // ============================================
    
    // For multiple DB support, you can use the following approach:
    // let db_configs = vec![
    //     DbConfig::Postgres(app_config.db_postgres.clone()),
    //     app_config.db_mysql.clone().map(DbConfig::MySql),
    //     app_config.mongodb.clone().map(DbConfig::MongoDB),
    // ].into_iter().flatten().collect::<Vec<_>>();
    // db::multiple_db_config::init_all_sql_db(&db_configs).await?;
    // db::multiple_db_config::init_all_mongo_db(&db_configs).await?;
    // RENEMBER TO TEST CONNECTIONS FOR EACH DB TYPE IN THE MULTIPLE DB CONFIG MODULE IF YOU USE IT.
    // TODO, DONT USE IT YET, IT IS FOR TESTING PURPOSES ONLY AND NOT PRODUCTION READY, IT LACKS TEST CONNECTION LOGIC AND OTHER IMPORTANT FEATURES, USE WITH CAUTION.

    // PostgreSQL (required)
    let pg_pool = db::postgres::init_pool(&app_config.db_postgres)
        .await
        .expect("Failed to initialize PostgreSQL pool");
    tracing::info!("✅ PostgreSQL connected");

    // MySQL (optional)
    if let Some(mysql_config) = &app_config.db_mysql {
        match db::mysql::init_pool(mysql_config).await {
            Ok(mysql_pool) => {
                // Test connection
                if let Err(e) = db::mysql::test_connection(&mysql_pool).await {
                    tracing::warn!("⚠️  MySQL connection test failed: {}", e);
                } else {
                    tracing::info!("✅ MySQL connected");
                    // TODO: Store mysql_pool in AppState if needed
                }
            }
            Err(e) => {
                tracing::warn!("⚠️  MySQL skipped: {}", e);
            }
        }
    }

    // let mysql_pool_option = if let Some(mysql_config) = &app_config.db_mysql {
    //     match db::mysql::init_pool(mysql_config).await {
    //         Ok(mysql_pool) => {
    //             if let Err(e) = db::mysql::test_connection(&mysql_pool).await {
    //                 tracing::warn!("⚠️  MySQL connection test failed: {}", e);
    //                 None
    //             } else {
    //                 tracing::info!("✅ MySQL connected");
    //                 Some(mysql_pool)
    //             }
    //         }
    //         Err(e) => {
    //             tracing::warn!("⚠️  MySQL skipped: {}", e);
    //             None
    //         }
    //     }
    // } else {
    //     None
    // };

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
    validate_security_config(&app_config);

    // ============================================
    // Bootstrap Application (DI Container)
    // ============================================
    let app_state = AppState::new(app_config.clone(), pg_pool);

    // ============================================
    //  Start Background Workers
    // ============================================
    let worker = Arc::new(Worker::new(
        app_state.pool.clone(),
        app_state.test_item_service.clone(),
        // TODO add more services.
        // app_state.email_service.clone(),
        // app_state.payment_service.clone(),
    ));
    
    // Start worker pool (10 workers)
    worker.start(10);
    tracing::info!("✅ Background workers started (10 workers)");

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    tracing::info!("🌐 Listening on http://{}", address);
    tracing::info!("🔗 Documentation: http://{}:{}/api/docs", app_config.server.host, app_config.server.port);
    tracing::info!("");

    // ============================================
    // Start HTTP Server
    // ============================================
    HttpServer::new(move || {
        let app = App::new();
        
        // Register services
        // mysql_pool_option # if needed in AppState, pass it here
        let app = register_services!(
            app,
            app_state,
            config,
            pool,
            queue_manager,
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