use std::env;

use std::fs::read_to_string;
use std::path::Path;
use tauri::{App, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

mod unzip;

#[tauri::command]
fn run_args(app: tauri::AppHandle) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    if cfg!(target_os = "windows") {
        args = env::args().skip(1).collect();
    } else if cfg!(target_os = "macos") {
        args = match app.deep_link().get_current().unwrap_or_default() {
            Some(urls) => urls.iter().map(|url| url.to_string()).collect(),
            None => Vec::new(),
        }
    }
    args
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            unzip::archive_list_files,
            unzip::archive_extract,
            run_args
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                use tauri::WebviewWindow;
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;

                let window: WebviewWindow = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
