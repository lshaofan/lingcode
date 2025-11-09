# Project Context

## Purpose

「聆码」(LingCode) 是一款完全本地化的跨应用语音听写工具,专为中文用户设计。项目致力于提供隐私安全、响应快速的语音转文字解决方案,无需依赖云服务,所有处理都在设备端完成。

### 核心目标
- 提供全局快捷键触发的语音录制功能
- 使用本地 AI 模型(Whisper)实现实时语音转文字
- 自动将转录文本插入到当前活动窗口
- 保护用户隐私,所有数据本地存储和处理

### 目标用户
- 中文使用者
- 关注隐私安全的用户
- 需要高效文字输入的办公人群
- 开源爱好者和贡献者

## Tech Stack

### 前端技术栈
- **React 19** - 现代化 UI 框架,支持最新特性(React Compiler, 改进的 Hooks)
- **TypeScript 5+** - 严格类型检查,提升代码质量
- **TailwindCSS v3** - Utility-first CSS 框架,快速构建 UI
- **Vite** - 快速的开发服务器和构建工具
- **Zustand** - 轻量级状态管理

### 桌面框架
- **Tauri 2.x** - 基于 Rust 的跨平台桌面框架
- **Rust 1.70+** - 系统级编程,高性能后端

### 数据和存储
- **SQLite** - 本地关系型数据库,存储设置和历史记录
- **Tauri Plugin SQL** - Tauri 的 SQLite 集成

### AI/ML
- **Whisper** - OpenAI 开源的语音识别模型
- **whisper.cpp** 或 **faster-whisper** - 高性能推理引擎

### 开发工具
- **pnpm** - 快速的包管理器
- **ESLint** - 代码质量检查
- **Prettier** - 代码格式化
- **Vitest** - 单元测试框架
- **React Testing Library** - 组件测试
- **Husky** - Git Hooks 管理
- **Commitlint** - 提交消息规范

### CI/CD
- **GitHub Actions** - 自动化测试和构建

## Project Conventions

### Code Style

#### TypeScript
- 严格模式启用 (`strict: true`)
- 显式类型标注函数参数和返回值
- 使用 `interface` 定义对象类型,`type` 定义联合类型
- 避免使用 `any`,优先使用 `unknown`
- 文件命名:PascalCase 用于组件,camelCase 用于工具函数

#### React
- 函数组件优先,使用 Hooks
- 组件文件使用 `.tsx` 扩展名
- 组件命名使用 PascalCase
- Props 接口命名格式:`{ComponentName}Props`
- 避免过度使用 `useEffect`,优先使用事件处理

#### CSS/TailwindCSS
- 优先使用 Tailwind 的 utility classes
- 自定义样式放在组件同目录的 `.module.css` 文件
- 使用语义化的 class 命名
- 响应式设计优先 (mobile-first)

#### 命名约定
- 变量和函数:camelCase
- 组件和类:PascalCase
- 常量:UPPER_SNAKE_CASE
- 私有方法/变量:前缀 `_` (Rust)
- 文件夹:kebab-case

### Architecture Patterns

#### 目录结构
```
src/
├── components/      # 可复用 UI 组件
├── hooks/          # 自定义 React Hooks
├── stores/         # Zustand 状态管理
├── utils/          # 工具函数
├── windows/        # 各个窗口的页面
│   ├── main/       # 主窗口(设置)
│   ├── recording/  # 录音窗口
│   └── history/    # 历史窗口
└── types/          # TypeScript 类型定义

src-tauri/
├── src/
│   ├── commands/   # Tauri Commands API
│   ├── db/         # 数据库访问层
│   ├── audio/      # 音频处理
│   └── whisper/    # Whisper 模型集成
```

#### 设计原则
- **单一职责**:每个组件/函数只做一件事
- **组合优于继承**:使用 Hooks 和组合模式
- **依赖注入**:通过 Props 传递依赖,便于测试
- **错误边界**:使用 ErrorBoundary 包裹关键组件
- **性能优化**:按需使用 React.memo, useMemo, useCallback

#### 状态管理
- **本地状态**:使用 `useState` 和 `useReducer`
- **全局状态**:使用 Zustand stores
- **服务端状态**:SQLite 作为单一数据源
- **URL 状态**:不适用(桌面应用无路由)

#### 窗口通信
- 使用 Tauri Event System 进行窗口间通信
- 事件命名格式:`{domain}:{action}` (例如:`recording:complete`)

### Testing Strategy

#### 测试金字塔
- **单元测试 (70%)**:测试工具函数和 Hooks
- **组件测试 (20%)**:测试 UI 组件交互
- **集成测试 (10%)**:测试 Tauri Commands 和数据库

#### 测试工具
- Vitest:单元测试运行器
- React Testing Library:组件测试
- Mock Service Worker (MSW):API mock (如需要)

#### 测试约定
- 测试文件放在被测文件同目录,命名为 `*.test.ts(x)`
- 每个测试用例应该独立,不依赖执行顺序
- 使用 `describe` 分组相关测试
- 测试命名:`should ... when ...`

#### 覆盖率目标
- 核心业务逻辑:>80%
- UI 组件:>60%
- 工具函数:>90%

### Git Workflow

#### 分支策略
- `main`:主分支,始终保持可发布状态
- `feature/{name}`:功能开发分支
- `fix/{name}`:Bug 修复分支
- `docs/{name}`:文档更新分支

#### 提交规范 (Conventional Commits)
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type:**
- `feat`:新功能
- `fix`:Bug 修复
- `docs`:文档更新
- `style`:代码格式(不影响功能)
- `refactor`:重构
- `test`:测试相关
- `chore`:构建/工具配置

**示例:**
```
feat(ui): add recording floating window

Implement a draggable, always-on-top recording window with
real-time waveform visualization.

Closes #12
```

#### PR 流程
1. 从 `main` 创建功能分支
2. 提交代码并 push
3. 创建 Pull Request
4. 通过 CI 检查(Lint, Test, Build)
5. Code Review
6. Squash and Merge 到 `main`

## Domain Context

### 语音识别领域
- **ASR (Automatic Speech Recognition)**:自动语音识别
- **Whisper**:OpenAI 的多语言语音识别模型
- **Fine-tuning**:针对特定场景优化模型
- **采样率**:音频质量指标,通常使用 16kHz
- **VAD (Voice Activity Detection)**:语音活动检测,区分语音和静音

### 桌面应用概念
- **系统托盘 (Tray)**:应用常驻的小图标
- **全局快捷键 (Global Hotkey)**:系统级快捷键,任何应用下都能触发
- **窗口置顶 (Always on Top)**:窗口始终在最前
- **剪贴板 (Clipboard)**:系统剪贴板,用于跨应用复制粘贴
- **输入法 (IME)**:输入文本到活动窗口的机制

### 隐私和安全
- **本地处理**:所有数据和计算都在设备上,不上传云端
- **数据加密**:敏感数据可选加密存储
- **权限管理**:麦克风、剪贴板等权限申请

## Important Constraints

### 技术约束
- 一期只支持 macOS (后续扩展 Windows/Linux)
- 需要麦克风权限
- 需要辅助功能权限(自动插入文本)
- Whisper 模型文件较大 (140MB - 1.5GB)
- 实时转录对 CPU/内存有一定要求

### 性能约束
- 语音转录延迟应小于 2 秒
- UI 响应时间应小于 100ms
- 历史记录查询应小于 200ms
- 应用启动时间应小于 3 秒
- 内存占用应小于 500MB (不含模型)

### 隐私约束
- 不得上传任何用户数据到云端
- 不得收集用户隐私信息
- 音频数据不持久化存储(仅转录文本)
- 用户应能完全控制和删除历史记录

### 许可约束
- 项目使用 MIT 或 Apache 2.0 开源协议
- 依赖的第三方库必须兼容开源协议
- Whisper 模型使用 MIT 协议

## External Dependencies

### 核心依赖
- **Whisper 模型**:从 Hugging Face 或 OpenAI 下载
- **Tauri 插件**:tauri-plugin-sql, tauri-plugin-global-shortcut 等

### 系统依赖
- **macOS**:10.15+ (Catalina 及以上)
- **Xcode Command Line Tools**:构建 Tauri 应用
- **Rust**:1.70+ (Tauri 后端编译)
- **Node.js**:18+ (前端开发)
- **pnpm**:8+ (包管理)

### 开发依赖
- **GitHub**:代码托管和 CI/CD
- **VS Code**:推荐的开发环境(配置文件已包含)

### 可选依赖
- **Sentry**:错误监控(未来可选)
- **Mixpanel/Plausible**:匿名使用统计(用户可选,默认关闭)
