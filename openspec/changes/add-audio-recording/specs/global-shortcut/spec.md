# Spec: Global Shortcut

## Requirements

### Requirement: 全局快捷键注册

系统 MUST 支持注册系统级全局快捷键。

#### Scenario: 注册默认快捷键

- **GIVEN** 应用首次启动
- **WHEN** 应用初始化完成
- **THEN** 应该注册默认快捷键 `Cmd+Shift+S`
- **AND** 快捷键在任何应用下都能触发

#### Scenario: 快捷键触发录音

- **GIVEN** 快捷键已成功注册
- **WHEN** 用户按下 `Cmd+Shift+S`
- **THEN** 应该打开或聚焦录音窗口
- **AND** 自动开始录音

#### Scenario: 注册失败处理

- **GIVEN** 快捷键已被其他应用占用
- **WHEN** 尝试注册快捷键
- **THEN** 应该捕获错误
- **AND** 显示提示："快捷键冲突，请更换"
- **AND** 提供快捷键设置入口

---

### Requirement: 快捷键配置

系统 MUST 允许用户自定义快捷键。

#### Scenario: 查看当前快捷键

- **WHEN** 用户打开设置 → 快捷键
- **THEN** 应该显示当前配置的快捷键
- **AND** 显示快捷键状态（已注册/冲突）

#### Scenario: 修改快捷键

- **GIVEN** 用户在快捷键设置页面
- **WHEN** 用户点击输入框并按下新快捷键 `Cmd+Opt+R`
- **THEN** 系统应该验证快捷键格式
- **AND** 尝试注册新快捷键
- **AND** 如果成功，保存到数据库
- **AND** 如果失败，显示冲突提示

#### Scenario: 重置为默认

- **WHEN** 用户点击"恢复默认快捷键"
- **THEN** 应该注销当前快捷键
- **AND** 注册默认快捷键 `Cmd+Shift+S`
- **AND** 更新数据库配置

---

### Requirement: 快捷键冲突检测

系统 SHALL 提供快捷键冲突提示。

#### Scenario: 常见冲突检测

- **GIVEN** 用户设置快捷键为 `Cmd+Space`
- **WHEN** 验证快捷键
- **THEN** 应该警告："此快捷键与 Spotlight 冲突"
- **AND** 建议使用其他组合

#### Scenario: 冲突数据库

- **WHEN** 系统初始化
- **THEN** 应该加载常见冲突快捷键列表
- **AND** 包含 macOS 系统快捷键
- **AND** 包含常见应用快捷键（Chrome, VSCode 等）

---

### Requirement: 快捷键格式

系统 MUST 支持标准快捷键组合格式。

#### Scenario: 支持的修饰键

- **WHEN** 用户配置快捷键
- **THEN** 应该支持以下修饰键：
  - `Cmd` (Command)
  - `Ctrl` (Control)
  - `Opt` (Option/Alt)
  - `Shift`

#### Scenario: 组合键验证

- **WHEN** 用户输入快捷键
- **THEN** 应该要求至少包含一个修饰键
- **AND** 主键应该是字母、数字或功能键
- **AND** 不允许只有修饰键的组合

#### Scenario: 格式化显示

- **GIVEN** 快捷键配置为 `Cmd+Shift+S`
- **WHEN** 显示在 UI 上
- **THEN** macOS 应该显示为 `⌘⇧S`
- **AND** Windows 应该显示为 `Ctrl+Shift+S`

---

### Requirement: 多快捷键支持

系统 SHALL 支持配置多个快捷键。

#### Scenario: 录音快捷键

- **WHEN** 配置录音快捷键
- **THEN** 默认为 `Cmd+Shift+S`
- **AND** 可配置为任意组合

#### Scenario: 停止录音快捷键

- **WHEN** 正在录音
- **THEN** 按下 `Esc` 应该停止录音
- **AND** 按下原录音快捷键也应该停止

#### Scenario: 快捷键优先级

- **GIVEN** 多个快捷键配置
- **WHEN** 快捷键冲突
- **THEN** 应该按优先级处理
- **AND** 全局快捷键优先于应用内快捷键

---

### Requirement: 快捷键持久化

系统 MUST 保存快捷键配置。

#### Scenario: 保存配置

- **WHEN** 用户修改快捷键
- **THEN** 应该保存到 SQLite settings 表
- **AND** 字段为 `hotkey_recording`

#### Scenario: 加载配置

- **WHEN** 应用启动
- **THEN** 应该从数据库读取快捷键配置
- **AND** 如果不存在，使用默认值
- **AND** 注册快捷键

---

### Requirement: 快捷键禁用

系统 SHALL 允许临时禁用快捷键。

#### Scenario: 禁用全局快捷键

- **WHEN** 用户在设置中禁用快捷键
- **THEN** 应该注销快捷键
- **AND** 快捷键不再响应
- **AND** 显示"已禁用"状态

#### Scenario: 重新启用

- **WHEN** 用户重新启用快捷键
- **THEN** 应该注册快捷键
- **AND** 快捷键恢复响应

---

### Requirement: 错误处理

系统 MUST 处理快捷键相关错误。

#### Scenario: 注册失败

- **GIVEN** 系统权限不足
- **WHEN** 尝试注册快捷键
- **THEN** 应该捕获错误
- **AND** 显示提示："无法注册快捷键，请检查权限"

#### Scenario: 快捷键响应超时

- **GIVEN** 快捷键被触发
- **WHEN** 录音窗口 2 秒内未响应
- **THEN** 应该记录错误日志
- **AND** 重置快捷键监听

---

### Requirement: 快捷键帮助

系统 SHALL 提供快捷键使用帮助。

#### Scenario: 快捷键列表

- **WHEN** 用户打开帮助页面
- **THEN** 应该显示所有可用快捷键
- **AND** 包含快捷键说明和用途

#### Scenario: 首次使用提示

- **GIVEN** 用户首次启动应用
- **WHEN** 主窗口打开
- **THEN** 应该显示快捷键引导提示
- **AND** 包含快捷键图示和说明
