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
}
