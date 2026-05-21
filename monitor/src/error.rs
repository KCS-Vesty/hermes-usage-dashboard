#[cfg(feature = "types")]
use thiserror::Error;

#[cfg_attr(feature = "types", derive(Debug, Error))]
pub enum DashboardError {
    #[cfg_attr(feature = "types", error("InfluxDB write failed: {0}"))]
    WriteFailed(String),

    #[cfg_attr(feature = "types", error("InfluxDB query failed: {0}"))]
    QueryFailed(String),

    #[cfg_attr(feature = "types", error("Configuration error: {0}"))]
    Config(String),

    #[cfg_attr(feature = "types", error("Storage error: {0}"))]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, DashboardError>;
