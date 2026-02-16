pub mod http;
pub mod persistence;

// Persistence layer
pub use persistence::PostgresUserRepository;
pub use persistence::PostgresTestItemRepository;

// HTTP layer - Authentication
pub use http::authentication::{
    AuthUser,
    AdminUser,
    ModeratorUser,
    PremiumUser,
    RoleUser,
};

// HTTP layer - Controllers
pub use http::controllers::{
    AuthController,
    UserController,
    TestItemController,
    HealthController,
};