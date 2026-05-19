#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::Manager;
use std::time::Duration;
use tokio::time::sleep;

#[tauri::command]
fn _() {}

#[tokio::main]
async fn main() {
    env_logger::init();

    tauri::async_runtime::spawn(hermes_monitor::db::start_writer());

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![_])
        .setup(|app| {
            let app_handle = app.handle();

            tauri::async_runtime::spawn(async move {
                let client =
                    influxdb2::Client::new("http://127.0.0.1:8086", "hermes-token")
                        .with_org("hermes")
                        .with_bucket("hermes_usage");

                loop {
                    let q = r#"
from(bucket: "hermes_usage")
  |> range(start: -30s)
  |> filter(fn: (r) => r._measurement == "usage")
  |> aggregateWindow(every: 5s, fn: sum, createEmpty: false)
  |> yield(name: "last")
"#;
                    let res = client.query(q).await;
                    if let Ok(result) = res {
                        let _ = app_handle.emit_all("usage_update", serde_json::to_string(&result).unwrap());
                    }
                    sleep(Duration::from_secs(5)).await;
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {});
}
