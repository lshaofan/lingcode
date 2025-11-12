use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use image::GenericImageView;

// Embed the tray icon at compile time
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray/iconTemplate.png");

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    // Decode PNG image
    let image = image::load_from_memory(TRAY_ICON)
        .map_err(|e| tauri::Error::AssetNotFound(format!("Failed to load icon: {}", e)))?;
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8().into_raw();

    let icon = Image::new_owned(rgba, width, height);

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
