use cocoa::base::{id, nil};
use cocoa::foundation::{NSPoint, NSRect};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use std::ptr;

/// Information about the currently focused text element
#[derive(Debug, Clone)]
pub struct TextCursorInfo {
    /// Screen coordinates of the cursor (x, y)
    pub position: (f64, f64),
    /// Whether a text field is currently focused
    pub is_text_field_focused: bool,
}

/// Get information about the currently focused text element and cursor position
pub fn get_text_cursor_info() -> Result<TextCursorInfo, String> {
    unsafe {
        // Get the system-wide focused element using AX API
        let focused_element = get_focused_text_element()?;

        if focused_element.is_null() {
            return Ok(TextCursorInfo {
                position: (0.0, 0.0),
                is_text_field_focused: false,
            });
        }

        // Check if it's a text field
        let is_text_field = is_text_element(focused_element);

        if !is_text_field {
            release_ax_element(focused_element);
            return Ok(TextCursorInfo {
                position: (0.0, 0.0),
                is_text_field_focused: false,
            });
        }

        // Get the cursor position (insertion point)
        let cursor_pos = get_cursor_position(focused_element)?;

        release_ax_element(focused_element);

        Ok(TextCursorInfo {
            position: cursor_pos,
            is_text_field_focused: true,
        })
    }
}

unsafe fn get_focused_text_element() -> Result<id, String> {
    // Import AX framework symbols
    let ax_lib = libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
        .map_err(|e| format!("Failed to load AX framework: {}", e))?;

    type AXUIElementCreateSystemWideType = unsafe extern "C" fn() -> id;
    type AXUIElementCopyAttributeValueType = unsafe extern "C" fn(id, CFStringRef, *mut id) -> i32;

    let create_system_wide: libloading::Symbol<AXUIElementCreateSystemWideType> = ax_lib
        .get(b"AXUIElementCreateSystemWide")
        .map_err(|e| format!("Failed to get AXUIElementCreateSystemWide: {}", e))?;

    let copy_attr: libloading::Symbol<AXUIElementCopyAttributeValueType> = ax_lib
        .get(b"AXUIElementCopyAttributeValue")
        .map_err(|e| format!("Failed to get AXUIElementCopyAttributeValue: {}", e))?;

    let system_wide = create_system_wide();
    if system_wide.is_null() {
        return Err("Failed to create system-wide accessibility element".to_string());
    }

    let mut focused_app: id = ptr::null_mut();
    let focused_app_key = CFString::new("AXFocusedApplication");
    let result = copy_attr(system_wide, focused_app_key.as_concrete_TypeRef(), &mut focused_app);

    if result != 0 || focused_app.is_null() {
        return Err("No focused application found".to_string());
    }

    let mut focused_element: id = ptr::null_mut();
    let focused_elem_key = CFString::new("AXFocusedUIElement");
    let result = copy_attr(focused_app, focused_elem_key.as_concrete_TypeRef(), &mut focused_element);

    if result != 0 || focused_element.is_null() {
        return Err("No focused UI element found".to_string());
    }

    Ok(focused_element)
}

unsafe fn is_text_element(element: id) -> bool {
    if element.is_null() {
        return false;
    }

    let ax_lib = match libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices") {
        Ok(lib) => lib,
        Err(_) => return false,
    };

    type AXUIElementCopyAttributeValueType = unsafe extern "C" fn(id, CFStringRef, *mut id) -> i32;
    let copy_attr: libloading::Symbol<AXUIElementCopyAttributeValueType> = match ax_lib.get(b"AXUIElementCopyAttributeValue") {
        Ok(f) => f,
        Err(_) => return false,
    };

    // Check the role of the element
    let mut role_value: id = ptr::null_mut();
    let role_key = CFString::new("AXRole");
    let result = copy_attr(element, role_key.as_concrete_TypeRef(), &mut role_value);

    if result != 0 || role_value.is_null() {
        println!("[Accessibility] Failed to get role for focused element");
        return false;
    }

    let role_str: CFString = CFString::wrap_under_get_rule(role_value as CFStringRef);
    let role = role_str.to_string();

    // Debug: Always print the role to help identify new types
    println!("[Accessibility] Focused element role: {}", role);

    // 扩展的文本字段角色列表，支持更多类型的输入控件
    let is_known_text_field = matches!(
        role.as_str(),
        // 标准文本输入控件
        "AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField" |
        // 富文本编辑器
        "AXTextView" | "AXStaticText" |
        // Web 内容和浏览器
        "AXWebArea" | "AXGroup" | "AXScrollArea" |
        // Electron 和其他应用
        "AXSplitGroup" | "AXLayoutArea"
    );

    if is_known_text_field {
        println!("[Accessibility] ✅ Role '{}' is a known text field type", role);
        return true;
    }

    // Fallback: 检查是否有 AXValue 属性（可编辑的标志）
    let mut value: id = ptr::null_mut();
    let value_key = CFString::new("AXValue");
    let has_value = copy_attr(element, value_key.as_concrete_TypeRef(), &mut value) == 0 && !value.is_null();

    // Fallback: 检查是否有 AXSelectedText 属性
    let mut selected_text: id = ptr::null_mut();
    let selected_text_key = CFString::new("AXSelectedText");
    let has_selected_text = copy_attr(element, selected_text_key.as_concrete_TypeRef(), &mut selected_text) == 0;

    if has_value || has_selected_text {
        println!("[Accessibility] ✅ Role '{}' has text editing attributes", role);
        return true;
    }

    println!("[Accessibility] ⚠️ Role '{}' not recognized, will try fallback method", role);
    false
}

unsafe fn get_cursor_position(element: id) -> Result<(f64, f64), String> {
    let ax_lib = libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
        .map_err(|e| format!("Failed to load AX framework: {}", e))?;

    type AXUIElementCopyAttributeValueType = unsafe extern "C" fn(id, CFStringRef, *mut id) -> i32;
    let copy_attr: libloading::Symbol<AXUIElementCopyAttributeValueType> = ax_lib
        .get(b"AXUIElementCopyAttributeValue")
        .map_err(|e| format!("Failed to get AXUIElementCopyAttributeValue: {}", e))?;

    // First, try to get the selected text range
    let mut selected_range: id = ptr::null_mut();
    let selected_range_key = CFString::new("AXSelectedTextRange");
    let result = copy_attr(element, selected_range_key.as_concrete_TypeRef(), &mut selected_range);

    if result != 0 || selected_range.is_null() {
        // Fallback to element position
        return get_element_position(element);
    }

    // Get the bounds for the selected range
    let mut range_bounds: id = ptr::null_mut();
    let bounds_key = CFString::new("AXBoundsForRange");

    // Note: This is a parameterized attribute, which is more complex to call
    // For simplicity, we'll fall back to element position
    return get_element_position(element);
}

unsafe fn get_element_position(element: id) -> Result<(f64, f64), String> {
    let ax_lib = libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
        .map_err(|e| format!("Failed to load AX framework: {}", e))?;

    type AXUIElementCopyAttributeValueType = unsafe extern "C" fn(id, CFStringRef, *mut id) -> i32;
    let copy_attr: libloading::Symbol<AXUIElementCopyAttributeValueType> = ax_lib
        .get(b"AXUIElementCopyAttributeValue")
        .map_err(|e| format!("Failed to get AXUIElementCopyAttributeValue: {}", e))?;

    // Get position
    let mut position_value: id = ptr::null_mut();
    let position_key = CFString::new("AXPosition");
    let result = copy_attr(element, position_key.as_concrete_TypeRef(), &mut position_value);

    if result != 0 || position_value.is_null() {
        return Err("Failed to get element position".to_string());
    }

    // Get size
    let mut size_value: id = ptr::null_mut();
    let size_key = CFString::new("AXSize");
    let result = copy_attr(element, size_key.as_concrete_TypeRef(), &mut size_value);

    if result != 0 || size_value.is_null() {
        return Err("Failed to get element size".to_string());
    }

    // Extract NSPoint and NSSize from AXValue
    type AXValueGetValueType = unsafe extern "C" fn(id, i32, *mut NSPoint) -> bool;
    let get_value: libloading::Symbol<AXValueGetValueType> = ax_lib
        .get(b"AXValueGetValue")
        .map_err(|e| format!("Failed to get AXValueGetValue: {}", e))?;

    let mut point = NSPoint { x: 0.0, y: 0.0 };
    let kAXValueTypeCGPoint: i32 = 1; // From AXValue.h

    if !get_value(position_value, kAXValueTypeCGPoint, &mut point) {
        return Err("Failed to extract position value".to_string());
    }

    // Return the position (top-left corner of the text field)
    // We'll position the floating ball near this location
    Ok((point.x, point.y))
}

unsafe fn release_ax_element(element: id) {
    if !element.is_null() {
        let _: () = msg_send![element, release];
    }
}

/// Insert text at the current cursor position
pub fn insert_text_at_cursor(text: &str) -> Result<(), String> {
    println!("[Accessibility] Starting text insertion");

    // 🔑 关键：先激活之前保存的应用
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = crate::app_tracker::activate_previous_app() {
            println!("[Accessibility] Warning: Failed to activate previous app: {}", e);
            // 如果激活失败,等待更长时间让用户手动切换回去或让系统自动恢复焦点
            println!("[Accessibility] Waiting 300ms for focus to be restored...");
            std::thread::sleep(std::time::Duration::from_millis(300));
        } else {
            println!("[Accessibility] Successfully activated previous app");
            // 等待应用切换完成并获得焦点
            println!("[Accessibility] Waiting 200ms for app activation...");
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    // 策略：优先使用 AX API（不污染剪贴板），失败时才用剪贴板方式
    unsafe {
        // 尝试获取焦点元素
        match get_focused_text_element() {
            Ok(focused_element) if !focused_element.is_null() => {
                // 检查是否是文本字段
                if is_text_element(focused_element) {
                    println!("[Accessibility] Found text element, trying AX API insertion");
                    // 尝试使用 AX API 直接插入（不经过剪贴板）
                    let result = insert_text_to_element(focused_element, text);
                    release_ax_element(focused_element);

                    if result.is_ok() {
                        println!("[Accessibility] ✅ Successfully inserted using AX API (no clipboard)");
                        return Ok(());
                    } else {
                        println!("[Accessibility] ❌ AX API failed: {:?}", result);
                        return Err("无法使用 AX API 插入文字,拒绝使用剪贴板方式".to_string());
                    }
                } else {
                    release_ax_element(focused_element);
                    println!("[Accessibility] ❌ Focused element is not a text field");
                    return Err("焦点不在文本输入框中".to_string());
                }
            }
            Ok(_) => {
                // 处理 Ok 但返回空指针的情况
                println!("[Accessibility] ❌ Got Ok but element is null");
                return Err("没有检测到焦点元素或元素为空".to_string());
            }
            Err(e) => {
                println!("[Accessibility] ❌ Failed to get focused element: {}", e);
                return Err(format!("无法获取焦点元素: {}", e));
            }
        }
    }
}

unsafe fn insert_text_to_element(element: id, text: &str) -> Result<(), String> {
    use objc::{msg_send, sel, sel_impl};

    let ax_lib = libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
        .map_err(|e| format!("Failed to load AX framework: {}", e))?;

    type AXUIElementSetAttributeValueType = unsafe extern "C" fn(id, CFStringRef, id) -> i32;
    type AXUIElementCopyAttributeValueType = unsafe extern "C" fn(id, CFStringRef, *mut id) -> i32;

    let set_attr: libloading::Symbol<AXUIElementSetAttributeValueType> = ax_lib
        .get(b"AXUIElementSetAttributeValue")
        .map_err(|e| format!("Failed to get AXUIElementSetAttributeValue: {}", e))?;

    let copy_attr: libloading::Symbol<AXUIElementCopyAttributeValueType> = ax_lib
        .get(b"AXUIElementCopyAttributeValue")
        .map_err(|e| format!("Failed to get AXUIElementCopyAttributeValue: {}", e))?;

    // 🔑 方法1：使用 AXSelectedText 属性直接插入/替换文本
    let selected_text_key = CFString::new("AXSelectedText");
    let text_cf = CFString::new(text);

    println!("[Accessibility] Trying AXSelectedText...");
    let result = set_attr(
        element,
        selected_text_key.as_concrete_TypeRef(),
        text_cf.as_concrete_TypeRef() as id,
    );

    if result == 0 {
        // 返回成功，但验证是否真的插入了
        // 等待一小段时间让更新生效
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 尝试读取 AXValue 来验证
        let mut value: id = ptr::null_mut();
        let value_key = CFString::new("AXValue");
        let read_result = copy_attr(element, value_key.as_concrete_TypeRef(), &mut value);

        if read_result == 0 && !value.is_null() {
            let value_str: *const i8 = msg_send![value, UTF8String];
            if !value_str.is_null() {
                let value_cstr = std::ffi::CStr::from_ptr(value_str);
                if let Ok(value_string) = value_cstr.to_str() {
                    if value_string.contains(text) {
                        println!("[Accessibility] ✅ Verified: text successfully inserted via AXSelectedText");
                        return Ok(());
                    } else {
                        println!("[Accessibility] ⚠️ AXSelectedText returned success but text not found in AXValue");
                        println!("[Accessibility] This app doesn't properly support AXSelectedText");
                    }
                }
            }
        }
    } else {
        println!("[Accessibility] AXSelectedText failed with error {}", result);
    }

    // AXSelectedText 失败，说明这个应用不支持正确的 AX API
    // 直接返回错误，让上层使用剪贴板方式
    println!("[Accessibility] AX API not working properly for this app");
    Err("AX API not supported or not working".to_string())
}

/// Check if accessibility permissions are granted
pub fn check_accessibility_permission() -> bool {
    unsafe {
        let ax_lib = match libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices") {
            Ok(lib) => lib,
            Err(_) => return false,
        };

        type AXIsProcessTrustedType = unsafe extern "C" fn() -> bool;
        let is_trusted: libloading::Symbol<AXIsProcessTrustedType> = match ax_lib.get(b"AXIsProcessTrusted") {
            Ok(f) => f,
            Err(_) => return false,
        };

        is_trusted()
    }
}

/// Request accessibility permissions (opens System Preferences)
pub fn request_accessibility_permission() -> Result<(), String> {
    unsafe {
        let ax_lib = libloading::Library::new("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
            .map_err(|e| format!("Failed to load AX framework: {}", e))?;

        type AXIsProcessTrustedWithOptionsType = unsafe extern "C" fn(id) -> bool;
        let is_trusted_with_options: libloading::Symbol<AXIsProcessTrustedWithOptionsType> = ax_lib
            .get(b"AXIsProcessTrustedWithOptions")
            .map_err(|e| format!("Failed to get AXIsProcessTrustedWithOptions: {}", e))?;

        // Create options dictionary to show prompt
        let options: id = msg_send![class!(NSMutableDictionary), dictionary];
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value: id = msg_send![class!(NSNumber), numberWithBool: true];
        let _: () = msg_send![options, setObject:value forKey:key.as_concrete_TypeRef()];

        is_trusted_with_options(options);

        Ok(())
    }
}

/// Fallback method: Insert text using clipboard and paste simulation
/// This works for apps that don't support AX API (like browsers, Electron apps, etc.)
/// ⚠️ 注意：此方法会暂时污染剪贴板历史
fn insert_text_via_paste(text: &str) -> Result<(), String> {
    use std::process::Command;

    println!("[Accessibility] ⚠️  Using clipboard fallback (will appear in clipboard history)");

    // 1. Save current clipboard content
    let original_clipboard = Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());

    // 2. Copy new text to clipboard
    let copy_result = Command::new("sh")
        .arg("-c")
        .arg(format!("printf '%s' {} | pbcopy", shell_escape(text)))
        .status();

    if copy_result.is_err() || !copy_result.unwrap().success() {
        println!("[Accessibility] Failed to copy to clipboard");
        return Err("Failed to copy text to clipboard".to_string());
    }

    // 3. Wait a bit for clipboard to update
    std::thread::sleep(std::time::Duration::from_millis(30));

    // 4. Simulate Cmd+V to paste
    let paste_result = simulate_paste_shortcut();

    // 5. 快速恢复原剪贴板内容（减少污染时间窗口）
    if let Some(original) = original_clipboard {
        std::thread::spawn(move || {
            // 缩短等待时间，减少在剪贴板历史中停留的时间
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = Command::new("sh")
                .arg("-c")
                .arg(format!("printf '%s' {} | pbcopy", shell_escape(&original)))
                .status();
            println!("[Accessibility] Restored original clipboard content");
        });
    }

    paste_result
}

/// Escape string for shell command
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace("'", "'\\''"))
}

/// Simulate Cmd+V keyboard shortcut using CGEvent
fn simulate_paste_shortcut() -> Result<(), String> {
    unsafe {
        let core_graphics = libloading::Library::new("/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics")
            .map_err(|e| format!("Failed to load CoreGraphics: {}", e))?;

        type CGEventCreateKeyboardEventType = unsafe extern "C" fn(id, u16, bool) -> id;
        type CGEventPostType = unsafe extern "C" fn(u32, id);
        type CGEventSetFlagsType = unsafe extern "C" fn(id, u64);

        let create_event: libloading::Symbol<CGEventCreateKeyboardEventType> = core_graphics
            .get(b"CGEventCreateKeyboardEvent")
            .map_err(|e| format!("Failed to get CGEventCreateKeyboardEvent: {}", e))?;

        let post_event: libloading::Symbol<CGEventPostType> = core_graphics
            .get(b"CGEventPost")
            .map_err(|e| format!("Failed to get CGEventPost: {}", e))?;

        let set_flags: libloading::Symbol<CGEventSetFlagsType> = core_graphics
            .get(b"CGEventSetFlags")
            .map_err(|e| format!("Failed to get CGEventSetFlags: {}", e))?;

        let kCGHIDEventTap: u32 = 0;
        let kCGEventFlagMaskCommand: u64 = 0x00100000; // Cmd key
        let v_key_code: u16 = 9; // V key

        // Create Cmd+V down event
        let event_down = create_event(ptr::null_mut(), v_key_code, true);
        set_flags(event_down, kCGEventFlagMaskCommand);
        post_event(kCGHIDEventTap, event_down);

        // Small delay
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create Cmd+V up event
        let event_up = create_event(ptr::null_mut(), v_key_code, false);
        set_flags(event_up, kCGEventFlagMaskCommand);
        post_event(kCGHIDEventTap, event_up);

        println!("[Accessibility] Simulated Cmd+V paste");
        Ok(())
    }
}
