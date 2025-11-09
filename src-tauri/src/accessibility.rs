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

    // Common text field roles
    let is_text_field = matches!(
        role.as_str(),
        "AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField"
    );

    if !is_text_field {
        println!("[Accessibility] Role '{}' is not recognized as a text field", role);
    }

    is_text_field
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
