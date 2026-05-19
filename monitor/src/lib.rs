pub mod usage;
pub mod collector;
pub mod db;
pub mod error;

// Re-export the domain types and error types at the crate root
// so downstream crates (tauri-backend, ui) import from one place.
pub use error::{DashboardError, Result};
pub use usage::{RateLimitRecord, UsageRecord};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod error_tests;
