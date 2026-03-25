use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

pub struct AlertService;

impl AlertService {
    pub async fn send(alert: Alert) {
        match alert.level {
            AlertLevel::Info => tracing::info!(?alert, "INFO ALERT"),
            AlertLevel::Warning => tracing::warn!(?alert, "WARNING ALERT"),
            AlertLevel::Critical => tracing::error!(?alert, "CRITICAL ALERT"),
        }
        // Here you could also add logic to send alerts to external systems (e.g., email, Slack, etc.)}
    }
}