use crate::monitoring::alerts::{Alert, AlertLevel};

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