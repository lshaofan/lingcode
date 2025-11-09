use tauri::State;
use std::sync::Arc;
use crate::db::Database;

/// 设置开机自启动
#[tauri::command]
pub fn set_auto_launch(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let app_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get app path: {}", e))?;

        let app_bundle = app_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .ok_or_else(|| "Failed to locate app bundle".to_string())?;

        if enable {
            // 使用 osascript 添加登录项
            let script = format!(
                r#"tell application "System Events" to make login item at end with properties {{path:"{}", hidden:false}}"#,
                app_bundle.display()
            );

            let output = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("Failed to enable auto launch: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "Failed to enable auto launch: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        } else {
            // 使用 osascript 删除登录项
            let app_name = app_bundle
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Failed to get app name".to_string())?;

            let script = format!(
                r#"tell application "System Events" to delete login item "{}""#,
                app_name
            );

            let output = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("Failed to disable auto launch: {}", e))?;

            if !output.status.success() {
                // 删除不存在的登录项不算错误
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("not found") {
                    return Err(format!("Failed to disable auto launch: {}", stderr));
                }
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Auto launch is only supported on macOS".to_string())
    }
}

/// 获取开机自启动状态
#[tauri::command]
pub fn get_auto_launch() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let app_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get app path: {}", e))?;

        let app_bundle = app_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .ok_or_else(|| "Failed to locate app bundle".to_string())?;

        let app_name = app_bundle
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Failed to get app name".to_string())?;

        let script = r#"tell application "System Events" to get the name of every login item"#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to get login items: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to get login items: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(app_name))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Auto launch is only supported on macOS".to_string())
    }
}

/// 列出可用的音频输入设备
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to get input devices: {}", e))?;

    let mut result = Vec::new();
    for (index, device) in devices.enumerate() {
        let name = device
            .name()
            .unwrap_or_else(|_| format!("Unknown Device {}", index));

        result.push(AudioDevice {
            id: format!("device_{}", index),
            name,
            is_default: false, // 会在下面更新
        });
    }

    // 标记默认设备
    if let Some(default_device) = host.default_input_device() {
        if let Ok(default_name) = default_device.name() {
            for device in &mut result {
                if device.name == default_name {
                    device.is_default = true;
                    break;
                }
            }
        }
    }

    Ok(result)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}