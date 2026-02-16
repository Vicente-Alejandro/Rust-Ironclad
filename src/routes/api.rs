use actix_web::web;
use crate::infrastructure::http::{AuthController, UserController, TestItemController};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
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
            ),
            
    );
}