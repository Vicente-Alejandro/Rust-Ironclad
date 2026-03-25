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