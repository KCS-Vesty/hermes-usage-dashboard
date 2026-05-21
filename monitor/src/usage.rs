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
