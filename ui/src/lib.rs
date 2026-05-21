use serde::Deserialize;
use yew::prelude::*;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ProviderUsage {
    pub name: String,
    pub tokens_used: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UsageSummary {
    pub providers: Vec<ProviderUsage>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
}

fn get_usage_summary() -> Option<UsageSummary> {
    // When running inside Tauri, the backend provides data via window.__TAURI__.invoke().
    // When running standalone (dev/test), use mock data.
    // For now, always use mock data — the Tauri IPC layer is handled by the host.
    mock_data()
}

fn mock_data() -> Option<UsageSummary> {
    Some(UsageSummary {
        providers: vec![
            ProviderUsage { name: "openrouter".into(), tokens_used: 15_000, cost_usd: 0.45 },
            ProviderUsage { name: "anthropic".into(), tokens_used: 8_000, cost_usd: 0.24 },
            ProviderUsage { name: "openai".into(), tokens_used: 12_000, cost_usd: 0.36 },
        ],
        total_tokens: 35_000,
        total_cost_usd: 1.05,
    })
}

#[function_component(App)]
fn app() -> Html {
    let summary = get_usage_summary();

    html! {
        <div>
            <h1>{ "Hermes Usage Dashboard" }</h1>
            {
                if let Some(ref s) = summary {
                    html! {
                        <>
                            <div>
                                <p>{ "Total Tokens: " }{ format_number(s.total_tokens) }</p>
                                <p>{ "Total Cost: $" }{ format!("{:.2}", s.total_cost_usd) }</p>
                            </div>
                            <table>
                                <thead>
                                    <tr>
                                        <th>{ "Provider" }</th>
                                        <th>{ "Tokens" }</th>
                                        <th>{ "Cost" }</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { for s.providers.iter().map(|p| html! {
                                        <tr>
                                            <td>{ &p.name }</td>
                                            <td>{ format_number(p.tokens_used) }</td>
                                            <td>{ format!("${:.4}", p.cost_usd) }</td>
                                        </tr>
                                    }) }
                                </tbody>
                            </table>
                        </>
                    }
                } else {
                    html! { <p>{ "No data available" }</p> }
                }
            }
        </div>
    }
}

fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1_000), "1,000");
        assert_eq!(format_number(35_000), "35,000");
        assert_eq!(format_number(1_234_567), "1,234,567");
    }

    #[test]
    fn test_provider_usage_deserialization() {
        let json = r#"{"name":"openrouter","tokens_used":15000,"cost_usd":0.45}"#;
        let p: ProviderUsage = serde_json::from_str(json).unwrap();
        assert_eq!(p.name, "openrouter");
        assert_eq!(p.tokens_used, 15_000);
        assert!((p.cost_usd - 0.45).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_summary_deserialization() {
        let json = r#"{"providers":[{"name":"openrouter","tokens_used":15000,"cost_usd":0.45}],"total_tokens":15000,"total_cost_usd":0.45}"#;
        let s: UsageSummary = serde_json::from_str(json).unwrap();
        assert_eq!(s.providers.len(), 1);
        assert_eq!(s.total_tokens, 15_000);
    }

    #[test]
    fn test_mock_data() {
        let data = mock_data().unwrap();
        assert_eq!(data.providers.len(), 3);
        assert_eq!(data.total_tokens, 35_000);
        assert!((data.total_cost_usd - 1.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_usage_summary_returns_data() {
        let data = get_usage_summary().unwrap();
        assert_eq!(data.providers.len(), 3);
        assert_eq!(data.total_tokens, 35_000);
    }

    #[test]
    fn test_provider_names() {
        let data = get_usage_summary().unwrap();
        let names: Vec<&str> = data.providers.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"openrouter"));
        assert!(names.contains(&"anthropic"));
        assert!(names.contains(&"openai"));
    }

    #[test]
    fn test_costs_are_positive() {
        let data = get_usage_summary().unwrap();
        for p in &data.providers {
            assert!(p.cost_usd > 0.0, "cost should be positive for {}", p.name);
        }
        assert!(data.total_cost_usd > 0.0);
    }

    #[test]
    fn test_tokens_are_positive() {
        let data = get_usage_summary().unwrap();
        for p in &data.providers {
            assert!(p.tokens_used > 0, "tokens should be positive for {}", p.name);
        }
        assert!(data.total_tokens > 0);
    }
}
