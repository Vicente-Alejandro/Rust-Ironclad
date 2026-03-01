pub mod authentication;
pub mod controllers;
pub mod handlers;

// Re-export controllers
pub use controllers::{
    AuthController,
    UserController,
    TestItemController,
    HealthController,
};

pub use handlers::handle_not_found; // Default 404 handler