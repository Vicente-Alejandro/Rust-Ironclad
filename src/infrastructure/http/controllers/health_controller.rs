use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::time::Duration;
use sqlx::PgPool; 
use crate::errors::ApiResult;
use std::sync::{Mutex, OnceLock};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System, Disks};


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

// --- DTOs (Data Transfer Objects) ---

#[derive(Serialize)]
pub struct OsInfo {
    pub platform: String,     // e.g., "Linux", "macOS"
    pub distro: String,       // e.g., "Ubuntu 22.04"
    pub hostname: String,
    pub kernel_version: String,
    pub num_cpus: usize,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub used_percentage: f32,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub global_usage_percent: f32,
    pub brand: String,
    pub frequency_mhz: u64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub total_space_bytes: u64,
    pub available_space_bytes: u64,
    pub mount_point: String,
}

#[derive(Serialize)]
pub struct SystemInfoResponse {
    pub os_info: OsInfo,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disks_info: Vec<DiskInfo>,
}

// --- SHARED STATE ---

/// Global instance of the System monitor.
/// We use a Mutex because `sysinfo::System` is mutable (it needs to update its internal counters).
/// We use OnceLock to initialize it lazily and safely.
static SYSTEM_MONITOR: OnceLock<Mutex<System>> = OnceLock::new();

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

    /// Professional System Info Endpoint.
    /// Returns real-time hardware telemetry (CPU, RAM, Disk).
    pub async fn system_info() -> ApiResult<HttpResponse> {
        // 1. Get access to the global system monitor
        let mutex = SYSTEM_MONITOR.get_or_init(|| {
            // Initialize with specific refresh configuration to optimize performance.
            // We don't want to refresh processes or networks on every call, just CPU and RAM.
            let mut sys = System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything())
            );
            // First refresh to establish a baseline for CPU calculation
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            sys.refresh_cpu();
            Mutex::new(sys)
        });

        // 2. Lock the mutex to update the stats safely
        let mut sys = mutex.lock().map_err(|_| {
            // In a real scenario, handle poisoned mutex gracefully
            crate::errors::ApiError::InternalServerError("System monitor lock failed".to_string())
        })?;

        // 3. Refresh vital components
        // Only refresh what we need to keep latency low.
        sys.refresh_cpu(); 
        sys.refresh_memory();
        // Disks are usually refreshed less frequently or on a new instance, 
        // but for this example, we can use the `Disks` struct separately.
        
        // 4. Construct CPU Data
        let global_cpu = sys.global_cpu_info();
        let cpu_info = CpuInfo {
            global_usage_percent: global_cpu.cpu_usage(),
            brand: global_cpu.brand().to_string(),
            frequency_mhz: global_cpu.frequency(),
        };

        // 5. Construct Memory Data
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let memory_info = MemoryInfo {
            total_bytes: total_mem,
            used_bytes: used_mem,
            free_bytes: sys.free_memory(),
            // Avoid division by zero
            used_percentage: if total_mem > 0 {
                (used_mem as f32 / total_mem as f32) * 100.0
            } else {
                0.0
            },
        };

        // 6. Construct OS Data (Static mostly)
        let os_info = OsInfo {
            platform: System::name().unwrap_or_else(|| "Unknown".to_string()),
            distro: System::long_os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "localhost".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            num_cpus: sys.cpus().len(),
        };

        // 7. Construct Disk Data
        // Disks operations can be heavy, usually, we construct a new list
        let disks_list = Disks::new_with_refreshed_list();
        let disks_info: Vec<DiskInfo> = disks_list.iter().map(|disk| {
            DiskInfo {
                total_space_bytes: disk.total_space(),
                available_space_bytes: disk.available_space(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
            }
        }).collect();

        // 8. Final Response
        let response = SystemInfoResponse {
            os_info,
            cpu_info,
            memory_info,
            disks_info,
        };

        Ok(HttpResponse::Ok().json(response))
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