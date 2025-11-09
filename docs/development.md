# 聆码开发指南

## 环境要求

### 必需软件

- **Node.js** >= 18.0.0
- **pnpm** >= 8.0.0
- **Rust** >= 1.70.0
- **Xcode Command Line Tools** (macOS)

### 验证安装

```bash
node --version  # v18.0.0+
pnpm --version  # 8.0.0+
rustc --version # 1.70.0+
```

## 快速开始

### 1. 克隆项目

```bash
git clone <repository-url>
cd lingcode
```

### 2. 安装依赖

```bash
pnpm install
```

这会自动安装前端和 Tauri 依赖。

### 3. 启动开发服务器

```bash
pnpm tauri:dev
```

这会:
1. 启动 Vite 开发服务器 (端口 1420)
2. 编译 Rust 代码
3. 启动 Tauri 应用
4. 开启热重载

### 4. 构建生产版本

```bash
pnpm tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 项目脚本

```bash
# 开发相关
pnpm dev          # 仅启动 Vite 开发服务器
pnpm tauri:dev    # 启动完整 Tauri 开发环境
pnpm preview      # 预览生产构建

# 构建相关
pnpm build        # 构建前端资源
pnpm tauri:build  # 构建 Tauri 应用

# 代码质量
pnpm lint         # 运行 ESLint
pnpm format       # 格式化代码
pnpm test         # 运行测试

# Tauri CLI
pnpm tauri <command>  # 直接运行 Tauri 命令
```

## 开发工作流

### 前端开发

#### 1. 创建组件

组件位于 `src/components/`,使用 TypeScript + React:

```typescript
// src/components/MyComponent.tsx
import { FC } from 'react';

interface MyComponentProps {
  title: string;
}

export const MyComponent: FC<MyComponentProps> = ({ title }) => {
  return (
    <div className="p-4 bg-white rounded-lg shadow">
      <h2 className="text-xl font-bold">{title}</h2>
    </div>
  );
};
```

#### 2. 使用 Zustand Store

```typescript
// src/stores/myStore.ts
import { create } from 'zustand';

interface MyStore {
  count: number;
  increment: () => void;
}

export const useMyStore = create<MyStore>((set) => ({
  count: 0,
  increment: () => set((state) => ({ count: state.count + 1 })),
}));

// 在组件中使用
import { useMyStore } from '../stores/myStore';

export const Counter = () => {
  const { count, increment } = useMyStore();
  return <button onClick={increment}>Count: {count}</button>;
};
```

#### 3. 调用 Tauri Commands

```typescript
import { invoke } from '@tauri-apps/api/core';

// 调用后端命令
const setting = await invoke<string | null>('get_setting', { key: 'language' });

// 保存设置
await invoke('set_setting', { key: 'language', value: 'zh' });
```

### 后端开发 (Rust)

#### 1. 创建 Tauri Command

```rust
// src-tauri/src/commands/my_commands.rs
#[tauri::command]
pub fn my_command(param: String) -> Result<String, String> {
    // 业务逻辑
    Ok(format!("Received: {}", param))
}

// 在 lib.rs 中注册
.invoke_handler(tauri::generate_handler![
    my_command,
    // ... 其他命令
])
```

#### 2. 数据库操作

```rust
use crate::db::{Database, SettingsRepository};

#[tauri::command]
pub fn save_data(db: State<Arc<Database>>, key: String, value: String) -> Result<(), String> {
    let repo = SettingsRepository::new(db.connection());
    repo.set(&key, &value).map_err(|e| e.to_string())
}
```

#### 3. 系统集成

```rust
// 托盘菜单
use tauri::menu::{Menu, MenuItem};

// 全局快捷键
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

// 窗口管理
let window = app.get_webview_window("main").unwrap();
window.show()?;
```

## 调试技巧

### 前端调试

开发模式下会自动打开 DevTools:

```rust
#[cfg(debug_assertions)]
{
    let window = app.get_webview_window("main").unwrap();
    window.open_devtools();
}
```

### 后端调试

使用 tracing 记录日志:

```rust
use tracing::{info, warn, error};

info!("Application started");
warn!("Something unusual happened");
error!("An error occurred: {}", error);
```

查看日志:

```bash
# macOS
tail -f ~/Library/Logs/com.lingcode.app/lingcode.log
```

### Rust 编译检查

```bash
cd src-tauri
cargo check        # 快速类型检查
cargo clippy       # Lint 检查
cargo test         # 运行测试
```

## 代码规范

### TypeScript/React

- 使用 **ES6+** 语法
- 组件使用 **函数式组件** + Hooks
- Props 使用 **interface** 定义类型
- CSS 使用 **TailwindCSS** utility classes
- 文件名使用 **PascalCase** (组件) 或 **camelCase** (工具)

```typescript
// ✅ 好的实践
interface ButtonProps {
  onClick: () => void;
  children: React.ReactNode;
}

export const Button: FC<ButtonProps> = ({ onClick, children }) => {
  return (
    <button
      onClick={onClick}
      className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
    >
      {children}
    </button>
  );
};

// ❌ 避免
const Button = (props) => {  // 缺少类型
  return <button style={{ padding: '8px' }}>{props.children}</button>;  // 使用内联样式
};
```

### Rust

- 遵循 **Rust API Guidelines**
- 使用 **rustfmt** 格式化代码
- 错误处理使用 **Result<T, E>**
- 文档注释使用 **///{}**

```rust
// ✅ 好的实践
/// Saves a setting to the database
///
/// # Arguments
/// * `key` - The setting key
/// * `value` - The setting value
///
/// # Errors
/// Returns an error if the database operation fails
#[tauri::command]
pub fn save_setting(key: String, value: String) -> Result<(), String> {
    // 实现
}

// ❌ 避免
#[tauri::command]
pub fn save_setting(key: String, value: String) {  // 缺少错误处理
    // 使用 unwrap() 或 expect()
}
```

## 常见问题

### 1. 编译错误: "tauri command not found"

确保已安装 Tauri CLI:

```bash
pnpm install  # 会自动安装 @tauri-apps/cli
```

### 2. Rust 依赖下载慢

配置国内镜像 (~/.cargo/config.toml):

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
```

### 3. 前端热重载不工作

检查 Vite 开发服务器是否正常运行:

```bash
pnpm dev  # 应该在 localhost:1420 启动
```

### 4. 数据库文件位置

开发环境:

```
macOS: ~/Library/Application Support/com.lingcode.app/lingcode.db
```

### 5. 图标生成

```bash
# 从 SVG 生成所有平台图标
pnpm tauri icon app-icon.svg
```

## Git 工作流

### Commit 规范

使用 [Conventional Commits](https://www.conventionalcommits.org/):

```bash
feat: add voice recording feature
fix: resolve database connection issue
docs: update development guide
style: format code with prettier
refactor: simplify audio processing logic
test: add unit tests for settings store
chore: update dependencies
```

### Pre-commit Hooks

项目配置了 Husky + lint-staged:

```bash
# 每次 commit 前自动运行
- ESLint 检查并修复
- Prettier 格式化
- Commitlint 验证 commit 消息
```

## 性能优化

### 前端

1. **使用 React.memo** 避免不必要的重渲染
2. **虚拟列表** 处理大量数据
3. **Code Splitting** 按需加载
4. **图片优化** 使用 WebP 格式

### 后端

1. **数据库索引** 优化查询性能
2. **批处理** 减少 IPC 调用
3. **缓存** 常用数据
4. **异步处理** 耗时操作

## 测试

### 前端测试

```bash
# 运行测试
pnpm test

# 查看覆盖率
pnpm test --coverage

# 监听模式
pnpm test --watch
```

示例:

```typescript
// src/components/Button.test.tsx
import { render, fireEvent } from '@testing-library/react';
import { Button } from './Button';

test('button click triggers callback', () => {
  const onClick = vi.fn();
  const { getByText } = render(<Button onClick={onClick}>Click me</Button>);

  fireEvent.click(getByText('Click me'));
  expect(onClick).toHaveBeenCalledTimes(1);
});
```

### 后端测试

```bash
cd src-tauri
cargo test
```

示例:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_operations() {
        let db = Database::new(PathBuf::from(":memory:")).unwrap();
        let repo = SettingsRepository::new(db.connection());

        repo.set("key", "value").unwrap();
        assert_eq!(repo.get("key").unwrap(), Some("value".to_string()));
    }
}
```

## 发布流程

1. 更新版本号

```bash
# package.json 和 Cargo.toml
```

2. 提交更改

```bash
git add .
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

3. 构建

```bash
pnpm tauri:build
```

4. 上传 Release (手动或 GitHub Actions)

## 资源链接

- [Tauri 文档](https://tauri.app/v1/guides/)
- [React 19 文档](https://react.dev/)
- [Zustand 文档](https://github.com/pmndrs/zustand)
- [TailwindCSS 文档](https://tailwindcss.com/)
- [Rust 文档](https://doc.rust-lang.org/)
- [项目 OpenSpec](../openspec/)

## 获取帮助

- GitHub Issues
- Discussions
- Contributing Guidelines

---

Happy Coding! 🚀
