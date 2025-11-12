use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub fn show_recording_float<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("recording-float") {
        // 使用 macOS 原生 API 显示窗口而不激活
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSWindow;
            use cocoa::base::id;
            use objc::{msg_send, sel, sel_impl};

            let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;
            unsafe {
                let _: () = msg_send![ns_window, orderFrontRegardless];
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            window.show().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn hide_recording_float<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("recording-float") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_recording_float<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    println!("toggle_recording_float 命令被调用");

    if let Some(window) = app.get_webview_window("recording-float") {
        println!("找到了 recording-float 窗口");
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        println!("窗口当前可见性: {}", is_visible);

        if is_visible {
            println!("隐藏窗口");
            window.hide().map_err(|e| e.to_string())?;
        } else {
            println!("显示窗口，不抢夺焦点");
            // 使用 macOS 原生 API 显示窗口而不激活
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::NSWindow;
                use cocoa::base::id;
                use objc::{msg_send, sel, sel_impl};

                let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;
                unsafe {
                    let _: () = msg_send![ns_window, orderFrontRegardless];
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                window.show().map_err(|e| e.to_string())?;
            }
        }
    } else {
        println!("警告: 未找到 recording-float 窗口!");
        return Err("Recording float window not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn set_float_ignore_cursor(app: AppHandle, ignore: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("recording-float") {
        // Use Tauri's built-in method to ignore cursor events
        window
            .set_ignore_cursor_events(ignore)
            .map_err(|e| e.to_string())?;
        println!("[Window] Set ignore cursor events: {}", ignore);
    }
    Ok(())
}

#[tauri::command]
pub fn resize_recording_float<R: Runtime>(
    app: AppHandle<R>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("recording-float") {
        let size = tauri::Size::Physical(tauri::PhysicalSize {
            width: width as u32,
            height: height as u32,
        });
        window.set_size(size).map_err(|e| e.to_string())?;
        println!("[Window] Resized recording-float to {}x{}", width, height);
    }
    Ok(())
}
