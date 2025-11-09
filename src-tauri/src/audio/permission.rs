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
    use cocoa::appkit::NSApplication;
    use cocoa::base::nil;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let av_capture_device: *mut objc::runtime::Object =
            msg_send![class!(AVCaptureDevice), class];

        if av_capture_device.is_null() {
            return PermissionStatus::NotDetermined;
        }

        let auth_status: isize = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: cocoa::foundation::NSString::alloc(nil)
                .init_str("AVMediaTypeAudio")
        ];

        match auth_status {
            0 => PermissionStatus::NotDetermined,
            1 => PermissionStatus::Restricted,
            2 => PermissionStatus::Denied,
            3 => PermissionStatus::Granted,
            _ => PermissionStatus::NotDetermined,
        }
    }
}

#[cfg(target_os = "macos")]
async fn request_macos_permission() -> Result<PermissionStatus, String> {
    use cocoa::base::nil;
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;

    let status = Arc::new(Mutex::new(PermissionStatus::NotDetermined));
    let status_clone = status.clone();

    // Request permission on main thread
    unsafe {
        let media_type = NSString::alloc(nil).init_str("AVMediaTypeAudio");

        // Note: This is a simplified version. In production, you'd use proper async callback
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: media_type
            completionHandler: |granted: bool| {
                let mut s = status_clone.lock().unwrap();
                *s = if granted {
                    PermissionStatus::Granted
                } else {
                    PermissionStatus::Denied
                };
            }
        ];
    }

    // Wait for callback (timeout after 10 seconds)
    for _ in 0..100 {
        sleep(Duration::from_millis(100)).await;
        let current_status = *status.lock().unwrap();
        if current_status != PermissionStatus::NotDetermined {
            return Ok(current_status);
        }
    }

    Ok(PermissionStatus::NotDetermined)
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
