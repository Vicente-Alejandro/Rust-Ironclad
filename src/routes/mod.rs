use crate::handlers::{auth, users};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/verify-admin", web::get().to(auth::verify_admin))
            )
            .service(
                web::scope("/user")
                    .route("/update", web::put().to(users::update_profile))
                    .route("/delete", web::delete().to(users::delete_profile))
            ),
    );
}
