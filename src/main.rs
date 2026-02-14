mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use tracing_subscriber;

use config::AppConfig;
use routes::config_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_config = AppConfig::from_env().expect("Failed to load config");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("template_project=debug".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting server on {}:{}", app_config.server.host, app_config.server.port);

    let pool = db::postgres::init_pool(&app_config.database)
        .await
        .expect("Failed to initialize database pool");

    tracing::info!("Database connected successfully");

    if let Some(mongo_config) = &app_config.mongodb {
        match db::mongo::init_mongodb(mongo_config).await {
            Ok(_) => tracing::info!("MongoDB connected successfully"),
            Err(e) => tracing::warn!("MongoDB connection failed: {}", e),
        }
    }

    let app_config_data = web::Data::new(app_config.clone());
    let pool_data = web::Data::new(pool);

    let address = format!("{}:{}", app_config.server.host, app_config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_config_data.clone())
            .app_data(pool_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .wrap(Logger::default())
            .configure(config_routes)
            .default_service(web::route().to(|| async {
                actix_web::HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Not Found",
                    "message": "The requested endpoint does not exist"
                }))
            }))
    })
    .bind(&address)?
    .run()
    .await
}