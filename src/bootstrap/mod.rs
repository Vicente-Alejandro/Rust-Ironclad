//! Bootstrap module - Application initialization and setup
//!
//! This module contains core components for bootstrapping the application:
//! - AppState: Dependency injection container
//! - Macros: Framework-level macros
//! - Providers: Service providers (future)

mod app_state;
mod macros;

pub use app_state::AppState;
pub use macros::*;

// Future: Service providers
// mod providers;
// pub use providers::*;