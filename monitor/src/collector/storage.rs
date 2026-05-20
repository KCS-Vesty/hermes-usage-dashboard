use crate::usage::{UsageRecord, RateLimitRecord};
use std::collections::VecDeque;
use tokio::sync::Mutex;

/// The interface for buffering metrics before they are written to persistent storage.
/// This is the seam: callers depend on `Storage`, not on the concrete queue implementation.
#[async_trait::async_trait]
pub trait Storage<T>: Send + Sync {
    async fn push(&self, item: T);
    async fn drain(&self) -> Vec<T>;
    async fn len(&self) -> usize;
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// In-memory adapter for `Storage`. Wraps a `VecDeque` behind a `Mutex`.
/// This is the test adapter and the default production adapter.
pub struct InMemoryStorage<T> {
    inner: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T> InMemoryStorage<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::with_capacity(1024)),
            capacity: 1024,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }
}

impl<T> Default for InMemoryStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<T: Send + Clone + 'static> Storage<T> for InMemoryStorage<T> {
    async fn push(&self, item: T) {
        let mut q = self.inner.lock().await;
        if q.len() == self.capacity {
            q.pop_front();
        }
        q.push_back(item);
    }

    async fn drain(&self) -> Vec<T> {
        let mut q = self.inner.lock().await;
        q.drain(..).collect()
    }

    async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }
}

// Re-export the old global-queue API for backward compatibility.
// These now delegate to the Storage trait via thread-local adapters.
pub mod legacy {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Arc;

    static USAGE_STORAGE: Lazy<Arc<InMemoryStorage<UsageRecord>>> =
        Lazy::new(|| Arc::new(InMemoryStorage::new()));
    static RATE_STORAGE: Lazy<Arc<InMemoryStorage<RateLimitRecord>>> =
        Lazy::new(|| Arc::new(InMemoryStorage::new()));

    pub async fn record_usage(rec: UsageRecord) {
        USAGE_STORAGE.push(rec).await;
    }

    pub async fn record_rate(rec: RateLimitRecord) {
        RATE_STORAGE.push(rec).await;
    }

    pub async fn drain_usage() -> Vec<UsageRecord> {
        USAGE_STORAGE.drain().await
    }

    pub async fn drain_rate() -> Vec<RateLimitRecord> {
        RATE_STORAGE.drain().await
    }

    /// Clear all usage records. Useful for test isolation.
    pub async fn clear_usage() {
        let mut q = USAGE_STORAGE.inner.lock().await;
        q.clear();
    }

    /// Clear all rate limit records. Useful for test isolation.
    pub async fn clear_rate() {
        let mut q = RATE_STORAGE.inner.lock().await;
        q.clear();
    }
}
