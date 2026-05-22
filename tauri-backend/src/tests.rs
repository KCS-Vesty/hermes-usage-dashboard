#[cfg(test)]
mod tests {
    use hermes_monitor::{ProviderUsage, UsageSummary};
    use hermes_monitor::build_usage_summary;

    #[test]
    fn test_get_dashboard_data_returns_ok() {
        let result = crate::get_dashboard_data();
        assert_eq!(result["status"], "ok");
        assert_eq!(result["message"], "Hermes Usage Dashboard");
        assert!(result.get("version").is_some(), "version field should be present");
    }

    #[test]
    fn test_build_usage_summary_returns_providers() {
        let summary = build_usage_summary();
        assert_eq!(summary.providers.len(), 5);
    }

    #[test]
    fn test_build_usage_summary_total_tokens() {
        let summary = build_usage_summary();
        assert_eq!(summary.total_tokens, 38_000);
    }

    #[test]
    fn test_build_usage_summary_total_cost() {
        let summary = build_usage_summary();
        assert!((summary.total_cost_usd - 1.29).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_usage_summary_provider_names() {
        let summary = build_usage_summary();
        let names: Vec<&str> = summary.providers.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"openrouter"));
        assert!(names.contains(&"anthropic"));
        assert!(names.contains(&"openai"));
        assert!(names.contains(&"opencode zen"));
        assert!(names.contains(&"opencode go"));
    }

    #[test]
    fn test_provider_usage_struct() {
        let provider = ProviderUsage {
            name: "test".to_string(),
            tokens_used: 1000,
            cost_usd: 0.002,
        };
        assert_eq!(provider.name, "test");
        assert_eq!(provider.tokens_used, 1000);
    }

    #[test]
    fn test_usage_summary_serde_roundtrip() {
        let summary = build_usage_summary();
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: UsageSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, deserialized);
    }

    #[tokio::test]
    async fn test_get_usage_summary_command_valid_json() {
        let result = crate::get_usage_summary().await;
        let parsed: UsageSummary = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.providers.len(), 5);
    }
}