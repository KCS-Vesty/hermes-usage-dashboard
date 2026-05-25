#[cfg(test)]
mod tests {
    use hermes_monitor::{ProviderUsage, UsageSummary};
    use hermes_monitor::build_usage_summary;
    use serde_json::Value;

    // --- Command existence & response shape tests ---

    #[test]
    fn test_get_dashboard_data_returns_ok() {
        let result = crate::get_dashboard_data();
        assert_eq!(result["status"], "ok");
        assert_eq!(result["message"], "Hermes Usage Dashboard");
        assert!(
            result.get("version").is_some(),
            "version field should be present"
        );
    }

    #[test]
    fn test_get_dashboard_data_has_required_keys() {
        let result = crate::get_dashboard_data();
        let keys: Vec<&str> = result.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert!(keys.contains(&"status"), "missing 'status' key");
        assert!(keys.contains(&"message"), "missing 'message' key");
        assert!(keys.contains(&"version"), "missing 'version' key");
    }

    // --- Usage summary tests ---

    #[test]
    fn test_build_usage_summary_returns_providers() {
        let summary = build_usage_summary();
        assert_eq!(summary.providers.len(), 5);
    }

    #[test]
    fn test_build_usage_summary_total_tokens() {
        let summary = build_usage_summary();
        assert_eq!(summary.total_tokens, 43_000);
    }

    #[test]
    fn test_build_usage_summary_total_cost() {
        let summary = build_usage_summary();
        assert!((summary.total_cost_usd - 1.29).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_usage_summary_provider_names() {
        let summary = build_usage_summary();
        let names: Vec<&str> = summary
            .providers
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(names.contains(&"openrouter"), "missing openrouter");
        assert!(names.contains(&"anthropic"), "missing anthropic");
        assert!(names.contains(&"openai"), "missing openai");
        assert!(names.contains(&"opencode zen"), "missing opencode zen");
        assert!(names.contains(&"opencode go"), "missing opencode go");
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

    // --- Async command tests ---

    #[tokio::test]
    async fn test_get_usage_summary_command_valid_json() {
        let result = crate::get_usage_summary().await;
        let parsed: UsageSummary = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.providers.len(), 5);
    }

    #[tokio::test]
    async fn test_get_usage_summary_json_has_required_fields() {
        let result = crate::get_usage_summary().await;
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(
            parsed.get("providers").is_some(),
            "JSON output missing 'providers' field"
        );
        assert!(
            parsed.get("total_tokens").is_some(),
            "JSON output missing 'total_tokens' field"
        );
        assert!(
            parsed.get("total_cost_usd").is_some(),
            "JSON output missing 'total_cost_usd' field"
        );
    }

    #[tokio::test]
    async fn test_get_usage_summary_provider_names_in_json() {
        let result = crate::get_usage_summary().await;
        let parsed: UsageSummary = serde_json::from_str(&result).unwrap();
        let names: Vec<&str> = parsed
            .providers
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(
            names.contains(&"openrouter"),
            "JSON output missing openrouter"
        );
        assert!(
            names.contains(&"anthropic"),
            "JSON output missing anthropic"
        );
        assert!(
            names.contains(&"openai"),
            "JSON output missing openai"
        );
        assert!(
            names.contains(&"opencode zen"),
            "JSON output missing opencode zen"
        );
        assert!(
            names.contains(&"opencode go"),
            "JSON output missing opencode go"
        );
    }

    // --- InfluxDB command tests ---

    #[tokio::test]
    #[ignore = "requires a running InfluxDB server"]
    async fn test_influx_connection_returns_ok_on_reachable() {
        let config = crate::InfluxConfig {
            url: "http://localhost:8086".to_string(),
            org: "test".to_string(),
            bucket: "test".to_string(),
            token: "test".to_string(),
        };
        // This will fail to connect (no InfluxDB running), but it should
        // return an Err(String) — not panic
        let result = crate::test_influx_connection(config).await;
        match result {
            Ok(v) => {
                // If it somehow connected, verify response shape
                assert!(v["ok"].as_bool().unwrap_or(false));
                assert!(v.get("status").is_some());
            }
            Err(e) => {
                // Expected: connection refused or similar network error
                assert!(
                    e.contains("Could not reach InfluxDB") || e.contains("error"),
                    "Error message should be descriptive, got: {}",
                    e
                );
            }
        }
    }

    // --- Provider command tests ---

    #[tokio::test]
    async fn test_provider_keys_with_empty_keys() {
        let keys = std::collections::HashMap::new();
        let result = crate::test_provider_keys(crate::ProviderKeys { keys }).await;
        let ok = result["ok"].as_array().unwrap();
        let failed = result["failed"].as_array().unwrap();
        assert_eq!(ok.len(), 0, "no keys provided, ok list should be empty");
        assert_eq!(failed.len(), 0, "no keys provided, failed list should be empty");
    }

    #[tokio::test]
    #[ignore = "requires network access to provider APIs"]
    async fn test_provider_keys_with_invalid_key() {
        let mut keys = std::collections::HashMap::new();
        keys.insert("openrouter".to_string(), "invalid-key".to_string());
        let result = crate::test_provider_keys(crate::ProviderKeys { keys }).await;
        let ok = result["ok"].as_array().unwrap();
        let failed = result["failed"].as_array().unwrap();
        // Invalid key should result in failure
        assert!(
            ok.len() == 0 || failed.len() > 0,
            "Invalid key should not succeed"
        );
        if failed.len() > 0 {
            let first = &failed[0];
            assert!(
                first.get("name").is_some(),
                "Failed entry should have 'name' field"
            );
            assert!(
                first.get("error").is_some(),
                "Failed entry should have 'error' field"
            );
        }
    }

    #[tokio::test]
    async fn test_provider_keys_response_shape() {
        let keys = std::collections::HashMap::new();
        let result = crate::test_provider_keys(crate::ProviderKeys { keys }).await;
        assert!(
            result.get("ok").is_some(),
            "Response missing 'ok' field"
        );
        assert!(
            result.get("failed").is_some(),
            "Response missing 'failed' field"
        );
        assert!(
            result["ok"].is_array(),
            "'ok' should be an array"
        );
        assert!(
            result["failed"].is_array(),
            "'failed' should be an array"
        );
    }

    #[tokio::test]
    async fn test_query_providers_with_empty_keys() {
        let keys = std::collections::HashMap::new();
        let result = crate::query_providers(crate::ProviderKeys { keys }).await;
        assert!(result.is_ok(), "Empty keys should not error");
        let value = result.unwrap();
        let providers = value["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 0);
        assert_eq!(value["total_tokens"], 0);
    }

    #[tokio::test]
    async fn test_query_providers_response_shape() {
        let keys = std::collections::HashMap::new();
        let result = crate::query_providers(crate::ProviderKeys { keys }).await;
        let value = result.unwrap();
        assert!(
            value.get("providers").is_some(),
            "Response missing 'providers'"
        );
        assert!(
            value.get("total_tokens").is_some(),
            "Response missing 'total_tokens'"
        );
        assert!(
            value.get("total_cost_usd").is_some(),
            "Response missing 'total_cost_usd'"
        );
        assert!(value["providers"].is_array());
        assert!(value["total_tokens"].is_number());
        assert!(value["total_cost_usd"].is_number());
    }

    // --- Command name consistency test (would have caught the _cmd mismatch) ---

    /// Verify that every Tauri command function exists and is callable.
    /// This test ensures the command names registered in `tauri::generate_handler!`
    /// match the names the frontend expects to call.
    #[test]
    fn test_all_tauri_commands_exist() {
        // These are the 6 commands the frontend calls:
        // 1. get_dashboard_data  — tested by test_get_dashboard_data_returns_ok
        // 2. get_usage_summary   — tested by test_get_usage_summary_command_valid_json
        // 3. test_influx_connection — tested by test_influx_connection_returns_ok_on_reachable
        // 4. query_influxdb     — exists as Tauri command (not query_influxdb_cmd)
        // 5. test_provider_keys — tested by test_provider_keys_response_shape
        // 6. query_providers    — tested by test_query_providers_response_shape

        // Verify query_influxdb function exists at crate root (not just in influx module)
        // This would have caught the _cmd naming mismatch
        let _: fn(crate::InfluxConfig) -> _ = crate::query_influxdb;

        // Verify it's NOT named query_influxdb_cmd (the old broken name)
        // This line won't compile if someone renames it back to _cmd
        let check: fn(crate::InfluxConfig) -> _ = crate::query_influxdb;
        let _ = check;
    }
}
