use actix_web::web;
use actix_files::Files; 
use crate::infrastructure::http::{AuthController, UserController, TestItemController, HealthController};

pub fn configure(cfg: &mut web::ServiceConfig) {
    // 1. Configure the static files route (CSS, JS, Images)
    // The first argument "/static" is the URL in the browser.
    // The second argument "./static" is the physical folder on your disk.
    cfg.service(Files::new("/static", "./static"));

    // 2. API routes configuration 
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/docs")
                    .route("", web::get().to(|| async {
                        actix_web::HttpResponse::Ok().json(serde_json::json!({
                            "message": "API Documentation is not implemented yet. Please check back later.",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "hint": "In the future, this endpoint will provide interactive API documentation using Swagger or similar tools."
                        }))
                    }))
            )
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(AuthController::register))
                    .route("/login", web::post().to(AuthController::login))
                    .route("/verify-admin", web::get().to(AuthController::verify_admin))
            )
            .service(
                web::scope("/user")
                    .route("/profile", web::get().to(UserController::get_profile))
                    .route("/all", web::get().to(UserController::get_all_users))
                    .route("/{id}", web::get().to(UserController::get_user))
            )
            .service(
                web::scope("/noauth")
                    .route("/users", web::get().to(UserController::get_all_users_no_session))
            )
            .service(
                web::scope("/test-items")
                    .route("", web::post().to(TestItemController::create))
                    .route("", web::get().to(TestItemController::get_all))
                    .route("/{id}", web::get().to(TestItemController::get_by_id))
                    .route("/{id}", web::put().to(TestItemController::update))
                    .route("/{id}", web::delete().to(TestItemController::delete))
            )
            .service(
                web::scope("/administration")
                    .route("/health", web::get().to(HealthController::health_check))
                    .route("/uptime", web::get().to(HealthController::uptime))
                    .route("/system", web::get().to(HealthController::system_dashboard)) 
                    .route("/system-json", web::get().to(HealthController::system_info_json))  
            )
    );
}