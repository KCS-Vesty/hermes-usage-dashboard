#[cfg(test)]
mod storage_tests {
    use crate::usage::UsageRecord;
    use crate::collector::storage::{InMemoryStorage, Storage};

    fn make_record(provider: &str, tokens: i64) -> UsageRecord {
        UsageRecord {
            provider: provider.to_string(),
            model: None,
            tokens_used: tokens,
            cost_usd: 0.001,
            ts: 1_700_000_000,
        }
    }

    #[tokio::test]
    async fn test_inmemory_storage_push_and_drain() {
        let storage: InMemoryStorage<UsageRecord> = InMemoryStorage::new();
        assert_eq!(storage.len().await, 0);

        storage.push(make_record("openrouter", 1000)).await;
        assert_eq!(storage.len().await, 1);

        storage.push(make_record("anthropic", 500)).await;
        assert_eq!(storage.len().await, 2);

        let drained = storage.drain().await;
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].provider, "openrouter");
        assert_eq!(drained[1].provider, "anthropic");
        assert_eq!(storage.len().await, 0);
    }

    #[tokio::test]
    async fn test_inmemory_storage_drain_empty() {
        let storage: InMemoryStorage<UsageRecord> = InMemoryStorage::new();
        let drained = storage.drain().await;
        assert!(drained.is_empty());
    }

    #[tokio::test]
    async fn test_inmemory_storage_capacity_eviction() {
        let storage: InMemoryStorage<UsageRecord> = InMemoryStorage::with_capacity(4);
        for i in 0..4 {
            storage.push(make_record(&format!("p-{}", i), 100)).await;
        }
        assert_eq!(storage.len().await, 4);

        // Pushing beyond capacity should evict the oldest
        storage.push(make_record("overflow", 999)).await;
        assert_eq!(storage.len().await, 4);

        let drained = storage.drain().await;
        assert!(!drained.iter().any(|r| r.provider == "p-0"));
        assert!(drained.iter().any(|r| r.provider == "overflow"));
    }

    // --- Legacy API tests ---

    #[tokio::test]
    async fn test_legacy_record_and_drain_usage() {
        // Clear any leftover state from other tests
        crate::collector::storage::legacy::clear_usage().await;

        crate::collector::record_usage(make_record("openrouter", 1000)).await;
        crate::collector::record_usage(make_record("anthropic", 500)).await;

        let drained = crate::collector::drain_usage().await;
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].provider, "openrouter");
        assert_eq!(drained[1].provider, "anthropic");

        // After drain, the queue should be empty
        let drained_again = crate::collector::drain_usage().await;
        assert!(drained_again.is_empty());
    }

    #[tokio::test]
    async fn test_legacy_record_and_drain_rate() {
        crate::collector::storage::legacy::clear_rate().await;

        let rate = crate::usage::RateLimitRecord {
            provider: "openrouter".to_string(),
            model: Some("claude-3".to_string()),
            hard_limit: 10000,
            soft_limit: Some(5000),
            remaining: 7500,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        };
        crate::collector::record_rate(rate.clone()).await;

        let drained = crate::collector::drain_rate().await;
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].provider, "openrouter");
        assert_eq!(drained[0].hard_limit, 10000);
    }
}
