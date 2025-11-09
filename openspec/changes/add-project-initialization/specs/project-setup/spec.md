# Spec Delta: Project Setup

## ADDED Requirements

### Requirement: 项目技术栈配置

系统 SHALL 使用 React 19 + Tauri + TypeScript + TailwindCSS v3 作为核心技术栈,并 MUST 配置完整的开发工具链。

#### Scenario: 项目初始化成功

- **WHEN** 开发者执行 `pnpm install` 和 `pnpm dev`
- **THEN** 应用应该正常启动,显示开发窗口,无错误信息
- **AND** 热重载功能正常工作

#### Scenario: TypeScript 严格模式检查

- **WHEN** 开发者编写代码时
- **THEN** TypeScript 编译器应该启用严格模式 (`strict: true`)
- **AND** 类型错误应该在编译时被捕获

#### Scenario: TailwindCSS 样式生效

- **WHEN** 开发者使用 TailwindCSS class
- **THEN** 样式应该正确渲染
- **AND** 未使用的样式应该在构建时被自动清除

---

### Requirement: 代码质量保障

系统 MUST 配置 ESLint、Prettier、Commitlint 和 Git Hooks,以确保代码质量和提交规范。

#### Scenario: 代码格式自动修复

- **WHEN** 开发者运行 `pnpm lint`
- **THEN** ESLint 应该检查所有 TypeScript 和 React 代码
- **AND** 自动修复可修复的问题

#### Scenario: 提交前代码检查

- **WHEN** 开发者执行 `git commit`
- **THEN** Husky 应该触发 lint-staged
- **AND** 只有通过 Lint 的代码才能提交

#### Scenario: 提交消息规范验证

- **WHEN** 开发者提交不符合约定式提交规范的消息
- **THEN** Commitlint 应该拒绝提交
- **AND** 显示错误提示和正确格式示例

---

### Requirement: 测试框架配置

系统 MUST 配置 Vitest 和 React Testing Library,以支持单元测试和组件测试。

#### Scenario: 运行测试套件

- **WHEN** 开发者执行 `pnpm test`
- **THEN** Vitest 应该运行所有测试用例
- **AND** 显示测试结果和覆盖率报告

#### Scenario: 组件测试

- **WHEN** 开发者为 React 组件编写测试
- **THEN** 可以使用 React Testing Library 的查询和断言
- **AND** 测试应该在 jsdom 环境中运行

---

### Requirement: 构建和打包

系统 MUST 支持开发模式和生产构建,并 SHALL 能够打包为 macOS 应用。

#### Scenario: 开发模式启动

- **WHEN** 开发者执行 `pnpm dev`
- **THEN** Vite 开发服务器应该启动
- **AND** Tauri 应该打开应用窗口
- **AND** 支持热模块替换 (HMR)

#### Scenario: 生产构建

- **WHEN** 开发者执行 `pnpm build`
- **THEN** Vite 应该构建优化后的前端资源
- **AND** Tauri 应该打包为 macOS .app 或 .dmg
- **AND** 构建产物应该小于 50MB (不含 Whisper 模型)

---

### Requirement: 项目文档

系统 MUST 提供完整的项目文档,包括 README、贡献指南和开发指南。

#### Scenario: 新贡献者阅读文档

- **WHEN** 新贡献者打开 README.md
- **THEN** 应该能了解项目目的、特性、安装和使用方法
- **AND** 应该能找到 CONTRIBUTING.md 获取贡献指南

#### Scenario: 开发者搭建环境

- **WHEN** 开发者按照 docs/development.md 操作
- **THEN** 应该能成功搭建开发环境
- **AND** 应该能运行和调试应用

#### Scenario: 了解项目架构

- **WHEN** 开发者阅读 docs/architecture.md
- **THEN** 应该能理解项目的目录结构、模块划分和技术决策
- **AND** 应该能找到关键代码的位置
