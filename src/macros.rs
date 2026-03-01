/// Macro para registrar múltiples servicios automáticamente
#[macro_export]
macro_rules! register_services {
    ($app:expr, $state:expr, $($field:ident),+ $(,)?) => {
        $app
        $(
            .app_data(web::Data::new($state.$field.clone()))
        )+
    };
}