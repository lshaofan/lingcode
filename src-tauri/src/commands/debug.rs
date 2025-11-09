use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub fn list_windows<R: Runtime>(app: AppHandle<R>) -> Vec<String> {
    app.webview_windows()
        .iter()
        .map(|(label, _)| label.clone())
        .collect()
}
