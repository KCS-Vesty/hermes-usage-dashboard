use hermes_monitor::UsageRecord;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let usage = use_state(Vec::<UsageRecord>::new);

    html! {
        <div class="p-4">
            <h1 class="text-xl font-bold">{ "Hermes Usage Dashboard" }</h1>
            <div>
                { for usage.iter().map(|u| html! {
                    <div key={u.provider.clone()}>
                        <span>{ &u.provider }</span>
                        <span>{ format!(" - {} tokens", u.tokens_used) }</span>
                        <span>{ format!(" - ${:.4}", u.cost_usd) }</span>
                    </div>
                }) }
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_record_from_monitor() {
        let rec = UsageRecord {
            provider: "openrouter".to_string(),
            model: Some("claude-3".to_string()),
            tokens_used: 1000,
            cost_usd: 0.002,
            ts: 1_700_000_000,
        };
        assert_eq!(rec.provider, "openrouter");
        assert_eq!(rec.tokens_used, 1000);
    }

    #[test]
    fn test_usage_record_serde_roundtrip() {
        let original = UsageRecord {
            provider: "anthropic".to_string(),
            model: None,
            tokens_used: 500,
            cost_usd: 0.015,
            ts: 1_700_000_000,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: UsageRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(original.provider, deserialized.provider);
        assert_eq!(original.tokens_used, deserialized.tokens_used);
    }

    #[test]
    fn test_usage_record_with_model() {
        let rec = UsageRecord {
            provider: "openai".to_string(),
            model: Some("gpt-4o".to_string()),
            tokens_used: 2000,
            cost_usd: 0.04,
            ts: 1_700_000_000,
        };
        assert_eq!(rec.model, Some("gpt-4o".to_string()));
        assert_eq!(rec.provider, "openai");
    }

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
    fn test_usage_record_cost_formatting() {
        // Verify cost values serialize with enough precision for display
        let rec = UsageRecord {
            provider: "test".to_string(),
            model: None,
            tokens_used: 100,
            cost_usd: 0.001,
            ts: 1_700_000_000,
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("0.001"));
    }
}
