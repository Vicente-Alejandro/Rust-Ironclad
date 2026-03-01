use actix_web::HttpResponse;
use chrono::Utc;

/// Default 404 handler for routes that don't exist
pub async fn handle_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Not Found",
        "message": "The requested endpoint does not exist",
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

/// Custom 404 with additional context
pub async fn handle_not_found_with_hint() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Not Found",
        "message": "The requested endpoint does not exist",
        "timestamp": Utc::now().to_rfc3339(),
        "hint": "Check the API documentation at /api/docs",
    }))
}