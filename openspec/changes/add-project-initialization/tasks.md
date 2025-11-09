# Tasks: 项目初始化实施清单

## 1. 项目脚手架搭建
- [x] 1.1 初始化 Tauri 项目 (`create-tauri-app`)
- [x] 1.2 配置 React 19 + Vite
- [x] 1.3 配置 TypeScript (strict mode)
- [x] 1.4 集成 TailwindCSS v3
- [ ] 1.5 配置 pnpm workspace (为未来 monorepo 预留)
- [x] 1.6 设置项目目录结构 (src/components, src/hooks, src/stores, src/windows)

## 2. 开发工具配置
- [x] 2.1 配置 ESLint (TypeScript + React rules)
- [x] 2.2 配置 Prettier (代码格式化)
- [x] 2.3 配置 Vitest + React Testing Library
- [x] 2.4 配置 Git Hooks (husky + lint-staged)
- [x] 2.5 配置 Commitlint (约定式提交)
- [x] 2.6 创建 .editorconfig (编辑器配置)
- [x] 2.7 配置 VS Code 推荐扩展 (.vscode/extensions.json)

## 3. Tauri 基础配置
- [x] 3.1 配置 tauri.conf.json (窗口、图标、权限)
- [x] 3.2 设置系统托盘菜单
- [x] 3.3 配置全局快捷键注册能力
- [ ] 3.4 配置窗口管理 (多窗口创建、显示、隐藏)
- [x] 3.5 配置 Tauri 权限 (麦克风、文件系统、剪贴板)
- [x] 3.6 创建应用图标 (macOS .icns)

## 4. 本地数据存储
- [x] 4.1 集成 SQLite (使用 tauri-plugin-sql 或 rusqlite)
- [x] 4.2 创建数据库 schema (settings, transcriptions 表)
- [x] 4.3 实现数据访问层 (Rust Commands)
- [x] 4.4 创建初始化脚本 (首次启动创建数据库)
- [x] 4.5 实现数据迁移机制 (版本管理)

## 5. 基础 UI 组件库
- [x] 5.1 创建 Button 组件 (primary, secondary, ghost 变体)
- [x] 5.2 创建 Input 组件 (文本输入、密码、搜索)
- [x] 5.3 创建 Modal 组件 (对话框、确认框)
- [x] 5.4 创建 Toast 组件 (通知提示)
- [ ] 5.5 创建 Tooltip 组件 (工具提示)
- [x] 5.6 创建 Icon 组件库 (使用 lucide-react 或 heroicons)
- [ ] 5.7 创建 Layout 组件 (Header, Sidebar, Content)

## 6. 窗口页面结构
- [ ] 6.1 创建主窗口 (设置页面)
- [ ] 6.2 创建录音悬浮窗口 (半透明、可拖动)
- [ ] 6.3 创建历史记录窗口 (列表、搜索、导出)
- [ ] 6.4 实现窗口间通信 (Tauri Event System)
- [ ] 6.5 实现窗口状态管理 (Zustand)

## 7. 状态管理
- [x] 7.1 集成 Zustand
- [x] 7.2 创建 settings store (用户设置)
- [x] 7.3 创建 recording store (录音状态)
- [x] 7.4 创建 history store (历史记录)
- [x] 7.5 实现状态持久化 (localStorage + SQLite)

## 8. 项目文档
- [x] 8.1 编写 README.md (项目介绍、特性、安装、使用)
- [x] 8.2 编写 CONTRIBUTING.md (贡献指南、开发流程)
- [x] 8.3 编写 docs/architecture.md (架构说明)
- [x] 8.4 编写 docs/development.md (开发指南、环境搭建)
- [ ] 8.5 编写 docs/api.md (Tauri Commands API 文档)
- [x] 8.6 选择开源协议 (建议 MIT 或 Apache 2.0)
- [ ] 8.7 更新 openspec/project.md (项目约定)

## 9. 测试基础
- [x] 9.1 配置 Vitest (单元测试)
- [x] 9.2 编写组件测试示例 (Button, Input)
- [x] 9.3 配置测试覆盖率报告
- [ ] 9.4 编写 Rust 单元测试 (数据库操作)
- [ ] 9.5 配置 E2E 测试框架 (Playwright 或 Tauri WebDriver)

## 10. 构建和发布
- [ ] 10.1 配置 Tauri 构建脚本
- [ ] 10.2 配置 macOS 代码签名 (开发阶段可选)
- [ ] 10.3 配置 GitHub Actions (CI: lint, test, build)
- [ ] 10.4 配置 GitHub Actions (发布: 自动打包 dmg)
- [ ] 10.5 配置版本管理 (semantic-release 或手动)
- [ ] 10.6 创建第一个 release (v0.1.0-alpha)

## 11. 开发脚本
- [x] 11.1 创建 `pnpm dev` (启动开发服务器)
- [x] 11.2 创建 `pnpm build` (构建生产版本)
- [x] 11.3 创建 `pnpm test` (运行测试)
- [x] 11.4 创建 `pnpm lint` (代码检查)
- [x] 11.5 创建 `pnpm format` (代码格式化)
- [x] 11.6 创建 `pnpm tauri` (Tauri CLI 快捷方式)

## 12. 验收测试
- [ ] 12.1 验证开发环境启动正常 (`pnpm dev`)
- [x] 12.2 验证应用可以打包 (`pnpm build`)
- [ ] 12.3 验证所有测试通过 (`pnpm test`)
- [ ] 12.4 验证代码规范通过 (`pnpm lint`)
- [ ] 12.5 验证托盘图标显示正常
- [ ] 12.6 验证多窗口创建和通信正常
- [ ] 12.7 验证 SQLite 数据库读写正常
- [ ] 12.8 验证基础 UI 组件渲染正常
- [ ] 12.9 验证 Git Hooks 工作正常
- [x] 12.10 验证项目文档完整清晰

## 注意事项

### 依赖关系
- 任务 1 必须先完成,其他任务才能开始
- 任务 3、4、5 可以并行开发
- 任务 6 依赖任务 5 (需要 UI 组件)
- 任务 9 依赖任务 5、6 (需要组件和页面)
- 任务 12 在所有任务完成后进行

### 优先级
- **P0 (必须)**: 任务 1, 2, 3, 4, 8.1-8.3
- **P1 (重要)**: 任务 5, 6, 7, 8.4-8.7
- **P2 (可选)**: 任务 9, 10.2, 10.5

### 时间估算
- 任务 1-4: 2-3 天
- 任务 5-7: 3-4 天
- 任务 8: 1-2 天
- 任务 9-11: 2-3 天
- 任务 12: 1 天
- **总计**: 约 2 周
