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
}
