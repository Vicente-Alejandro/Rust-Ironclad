use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;
use sqlx::PgPool; // Important for the pool data type
use crate::errors::ApiResult;

// --- STRUCTURES ---

/// DTO for the Uptime response.
/// Using a struct ensures the response is documentable (OpenAPI)
/// and consistent, instead of a loose "JSON".
#[derive(Serialize)]
pub struct UptimeResponse {
    /// Exact time the service started (ISO 8601)
    pub started_at: String,
    /// Total execution time in seconds (useful for metrics/alerts)
    pub uptime_seconds: u64,
    /// Human-readable representation (e.g., "2 days, 4 hours, 12 minutes")
    pub uptime_human: String,
    /// Current server timestamp (ISO 8601) for latency reference
    pub check_time: String,
}

// --- GLOBAL STATE ---

/// Stores the application start time.
/// 'OnceLock' is the modern and thread-safe way in Rust (std) to initialize statics.
static APP_START_TIME: OnceLock<DateTime<Utc>> = OnceLock::new();

// --- DATA STRUCTURES ---

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Up,
    Down,
    Degraded,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub timestamp: String,
    pub components: Vec<ComponentHealth>,
}

pub struct HealthController;

impl HealthController {
    
    /// Professional Uptime Endpoint.
    /// Initializes the reference time on the first call if it doesn't exist,
    /// and calculates the difference against the current UTC time.
    pub async fn uptime() -> ApiResult<HttpResponse> {
        // 1. Get or initialize the start time.
        // In a real environment, this is ideally initialized in 'main.rs',
        // but here we use 'get_or_init' to ensure it works autonomously.
        let start_time = *APP_START_TIME.get_or_init(Utc::now);
        let current_time = Utc::now();

        // 2. Calculate duration
        // We use signed_duration_since for safety, although it will always be positive here.
        let duration = current_time.signed_duration_since(start_time);
        
        // 3. Format the response
        let response = UptimeResponse {
            started_at: start_time.to_rfc3339(),
            uptime_seconds: duration.num_seconds() as u64,
            uptime_human: Self::format_duration(duration),
            check_time: current_time.to_rfc3339(),
        };

        Ok(HttpResponse::Ok().json(response))
    }

    /// Private helper to format the duration in a readable way.
    /// Splits seconds into days, hours, minutes, and seconds.
    fn format_duration(duration: chrono::Duration) -> String {
        let total_seconds = duration.num_seconds();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{} days, {} hours, {} minutes, {} seconds", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
        } else {
            format!("{} minutes, {} seconds", minutes, seconds)
        }
    }

    // TODO: Add a more comprehensive system info endpoint in the future that includes CPU, memory, disk usage, etc.
    pub async fn system_info() -> ApiResult<HttpResponse> {
        // Placeholder for actual system info logic
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "os": "Linux",
            "version": "1.0.0",
            "status": "operational"
        })))
    }

    /// Health Check - Readiness Probe
    /// Receives the connection pool injected by Actix (web::Data<PgPool>)
    pub async fn health_check(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
        
        // 1. Real Database Verification
        // We pass a reference to the pool to avoid consuming it
        let db_check = Self::check_database(&pool).await;

        // Here you would add other checks in the future (Redis, external API, etc.)
        let components = vec![db_check];

        // 2. Determine global status
        let global_status = if components.iter().any(|c| matches!(c.status, HealthStatus::Down)) {
            HealthStatus::Down
        } else {
            HealthStatus::Up
        };

        let response_body = HealthResponse {
            status: global_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now().to_rfc3339(),
            components,
        };

        // 3. HTTP Response
        match response_body.status {
            HealthStatus::Up => Ok(HttpResponse::Ok().json(response_body)),
            // 503 indicates to the Load Balancer not to send traffic because the DB is down
            _ => Ok(HttpResponse::ServiceUnavailable().json(response_body)),
        }
    }

    /// Performs a real ping to the database using "SELECT 1"
    /// This is the industry standard practice for verifying SQL connectivity.
    async fn check_database(pool: &PgPool) -> ComponentHealth {
        // We execute a trivial query. If this passes, authentication and network are working.
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => ComponentHealth {
                name: "postgres_primary".to_string(),
                status: HealthStatus::Up,
                error: None,
            },
            Err(e) => ComponentHealth {
                name: "postgres_primary".to_string(),
                status: HealthStatus::Down,
                error: Some(e.to_string()), // Captures the actual error (timeout, auth failure, etc.)
            },
        }
    }
}