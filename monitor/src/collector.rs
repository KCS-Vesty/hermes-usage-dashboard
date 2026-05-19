pub mod storage;

// Re-export the legacy global-queue API so existing callers (db.rs, tests) keep working.
// These are thin wrappers around the new Storage trait.
pub use storage::legacy::{drain_rate, drain_usage, record_rate, record_usage};

#[cfg(test)]
mod storage_tests;
