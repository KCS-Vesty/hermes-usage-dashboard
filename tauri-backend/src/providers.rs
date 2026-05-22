// tauri-backend/src/providers.rs
// Provider API implementations — each provider has a test_key() and query_usage() method.

use serde_json::Value;

pub struct ProviderData {
    pub tokens: i64,
    pub cost: f64,
}

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

pub async fn test_openrouter_key(key: &str) -> Result<(), ProviderError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    if resp.status().is_success() {
        Ok(())
    } else if resp.status().as_u16() == 401 {
        Err(ProviderError::Auth("Invalid API key".into()))
    } else {
        Err(ProviderError::Network(format!("Status: {}", resp.status())))
    }
}

pub async fn query_openrouter(key: &str) -> Result<ProviderData, ProviderError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/usage")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    let json: Value = resp.json().await.map_err(|e| ProviderError::Parse(e.to_string()))?;
    let usage = json["data"]["total_usage"].as_f64().unwrap_or(0.0);
    let tokens = json["data"]["total_tokens"].as_i64().unwrap_or(0);
    Ok(ProviderData {
        tokens,
        cost: usage / 100.0,
    })
}

// --- Anthropic ---

pub async fn test_anthropic_key(key: &str) -> Result<(), ProviderError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    if resp.status().is_success() {
        Ok(())
    } else if resp.status().as_u16() == 401 {
        Err(ProviderError::Auth("Invalid API key".into()))
    } else {
        Err(ProviderError::Network(format!("Status: {}", resp.status())))
    }
}

pub async fn query_anthropic(_key: &str) -> Result<ProviderData, ProviderError> {
    // Anthropic doesn't have a public usage API endpoint.
    // Their billing data is only available through the web dashboard.
    // TODO: Implement via Anthropic's admin API or scraping the billing page.
    Err(ProviderError::Unsupported(
        "Anthropic usage API not yet implemented — use InfluxDB data source instead".into(),
    ))
}

// --- OpenAI ---

pub async fn test_openai_key(key: &str) -> Result<(), ProviderError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    if resp.status().is_success() {
        Ok(())
    } else if resp.status().as_u16() == 401 {
        Err(ProviderError::Auth("Invalid API key".into()))
    } else {
        Err(ProviderError::Network(format!("Status: {}", resp.status())))
    }
}

pub async fn query_openai(key: &str) -> Result<ProviderData, ProviderError> {
    let client = reqwest::Client::new();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // OpenAI usage endpoint: GET /dashboard/billing/usage?start_date=...&end_date=...
    let resp = client
        .get(format!(
            "https://api.openai.com/v1/dashboard/billing/usage?start_date={}&end_date={}",
            today, today
        ))
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ProviderError::Network(format!(
            "OpenAI returned status {}",
            resp.status()
        )));
    }
    let json: Value = resp.json().await.map_err(|e| ProviderError::Parse(e.to_string()))?;
    let total_usage = json["total_usage"].as_f64().unwrap_or(0.0) / 100.0;
    // Token count requires summing across usage breakdown
    let tokens: i64 = json["usage_breakdown"]
        .as_array()
        .map(|arr| arr.iter().map(|item| item["n_tokens"].as_i64().unwrap_or(0)).sum())
        .unwrap_or(0);
    Ok(ProviderData {
        tokens,
        cost: total_usage,
    })
}

// --- Opencode Zen ---

pub async fn test_opencode_zen_key(key: &str) -> Result<(), ProviderError> {
    if key.len() > 10 {
        Ok(())
    } else {
        Err(ProviderError::Auth("Key too short".into()))
    }
}

pub async fn query_opencode_zen(_key: &str) -> Result<ProviderData, ProviderError> {
    Err(ProviderError::Unsupported(
        "Opencode Zen API endpoint not yet configured — use InfluxDB data source instead".into(),
    ))
}

// --- Opencode Go ---

pub async fn test_opencode_go_key(key: &str) -> Result<(), ProviderError> {
    if key.len() > 10 {
        Ok(())
    } else {
        Err(ProviderError::Auth("Key too short".into()))
    }
}

pub async fn query_opencode_go(_key: &str) -> Result<ProviderData, ProviderError> {
    Err(ProviderError::Unsupported(
        "Opencode Go API endpoint not yet configured — use InfluxDB data source instead".into(),
    ))
}

// --- Provider registry ---

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
}

pub async fn test_all_keys(config: &ProviderConfig) -> Vec<(String, Result<(), ProviderError>)> {
    let mut results = Vec::new();

    if let Some(ref key) = config.openrouter {
        if !key.trim().is_empty() {
            results.push(("openrouter".into(), test_openrouter_key(key).await));
        }
    }
    if let Some(ref key) = config.anthropic {
        if !key.trim().is_empty() {
            results.push(("anthropic".into(), test_anthropic_key(key).await));
        }
    }
    if let Some(ref key) = config.openai {
        if !key.trim().is_empty() {
            results.push(("openai".into(), test_openai_key(key).await));
        }
    }
    if let Some(ref key) = config.opencode_zen {
        if !key.trim().is_empty() {
            results.push(("opencode-zen".into(), test_opencode_zen_key(key).await));
        }
    }
    if let Some(ref key) = config.opencode_go {
        if !key.trim().is_empty() {
            results.push(("opencode-go".into(), test_opencode_go_key(key).await));
        }
    }

    results
}

pub async fn query_all(config: &ProviderConfig) -> Vec<(String, ProviderData)> {
    let mut results = Vec::new();

    if let Some(ref key) = config.openrouter {
        if !key.trim().is_empty() {
            if let Ok(data) = query_openrouter(key).await {
                results.push(("openrouter".into(), data));
            }
        }
    }
    if let Some(ref key) = config.anthropic {
        if !key.trim().is_empty() {
            if let Ok(data) = query_anthropic(key).await {
                results.push(("anthropic".into(), data));
            }
        }
    }
    if let Some(ref key) = config.openai {
        if !key.trim().is_empty() {
            if let Ok(data) = query_openai(key).await {
                results.push(("openai".into(), data));
            }
        }
    }
    if let Some(ref key) = config.opencode_zen {
        if !key.trim().is_empty() {
            if let Ok(data) = query_opencode_zen(key).await {
                results.push(("opencode-zen".into(), data));
            }
        }
    }
    if let Some(ref key) = config.opencode_go {
        if !key.trim().is_empty() {
            if let Ok(data) = query_opencode_go(key).await {
                results.push(("opencode-go".into(), data));
            }
        }
    }

    results
}
