/// Route configuration modules
pub mod api;
use actix_web::web;

/// Configure all application routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    api::configure(cfg);
}
