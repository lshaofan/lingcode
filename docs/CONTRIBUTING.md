# 贡献指南

感谢您对聆码项目的关注！我们欢迎各种形式的贡献。

## 开发流程

### 1. Fork 项目

点击 GitHub 页面右上角的 "Fork" 按钮，将项目 fork 到您的账号下。

### 2. 克隆仓库

```bash
git clone https://github.com/YOUR_USERNAME/lingcode.git
cd lingcode
```

### 3. 创建分支

```bash
git checkout -b feature/your-feature-name
```

分支命名规范：
- `feature/xxx` - 新功能
- `fix/xxx` - bug 修复
- `docs/xxx` - 文档更新
- `refactor/xxx` - 代码重构

### 4. 安装依赖

```bash
pnpm install
```

### 5. 开发

启动开发服务器：
```bash
pnpm tauri:dev
```

### 6. 代码规范

在提交代码前，请确保：

- 代码通过 ESLint 检查: `pnpm lint`
- 代码已格式化: `pnpm format`
- 测试通过: `pnpm test`

### 7. 提交代码

我们使用约定式提交 (Conventional Commits)：

```bash
git commit -m "feat: 添加新功能"
git commit -m "fix: 修复某个 bug"
git commit -m "docs: 更新文档"
git commit -m "refactor: 重构代码"
git commit -m "test: 添加测试"
git commit -m "chore: 更新构建配置"
```

### 8. 推送代码

```bash
git push origin feature/your-feature-name
```

### 9. 创建 Pull Request

在 GitHub 上创建 Pull Request，并：
- 填写清晰的标题和描述
- 关联相关的 Issue
- 等待代码审查

## 代码规范

### TypeScript/React

- 使用 TypeScript 严格模式
- 组件使用函数式组件 + Hooks
- 使用 TypeScript 类型而非 PropTypes
- 避免使用 `any` 类型

### Rust

- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码

### 样式

- 使用 TailwindCSS 编写样式
- 避免内联样式
- 使用语义化的类名

### 命名规范

- 文件名: kebab-case (`user-profile.tsx`)
- 组件名: PascalCase (`UserProfile`)
- 函数名: camelCase (`getUserData`)
- 常量名: UPPER_CASE (`MAX_COUNT`)

## 提交 Pull Request 前的检查清单

- [ ] 代码通过所有测试
- [ ] 代码通过 ESLint 检查
- [ ] 代码已格式化
- [ ] 添加了必要的注释
- [ ] 更新了相关文档
- [ ] 提交信息符合约定式提交规范

## 报告 Bug

如果您发现了 bug，请：

1. 在 Issues 中搜索是否已有相关问题
2. 如果没有，创建新 Issue，包含：
   - 清晰的标题和描述
   - 复现步骤
   - 预期行为和实际行为
   - 系统环境信息
   - 相关的错误日志或截图

## 功能建议

如果您有功能建议，请：

1. 在 Issues 中创建 Feature Request
2. 描述清楚功能的使用场景
3. 说明为什么需要这个功能
4. 如果可能，提供实现思路

## 行为准则

- 尊重所有贡献者
- 建设性的讨论
- 包容不同观点
- 专注于对项目最有利的事情

## 获取帮助

如果您在开发过程中遇到问题：

- 查看 [文档](../README.md)
- 在 Issues 中提问
- 加入社区讨论

感谢您的贡献！
