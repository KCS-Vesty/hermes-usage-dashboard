#[cfg_attr(feature = "types", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub provider: String,
    pub model: Option<String>,
    pub tokens_used: i64,
    pub cost_usd: f64,
    pub ts: i64,
}

#[cfg_attr(feature = "types", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct RateLimitRecord {
    pub provider: String,
    pub model: Option<String>,
    pub hard_limit: i64,
    pub soft_limit: Option<i64>,
    pub remaining: i64,
    pub reset_ts: i64,
    pub ts: i64,
}

// --- Dashboard types (feature-gated with "types") ---

#[cfg_attr(feature = "types", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderUsage {
    pub name: String,
    pub tokens_used: i64,
    pub cost_usd: f64,
}

#[cfg_attr(feature = "types", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummary {
    pub providers: Vec<ProviderUsage>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
}

/// Build a usage summary from current data.
/// Pure function — no I/O, easily testable.
/// NOTE: This is legacy mock data for when no real data sources are configured.
#[cfg(feature = "types")]
pub fn build_usage_summary() -> UsageSummary {
    UsageSummary {
        providers: vec![
            ProviderUsage {
                name: "openrouter".to_string(),
                tokens_used: 15_000,
                cost_usd: 0.45,
            },
            ProviderUsage {
                name: "anthropic".to_string(),
                tokens_used: 8_000,
                cost_usd: 0.24,
            },
            ProviderUsage {
                name: "openai".to_string(),
                tokens_used: 12_000,
                cost_usd: 0.36,
            },
            ProviderUsage {
                name: "opencode zen".to_string(),
                tokens_used: 5_000,
                cost_usd: 0.15,
            },
            ProviderUsage {
                name: "opencode go".to_string(),
                tokens_used: 3_000,
                cost_usd: 0.09,
            },
        ],
        total_tokens: 43_000,
        total_cost_usd: 1.29,
    }
}

/// Build a usage summary from a list of provider usages.
/// This function is pure and can be used to construct a UsageSummary from
/// data fetched from any source (InfluxDB, provider APIs, etc.).
#[cfg(feature = "types")]
pub fn build_usage_summary_from_providers(providers: Vec<ProviderUsage>) -> UsageSummary {
    let total_tokens = providers.iter().map(|p| p.tokens_used).sum();
    let total_cost_usd = providers.iter().map(|p| p.cost_usd).sum();
    UsageSummary {
        providers,
        total_tokens,
        total_cost_usd,
    }
}