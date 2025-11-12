use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::db::{Database, SettingsRepository};

// Recording state tracking
static RECORDING_STATE: Mutex<RecordingState> = Mutex::new(RecordingState::Idle);

// Track last frontend ready notification time to prevent duplicates
static LAST_FRONTEND_READY: Mutex<Option<Instant>> = Mutex::new(None);
const FRONTEND_READY_DEBOUNCE_MS: u64 = 500; // 500ms debounce window

#[derive(Debug, Clone, Copy, PartialEq)]
enum RecordingState {
    Idle,
    Recording,
    Processing,
}

// Helper function to get operation mode from settings
fn get_operation_mode(app: &AppHandle<impl Runtime>) -> String {
    // Try to get from database
    if let Some(db) = app.try_state::<Arc<Database>>() {
        let repo = SettingsRepository::new(db.connection());
        if let Ok(Some(mode_value)) = repo.get("operationMode") {
            if let Ok(mode) = serde_json::from_str::<String>(&mode_value) {
                println!("[Shortcut] Operation mode from DB: {}", mode);
                return mode;
            }
        }
    }

    // Default to preview mode for safety
    println!("[Shortcut] Using default operation mode: preview");
    "preview".to_string()
}

pub fn register_shortcuts<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // Register default shortcut: Cmd+Shift+S (macOS) or Ctrl+Shift+S (other platforms)
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::META | Modifiers::SHIFT
    } else {
        Modifiers::CONTROL | Modifiers::SHIFT
    };

    let shortcut = Shortcut::new(Some(modifiers), Code::KeyS);
    println!("[Shortcut] Registering shortcut: {:?}", shortcut);

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            // Get operation mode
            let operation_mode = get_operation_mode(&app_handle);
            let is_direct_mode = operation_mode == "direct";

            println!("[Shortcut] Shortcut event: {:?}, Mode: {}", event.state, operation_mode);

            // Get current state
            let mut state = RECORDING_STATE.lock().unwrap();
            let current_state = *state;

            // Handle different modes
            if is_direct_mode {
                // Direct mode: Press to start, Release to stop
                match (event.state, current_state) {
                    (ShortcutState::Pressed, RecordingState::Idle) => {
                        // Start recording on press
                        println!("[Shortcut] Direct mode: Press detected -> Start recording");
                        *state = RecordingState::Recording;
                        drop(state);

                        // Create/show window and start recording
                        handle_start_recording(&app_handle);
                    }
                    (ShortcutState::Released, RecordingState::Recording) => {
                        // Stop recording on release
                        println!("[Shortcut] Direct mode: Release detected -> Stop recording");
                        *state = RecordingState::Processing;
                        drop(state);

                        // Stop recording
                        handle_stop_recording(&app_handle);
                    }
                    _ => {
                        // Ignore other combinations
                        drop(state);
                    }
                }
            } else {
                // Preview mode: Toggle window visibility on press
                if event.state != ShortcutState::Pressed {
                    drop(state);
                    return;
                }

                println!("[Shortcut] Preview mode: Toggle window visibility");

                // Check if window is visible
                if let Some(window) = app_handle.get_webview_window("recording-float") {
                    // Window exists - toggle visibility
                    if window.is_visible().unwrap_or(false) {
                        println!("[Shortcut] Window visible - hiding and stopping recording");
                        let _ = window.hide();
                        let _ = window.emit("shortcut-stop-recording", ());
                        *state = RecordingState::Idle;
                        drop(state);
                    } else {
                        println!("[Shortcut] Window hidden - showing and starting recording");
                        *state = RecordingState::Recording;
                        drop(state);
                        handle_start_recording(&app_handle);
                    }
                } else {
                    // Window doesn't exist - create and start recording
                    println!("[Shortcut] Window doesn't exist - creating and starting recording");
                    *state = RecordingState::Recording;
                    drop(state);
                    handle_start_recording(&app_handle);
                }
            }
        })
        .map_err(|e| e.to_string())?;

    println!("[Shortcut] Shortcut registration complete");
    Ok(())
}

fn handle_start_recording<R: Runtime>(app_handle: &AppHandle<R>) {
    // 🚨 CRITICAL: 首先检查麦克风权限，如果没有授权则阻止录音
    use crate::audio::check_permission;
    let permission = check_permission();
    println!("[Shortcut] Checking microphone permission: {:?}", permission);

    if permission != crate::audio::PermissionStatus::Granted {
        println!("[Shortcut] ❌ Microphone permission not granted, blocking recording");
        // 发送事件到前端显示错误提示
        if let Some(main_window) = app_handle.get_webview_window("main") {
            let _ = main_window.emit("permission-error", "microphone");
        }
        return;
    }

    println!("[Shortcut] ✅ Microphone permission granted, proceeding with recording");

    // 🔑 关键：在显示窗口前，先保存当前活跃的应用
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = crate::app_tracker::save_current_app() {
            println!("[Shortcut] Warning: Failed to save current app: {}", e);
        }
    }

    // 🎯 新架构：不再在后端启动录音
    // 前端会通过 getUserMedia 自己启动录音，确保麦克风图标正确显示
    println!("[Shortcut] 📱 Preparing to show window - frontend will handle audio capture");

    // Get or create the recording float window
    let is_new_window = app_handle.get_webview_window("recording-float").is_none();

    let window = if let Some(existing_window) = app_handle.get_webview_window("recording-float") {
        println!("[Shortcut] Using existing recording-float window");
        existing_window
    } else {
        println!("[Shortcut] Creating new recording-float window");
        use tauri::{WebviewUrl, WebviewWindowBuilder};

        // 在开发模式和生产模式都使用主入口,根据窗口label在前端决定渲染内容
        let window_url = if cfg!(debug_assertions) {
            WebviewUrl::External("http://localhost:1420/".parse().unwrap())
        } else {
            WebviewUrl::App("index.html".into())
        };

        let new_window = WebviewWindowBuilder::new(app_handle, "recording-float", window_url)
            .title("Recording Float")
            .inner_size(300.0, 70.0)  // Initial small size for idle/recording state
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .focused(false)
            .accept_first_mouse(true)
            .build()
            .expect("Failed to create recording float window");

        // Configure macOS-specific window behavior
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
            use cocoa::base::id;
            use objc::{class, msg_send, sel, sel_impl};

            let ns_window = new_window.ns_window().unwrap() as id;
            unsafe {
                // 设置窗口层级为浮动窗口（NSFloatingWindowLevel = 3）
                ns_window.setLevel_(3);

                // 设置窗口行为
                let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;
                ns_window.setCollectionBehavior_(behavior);

                // 设置透明背景
                ns_window.setOpaque_(false);
                let clear_color: id = msg_send![class!(NSColor), clearColor];
                ns_window.setBackgroundColor_(clear_color);
                let _: () = msg_send![ns_window, setHasShadow: false];

                // 🔑 关键修复：设置窗口样式，阻止它接受键盘焦点
                // NSWindowStyleMaskNonactivatingPanel = 1 << 7 = 128 (0x80)
                let current_style: u64 = msg_send![ns_window, styleMask];
                let non_activating_panel_mask: u64 = 1 << 7; // 128
                let new_style = current_style | non_activating_panel_mask;
                let _: () = msg_send![ns_window, setStyleMask: new_style];

                // 防止窗口接受鼠标移动事件（但允许点击）
                let _: () = msg_send![ns_window, setAcceptsMouseMovedEvents: false];
                let _: () = msg_send![ns_window, setIgnoresMouseEvents: false];

                println!("[Window] Configured macOS-specific window behavior (non-activating panel, transparent)");
            }
        }

        new_window
    };

    // Position and show the window
    let window_size = 300; // Window is 300x60
    let spacing = 20; // Distance from cursor

    // Get screen size and cursor position
    let (window_x, window_y) = if let Ok(cursor_pos) = app_handle.cursor_position() {
        let cursor_x = cursor_pos.x as i32;
        let cursor_y = cursor_pos.y as i32;

        // Get primary monitor to determine screen dimensions
        let screen_width = if let Some(monitor) = app_handle.primary_monitor().ok().flatten() {
            monitor.size().width as i32
        } else {
            3440 // Default fallback
        };

        let screen_height = if let Some(monitor) = app_handle.primary_monitor().ok().flatten() {
            monitor.size().height as i32
        } else {
            1440 // Default fallback
        };

        // Determine which quadrant the cursor is in
        let is_upper_half = cursor_y < screen_height / 2;
        let is_left_half = cursor_x < screen_width / 2;

        // Calculate position based on quadrant
        let x = if is_left_half {
            // Left half: position to the right of cursor
            cursor_x + spacing
        } else {
            // Right half: position to the left of cursor
            cursor_x - window_size - spacing
        };

        let y = if is_upper_half {
            // Upper half: position below cursor
            cursor_y + spacing
        } else {
            // Lower half: position above cursor
            cursor_y - 60 - spacing  // Window height is 60px
        };

        println!("[Shortcut] Screen: {}x{}, Cursor: ({}, {}), Quadrant: {}, {}",
            screen_width, screen_height, cursor_x, cursor_y,
            if is_left_half { "left" } else { "right" },
            if is_upper_half { "upper" } else { "lower" });

        (x, y)
    } else {
        (0, 0)
    };

    println!("[Shortcut] Positioning window at ({}, {}) near mouse cursor", window_x, window_y);

    let _ = window.set_position(tauri::Position::Physical(
        tauri::PhysicalPosition { x: window_x, y: window_y }
    ));

    // 🚀 CRITICAL FIX: For NEW windows, wait for frontend ready signal
    // For EXISTING windows, directly emit start recording event
    if is_new_window {
        println!("[Shortcut] New window - will wait for frontend ready signal");
        // Frontend will notify when ready, then recording_window_ready command will emit start event
    } else {
        println!("[Shortcut] Existing window - directly emitting start recording event");
        // Window already exists, React component is already mounted (but may be in background)
        // Directly emit start recording event
        let _ = app_handle.emit("shortcut-start-recording", ());
        println!("[Shortcut] ✅ Start recording event emitted for existing window");
    }

    // Show window without stealing focus using macOS native API
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSWindow;
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};

        let ns_window = window.ns_window().unwrap() as id;
        unsafe {
            // 使用 orderFrontRegardless 而不是 makeKeyAndOrderFront
            // 这样窗口会显示但不会抢夺焦点
            let _: () = msg_send![ns_window, orderFrontRegardless];
        }
        println!("[Shortcut] Window shown using orderFrontRegardless (non-activating)");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.show();
        println!("[Shortcut] Window shown");
    }
}

fn handle_stop_recording<R: Runtime>(app_handle: &AppHandle<R>) {
    // 🚀 CRITICAL FIX: Use global event channel to avoid window event listener issues
    // When window is hidden/shown repeatedly, window.emit() may not work reliably
    // Using app_handle.emit() with global event listener is more reliable
    println!("[Shortcut] 🎯 Emitting shortcut-stop-recording event globally");
    match app_handle.emit("shortcut-stop-recording", ()) {
        Ok(_) => println!("[Shortcut] ✅ Stop recording event emitted successfully (global)"),
        Err(e) => println!("[Shortcut] ❌ Failed to emit global stop recording event: {:?}", e),
    }

    // Also try emitting to window as fallback
    if let Some(window) = app_handle.get_webview_window("recording-float") {
        let _ = window.emit("shortcut-stop-recording", ());
    }

    // Reset to idle state after processing
    // In a real app, this would be done after transcription completes
    *RECORDING_STATE.lock().unwrap() = RecordingState::Idle;
}

pub fn unregister_all<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Command called by frontend when recording window is ready
#[tauri::command]
pub fn recording_window_ready(app: AppHandle) -> Result<(), String> {
    println!("[Shortcut] Frontend ready signal received");

    // 🚀 CRITICAL FIX: Debounce frontend ready signals to prevent duplicate start events
    // React StrictMode can cause component to mount multiple times, sending multiple ready signals
    let mut last_ready = LAST_FRONTEND_READY.lock().unwrap();
    let now = Instant::now();

    if let Some(last_time) = *last_ready {
        let elapsed = now.duration_since(last_time);
        if elapsed < Duration::from_millis(FRONTEND_READY_DEBOUNCE_MS) {
            println!("[Shortcut] ⚠️  Ignoring duplicate frontend ready signal (within {}ms debounce window)", FRONTEND_READY_DEBOUNCE_MS);
            return Ok(());
        }
    }

    *last_ready = Some(now);
    drop(last_ready); // Release lock

    println!("[Shortcut] ✅ Processing frontend ready signal");

    // Check current state and send appropriate event
    let mut state = RECORDING_STATE.lock().unwrap();
    let current_state = *state;

    if let Some(window) = app.get_webview_window("recording-float") {
        match current_state {
            RecordingState::Recording => {
                // Normal case: user is still holding the key, start recording
                println!("[Shortcut] ✅ User still holding key, emitting start recording event");
                drop(state); // Release lock before emit
                let _ = window.emit("shortcut-start-recording", ());
                println!("[Shortcut] ✅ Start recording event sent after frontend ready");
            }
            RecordingState::Processing | RecordingState::Idle => {
                // User released the key before window was ready
                // Reset state to idle (it's already Processing or Idle)
                // Note: We don't emit stop event here - user already released, no recording happened
                println!("[Shortcut] ⚠️  User released key too quickly (before window ready), window will auto-hide");
            }
        }
    }

    Ok(())
}
