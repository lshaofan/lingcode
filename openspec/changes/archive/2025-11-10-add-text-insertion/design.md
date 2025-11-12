# Design: 文本自动插入功能技术设计

## Context

文本自动插入是将转录文字自动输入到用户当前活动应用的关键功能。在 macOS 上实现文本插入有多种方案，需要根据权限、兼容性和可靠性选择最佳策略。

## Goals / Non-Goals

### Goals
- 实现自动文本插入到当前光标位置
- 支持主流应用（浏览器、编辑器、聊天工具）
- 插入延迟 < 100ms
- 提供多种插入策略和自动降级
- 保护用户剪贴板数据

### Non-Goals
- 不支持富文本插入（仅纯文本）
- 不支持自定义格式化
- 不支持跨设备同步
- 不支持插入历史撤销

## Architecture

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                   用户界面层                             │
│  ┌──────────────┐         ┌──────────────┐             │
│  │ 插入设置      │         │ 插入状态显示  │             │
│  └──────────────┘         └──────────────┘             │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                Tauri Commands (Rust)                    │
│  - insert_text()                                        │
│  - check_accessibility_permission()                     │
│  - get_active_application()                             │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│             文本插入引擎 (策略模式)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 策略 A:      │  │ 策略 B:      │  │ 策略 C:      │ │
│  │ 剪贴板粘贴    │  │ 键盘模拟     │  │ Accessibility│ │
│  │ (最兼容)     │  │ (快速)       │  │ (原生)       │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                          │                              │
│                          ▼                              │
│                   ┌──────────────┐                      │
│                   │ 自动降级      │                      │
│                   │ A → B → C    │                      │
│                   └──────────────┘                      │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
                 活动窗口应用
              (接收文本输入)
```

### 插入策略决策树

```
开始插入文本
    │
    ├─> 检查 Accessibility 权限
    │   ├─ 有权限 ─> 策略 C: Accessibility API
    │   └─ 无权限 ─> 检查应用兼容性
    │                 ├─ 兼容 ─> 策略 B: 键盘模拟
    │                 └─ 不兼容 ─> 策略 A: 剪贴板粘贴
    │
    └─> 执行插入
        ├─ 成功 ─> 显示成功提示
        └─ 失败 ─> 降级到下一策略
                   └─> 最终降级: 复制到剪贴板 + 提示用户手动粘贴
```

## Decisions

### 决策 1: 主要插入策略 - 剪贴板 vs 键盘模拟 vs Accessibility

**决策**: **剪贴板粘贴为主，其他为辅**

**原因**:
- **兼容性最好**: 99% 应用支持 Cmd+V
- **可靠性高**: 不依赖特殊权限
- **速度快**: 无需逐字符输入

**实现优先级**:
1. 剪贴板粘贴（默认）
2. 键盘模拟（特殊情况）
3. Accessibility API（最优但需权限）

### 决策 2: 剪贴板备份方案

**决策**: **每次插入前备份，插入后延迟恢复**

**原因**:
- 保护用户数据
- 防止覆盖重要内容
- 延迟恢复避免竞态

**实现**:
```
1. 读取当前剪贴板内容 -> backup
2. 写入转录文本到剪贴板
3. 模拟 Cmd+V 粘贴
4. 延迟 100ms 后恢复 backup
```

### 决策 3: 长文本处理

**决策**: **分段插入** (超过 1000 字符)

**原因**:
- 避免剪贴板溢出
- 减少系统压力
- 提升可靠性

**实现**:
```rust
if text.len() > 1000 {
    for chunk in text.chunks(1000) {
        insert_chunk(chunk);
        std::thread::sleep(Duration::from_millis(50));
    }
}
```

### 决策 4: 应用兼容性检测

**决策**: 维护**兼容性数据库**

**原因**:
- 不同应用行为不同
- 提前规避已知问题
- 优化插入策略

**数据结构**:
```rust
struct AppCompatibility {
    bundle_id: String,
    name: String,
    preferred_method: InsertionMethod,
    known_issues: Vec<String>,
}

enum InsertionMethod {
    Clipboard,      // 剪贴板粘贴
    Keyboard,       // 键盘模拟
    Accessibility,  // Accessibility API
}
```

## Implementation Details

### 1. 活动窗口检测 (macOS)

```rust
// src-tauri/src/insertion/active_window.rs

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use cocoa::appkit::NSWorkspace;
use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

pub struct ActiveApplication {
    pub bundle_id: String,
    pub name: String,
    pub pid: i32,
}

pub fn get_active_application() -> Result<ActiveApplication, String> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let active_app: id = msg_send![workspace, frontmostApplication];

        if active_app == nil {
            return Err("No active application".to_string());
        }

        // 获取 Bundle ID
        let bundle_id: id = msg_send![active_app, bundleIdentifier];
        let bundle_id_str: String = nsstring_to_string(bundle_id);

        // 获取应用名称
        let name: id = msg_send![active_app, localizedName];
        let name_str: String = nsstring_to_string(name);

        // 获取 PID
        let pid: i32 = msg_send![active_app, processIdentifier];

        Ok(ActiveApplication {
            bundle_id: bundle_id_str,
            name: name_str,
            pid,
        })
    }
}

unsafe fn nsstring_to_string(ns_string: id) -> String {
    let utf8: *const u8 = msg_send![ns_string, UTF8String];
    let len = msg_send![ns_string, lengthOfBytesUsingEncoding: 4]; // UTF8 = 4
    let bytes = std::slice::from_raw_parts(utf8, len);
    String::from_utf8_lossy(bytes).to_string()
}
```

### 2. 剪贴板插入策略

```rust
// src-tauri/src/insertion/strategies/clipboard.rs

use clipboard::{ClipboardProvider, ClipboardContext};
use enigo::{Enigo, Key, KeyboardControllable};
use std::thread;
use std::time::Duration;

pub struct ClipboardInsertion;

impl ClipboardInsertion {
    pub fn insert(text: &str) -> Result<(), String> {
        // 1. 备份剪贴板
        let backup = Self::backup_clipboard()?;

        // 2. 写入文本到剪贴板
        Self::set_clipboard(text)?;

        // 3. 等待剪贴板更新
        thread::sleep(Duration::from_millis(50));

        // 4. 模拟 Cmd+V
        Self::simulate_paste()?;

        // 5. 延迟恢复剪贴板
        thread::sleep(Duration::from_millis(100));
        Self::restore_clipboard(&backup)?;

        Ok(())
    }

    fn backup_clipboard() -> Result<String, String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| e.to_string())?;

        ctx.get_contents()
            .unwrap_or_else(|_| String::new())
            .into()
    }

    fn set_clipboard(text: &str) -> Result<(), String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| e.to_string())?;

        ctx.set_contents(text.to_string())
            .map_err(|e| e.to_string())
    }

    fn restore_clipboard(backup: &str) -> Result<(), String> {
        if !backup.is_empty() {
            Self::set_clipboard(backup)?;
        }
        Ok(())
    }

    fn simulate_paste() -> Result<(), String> {
        let mut enigo = Enigo::new();

        // macOS: Cmd+V
        enigo.key_down(Key::Meta); // Cmd
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Meta);

        Ok(())
    }
}
```

### 3. 键盘模拟插入策略

```rust
// src-tauri/src/insertion/strategies/keyboard.rs

use enigo::{Enigo, KeyboardControllable};
use std::thread;
use std::time::Duration;

pub struct KeyboardInsertion;

impl KeyboardInsertion {
    pub fn insert(text: &str) -> Result<(), String> {
        let mut enigo = Enigo::new();

        // 处理特殊字符
        let text = Self::escape_special_chars(text);

        // 逐字符输入（对于短文本）
        if text.len() < 100 {
            for ch in text.chars() {
                enigo.key_sequence(&ch.to_string());
                thread::sleep(Duration::from_millis(5)); // 防止输入过快
            }
        } else {
            // 长文本分段
            for chunk in text.as_bytes().chunks(100) {
                let chunk_str = String::from_utf8_lossy(chunk);
                enigo.key_sequence(&chunk_str);
                thread::sleep(Duration::from_millis(20));
            }
        }

        Ok(())
    }

    fn escape_special_chars(text: &str) -> String {
        // 转义特殊字符（如换行符）
        text.replace('\n', "\\n")
            .replace('\t', "\\t")
    }
}
```

### 4. Accessibility API 插入策略

```rust
// src-tauri/src/insertion/strategies/accessibility.rs

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use accessibility::AXUIElement;

pub struct AccessibilityInsertion;

impl AccessibilityInsertion {
    pub fn insert(text: &str) -> Result<(), String> {
        // 获取当前聚焦的元素
        let focused_element = Self::get_focused_element()?;

        // 直接设置文本值
        Self::set_text_value(&focused_element, text)?;

        Ok(())
    }

    fn get_focused_element() -> Result<AXUIElement, String> {
        // 使用 Accessibility API 获取聚焦元素
        // 需要 macOS Accessibility 权限
        unsafe {
            let system_wide = AXUIElement::system_wide();
            // 实现获取聚焦元素逻辑
            // ...
        }
        Err("Not implemented".to_string())
    }

    fn set_text_value(element: &AXUIElement, text: &str) -> Result<(), String> {
        let cf_text = CFString::new(text);
        // 设置 AXValue 属性
        // element.set_attribute("AXValue", &cf_text)?;
        Ok(())
    }
}
```

### 5. 插入引擎主控制器

```rust
// src-tauri/src/insertion/engine.rs

use crate::insertion::strategies::*;
use crate::insertion::active_window::get_active_application;

pub struct InsertionEngine {
    compatibility_db: AppCompatibilityDB,
}

impl InsertionEngine {
    pub fn new() -> Self {
        Self {
            compatibility_db: AppCompatibilityDB::load(),
        }
    }

    pub fn insert(&self, text: &str) -> Result<(), String> {
        // 1. 检测当前活动应用
        let app = get_active_application()?;

        // 2. 查询兼容性数据库
        let preferred_method = self.compatibility_db
            .get_preferred_method(&app.bundle_id);

        // 3. 尝试插入（带降级）
        self.try_insert_with_fallback(text, preferred_method)
    }

    fn try_insert_with_fallback(
        &self,
        text: &str,
        preferred: InsertionMethod,
    ) -> Result<(), String> {
        let methods = match preferred {
            InsertionMethod::Clipboard => vec![
                InsertionMethod::Clipboard,
                InsertionMethod::Keyboard,
            ],
            InsertionMethod::Keyboard => vec![
                InsertionMethod::Keyboard,
                InsertionMethod::Clipboard,
            ],
            InsertionMethod::Accessibility => vec![
                InsertionMethod::Accessibility,
                InsertionMethod::Clipboard,
            ],
        };

        for method in methods {
            let result = match method {
                InsertionMethod::Clipboard => {
                    ClipboardInsertion::insert(text)
                }
                InsertionMethod::Keyboard => {
                    KeyboardInsertion::insert(text)
                }
                InsertionMethod::Accessibility => {
                    AccessibilityInsertion::insert(text)
                }
            };

            if result.is_ok() {
                return result;
            }
        }

        // 最终降级：复制到剪贴板
        ClipboardInsertion::set_clipboard(text)?;
        Err("Insertion failed, copied to clipboard".to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InsertionMethod {
    Clipboard,
    Keyboard,
    Accessibility,
}
```

### 6. 应用兼容性数据库

```rust
// src-tauri/src/insertion/compatibility.rs

use std::collections::HashMap;

pub struct AppCompatibilityDB {
    apps: HashMap<String, AppCompatibility>,
}

impl AppCompatibilityDB {
    pub fn load() -> Self {
        let mut apps = HashMap::new();

        // Chrome
        apps.insert("com.google.Chrome".to_string(), AppCompatibility {
            bundle_id: "com.google.Chrome".to_string(),
            name: "Google Chrome".to_string(),
            preferred_method: InsertionMethod::Clipboard,
            known_issues: vec![],
        });

        // VSCode
        apps.insert("com.microsoft.VSCode".to_string(), AppCompatibility {
            bundle_id: "com.microsoft.VSCode".to_string(),
            name: "Visual Studio Code".to_string(),
            preferred_method: InsertionMethod::Clipboard,
            known_issues: vec![],
        });

        // 微信
        apps.insert("com.tencent.xinWeChat".to_string(), AppCompatibility {
            bundle_id: "com.tencent.xinWeChat".to_string(),
            name: "微信".to_string(),
            preferred_method: InsertionMethod::Clipboard,
            known_issues: vec!["某些输入框可能不支持".to_string()],
        });

        // Terminal
        apps.insert("com.apple.Terminal".to_string(), AppCompatibility {
            bundle_id: "com.apple.Terminal".to_string(),
            name: "Terminal".to_string(),
            preferred_method: InsertionMethod::Keyboard,
            known_issues: vec!["剪贴板粘贴可能有问题".to_string()],
        });

        Self { apps }
    }

    pub fn get_preferred_method(&self, bundle_id: &str) -> InsertionMethod {
        self.apps
            .get(bundle_id)
            .map(|app| app.preferred_method)
            .unwrap_or(InsertionMethod::Clipboard) // 默认使用剪贴板
    }
}

pub struct AppCompatibility {
    pub bundle_id: String,
    pub name: String,
    pub preferred_method: InsertionMethod,
    pub known_issues: Vec<String>,
}
```

### 7. Tauri Command

```rust
// src-tauri/src/commands/insertion.rs

use tauri::State;
use std::sync::Mutex;
use crate::insertion::engine::InsertionEngine;

#[tauri::command]
pub async fn insert_text(
    text: String,
    engine: State<'_, Mutex<InsertionEngine>>,
) -> Result<(), String> {
    let engine = engine.lock().unwrap();
    engine.insert(&text)
}

#[tauri::command]
pub async fn check_accessibility_permission() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        // 尝试创建事件源，需要 Accessibility 权限
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        Ok(source.is_ok())
    }

    #[cfg(not(target_os = "macos"))]
    Ok(true)
}

#[tauri::command]
pub async fn get_active_application() -> Result<serde_json::Value, String> {
    let app = crate::insertion::active_window::get_active_application()?;

    Ok(serde_json::json!({
        "bundleId": app.bundle_id,
        "name": app.name,
        "pid": app.pid,
    }))
}
```

## Performance & Reliability

### 性能指标
- 插入延迟: < 100ms (剪贴板方案)
- CPU 占用: < 1%
- 成功率: > 95%

### 可靠性保证
1. **多重降级**: 3 种策略自动切换
2. **备份恢复**: 剪贴板数据保护
3. **错误处理**: 所有异常捕获
4. **用户提示**: 失败时明确提示

## Testing Strategy

### 测试矩阵

| 应用类型 | 应用名称 | 测试场景 | 预期结果 |
|---------|---------|---------|---------|
| 浏览器 | Chrome | 输入框插入 | ✓ |
| 浏览器 | Safari | 文本域插入 | ✓ |
| 编辑器 | VSCode | 代码编辑器 | ✓ |
| 编辑器 | Sublime | 文本文件 | ✓ |
| 通讯 | 微信 | 聊天窗口 | ✓ |
| 终端 | Terminal | 命令行 | ✓ |
| Office | Word | 文档编辑 | ✓ |

### 手动测试清单
- [ ] 短文本插入 (< 100 字符)
- [ ] 长文本插入 (> 1000 字符)
- [ ] 特殊字符（换行、Tab、表情）
- [ ] 剪贴板恢复正确性
- [ ] 不同应用兼容性
- [ ] 权限被拒绝降级
- [ ] 插入失败重试

## Security & Privacy

- ✅ 仅读取必要的剪贴板数据
- ✅ 插入后立即恢复剪贴板
- ✅ 不记录插入历史（可选）
- ✅ 权限申请透明化
- ✅ 不上传任何数据
