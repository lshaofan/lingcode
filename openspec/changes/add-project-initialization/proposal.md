# Proposal: 聆码项目初始化

## Why

「聆码」是一款跨应用语音听写工具,旨在为中文用户提供完全本地化、隐私安全的语音转文字解决方案。项目参考 Wispr Flow 和 OpenWhispr,但专注于中文使用场景,基于设备端本地处理,无需云端服务。

当前项目处于空白状态,需要建立完整的技术栈、开发工作流和基础架构,为后续核心功能(语音录制、本地转录、自动插入)的开发奠定基础。

## What Changes

### 技术栈确立
- **前端框架**: React 19 + TypeScript
- **桌面框架**: Tauri (Rust 后端)
- **样式方案**: TailwindCSS v3
- **语音模型**: Whisper (后续支持 fine-tune 优化)
- **本地存储**: SQLite (历史记录、设置)
- **包管理器**: pnpm
- **目标平台**: macOS (一期)

### 项目脚手架
- 初始化 Tauri + React 19 项目结构
- 配置 TypeScript 严格模式
- 集成 TailwindCSS v3
- 配置构建和开发工具链

### 开发工具配置
- ESLint + Prettier 代码规范
- Git 工作流(commitlint, husky)
- 测试框架(Vitest + React Testing Library)
- CI/CD 基础配置

### 基础组件库
- UI 组件库基础(Button, Input, Modal 等)
- 托盘图标和菜单
- 窗口管理基础(主窗口、悬浮窗)

### 项目文档
- README.md (项目介绍、安装、使用)
- CONTRIBUTING.md (贡献指南)
- 开发者文档(架构说明、开发指南)
- LICENSE (开源协议)

### 基础数据层
- SQLite 数据库初始化
- 数据模型设计(设置表、历史记录表)
- 数据访问层封装

## Impact

- **受影响的 specs**:
  - `project-setup` (新增) - 项目基础设施
  - `ui-framework` (新增) - UI 组件和窗口管理
  - `data-storage` (新增) - 本地数据存储

- **受影响的代码**:
  - 整个项目的基础架构
  - 所有配置文件 (package.json, tsconfig.json, tauri.conf.json 等)
  - src/ 目录结构

- **依赖**:
  - Node.js 18+
  - Rust 1.70+
  - pnpm 8+
  - 开发环境: Xcode Command Line Tools (macOS)

- **后续影响**:
  - 为核心功能模块提供统一的开发基础
  - 建立代码规范和质量保障体系
  - 为多语言、跨平台扩展预留架构空间
