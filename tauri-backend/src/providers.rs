// tauri-backend/src/providers.rs
// Provider API implementations.

use serde_json::Value;

pub struct ProviderData {
    pub tokens: i64,
    pub cost: f64,
}

#[derive(Debug)]
pub enum ProviderError {
    Network(String),
    Auth(String),
    Parse(String),
    Unsupported(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Auth(e) => write!(f, "Auth error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::Unsupported(e) => write!(f, "Unsupported: {}", e),
        }
    }
}

// --- OpenRouter ---

async fn test_openrouter_key(key: &str) -> Result<(), ProviderError> {
    let resp = reqwest::Client::new()
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 => Err(ProviderError::Auth("Invalid API key".into())),
        s => Err(ProviderError::Network(format!("Status: {}", s))),
    }
}

async fn query_openrouter(key: &str) -> Result<ProviderData, ProviderError> {
    let json: Value = reqwest::Client::new()
        .get("https://openrouter.ai/api/v1/usage")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?
        .json()
        .await
        .map_err(|e| ProviderError::Parse(e.to_string()))?;
    Ok(ProviderData {
        tokens: json["data"]["total_tokens"].as_i64().unwrap_or(0),
        cost: json["data"]["total_usage"].as_f64().unwrap_or(0.0) / 100.0,
    })
}

// --- Anthropic ---

async fn test_anthropic_key(key: &str) -> Result<(), ProviderError> {
    let resp = reqwest::Client::new()
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 => Err(ProviderError::Auth("Invalid API key".into())),
        s => Err(ProviderError::Network(format!("Status: {}", s))),
    }
}

async fn query_anthropic() -> Result<ProviderData, ProviderError> {
    Err(ProviderError::Unsupported(
        "Anthropic usage API not yet implemented — use InfluxDB data source instead".into(),
    ))
}

// --- OpenAI ---

async fn test_openai_key(key: &str) -> Result<(), ProviderError> {
    let resp = reqwest::Client::new()
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 => Err(ProviderError::Auth("Invalid API key".into())),
        s => Err(ProviderError::Network(format!("Status: {}", s))),
    }
}

async fn query_openai(key: &str) -> Result<ProviderData, ProviderError> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let json: Value = reqwest::Client::new()
        .get(format!(
            "https://api.openai.com/v1/dashboard/billing/usage?start_date={}&end_date={}",
            today, today
        ))
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| ProviderError::Network(format!("OpenAI returned {}", e.status().unwrap())))?
        .json()
        .await
        .map_err(|e| ProviderError::Parse(e.to_string()))?;
    Ok(ProviderData {
        tokens: json["usage_breakdown"]
            .as_array()
            .map(|a| a.iter().map(|i| i["n_tokens"].as_i64().unwrap_or(0)).sum())
            .unwrap_or(0),
        cost: json["total_usage"].as_f64().unwrap_or(0.0) / 100.0,
    })
}

// --- Opencode Zen ---

async fn test_opencode_zen_key(key: &str) -> Result<(), ProviderError> {
    if key.len() > 10 { Ok(()) } else { Err(ProviderError::Auth("Key too short".into())) }
}

async fn query_opencode_zen() -> Result<ProviderData, ProviderError> {
    Err(ProviderError::Unsupported(
        "Opencode Zen API endpoint not yet configured — use InfluxDB data source instead".into(),
    ))
}

// --- Opencode Go ---

async fn test_opencode_go_key(key: &str) -> Result<(), ProviderError> {
    if key.len() > 10 { Ok(()) } else { Err(ProviderError::Auth("Key too short".into())) }
}

async fn query_opencode_go() -> Result<ProviderData, ProviderError> {
    Err(ProviderError::Unsupported(
        "Opencode Go API endpoint not yet configured — use InfluxDB data source instead".into(),
    ))
}

// --- Provider config ---

pub struct ProviderConfig {
    pub openrouter: Option<String>,
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub opencode_zen: Option<String>,
    pub opencode_go: Option<String>,
}

impl ProviderConfig {
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Self {
        Self {
            openrouter: map.get("openrouter").cloned(),
            anthropic: map.get("anthropic").cloned(),
            openai: map.get("openai").cloned(),
            opencode_zen: map.get("opencode-zen").cloned(),
            opencode_go: map.get("opencode-go").cloned(),
        }
    }

    /// Return configured provider entries as (name, key) pairs.
    fn entries(&self) -> Vec<(&str, &str)> {
        [
            ("openrouter", self.openrouter.as_deref()),
            ("anthropic", self.anthropic.as_deref()),
            ("openai", self.openai.as_deref()),
            ("opencode-zen", self.opencode_zen.as_deref()),
            ("opencode-go", self.opencode_go.as_deref()),
        ]
        .iter()
        .filter_map(|(name, key)| key.filter(|k| !k.trim().is_empty()).map(|k| (*name, k)))
        .collect()
    }
}

// --- Public API ---

pub async fn test_all_keys(config: &ProviderConfig) -> Vec<(String, Result<(), ProviderError>)> {
    let mut results = Vec::new();
    for (name, key) in config.entries() {
        let result = match name {
            "openrouter" => test_openrouter_key(key).await,
            "anthropic" => test_anthropic_key(key).await,
            "openai" => test_openai_key(key).await,
            "opencode-zen" => test_opencode_zen_key(key).await,
            "opencode-go" => test_opencode_go_key(key).await,
            _ => Err(ProviderError::Unsupported(name.into())),
        };
        results.push((name.into(), result));
    }
    results
}

pub async fn query_all(config: &ProviderConfig) -> Vec<(String, ProviderData)> {
    let mut results = Vec::new();
    for (name, key) in config.entries() {
        let result = match name {
            "openrouter" => query_openrouter(key).await,
            "anthropic" => query_anthropic().await,
            "openai" => query_openai(key).await,
            "opencode-zen" => query_opencode_zen().await,
            "opencode-go" => query_opencode_go().await,
            _ => Err(ProviderError::Unsupported(name.into())),
        };
        if let Ok(data) = result {
            results.push((name.into(), data));
        }
    }
    results
}
