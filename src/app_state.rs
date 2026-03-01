use std::sync::Arc;
use sqlx::PgPool;
// use actix_web::web;

use crate::config::AppConfig;
use crate::infrastructure::{PostgresUserRepository, PostgresTestItemRepository};
use crate::application::{AuthService, UserService, TestItemService};
use crate::interfaces::{UserRepository, TestItemRepository};

/// Global application state containing all services and dependencies
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: PgPool,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub test_item_service: Arc<TestItemService>,
}

impl AppState {
    /// Initialize all services and dependencies
    pub fn new(config: AppConfig, pg_pool: PgPool) -> Self {
        let config = Arc::new(config);
        
        // ============================================
        // Repositories
        // ============================================
        let user_repository: Arc<dyn UserRepository> =
            Arc::new(PostgresUserRepository::new(pg_pool.clone()));
        
        let test_item_repository: Arc<dyn TestItemRepository> =
            Arc::new(PostgresTestItemRepository::new(pg_pool.clone()));

        // ============================================
        // Services
        // ============================================
        let auth_service = Arc::new(AuthService::new(
            user_repository.clone(),
            config.clone(),
        ));

        let user_service = Arc::new(UserService::new(
            user_repository.clone(),
            config.clone(),
        ));

        let test_item_service = Arc::new(TestItemService::new(
            test_item_repository.clone(),
        ));

        // ============================================
        // Return AppState
        // ============================================
        Self {
            config,
            pool: pg_pool,
            auth_service,
            user_service,
            test_item_service,
        }
    }
}

// #[derive(Clone)]
// pub struct AppState {
//     pub config: Arc<AppConfig>,
//     pub pool: PgPool,
//     pub auth_service: Arc<AuthService>,
//     pub user_service: Arc<UserService>,
//     pub test_item_service: Arc<TestItemService>,
//     pub new_service: Arc<NewService>,  // 🆕 Add here
// }

// impl AppState {
//     pub fn new(config: AppConfig, pg_pool: PgPool) -> Self {
//         // ... existing code ...
        
//         // 🆕 Create new Service & Repository
//         let new_repository: Arc<dyn NewRepository> =
//             Arc::new(PostgresNewRepository::new(pg_pool.clone()));
//         let new_service = Arc::new(NewService::new(new_repository));

//         Self {
//             // ... existing fields ...
//             new_service,  // 🆕 Add to the constructor
//         }
//     }
// }

// let app = register_services!(
//     app,
//     app_state,
//     config,
//     pool,
//     auth_service,
//     user_service,
//     test_item_service,
//     new_service  // 🆕 Add here in main.rs
// );