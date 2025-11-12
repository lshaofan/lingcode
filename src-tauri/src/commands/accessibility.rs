#[cfg(target_os = "macos")]
use crate::accessibility::{
    check_accessibility_permission, insert_text_at_cursor, request_accessibility_permission,
};

/// Check if accessibility permissions are granted
#[tauri::command]
pub async fn check_accessibility_permission_cmd() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(check_accessibility_permission())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Non-macOS platforms don't need accessibility permissions
    }
}

/// Request accessibility permissions (opens System Preferences)
#[tauri::command]
pub async fn request_accessibility_permission_cmd() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        request_accessibility_permission()
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(()) // No-op on non-macOS platforms
    }
}

/// Insert text at the current cursor position
#[tauri::command]
pub async fn insert_text_at_cursor_cmd(text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        insert_text_at_cursor(&text)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, we could use clipboard + paste
        // For now, just return an error
        Err("Text insertion not supported on this platform".to_string())
    }
}
