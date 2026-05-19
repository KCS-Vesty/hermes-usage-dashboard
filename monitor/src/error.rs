use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("InfluxDB write failed: {0}")]
    WriteFailed(String),

    #[error("InfluxDB query failed: {0}")]
    QueryFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, DashboardError>;
