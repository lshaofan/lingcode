# Design: 聆码项目初始化技术设计

## Context

「聆码」是一款桌面语音听写工具,核心特点是完全本地化处理,保护用户隐私。项目需要在 macOS 平台上实现全局快捷键触发、实时语音转文字、自动插入文本等功能。

技术栈选择需要平衡以下因素:
- **性能**: 语音处理需要低延迟,UI 响应流畅
- **跨平台**: 虽然一期只支持 macOS,但需要为 Windows/Linux 扩展预留空间
- **本地化**: 所有数据和处理都在本地,不依赖云服务
- **开发效率**: 前端技术栈成熟,社区资源丰富
- **可维护性**: 代码规范统一,架构清晰

## Goals / Non-Goals

### Goals
- 建立 Tauri + React 19 + TypeScript 的现代化开发环境
- 配置完整的代码质量保障体系(Lint, Format, Test)
- 实现基础 UI 组件库和窗口管理框架
- 建立本地数据存储方案(SQLite)
- 提供清晰的项目文档和开发指南

### Non-Goals
- 不在此阶段实现具体业务功能(语音录制、转录等)
- 不涉及 Whisper 模型的集成和调优
- 不实现多平台打包(仅 macOS 开发环境)
- 不进行性能优化和压力测试

## Decisions

### 1. 桌面框架: Tauri vs Electron

**决策**: 选择 Tauri

**原因**:
- **体积小**: 打包体积比 Electron 小 10 倍以上 (< 10MB vs > 100MB)
- **性能好**: Rust 后端性能优异,内存占用低
- **安全性**: Rust 的内存安全特性,减少漏洞
- **系统集成**: 原生系统 API 调用更方便(托盘、快捷键、窗口管理)
- **前端灵活**: 支持任何前端框架,不绑定特定版本

**权衡**:
- 社区规模比 Electron 小,部分场景需要自己实现
- Rust 学习曲线较陡,但核心逻辑可以用 TypeScript 实现

### 2. 前端框架: React 19

**决策**: React 19 + TypeScript

**原因**:
- **成熟稳定**: 社区资源丰富,问题容易解决
- **React 19 新特性**:
  - React Compiler 自动优化渲染性能
  - 改进的 Hooks (useTransition, useDeferredValue)
  - Server Components (为未来扩展预留)
- **生态丰富**: UI 库、状态管理、路由等工具链完善
- **团队熟悉度**: 主要贡献者熟悉 React

### 3. 样式方案: TailwindCSS v3

**决策**: TailwindCSS v3

**原因**:
- **开发效率**: Utility-first 快速构建 UI
- **体积可控**: PurgeCSS 自动移除未使用样式
- **一致性**: 设计系统内置,避免样式不一致
- **响应式**: 断点系统方便适配不同屏幕

**替代方案**:
- CSS Modules: 更灵活但需要更多手写样式
- Styled Components: 运行时开销,打包体积大

### 4. 包管理器: pnpm

**决策**: pnpm

**原因**:
- **快速**: 硬链接机制,安装速度快,节省磁盘空间
- **严格**: 防止幽灵依赖,依赖关系更清晰
- **Monorepo 友好**: 为未来多包管理预留空间

### 5. 本地存储: SQLite

**决策**: SQLite

**原因**:
- **轻量级**: 单文件数据库,无需服务器
- **性能**: 本地读写快,适合桌面应用
- **SQL 支持**: 复杂查询、事务、索引等特性完善
- **跨平台**: 所有平台支持良好
- **备份简单**: 单文件备份和迁移方便

**数据模型设计**:
```sql
-- 设置表
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 历史记录表
CREATE TABLE transcriptions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  text TEXT NOT NULL,
  audio_duration REAL,
  model_version TEXT,
  language TEXT DEFAULT 'zh',
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  app_context TEXT  -- 记录当时的应用上下文
);

-- 索引
CREATE INDEX idx_transcriptions_created_at ON transcriptions(created_at DESC);
```

### 6. 语音模型: Whisper (后续 fine-tune)

**决策**: OpenAI Whisper

**原因**:
- **多语言支持**: 虽然中文不如 FunASR,但多语言场景更通用
- **社区生态**: whisper.cpp 等高性能实现成熟
- **Fine-tune 潜力**: 可以针对中文场景微调优化
- **开源友好**: MIT 协议,商业友好

**集成方案**:
- 使用 whisper.cpp 或 faster-whisper
- 通过 Tauri Command 调用后端推理
- 初期使用 base/small 模型平衡速度和准确度

### 7. 窗口架构

**决策**: 多窗口 + 托盘常驻

**窗口类型**:
1. **托盘菜单**: 常驻后台,快速访问
2. **悬浮录音窗口**: 录音时显示,半透明,可拖动
3. **设置窗口**: 配置快捷键、模型、语言等
4. **历史记录窗口**: 查看和管理历史转录

**通信机制**:
- Tauri Event System: 窗口间通信
- Zustand: 前端状态管理
- SQLite: 持久化数据共享

## Risks / Trade-offs

### 风险 1: Tauri 生态不如 Electron 成熟
- **影响**: 某些功能需要自己实现或等待社区支持
- **缓解**:
  - 核心功能用 Rust 插件实现
  - 参考 Tauri 官方示例和插件
  - 贡献代码回社区

### 风险 2: React 19 新版本稳定性
- **影响**: 可能遇到 bug 或第三方库不兼容
- **缓解**:
  - 锁定依赖版本
  - 使用 React 19 稳定特性,避免实验性 API
  - 关注 React 社区更新

### 风险 3: Whisper 中文识别准确度
- **影响**: 初期用户体验可能不如 FunASR
- **缓解**:
  - 明确说明这是开源项目,后续可微调
  - 提供模型切换能力(预留接口)
  - 收集用户反馈数据用于 fine-tune

### 权衡 1: 性能 vs 开发效率
- **选择**: 优先开发效率(React + TypeScript)
- **理由**: MVP 阶段快速验证,后续可以用 Rust 优化性能关键路径

### 权衡 2: 功能完整性 vs 简洁性
- **选择**: 初期保持简洁,核心功能优先
- **理由**: 避免过度设计,根据用户反馈迭代

## Migration Plan

N/A (新项目无需迁移)

## Open Questions

### Q1: 是否需要支持离线模型下载?
- **背景**: Whisper 模型较大 (140MB - 1.5GB)
- **选项 A**: 打包时内置 base 模型 → 安装包大,但开箱即用
- **选项 B**: 首次启动下载 → 安装包小,但需要网络
- **建议**: 内置 base 模型 (140MB),提供下载更大模型的选项

### Q2: 是否需要自动更新机制?
- **背景**: 开源项目,用户可能手动下载更新
- **选项 A**: 实现自动更新 (Tauri Updater) → 用户体验好
- **选项 B**: 手动下载更新 → 开发成本低
- **建议**: 一期手动更新,二期加入自动更新

### Q3: 快捷键冲突检测?
- **背景**: 用户设置的快捷键可能与系统或其他应用冲突
- **选项 A**: 实现冲突检测和提示 → 复杂度高
- **选项 B**: 用户自行处理冲突 → 体验差
- **建议**: 提供常见冲突提示,不做全局检测

## Implementation Notes

### 项目结构
```
lingcode/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── hooks/              # 自定义 Hooks
│   ├── stores/             # 状态管理(Zustand)
│   ├── utils/              # 工具函数
│   ├── windows/            # 各个窗口的页面
│   │   ├── main/           # 主窗口(设置)
│   │   ├── recording/      # 录音窗口
│   │   └── history/        # 历史窗口
│   └── main.tsx            # 入口文件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── commands/       # Tauri Commands
│   │   ├── db/             # SQLite 封装
│   │   ├── audio/          # 音频处理
│   │   └── main.rs         # Rust 入口
│   ├── icons/              # 应用图标
│   └── tauri.conf.json     # Tauri 配置
├── docs/                   # 文档
├── tests/                  # 测试
└── scripts/                # 构建脚本
```

### 开发工作流
1. `pnpm install` - 安装依赖
2. `pnpm dev` - 启动开发服务器
3. `pnpm build` - 构建生产版本
4. `pnpm test` - 运行测试
5. `pnpm lint` - 代码检查
6. `pnpm format` - 代码格式化

### CI/CD 基础
- **GitHub Actions**:
  - PR 检查: Lint, Test, Build
  - 发布: 自动打包 macOS dmg
  - 版本管理: semantic-release
