use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::Mutex;

// Recording state tracking
static RECORDING_STATE: Mutex<RecordingState> = Mutex::new(RecordingState::Idle);

#[derive(Debug, Clone, Copy, PartialEq)]
enum RecordingState {
    Idle,
    Recording,
    Processing,
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
            // Only respond to key press events (ignore release events)
            if event.state != ShortcutState::Pressed {
                return;
            }

            println!("[Shortcut] Shortcut pressed, toggling recording state");

            // Get current state and decide action
            let mut state = RECORDING_STATE.lock().unwrap();
            let current_state = *state;

            match current_state {
                RecordingState::Idle => {
                    // Start recording
                    println!("[Shortcut] State: Idle -> Recording");
                    *state = RecordingState::Recording;
                    drop(state); // Release lock before window operations

                    // Get or create the recording float window
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

                        let new_window = WebviewWindowBuilder::new(&app_handle, "recording-float", window_url)
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
                            use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
                            use cocoa::base::id;
                            use objc::{class, msg_send, sel, sel_impl};

                            let ns_window = new_window.ns_window().unwrap() as id;
                            unsafe {
                                ns_window.setLevel_(3);
                                let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;
                                ns_window.setCollectionBehavior_(behavior);
                                ns_window.setOpaque_(false);
                                let clear_color: id = msg_send![class!(NSColor), clearColor];
                                ns_window.setBackgroundColor_(clear_color);
                                let _: () = msg_send![ns_window, setHasShadow: false];
                                let _: () = msg_send![ns_window, setIgnoresMouseEvents: false];
                                println!("[Window] Configured macOS-specific window behavior with transparency");
                            }
                        }

                        new_window
                    };

                    // Position and show the window
                    let window_size = 300; // Window is 300x60
                    let spacing = 20; // Distance from cursor

                    // Get screen size and cursor position
                    let (window_x, window_y) = if let Ok(cursor_pos) = app.cursor_position() {
                        let cursor_x = cursor_pos.x as i32;
                        let cursor_y = cursor_pos.y as i32;

                        // Get primary monitor to determine screen dimensions
                        let screen_width = if let Some(monitor) = app.primary_monitor().ok().flatten() {
                            monitor.size().width as i32
                        } else {
                            3440 // Default fallback
                        };

                        let screen_height = if let Some(monitor) = app.primary_monitor().ok().flatten() {
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

                    // Show window and start recording
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("shortcut-start-recording", ());
                    println!("[Shortcut] Window shown, recording started");
                }
                RecordingState::Recording => {
                    // Stop recording
                    println!("[Shortcut] State: Recording -> Processing");
                    *state = RecordingState::Processing;
                    drop(state); // Release lock

                    if let Some(window) = app_handle.get_webview_window("recording-float") {
                        let _ = window.emit("shortcut-stop-recording", ());
                        println!("[Shortcut] Stop recording event emitted");

                        // Reset to idle state after processing
                        // In a real app, this would be done after transcription completes
                        *RECORDING_STATE.lock().unwrap() = RecordingState::Idle;
                    } else {
                        println!("[Shortcut] Warning: recording-float window not found");
                        *RECORDING_STATE.lock().unwrap() = RecordingState::Idle;
                    }
                }
                RecordingState::Processing => {
                    // Ignore shortcut during processing
                    println!("[Shortcut] State: Processing - ignoring shortcut");
                    drop(state);
                }
            }
        })
        .map_err(|e| e.to_string())?;

    println!("[Shortcut] Shortcut registration complete");
    Ok(())
}

pub fn unregister_all<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    Ok(())
}
