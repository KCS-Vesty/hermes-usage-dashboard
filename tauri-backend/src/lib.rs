use hermes_monitor::build_usage_summary;
use serde_json::Value;
use std::collections::HashMap;
use tauri::Emitter;
use tokio::time::{sleep, Duration};

// --- Existing commands ---

#[tauri::command]
fn get_dashboard_data() -> Value {
    serde_json::json!({
        "status": "ok",
        "message": "Hermes Usage Dashboard",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

#[tauri::command]
fn get_usage_summary() -> String {
    let summary = build_usage_summary();
    serde_json::to_string(&summary).unwrap_or_default()
}

// --- InfluxDB commands ---

#[derive(serde::Deserialize)]
struct InfluxConfig {
    url: String,
    org: String,
    bucket: String,
    token: String,
}

#[tauri::command]
async fn test_influx_connection(config: InfluxConfig) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", config.url);
    match client.get(&health_url).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() || status.as_u16() == 401 {
                // 401 means InfluxDB is running but needs auth — that's OK
                Ok(serde_json::json!({ "ok": true, "status": status.as_u16() }))
            } else {
                Err(format!("InfluxDB returned status {}", status))
            }
        }
        Err(e) => Err(format!("Could not reach InfluxDB: {}", e)),
    }
}

#[tauri::command]
async fn query_influxdb(config: InfluxConfig) -> Result<Value, String> {
    // Query InfluxDB for usage data from the last 24 hours
    let client = reqwest::Client::new();
    let query_url = format!("{}/api/v2/query", config.url);

    let flux_query = format!(
        r#"from(bucket: "{}")
        |> range(start: -24h)
        |> filter(fn: (r) => r._measurement == "usage")
        |> pivot(rowKey:["_time"], columnKey: ["_field"], valueColumn: "_value")
        |> group(columns: ["provider"])
        |> sum()"#,
        config.bucket
    );

    let body = serde_json::json!({
        "query": flux_query,
        "type": "flux"
    });

    let resp = client
        .post(&query_url)
        .header("Authorization", format!("Token {}", config.token))
        .header("Content-Type", "application/vnd.flux")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("InfluxDB query failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("InfluxDB returned status {}", resp.status()));
    }

    // Parse InfluxDB response and convert to UsageSummary format
    let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    let parsed = parse_influx_response(&text).map_err(|e| format!("Parse error: {}", e))?;
    Ok(parsed)
}

fn parse_influx_response(text: &str) -> Result<Value, String> {
    // InfluxDB returns CSV-like data in the response body
    // Parse it into our UsageSummary format
    let mut providers = Vec::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("result") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 4 {
            continue;
        }
        // Find provider, tokens_used, cost_usd columns
        let mut provider_name = String::new();
        let mut tokens: i64 = 0;
        let mut cost: f64 = 0.0;

        for (i, col) in cols.iter().enumerate() {
            let col = col.trim();
            if i == 3 && !col.is_empty() && col != "null" {
                // provider column
                provider_name = col.to_string();
            }
            // Look for tokens_used and cost_usd in the values
            if col.contains("tokens_used") {
                if let Some(val) = cols.get(i + 1) {
                    tokens = val.trim().parse::<i64>().unwrap_or(0);
                }
            }
            if col.contains("cost_usd") {
                if let Some(val) = cols.get(i + 1) {
                    cost = val.trim().parse::<f64>().unwrap_or(0.0);
                }
            }
        }

        if !provider_name.is_empty() && tokens > 0 {
            providers.push(serde_json::json!({
                "name": provider_name,
                "tokens_used": tokens,
                "cost_usd": cost
            }));
            total_tokens += tokens;
            total_cost += cost;
        }
    }

    Ok(serde_json::json!({
        "providers": providers,
        "total_tokens": total_tokens,
        "total_cost_usd": (total_cost * 100.0).round() / 100.0
    }))
}

// --- Provider API commands ---

#[derive(serde::Deserialize)]
struct ProviderKeys {
    #[serde(flatten)]
    keys: HashMap<String, String>,
}

#[tauri::command]
async fn test_provider_keys(keys: ProviderKeys) -> Value {
    let mut ok = Vec::new();
    let mut failed = Vec::new();

    for (provider, key) in &keys.keys {
        if key.trim().is_empty() {
            continue;
        }
        let result = match provider.as_str() {
            "openrouter" => test_openrouter_key(key).await,
            "anthropic" => test_anthropic_key(key).await,
            "openai" => test_openai_key(key).await,
            "opencode-zen" => test_opencode_zen_key(key).await,
            "opencode-go" => test_opencode_go_key(key).await,
            _ => Err(format!("Unknown provider: {}", provider)),
        };
        match result {
            Ok(_) => ok.push(provider.clone()),
            Err(e) => {
                log::warn!("Provider {} test failed: {}", provider, e);
                failed.push(provider.clone());
            }
        }
    }

    serde_json::json!({ "ok": ok, "failed": failed })
}

#[tauri::command]
async fn query_providers(keys: ProviderKeys) -> Result<Value, String> {
    let mut providers = Vec::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;

    for (provider, key) in &keys.keys {
        if key.trim().is_empty() {
            continue;
        }
        let result = match provider.as_str() {
            "openrouter" => query_openrouter(key).await,
            "anthropic" => query_anthropic(key).await,
            "openai" => query_openai(key).await,
            "opencode-zen" => query_opencode_zen(key).await,
            "opencode-go" => query_opencode_go(key).await,
            _ => continue,
        };
        match result {
            Ok(data) => {
                providers.push(serde_json::json!({
                    "name": provider,
                    "tokens_used": data.tokens,
                    "cost_usd": data.cost
                }));
                total_tokens += data.tokens;
                total_cost += data.cost;
            }
            Err(e) => {
                log::warn!("Failed to query {}: {}", provider, e);
            }
        }
    }

    Ok(serde_json::json!({
        "providers": providers,
        "total_tokens": total_tokens,
        "total_cost_usd": (total_cost * 100.0).round() / 100.0
    }))
}

// --- Provider API implementations ---

struct ProviderData {
    tokens: i64,
    cost: f64,
}

async fn test_openrouter_key(key: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Status: {}", resp.status()))
    }
}

async fn query_openrouter(key: &str) -> Result<ProviderData, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/usage")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let usage = json["data"]["total_usage"].as_f64().unwrap_or(0.0);
    let tokens = json["data"]["total_tokens"].as_i64().unwrap_or(0);
    Ok(ProviderData {
        tokens,
        cost: usage / 100.0, // OpenRouter returns cents
    })
}

async fn test_anthropic_key(key: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Status: {}", resp.status()))
    }
}

async fn query_anthropic(_key: &str) -> Result<ProviderData, String> {
    // Anthropic doesn't have a simple usage API; return 0 for now
    // In production, you'd parse billing dashboard or use their admin API
    Ok(ProviderData { tokens: 0, cost: 0.0 })
}

async fn test_openai_key(key: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Status: {}", resp.status()))
    }
}

async fn query_openai(key: &str) -> Result<ProviderData, String> {
    let client = reqwest::Client::new();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let resp = client
        .get(format!("https://api.openai.com/v1/usage?date={}", today))
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let tokens = json["data"].as_array().map(|d| {
        d.iter().map(|item| item["n_tokens"].as_i64().unwrap_or(0)).sum::<i64>()
    }).unwrap_or(0);
    let cost = json["total_usage"].as_f64().unwrap_or(0.0) / 100.0;
    Ok(ProviderData { tokens, cost })
}

async fn test_opencode_zen_key(key: &str) -> Result<(), String> {
    // Opencode Zen — placeholder; adjust URL when known
    if key.len() > 10 {
        Ok(())
    } else {
        Err("Key too short".to_string())
    }
}

async fn query_opencode_zen(_key: &str) -> Result<ProviderData, String> {
    // Placeholder — implement when Opencode Zen API is known
    Ok(ProviderData { tokens: 0, cost: 0.0 })
}

async fn test_opencode_go_key(key: &str) -> Result<(), String> {
    if key.len() > 10 {
        Ok(())
    } else {
        Err("Key too short".to_string())
    }
}

async fn query_opencode_go(_key: &str) -> Result<ProviderData, String> {
    // Placeholder — implement when Opencode Go API is known
    Ok(ProviderData { tokens: 0, cost: 0.0 })
}

// --- App entry ---

#[cfg(test)]
mod tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            get_dashboard_data,
            get_usage_summary,
            test_influx_connection,
            query_influxdb,
            test_provider_keys,
            query_providers,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = app_handle.emit("heartbeat", "ok");
                    sleep(Duration::from_secs(5)).await;
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {});
}
