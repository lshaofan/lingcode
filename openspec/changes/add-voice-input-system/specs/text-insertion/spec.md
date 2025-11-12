# Spec: Text Insertion

## ADDED Requirements

### Requirement: 自动文本插入

系统 MUST 能够将转录文本自动插入到当前活动窗口。

#### Scenario: 基础插入

- **GIVEN** 转录完成，文本为"你好世界"
- **WHEN** 触发自动插入
- **THEN** 文本应该插入到当前光标位置
- **AND** 插入延迟应该 < 100ms
- **AND** 显示插入成功提示

#### Scenario: 剪贴板备份

- **GIVEN** 用户剪贴板有内容
- **WHEN** 使用剪贴板插入方案
- **THEN** 应该先备份剪贴板内容
- **AND** 插入后恢复剪贴板

#### Scenario: 插入失败降级

- **WHEN** 自动插入失败
- **THEN** 应该自动复制文本到剪贴板
- **AND** 显示提示："已复制到剪贴板，请手动粘贴"

---

### Requirement: 辅助功能权限

系统 MUST 检查和申请 macOS Accessibility 权限。

#### Scenario: 权限检查

- **WHEN** 应用启动
- **THEN** 应该检查 Accessibility 权限状态
- **AND** 如果未授权，显示引导提示

#### Scenario: 权限申请

- **WHEN** 用户点击"授权辅助功能"
- **THEN** 应该打开系统设置页面
- **AND** 高亮显示应用权限选项

---

### Requirement: 应用兼容性

系统 SHALL 支持主流应用的文本插入。

#### Scenario: 兼容性测试

- **WHEN** 测试应用兼容性
- **THEN** 应该支持以下应用类型：
  - 浏览器 (Chrome, Safari, Firefox)
  - 编辑器 (VSCode, Sublime, Xcode)
  - 通讯工具 (微信, QQ, Slack)
  - Office (Word, Excel, Pages)

#### Scenario: 不兼容处理

- **GIVEN** 当前应用在黑名单中
- **WHEN** 尝试插入文本
- **THEN** 应该显示警告
- **AND** 自动复制到剪贴板
