# 聆码架构说明

## 概述

聆码是一款完全本地化的跨应用语音听写工具,采用 **Tauri + React 19** 的混合架构,所有语音处理完全在本地进行,无需云端服务。

## 架构图

```
┌─────────────────────────────────────────────────┐
│                  用户界面层                        │
│  React 19 + TypeScript + TailwindCSS v3        │
│  ┌──────────┬──────────┬──────────┬──────────┐  │
│  │ 主窗口    │ 录音窗口  │ 历史窗口  │  托盘菜单  │  │
│  └──────────┴──────────┴──────────┴──────────┘  │
└──────────────────┬──────────────────────────────┘
                   │ Tauri IPC
┌──────────────────▼──────────────────────────────┐
│              Rust 后端服务层                      │
│  ┌──────────┬──────────┬──────────┬──────────┐  │
│  │ 音频录制  │ 数据存储  │ 系统集成  │ 快捷键   │  │
│  │  cpal    │ SQLite   │  托盘    │  全局    │  │
│  │  hound   │ rusqlite │  权限    │ shortcut │  │
│  └──────────┴──────────┴──────────┴──────────┘  │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│            语音识别引擎 (计划中)                   │
│        whisper.cpp (C++ + Core ML)              │
└─────────────────────────────────────────────────┘
```

## 技术栈

### 前端

- **框架**: React 19
  - React Compiler 自动优化
  - 改进的并发特性
- **语言**: TypeScript (strict mode)
- **样式**: TailwindCSS v3
  - Utility-first CSS
  - 自动 PurgeCSS
- **状态管理**: Zustand
  - 轻量级 (1KB gzipped)
  - 支持持久化中间件
- **构建工具**: Vite 6
  - 快速HMR
  - 优化的生产构建

### 后端 (Rust)

- **桌面框架**: Tauri 2.2
  - 轻量级 (< 10MB)
  - 原生系统 API
- **音频处理**:
  - `cpal` - 跨平台音频捕获
  - `hound` - WAV 编码
- **数据库**: `rusqlite` (带 bundled feature)
  - 单文件数据库
  - 零配置
- **并发**: `tokio` + `parking_lot`
- **日志**: `tracing` + `tracing-subscriber`

## 核心模块

### 1. 数据库模块 (`src-tauri/src/db/`)

```rust
db/
├── mod.rs          // 模块入口,Database 结构体
├── models.rs       // 数据模型 (Setting, Transcription)
├── schema.rs       // Schema 定义和版本迁移
└── repository.rs   // 数据访问层 (Repository 模式)
```

**特性**:
- 版本化 schema 管理
- 自动迁移机制
- Repository 模式封装
- 线程安全 (Arc<Mutex<Connection>>)

**数据模型**:

```sql
-- 设置表
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 转录历史表
CREATE TABLE transcriptions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  text TEXT NOT NULL,
  audio_duration REAL,
  model_version TEXT,
  language TEXT DEFAULT 'zh',
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  app_context TEXT
);
```

### 2. 状态管理 (`src/stores/`)

```typescript
stores/
├── index.ts           // 统一导出
├── settingsStore.ts   // 用户设置 (持久化)
├── recordingStore.ts  // 录音状态
└── historyStore.ts    // 历史记录
```

**状态流**:

```
┌─────────────────┐
│ User Action     │
└────────┬────────┘
         ▼
┌─────────────────┐
│ Zustand Store   │ ← 本地状态管理
└────────┬────────┘
         ▼
┌─────────────────┐
│ Tauri Command   │ ← IPC 调用
└────────┬────────┘
         ▼
┌─────────────────┐
│ SQLite Database │ ← 持久化存储
└─────────────────┘
```

### 3. 系统集成

**托盘菜单** (`src-tauri/src/tray.rs`):
- 显示/隐藏主窗口
- 快速访问功能
- 优雅退出

**全局快捷键** (`src-tauri/src/shortcut.rs`):
- 默认: `Cmd+Shift+S` (macOS) / `Ctrl+Shift+S` (其他)
- 可自定义配置
- 触发录音功能

**权限管理** (`entitlements.plist`):
- 麦克风访问
- 辅助功能 (文本插入)
- 文件系统读写

### 4. Tauri Commands

所有 Tauri Commands 通过 `src-tauri/src/commands/` 暴露给前端:

```rust
commands/
├── mod.rs       // 命令模块入口
└── db.rs        // 数据库相关命令
```

**可用命令**:
- 设置: `get_setting`, `set_setting`, `get_all_settings`, `delete_setting`
- 转录: `create_transcription`, `get_transcription`, `get_recent_transcriptions`, `search_transcriptions`, `delete_transcription`, `delete_all_transcriptions`

## 数据流

### 语音录制流程 (计划中)

```
1. 用户按下快捷键 (Cmd+Shift+S)
   ▼
2. 全局快捷键监听器触发
   ▼
3. 显示录音悬浮窗
   ▼
4. cpal 开始捕获音频 (16kHz, mono, i16)
   ▼
5. 音频数据写入内存缓冲区
   ▼
6. 用户停止录音
   ▼
7. 将音频数据传递给 whisper.cpp
   ▼
8. 获取转录文本
   ▼
9. 保存到 SQLite 数据库
   ▼
10. 自动插入到当前应用 (通过辅助功能 API)
```

### 历史记录查询流程

```
1. 用户打开历史窗口
   ▼
2. React 组件调用 useHistoryStore.loadRecent()
   ▼
3. Store 调用 invoke('get_recent_transcriptions')
   ▼
4. Tauri Command 执行 TranscriptionRepository.get_recent()
   ▼
5. SQLite 查询数据
   ▼
6. 返回结果到前端
   ▼
7. 更新 Store 状态
   ▼
8. React 重新渲染 UI
```

## 性能考虑

### 1. 延迟优化

- **音频捕获**: < 50ms (无 GC,直接系统调用)
- **UI 响应**: < 100ms (React Compiler 优化)
- **数据库查询**: < 10ms (SQLite 索引优化)

### 2. 内存管理

- **目标**: < 500MB (不含语音模型)
- **策略**:
  - Rust 零拷贝优化
  - 音频缓冲区重用
  - 前端虚拟列表 (历史记录)

### 3. 打包体积

- **目标**: < 10MB (不含语音模型)
- **实现**:
  - Tauri 原生渲染
  - TailwindCSS PurgeCSS
  - Rust release 优化 (LTO, strip)

## 安全考虑

### 1. 数据隐私

- ✅ 所有数据本地存储
- ✅ 无网络上传
- ✅ 用户完全控制数据

### 2. 权限最小化

- 仅请求必要权限 (麦克风、辅助功能)
- 用户可随时在系统设置中撤销

### 3. 内存安全

- Rust 所有权系统防止内存泄漏
- 无数据竞争 (借用检查器)

## 扩展性

### 1. 多语言支持

- 架构预留多语言接口
- 语音模型可切换
- UI 国际化支持

### 2. 跨平台

- macOS (一期)
- Windows (二期)
- Linux (三期)

### 3. 插件系统 (未来)

- 自定义语音模型
- 第三方转录引擎
- 自定义后处理

## 开发工具链

### 代码质量

- **ESLint** - TypeScript/React 规范
- **Prettier** - 代码格式化
- **Clippy** - Rust linting
- **rustfmt** - Rust 格式化

### 测试

- **Vitest** - 前端单元测试
- **Rust test** - 后端单元测试
- **Tauri WebDriver** - E2E 测试 (计划中)

### CI/CD

- **Lint & Test** - 每次 PR
- **Build** - 每次提交
- **Release** - Tag 触发

## 目录结构

```
lingcode/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── stores/             # Zustand 状态管理
│   ├── windows/            # 各个窗口页面
│   └── main.tsx            # 入口
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── db/             # 数据库模块
│   │   ├── commands/       # Tauri Commands
│   │   ├── tray.rs         # 托盘管理
│   │   ├── shortcut.rs     # 快捷键管理
│   │   └── main.rs         # Rust 入口
│   ├── icons/              # 应用图标
│   ├── entitlements.plist  # macOS 权限
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── docs/                   # 项目文档
├── openspec/               # OpenSpec 规范
└── package.json            # Node 依赖
```

## 参考资料

- [Tauri 文档](https://tauri.app/)
- [React 19 文档](https://react.dev/)
- [Zustand 文档](https://github.com/pmndrs/zustand)
- [rusqlite 文档](https://docs.rs/rusqlite/)
- [cpal 文档](https://docs.rs/cpal/)
