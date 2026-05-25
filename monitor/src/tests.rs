#[cfg(test)]
mod tests {
    use crate::usage::{UsageRecord, RateLimitRecord};

    fn make_usage_record(provider: &str, tokens: i64, cost: f64) -> UsageRecord {
        UsageRecord {
            provider: provider.to_string(),
            model: Some("test-model".to_string()),
            tokens_used: tokens,
            cost_usd: cost,
            ts: 1_700_000_000,
        }
    }

    fn make_rate_record(provider: &str, hard: i64, remaining: i64) -> RateLimitRecord {
        RateLimitRecord {
            provider: provider.to_string(),
            model: Some("test-model".to_string()),
            hard_limit: hard,
            soft_limit: Some(hard / 2),
            remaining,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        }
    }

    // --- UsageRecord tests ---

    #[test]
    fn test_usage_record_creation() {
        let rec = make_usage_record("openrouter", 1000, 0.002);
        assert_eq!(rec.provider, "openrouter");
        assert_eq!(rec.model, Some("test-model".to_string()));
        assert_eq!(rec.tokens_used, 1000);
        assert!((rec.cost_usd - 0.002).abs() < f64::EPSILON);
        assert_eq!(rec.ts, 1_700_000_000);
    }

    #[test]
    fn test_usage_record_with_no_model() {
        let rec = UsageRecord {
            provider: "anthropic".to_string(),
            model: None,
            tokens_used: 500,
            cost_usd: 0.015,
            ts: 1_700_000_000,
        };
        assert_eq!(rec.model, None);
        assert_eq!(rec.provider, "anthropic");
    }

    #[test]
    fn test_usage_record_clone() {
        let rec = make_usage_record("openai", 2000, 0.04);
        let cloned = rec.clone();
        assert_eq!(rec.provider, cloned.provider);
        assert_eq!(rec.tokens_used, cloned.tokens_used);
    }

    #[test]
    fn test_usage_record_debug() {
        let rec = make_usage_record("openrouter", 1000, 0.002);
        let debug = format!("{:?}", rec);
        assert!(debug.contains("openrouter"));
    }

    #[test]
    fn test_usage_record_serialization() {
        let rec = make_usage_record("openrouter", 1000, 0.002);
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("\"provider\":\"openrouter\""));
        assert!(json.contains("\"tokens_used\":1000"));
    }

    #[test]
    fn test_usage_record_deserialization() {
        let json = r#"{"provider":"anthropic","model":"claude-3","tokens_used":500,"cost_usd":0.015,"ts":1700000000}"#;
        let rec: UsageRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.provider, "anthropic");
        assert_eq!(rec.model, Some("claude-3".to_string()));
        assert_eq!(rec.tokens_used, 500);
    }

    #[test]
    fn test_usage_record_roundtrip() {
        let original = make_usage_record("openrouter", 1500, 0.003);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: UsageRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(original.provider, deserialized.provider);
        assert_eq!(original.tokens_used, deserialized.tokens_used);
        assert!((original.cost_usd - deserialized.cost_usd).abs() < f64::EPSILON);
    }

    // --- RateLimitRecord tests ---

    #[test]
    fn test_rate_limit_record_creation() {
        let rec = make_rate_record("openrouter", 10000, 7500);
        assert_eq!(rec.provider, "openrouter");
        assert_eq!(rec.hard_limit, 10000);
        assert_eq!(rec.soft_limit, Some(5000));
        assert_eq!(rec.remaining, 7500);
    }

    #[test]
    fn test_rate_limit_record_no_soft_limit() {
        let rec = RateLimitRecord {
            provider: "openai".to_string(),
            model: None,
            hard_limit: 5000,
            soft_limit: None,
            remaining: 3000,
            reset_ts: 1_700_000_100,
            ts: 1_700_000_000,
        };
        assert_eq!(rec.soft_limit, None);
    }

    #[test]
    fn test_rate_limit_record_clone() {
        let rec = make_rate_record("anthropic", 20000, 15000);
        let cloned = rec.clone();
        assert_eq!(rec.provider, cloned.provider);
        assert_eq!(rec.hard_limit, cloned.hard_limit);
        assert_eq!(rec.remaining, cloned.remaining);
    }

    #[test]
    fn test_rate_limit_record_serialization() {
        let rec = make_rate_record("anthropic", 20000, 15000);
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("\"provider\":\"anthropic\""));
        assert!(json.contains("\"hard_limit\":20000"));
        assert!(json.contains("\"remaining\":15000"));
    }

    #[test]
    fn test_rate_limit_record_deserialization() {
        let json = r#"{"provider":"openai","model":null,"hard_limit":5000,"soft_limit":2500,"remaining":3000,"reset_ts":1700000100,"ts":1700000000}"#;
        let rec: RateLimitRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.provider, "openai");
        assert_eq!(rec.model, None);
        assert_eq!(rec.hard_limit, 5000);
        assert_eq!(rec.soft_limit, Some(2500));
    }

    #[test]
    fn test_rate_limit_record_roundtrip() {
        let original = make_rate_record("openrouter", 10000, 7500);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RateLimitRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(original.provider, deserialized.provider);
        assert_eq!(original.hard_limit, deserialized.hard_limit);
        assert_eq!(original.remaining, deserialized.remaining);
    }

    // --- Collector module tests (pure logic) ---

    #[test]
    fn test_usage_record_zero_tokens() {
        let rec = UsageRecord {
            provider: "test".to_string(),
            model: None,
            tokens_used: 0,
            cost_usd: 0.0,
            ts: 0,
        };
        assert_eq!(rec.tokens_used, 0);
        assert!((rec.cost_usd).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rate_limit_record_zero_remaining() {
        let rec = RateLimitRecord {
            provider: "test".to_string(),
            model: None,
            hard_limit: 100,
            soft_limit: Some(50),
            remaining: 0,
            reset_ts: 0,
            ts: 0,
        };
        assert_eq!(rec.remaining, 0);
    }

    // --- build_usage_summary_from_providers tests ---

    #[cfg(feature = "types")]
    #[test]
    fn test_build_summary_from_providers() {
        use crate::usage::{ProviderUsage, build_usage_summary_from_providers};
        let providers = vec![
            ProviderUsage { name: "a".into(), tokens_used: 1000, cost_usd: 0.01 },
            ProviderUsage { name: "b".into(), tokens_used: 2000, cost_usd: 0.02 },
        ];
        let summary = build_usage_summary_from_providers(providers);
        assert_eq!(summary.total_tokens, 3000);
        assert!((summary.total_cost_usd - 0.03).abs() < f64::EPSILON);
        assert_eq!(summary.providers.len(), 2);
    }

    #[cfg(feature = "types")]
    #[test]
    fn test_build_summary_from_empty() {
        use crate::usage::build_usage_summary_from_providers;
        let summary = build_usage_summary_from_providers(vec![]);
        assert_eq!(summary.total_tokens, 0);
        assert!((summary.total_cost_usd).abs() < f64::EPSILON);
        assert_eq!(summary.providers.len(), 0);
    }

    // --- Crate-level re-export tests ---
    // These test that the `pub use` statements in `lib.rs` are working.

    #[cfg(feature = "types")]
    #[test]
    fn test_build_summary_via_crate_reexport() {
        // Must be callable via `crate::build_usage_summary_from_providers`
        // thanks to `lib.rs` re-export
        let providers = vec![
            crate::usage::ProviderUsage {
                name: "test".into(),
                tokens_used: 500,
                cost_usd: 0.01,
            },
        ];
        let summary = crate::build_usage_summary_from_providers(providers);
        assert_eq!(summary.total_tokens, 500);
    }
}
