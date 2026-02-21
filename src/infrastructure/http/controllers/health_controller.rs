use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use std::sync::{Mutex, OnceLock};
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind,
    RefreshKind, System,
};

use crate::errors::ApiResult;

// --- UTILITY FUNCTIONS ---

/// Helper to convert raw bytes into human-readable strings (KB, MB, GB, TB).
fn format_bytes(bytes: u64) -> String {
    const UNIT: u64 = 1024;
    if bytes < UNIT {
        return format!("{} B", bytes);
    }
    let exp = (bytes as f64).ln() / (UNIT as f64).ln();
    let pre = "KMGTPE"
        .chars()
        .nth(exp as usize - 1)
        .unwrap_or('?');
    format!("{:.2} {}B", (bytes as f64) / (UNIT as f64).powf(exp.floor()), pre)
}

/// Helper to format date into "Day-Month-Year Hour:Minute:Second"
fn format_timestamp_human(date: DateTime<Utc>) -> String {
    date.format("%d-%m-%Y %H:%M:%S UTC").to_string()
}

// --- DTOs (Data Transfer Objects) ---

#[derive(Serialize)]
pub struct UptimeResponse {
    pub started_at: String,
    pub uptime_seconds: u64,
    pub uptime_human: String,
    pub check_time: String,
    pub check_time_human: String, // NEW: Readable format
}

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

// --- SYSTEM INFO STRUCTURES ---

#[derive(Serialize)]
pub struct OsInfo {
    pub platform: String,
    pub distro: String,
    pub hostname: String,
    pub kernel_version: String,
    pub num_cpus: usize,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub total_human: String,
    pub used_bytes: u64,
    pub used_human: String,
    pub free_bytes: u64,
    pub free_human: String,
    pub used_percentage: f32,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub global_usage_percent: f32,
    pub brand: String,
    pub frequency_mhz: u64,
    pub temperature_celsius: Option<f32>,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_space_bytes: u64,
    pub total_space_human: String,
    pub available_space_bytes: u64,
    pub available_space_human: String,
    pub used_percentage: f32,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub total_rx_bytes: u64,
    pub total_rx_human: String,
    pub total_tx_bytes: u64,
    pub total_tx_human: String,
}

#[derive(Serialize)]
pub struct AppTelemetry {
    pub app_memory_usage_bytes: u64,
    pub app_memory_human: String,
    pub app_cpu_usage_percent: f32,
    pub app_uptime_seconds: u64,
}

#[derive(Serialize)]
pub struct DatabaseTelemetry {
    pub status: String,
    pub database_name: String,
    pub size_bytes: i64,
    pub size_human: String,
    pub engine_version: String,
    pub server_address: Option<String>,
    pub data_directory: Option<String>, // NEW: Physical Path on Disk
}

#[derive(Serialize)]
pub struct FullSystemReport {
    pub host: String,
    pub check_time: String,
    pub check_time_human: String, // NEW: Readable format
    pub os_info: OsInfo,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disks_info: Vec<DiskInfo>,
    pub network_info: NetworkInfo,
    pub application: AppTelemetry,
    pub database: DatabaseTelemetry,
}

// --- GLOBAL STATE ---

static APP_START_TIME: OnceLock<DateTime<Utc>> = OnceLock::new();
static SYSTEM_MONITOR: OnceLock<Mutex<System>> = OnceLock::new();

// --- CONTROLLER IMPLEMENTATION ---

pub struct HealthController;

impl HealthController {
    
    /// Endpoint 1: Professional Uptime
    pub async fn uptime() -> ApiResult<HttpResponse> {
        let start_time = *APP_START_TIME.get_or_init(Utc::now);
        let current_time = Utc::now();
        let duration = current_time.signed_duration_since(start_time);
        
        let response = UptimeResponse {
            started_at: start_time.to_rfc3339(),
            uptime_seconds: duration.num_seconds() as u64,
            uptime_human: Self::format_duration(duration),
            check_time: current_time.to_rfc3339(),
            check_time_human: format_timestamp_human(current_time),
        };

        Ok(HttpResponse::Ok().json(response))
    }

    /// Endpoint 2: Full System Info (Real-time + DB + Human Readable)
    pub async fn system_info(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
        
        // 1. Trigger Async DB Check immediately
        let db_future = Self::get_database_details(&pool);

        // 2. Initialize or Get System Monitor
        let mutex = SYSTEM_MONITOR.get_or_init(|| {
            // Note: In sysinfo 0.30+, specific components are decoupled.
            // System only manages CPU, Memory and Processes here.
            let mut sys = System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything())
                    .with_processes(ProcessRefreshKind::everything())
            );
            // Wait for CPU baseline
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            sys.refresh_all(); 
            Mutex::new(sys)
        });

        // 3. Lock and Refresh Data
        let mut sys = mutex.lock().map_err(|_| {
            crate::errors::ApiError::InternalServerError("System monitor lock failed".to_string())
        })?;

        // Refresh only what System struct controls
        sys.refresh_cpu();
        sys.refresh_memory();
        // sys.refresh_networks() & sys.refresh_components() REMOVED (Handled separately below)
        
        // Refresh specific process (Our App)
        let current_pid = Pid::from_u32(std::process::id());
        sys.refresh_process(current_pid);

        // --- GATHER DATA ---

        // A. CPU (Stats + Temp)
        let global_cpu = sys.global_cpu_info();
        
        // Instantiate Components to find temperature sensors
        let components = Components::new_with_refreshed_list();
        let cpu_temp = components.iter()
            .find(|c| c.label().to_lowercase().contains("cpu") || c.label().to_lowercase().contains("core"))
            .map(|c| c.temperature());

        let cpu_info = CpuInfo {
            global_usage_percent: global_cpu.cpu_usage(),
            brand: global_cpu.brand().to_string(),
            frequency_mhz: global_cpu.frequency(),
            temperature_celsius: cpu_temp,
        };

        // B. Memory
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let memory_info = MemoryInfo {
            total_bytes: total_mem,
            total_human: format_bytes(total_mem),
            used_bytes: used_mem,
            used_human: format_bytes(used_mem),
            free_bytes: sys.free_memory(),
            free_human: format_bytes(sys.free_memory()),
            used_percentage: if total_mem > 0 { (used_mem as f32 / total_mem as f32) * 100.0 } else { 0.0 },
        };

        // C. OS
        let os_info = OsInfo {
            platform: System::name().unwrap_or_else(|| "Unknown".to_string()),
            distro: System::long_os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "localhost".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            num_cpus: sys.cpus().len(),
        };

        // D. Disks (Iterate ALL disks)
        // Instantiate Disks to get storage info
        let disks_list = Disks::new_with_refreshed_list();
        let disks_info: Vec<DiskInfo> = disks_list.iter().map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_space_bytes: total,
                total_space_human: format_bytes(total),
                available_space_bytes: available,
                available_space_human: format_bytes(available),
                used_percentage: percent,
            }
        }).collect();

        // E. Network
        // Instantiate Networks to get traffic stats
        let networks = Networks::new_with_refreshed_list();
        let (rx, tx) = networks.iter().fold((0, 0), |acc, (_, data)| {
            (acc.0 + data.total_received(), acc.1 + data.total_transmitted())
        });
        let network_info = NetworkInfo {
            total_rx_bytes: rx,
            total_rx_human: format_bytes(rx),
            total_tx_bytes: tx,
            total_tx_human: format_bytes(tx),
        };

        // F. Application Telemetry
        let app_telemetry = if let Some(process) = sys.process(current_pid) {
            AppTelemetry {
                app_memory_usage_bytes: process.memory(),
                app_memory_human: format_bytes(process.memory()),
                app_cpu_usage_percent: process.cpu_usage(),
                app_uptime_seconds: process.run_time(),
            }
        } else {
            AppTelemetry { 
                app_memory_usage_bytes: 0, 
                app_memory_human: "0 B".to_string(), 
                app_cpu_usage_percent: 0.0, 
                app_uptime_seconds: 0 
            }
        };

        // G. Database (Await result)
        let database_telemetry = db_future.await;

        // Final Report
        let current_time = Utc::now();
        let report = FullSystemReport {
            host: System::host_name().unwrap_or_default(),
            check_time: current_time.to_rfc3339(),
            check_time_human: format_timestamp_human(current_time),
            os_info,
            cpu_info,
            memory_info,
            disks_info,
            network_info,
            application: app_telemetry,
            database: database_telemetry,
        };

        Ok(HttpResponse::Ok().json(report))
    }

    /// Endpoint 3: Health Check (Readiness)
    pub async fn health_check(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
        let db_check = Self::check_database_simple(&pool).await;
        let components = vec![db_check];
        
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

        match response_body.status {
            HealthStatus::Up => Ok(HttpResponse::Ok().json(response_body)),
            _ => Ok(HttpResponse::ServiceUnavailable().json(response_body)),
        }
    }

    /// Endpoint 4: Dashboard HTML
    pub async fn system_dashboard() -> ApiResult<HttpResponse> {
        let html = include_str!("../../../../templates/system/dashboard.html");
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html))
    }

    /// Endpoint 5: System Info JSON (Same as system_info but only JSON, no HTML)
    pub async fn system_info_json(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
        // Reutilizar la lógica de system_info pero solo devolver JSON
        Self::system_info(pool).await
    }

    // --- PRIVATE HELPERS ---

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

    /// Simple ping for Readiness Probe
    async fn check_database_simple(pool: &PgPool) -> ComponentHealth {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => ComponentHealth {
                name: "postgres_primary".to_string(),
                status: HealthStatus::Up,
                error: None,
            },
            Err(e) => ComponentHealth {
                name: "postgres_primary".to_string(),
                status: HealthStatus::Down,
                error: Some(e.to_string()),
            },
        }
    }

    /// Detailed DB Stats for System Info
    async fn get_database_details(pool: &PgPool) -> DatabaseTelemetry {
        #[derive(FromRow)]
        struct DbRow {
            current_db: String,
            size: i64,
            version: String,
            server_addr: Option<String>,
            data_dir: Option<String>, // Captures 'data_directory' setting
        }

        // Query to get size, version, server IP, and physical data directory
        // We use current_setting('data_directory') to find the disk path.
        let query = r#"
            SELECT 
                current_database() as current_db,
                pg_database_size(current_database()) as size,
                version() as version,
                inet_server_addr()::text as server_addr,
                current_setting('data_directory') as data_dir
        "#;

        match sqlx::query_as::<_, DbRow>(query).fetch_one(pool).await {
            Ok(row) => DatabaseTelemetry {
                status: "Connected".to_string(),
                database_name: row.current_db,
                size_bytes: row.size,
                size_human: format_bytes(row.size as u64),
                engine_version: row.version,
                server_address: row.server_addr,
                data_directory: row.data_dir,
            },
            Err(e) => DatabaseTelemetry {
                status: "Error".to_string(),
                database_name: "Unknown".to_string(),
                size_bytes: 0,
                size_human: "0 B".to_string(),
                engine_version: format!("Check failed: {}", e),
                server_address: None,
                data_directory: None,
            },
        }
    }
}