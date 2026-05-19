use yew::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UsageData {
    pub provider: String,
    pub tokens_used: i64,
    pub cost_usd: f64,
}

#[function_component(App)]
fn app() -> Html {
    let usage = use_state(|| Vec::<UsageData>::new());

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
    fn usage_data_creation() {
        let data = UsageData {
            provider: "openrouter".to_string(),
            tokens_used: 1000,
            cost_usd: 0.002,
        };
        assert_eq!(data.provider, "openrouter");
        assert_eq!(data.tokens_used, 1000);
        assert!((data.cost_usd - 0.002).abs() < f64::EPSILON);
    }

    #[test]
    fn usage_data_serde() {
        let json = r#"{"provider":"anthropic","tokens_used":500,"cost_usd":0.015}"#;
        let data: UsageData = serde_json::from_str(json).unwrap();
        assert_eq!(data.provider, "anthropic");
        assert_eq!(data.tokens_used, 500);
        assert!((data.cost_usd - 0.015).abs() < f64::EPSILON);
    }
}
