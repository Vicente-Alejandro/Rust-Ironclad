//! Framework-level macros for common patterns

/// Macro to register multiple services with Actix App
///
/// Simplifies registering all AppState fields as web::Data in the Actix App.
///
/// # Example
/// ```rust
/// let app = App::new();
/// let app = register_services!(
///     app,
///     app_state,
///     config,
///     pool,
///     auth_service,
///     user_service,
///     test_item_service
/// );
/// ```
///
/// # Equivalent to
/// ```rust
/// app.app_data(web::Data::new(app_state.config.clone()))
///    .app_data(web::Data::new(app_state.pool.clone()))
///    .app_data(web::Data::new(app_state.auth_service.clone()))
///    .app_data(web::Data::new(app_state.user_service.clone()))
///    .app_data(web::Data::new(app_state.test_item_service.clone()))
/// ```
#[macro_export]
macro_rules! register_services {
    ($app:expr, $state:expr, $($field:ident),+ $(,)?) => {
        $app
        $(
            .app_data(web::Data::new($state.$field.clone()))
        )+
    };
}