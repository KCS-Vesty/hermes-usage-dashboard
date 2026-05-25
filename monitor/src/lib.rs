pub mod usage;
#[cfg(feature = "types")]
pub mod error;

// Re-export the domain types at the crate root
pub use usage::{RateLimitRecord, UsageRecord};
#[cfg(feature = "types")]
pub use usage::{ProviderUsage, UsageSummary, build_usage_summary, build_usage_summary_from_providers};
#[cfg(feature = "types")]
pub use error::{DashboardError, Result};

#[cfg(feature = "server")]
pub mod collector;
#[cfg(feature = "server")]
pub mod db;

#[cfg(feature = "server")]
pub use db::{InfluxDbConfig, start_writer, write_usage_record};

#[cfg(all(test, feature = "server"))]
mod tests;
#[cfg(all(test, feature = "server"))]
mod error_tests;
