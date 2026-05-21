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
        ],
        total_tokens: 35_000,
        total_cost_usd: 1.05,
    }
}
