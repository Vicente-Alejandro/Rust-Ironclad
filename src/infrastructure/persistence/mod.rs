pub mod postgres;

pub use postgres::PostgresUserRepository;
pub use postgres::PostgresTestItemRepository;

// TODO - Add Redis repositories for caching (e.g., UserCacheRepository)
// TODO - ADJUST MULTIPLE DATABASE SUPPORT (e.g., MySQL, SQLite) if needed in the future