/// 应用跟踪模块
/// 用于记录和恢复用户的活跃应用，确保文本插入到正确的位置

use std::sync::Mutex;

// 全局状态：记录录音开始前的活跃应用
static PREVIOUS_APP: Mutex<Option<String>> = Mutex::new(None);

/// 获取当前活跃应用的 Bundle ID
pub fn get_frontmost_app() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            // NSWorkspace 类
            let workspace_class = class!(NSWorkspace);
            let workspace: id = msg_send![workspace_class, sharedWorkspace];
            let frontmost_app: id = msg_send![workspace, frontmostApplication];

            if frontmost_app == nil {
                return Err("No frontmost application found".to_string());
            }

            // 获取 bundle identifier
            let bundle_id: id = msg_send![frontmost_app, bundleIdentifier];
            if bundle_id == nil {
                return Err("Failed to get bundle identifier".to_string());
            }

            // 转换为 Rust String
            let bundle_id_ptr: *const i8 = msg_send![bundle_id, UTF8String];
            let bundle_id_cstr = std::ffi::CStr::from_ptr(bundle_id_ptr);
            let bundle_id_str = bundle_id_cstr.to_str()
                .map_err(|e| format!("Failed to convert bundle ID: {}", e))?
                .to_string();

            println!("[AppTracker] Current frontmost app: {}", bundle_id_str);
            Ok(bundle_id_str)
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Not supported on this platform".to_string())
    }
}

/// 保存当前活跃应用（在录音开始前调用）
pub fn save_current_app() -> Result<(), String> {
    let app_id = get_frontmost_app()?;
    let mut previous = PREVIOUS_APP.lock().unwrap();
    *previous = Some(app_id.clone());
    println!("[AppTracker] Saved frontmost app: {}", app_id);
    Ok(())
}

/// 激活之前保存的应用（在插入文本前调用）
pub fn activate_previous_app() -> Result<(), String> {
    let previous = PREVIOUS_APP.lock().unwrap();

    if let Some(bundle_id) = previous.as_ref() {
        println!("[AppTracker] Activating previous app: {}", bundle_id);
        activate_app_by_bundle_id(bundle_id)?;

        // 等待应用激活完成
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    } else {
        Err("No previous app saved".to_string())
    }
}

/// 根据 Bundle ID 激活应用
fn activate_app_by_bundle_id(bundle_id: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            // NSWorkspace 类
            let workspace_class = class!(NSWorkspace);
            let workspace: id = msg_send![workspace_class, sharedWorkspace];

            // 获取正在运行的应用列表
            let running_apps: id = msg_send![workspace, runningApplications];
            let count: usize = msg_send![running_apps, count];

            for i in 0..count {
                let app: id = msg_send![running_apps, objectAtIndex: i];
                let app_bundle_id: id = msg_send![app, bundleIdentifier];

                if app_bundle_id != nil {
                    let app_bundle_str: *const i8 = msg_send![app_bundle_id, UTF8String];
                    let app_bundle_cstr = std::ffi::CStr::from_ptr(app_bundle_str);

                    if let Ok(app_bundle_string) = app_bundle_cstr.to_str() {
                        if app_bundle_string == bundle_id {
                            // 找到了应用，激活它
                            // NSApplicationActivateIgnoringOtherApps = 1 << 1
                            let options: u64 = 1 << 1;
                            let _: bool = msg_send![app, activateWithOptions: options];
                            println!("[AppTracker] Successfully activated app: {}", bundle_id);
                            return Ok(());
                        }
                    }
                }
            }

            Err(format!("Failed to find running application with bundle ID: {}", bundle_id))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Not supported on this platform".to_string())
    }
}

/// 清除保存的应用信息
pub fn clear_saved_app() {
    let mut previous = PREVIOUS_APP.lock().unwrap();
    *previous = None;
}
