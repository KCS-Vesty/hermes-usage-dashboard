#[cfg(feature = "server")]
pub mod storage;

#[cfg(feature = "server")]
// Re-export the legacy global-queue API so existing callers (db.rs, tests) keep working.
// These are thin wrappers around the new Storage trait.
pub use storage::legacy::{drain_rate, drain_usage, record_rate, record_usage};

#[cfg(all(test, feature = "server"))]
mod storage_tests;
