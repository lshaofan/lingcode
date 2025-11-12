use super::types::PermissionStatus;

/// Check microphone permission status
pub fn check_permission() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        check_macos_permission()
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On other platforms, assume granted if we can access the device
        PermissionStatus::Granted
    }
}

/// Request microphone permission
pub async fn request_permission() -> Result<PermissionStatus, String> {
    #[cfg(target_os = "macos")]
    {
        request_macos_permission().await
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(PermissionStatus::Granted)
    }
}

#[cfg(target_os = "macos")]
fn check_macos_permission() -> PermissionStatus {
    // 使用 tauri-plugin-macos-permissions 来准确检查麦克风权限
    use std::process::Command;

    // 使用 Swift/Objective-C 来直接检查 AVFoundation 权限状态
    // 执行一个简单的 Swift 脚本来检查权限
    let script = r#"
        import AVFoundation
        let status = AVCaptureDevice.authorizationStatus(for: .audio)
        switch status {
            case .authorized:
                print("granted")
            case .denied:
                print("denied")
            case .restricted:
                print("restricted")
            case .notDetermined:
                print("not_determined")
            @unknown default:
                print("not_determined")
        }
    "#;

    match Command::new("swift")
        .arg("-")
        .arg("-suppress-warnings")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(script.as_bytes());
            }

            if let Ok(output) = child.wait_with_output() {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                match result.as_str() {
                    "granted" => PermissionStatus::Granted,
                    "denied" => PermissionStatus::Denied,
                    "restricted" => PermissionStatus::Restricted,
                    "not_determined" => PermissionStatus::NotDetermined,
                    _ => {
                        // 如果 swift 命令失败，回退到 cpal 检查
                        fallback_check_permission()
                    }
                }
            } else {
                fallback_check_permission()
            }
        }
        Err(_) => {
            fallback_check_permission()
        }
    }
}

#[cfg(target_os = "macos")]
fn fallback_check_permission() -> PermissionStatus {
    // 回退方法：通过实际尝试列出音频设备来检查权限
    use cpal::traits::HostTrait;
    let host = cpal::default_host();

    match host.input_devices() {
        Ok(mut devices) => {
            if devices.next().is_some() {
                PermissionStatus::Granted
            } else {
                PermissionStatus::NotDetermined
            }
        }
        Err(_) => {
            PermissionStatus::Denied
        }
    }
}

#[cfg(target_os = "macos")]
async fn request_macos_permission() -> Result<PermissionStatus, String> {
    // Check current status first
    let current_status = check_macos_permission();

    if current_status == PermissionStatus::Granted {
        return Ok(current_status);
    }

    // 如果是 NotDetermined，尝试触发系统权限弹窗
    if current_status == PermissionStatus::NotDetermined {
        // 使用 Swift 来请求权限
        let script = r#"
            import AVFoundation
            import Dispatch

            let semaphore = DispatchSemaphore(value: 0)
            AVCaptureDevice.requestAccess(for: .audio) { granted in
                if granted {
                    print("granted")
                } else {
                    print("denied")
                }
                semaphore.signal()
            }
            semaphore.wait()
        "#;

        use std::process::Command;
        match Command::new("swift")
            .arg("-")
            .arg("-suppress-warnings")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(script.as_bytes());
                }

                if let Ok(output) = child.wait_with_output() {
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    match result.as_str() {
                        "granted" => return Ok(PermissionStatus::Granted),
                        "denied" => return Ok(PermissionStatus::Denied),
                        _ => {}
                    }
                }
            }
            Err(_) => {}
        }
    }

    // If permission is denied or restricted, ask user to manually grant permission
    Err(
        "麦克风权限未授权。请在系统设置中允许访问麦克风：\n系统设置 > 隐私与安全性 > 麦克风".to_string()
    )
}

/// Open system preferences to manually grant permission
pub fn open_system_preferences() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        open_macos_preferences()
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Not supported on this platform".to_string())
    }
}

#[cfg(target_os = "macos")]
fn open_macos_preferences() -> Result<(), String> {
    use std::process::Command;

    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        .spawn()
        .map_err(|e| format!("Failed to open system preferences: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permission() {
        let status = check_permission();
        assert!(matches!(
            status,
            PermissionStatus::Granted
                | PermissionStatus::Denied
                | PermissionStatus::NotDetermined
                | PermissionStatus::Restricted
        ));
    }
}
