use actix_web::web;
use crate::infrastructure::http::{AuthController, UserController};

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
                    .route("all/nosession", web::get().to(UserController::get_all_users_no_session))
                    .route("/{id}", web::get().to(UserController::get_user))
            ),
    );
}