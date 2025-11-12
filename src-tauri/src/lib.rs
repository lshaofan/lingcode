mod audio;
#[cfg(target_os = "macos")]
mod accessibility;
#[cfg(target_os = "macos")]
mod app_tracker;
mod commands;
mod config;
mod db;
mod funasr;
mod python;
mod shortcut;
mod tray;
mod whisper;

use commands::{
    audio::*, db::*, funasr::*, model::*, system::*, transcription::*, window::*,
};
use crate::commands::{
    check_accessibility_permission_cmd,
    request_accessibility_permission_cmd,
    insert_text_at_cursor_cmd,
};
use db::Database;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

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
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init());

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_macos_permissions::init());
    }

    builder
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
            // System commands
            set_auto_launch,
            get_auto_launch,
            list_audio_devices,
            // Model commands
            get_available_models,
            download_model,
            delete_model,
            get_downloaded_models,
            get_models_directory,
            setup_funasr_environment,
            // Transcription commands (Whisper)
            initialize_whisper,
            transcribe_audio,
            transcribe_last_recording,
            transcribe_audio_with_timestamps,
            get_current_model,
            // FunASR commands
            initialize_funasr,
            transcribe_last_recording_funasr,
            transcribe_audio_funasr,
            download_funasr_model,
            get_current_funasr_model,
            prewarm_funasr_cmd,
            // Audio commands
            initialize_audio_system,
            check_microphone_permission,
            request_microphone_permission,
            open_microphone_settings,
            get_audio_devices,
            set_audio_device,
            start_recording,
            stop_recording,
            pause_recording,
            resume_recording,
            get_recording_info,
            clear_audio_buffer,
            save_recording,
            get_recording_data,
            // Accessibility commands
            check_accessibility_permission_cmd,
            request_accessibility_permission_cmd,
            insert_text_at_cursor_cmd,
            // Shortcut commands
            shortcut::recording_window_ready,
        ])
        .setup(|app| {
            use crate::config::ConfigManager;
            use crate::funasr::{prewarm_funasr, quick_health_check};
            use tracing::{error, info};

            // Initialize database
            let db_path = get_db_path(&app.handle());
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let database = Database::new(db_path)
                .expect("Failed to initialize database");
            let db_arc = Arc::new(database);
            app.manage(db_arc.clone());

            // Load application configuration
            let config_manager = ConfigManager::new(db_arc.connection());
            let app_config = config_manager.load()
                .expect("Failed to load application config");

            info!("📋 Application config loaded: model_type={:?}, is_first_launch={}",
                  app_config.model_type, app_config.is_first_launch);

            // Check if this is first launch and emit event
            if app_config.is_first_launch {
                info!("👋 First launch detected");
                let _ = app.handle().emit("first-launch", ());
            }

            // Initialize Whisper state
            app.manage(WhisperState::new());

            // Initialize FunASR state
            app.manage(FunASRState::new());

            // Smart initialization based on configuration
            if app_config.model_type == config::ModelType::FunASR {
                info!("🔍 FunASR is configured, checking Python environment...");

                // Quick health check
                let app_handle = app.handle();
                let env_status = quick_health_check(&app_handle);
                info!("Python环境状态: {:?}", env_status.status);

                // 保存 status 用于后续判断
                let status = env_status.status.clone();

                // Emit environment status
                let _ = app_handle.emit("python-env-status", env_status);

                // 🚀 激进预热策略：只要启用了预热且环境就绪，就立即预热
                // 不再等待"非首次启动"，因为用户选择 FunASR 就应该获得最佳性能
                if app_config.enable_prewarming {
                    // 只有在环境已就绪时才预热（避免首次启动时安装依赖的同时预热）
                    if status == "ready" {
                        info!("🔥 Prewarming enabled and environment ready, starting background prewarming...");
                        prewarm_funasr(app_handle.clone());
                    } else {
                        info!("⏳ Prewarming enabled but environment not ready yet ({})", status);
                        info!("💡 Prewarming will start automatically after first successful transcription");
                    }
                } else {
                    info!("ℹ️ Prewarming disabled in config");
                }
            } else {
                info!("ℹ️ Whisper is configured, no Python environment needed");
            }

            // Initialize audio system with error handling (async)
            tauri::async_runtime::spawn(async {
                use commands::audio::initialize_audio_system;
                match initialize_audio_system().await {
                    Ok(_) => info!("✅ Audio system initialized successfully"),
                    Err(e) => error!("❌ Failed to initialize audio system: {}", e),
                }
            });

            // Load and apply microphone settings
            use crate::db::SettingsRepository;
            let settings_repo = SettingsRepository::new(db_arc.connection());
            if let Ok(Some(microphone_json)) = settings_repo.get("microphone") {
                // Parse JSON string (frontend stores values as JSON)
                if let Ok(microphone_device) = serde_json::from_str::<String>(&microphone_json) {
                    if microphone_device != "auto" {
                        use commands::audio::AUDIO_RECORDER;
                        info!("📱 Loading saved microphone device: {}", microphone_device);
                        let recorder = AUDIO_RECORDER.lock();
                        recorder.set_preferred_device(Some(microphone_device));
                    }
                }
            }

            // Note: Audio state is now managed via global static variables (AUDIO_RECORDER, LAST_RECORDING)
            // to avoid Send/Sync issues with cpal::Stream on macOS

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
