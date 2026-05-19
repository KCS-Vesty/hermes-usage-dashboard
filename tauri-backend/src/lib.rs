use std::time::Duration;
use tokio::time::sleep;
use tauri::Emitter;

#[tauri::command]
fn get_dashboard_data() -> String {
    serde_json::json!({
        "status": "ok",
        "message": "Hermes Usage Dashboard"
    })
    .to_string()
}

#[tauri::command]
fn get_usage_summary() -> String {
    serde_json::json!({
        "providers": [
            {"name": "openrouter", "tokens_used": 15000, "cost_usd": 0.45},
            {"name": "anthropic", "tokens_used": 8000, "cost_usd": 0.24},
            {"name": "openai", "tokens_used": 12000, "cost_usd": 0.36}
        ],
        "total_tokens": 35000,
        "total_cost_usd": 1.05
    })
    .to_string()
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
