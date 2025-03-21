use std::env;

use tauri::{App, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

mod file_ext;
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
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            unzip::archive_list_files,
            unzip::archive_extract,
            run_args
        ])
        .setup(|app| {
            let inspect = file_ext::Inspect::new(app.handle().clone())?;
            file_ext::load(inspect);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
