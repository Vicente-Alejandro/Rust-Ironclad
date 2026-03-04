/// Route configuration modules
pub mod api;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    api::configure(cfg);
}