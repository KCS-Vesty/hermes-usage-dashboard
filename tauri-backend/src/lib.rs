use hermes_monitor::build_usage_summary;
use tauri::Emitter;
use std::time::Duration;
use tokio::time::sleep;

/// Tauri command — returns dashboard status as structured JSON.
#[tauri::command]
fn get_dashboard_data() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "message": "Hermes Usage Dashboard",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// Tauri command — thin wrapper that serializes the typed struct from the domain layer.
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
