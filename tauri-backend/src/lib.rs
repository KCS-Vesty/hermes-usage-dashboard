mod providers;

use hermes_monitor::build_usage_summary;
use providers::ProviderConfig;
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
pub struct InfluxConfig {
    pub url: String,
    pub org: String,
    pub bucket: String,
    pub token: String,
}

#[tauri::command]
async fn test_influx_connection(config: InfluxConfig) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", config.url);
    match client.get(&health_url).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() || status.as_u16() == 401 {
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
    let client = reqwest::Client::new();
    let query_url = format!("{}/api/v2/query", config.url);

    // Query usage data from the last 24 hours, grouped by provider
    let flux_query = format!(
        r#"from(bucket: "{}")
        |> range(start: -24h)
        |> filter(fn: (r) => r._measurement == "usage")
        |> pivot(rowKey:["_time"], columnKey: ["_field"], valueColumn: "_value")
        |> group(columns: ["provider"])
        |> sum()"#,
        config.bucket
    );

    let resp = client
        .post(&query_url)
        .header("Authorization", format!("Token {}", config.token))
        .header("Content-Type", "application/vnd.flux")
        .json(&serde_json::json!({ "query": flux_query, "type": "flux" }))
        .send()
        .await
        .map_err(|e| format!("InfluxDB query failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("InfluxDB returned status {}", resp.status()));
    }

    let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    let parsed = parse_influx_csv(&text).map_err(|e| format!("Parse error: {}", e))?;
    Ok(parsed)
}

/// Parse InfluxDB's annotated CSV response into UsageSummary format.
/// InfluxDB returns CSV with comment lines starting with #, then headers, then data rows.
fn parse_influx_csv(text: &str) -> Result<Value, String> {
    let mut providers = Vec::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;

    // Parse header column positions
    let mut header_cols: Vec<&str> = Vec::new();
    let mut provider_col = None;
    let mut tokens_col = None;
    let mut cost_col = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            // Parse header line: #datatype,string,long,dateTime:RFC3339,...
            // The column names come after the datatype annotations
            continue;
        }

        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        // First non-comment line after headers contains column names
        if header_cols.is_empty() && !line.starts_with("result") {
            header_cols = cols.clone();
            // Find column indices
            for (i, col) in header_cols.iter().enumerate() {
                match *col {
                    "provider" => provider_col = Some(i),
                    "tokens_used" => tokens_col = Some(i),
                    "cost_usd" => cost_col = Some(i),
                    _ => {}
                }
            }
            continue;
        }

        // Skip the "result,table,_start,_stop" header line
        if cols.first() == Some(&"result") {
            continue;
        }

        // Data row
        let provider_name = provider_col.and_then(|i| cols.get(i)).unwrap_or(&"").to_string();
        let tokens = tokens_col
            .and_then(|i| cols.get(i))
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let cost = cost_col
            .and_then(|i| cols.get(i))
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

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
    let config = ProviderConfig::from_map(&keys.keys);
    let results = providers::test_all_keys(&config).await;

    let mut ok = Vec::new();
    let mut failed = Vec::new();

    for (name, result) in results {
        match result {
            Ok(_) => ok.push(name),
            Err(e) => {
                log::warn!("Provider {} test failed: {}", name, e);
                failed.push(serde_json::json!({ "name": name, "error": e.to_string() }));
            }
        }
    }

    serde_json::json!({ "ok": ok, "failed": failed })
}

#[tauri::command]
async fn query_providers(keys: ProviderKeys) -> Result<Value, String> {
    let config = ProviderConfig::from_map(&keys.keys);
    let results = providers::query_all(&config).await;

    let mut provider_list = Vec::new();
    let mut total_tokens: i64 = 0;
    let mut total_cost: f64 = 0.0;

    for (name, data) in results {
        provider_list.push(serde_json::json!({
            "name": name,
            "tokens_used": data.tokens,
            "cost_usd": data.cost
        }));
        total_tokens += data.tokens;
        total_cost += data.cost;
    }

    Ok(serde_json::json!({
        "providers": provider_list,
        "total_tokens": total_tokens,
        "total_cost_usd": (total_cost * 100.0).round() / 100.0
    }))
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
