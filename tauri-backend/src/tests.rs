#[cfg(test)]
mod tests {
    use crate::get_dashboard_data;
    use crate::get_usage_summary;

    #[test]
    fn test_get_dashboard_data_returns_ok() {
        let result = get_dashboard_data();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["message"], "Hermes Usage Dashboard");
    }

    #[test]
    fn test_get_usage_summary_returns_providers() {
        let result = get_usage_summary();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let providers = parsed["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 3);
    }

    #[test]
    fn test_get_usage_summary_total_tokens() {
        let result = get_usage_summary();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total_tokens"], 35000);
    }

    #[test]
    fn test_get_usage_summary_total_cost() {
        let result = get_usage_summary();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let total_cost = parsed["total_cost_usd"].as_f64().unwrap();
        assert!((total_cost - 1.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_usage_summary_provider_names() {
        let result = get_usage_summary();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let providers = parsed["providers"].as_array().unwrap();
        let names: Vec<&str> = providers
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"openrouter"));
        assert!(names.contains(&"anthropic"));
        assert!(names.contains(&"openai"));
    }
}
