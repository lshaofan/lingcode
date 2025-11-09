// mod audio;  // TODO: Uncomment when audio implementation is complete
// #[cfg(target_os = "macos")]
// mod accessibility;  // TODO: Re-enable when implementing text insertion
mod commands;
mod db;
mod shortcut;
mod tray;

use commands::{db::*, window::*};
use db::Database;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! 欢迎使用聆码！", name)
}

fn get_db_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data directory")
        .join("lingcode.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_setting,
            set_setting,
            get_all_settings,
            delete_setting,
            create_transcription,
            get_transcription,
            get_recent_transcriptions,
            search_transcriptions,
            delete_transcription,
            delete_all_transcriptions,
            show_recording_float,
            hide_recording_float,
            toggle_recording_float,
            set_float_ignore_cursor,
            resize_recording_float,
        ])
        .setup(|app| {
            // Initialize database
            let db_path = get_db_path(&app.handle());
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let database = Database::new(db_path)
                .expect("Failed to initialize database");
            app.manage(Arc::new(database));

            // Initialize system tray
            tray::create_tray(&app.handle())
                .expect("Failed to create system tray");

            // Register global shortcuts
            shortcut::register_shortcuts(&app.handle())
                .expect("Failed to register shortcuts");

            // Note: recording-float window will be created lazily on first shortcut press
            // This ensures the webview loads properly when the window is actually needed
            println!("[Setup] Recording float window will be created on first use");

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
