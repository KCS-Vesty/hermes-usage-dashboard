mod influx;
mod providers;

use hermes_monitor::build_usage_summary;
use influx::{query_influxdb, InfluxConfig};
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
async fn get_usage_summary() -> String {
    let summary = build_usage_summary();
    serde_json::to_string(&summary).unwrap_or_default()
}

// --- InfluxDB commands ---

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
async fn query_influxdb_cmd(config: InfluxConfig) -> Result<Value, String> {
    query_influxdb(&config).await
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
            query_influxdb_cmd,
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
