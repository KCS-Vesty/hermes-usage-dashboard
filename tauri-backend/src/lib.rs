use std::time::Duration;
use tokio::time::sleep;
use tauri::Emitter;

/// Plain function (no #[tauri::command]) — testable without Tauri runtime.
/// Returns usage summary data as a typed struct.
pub fn build_usage_summary() -> UsageSummary {
    UsageSummary {
        providers: vec![
            ProviderUsage {
                name: "openrouter".to_string(),
                tokens_used: 15000,
                cost_usd: 0.45,
            },
            ProviderUsage {
                name: "anthropic".to_string(),
                tokens_used: 8000,
                cost_usd: 0.24,
            },
            ProviderUsage {
                name: "openai".to_string(),
                tokens_used: 12000,
                cost_usd: 0.36,
            },
        ],
        total_tokens: 35000,
        total_cost_usd: 1.05,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProviderUsage {
    pub name: String,
    pub tokens_used: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UsageSummary {
    pub providers: Vec<ProviderUsage>,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
}

/// Tauri command — thin wrapper that calls the plain function.
#[tauri::command]
fn get_dashboard_data() -> String {
    serde_json::json!({
        "status": "ok",
        "message": "Hermes Usage Dashboard"
    })
    .to_string()
}

/// Tauri command — thin wrapper that serializes the typed struct.
#[tauri::command]
fn get_usage_summary() -> String {
    let summary = build_usage_summary();
    serde_json::to_string(&summary).unwrap_or_default()
}

#[cfg(test)]
mod tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![get_dashboard_data, get_usage_summary])
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
