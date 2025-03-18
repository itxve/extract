use std::env;

use tauri::Manager;

mod unzip;

#[tauri::command]
fn run_args() -> (String, String) {
    use std::fs::read_to_string;
    use std::path::Path;
    let mut result = "".to_owned();
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let path = Path::new(&args[1]);
        let extension = format!(".{}", path.extension().unwrap().to_string_lossy());
        result = read_to_string(path).expect("read error");
        (extension, result)
    } else {
        (result, "".to_owned())
    }
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
