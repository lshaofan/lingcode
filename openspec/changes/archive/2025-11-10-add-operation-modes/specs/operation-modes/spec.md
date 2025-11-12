# Specification: Operation Modes

## ADDED Requirements

### Requirement: Operation Mode Configuration
系统 SHALL 提供两种操作模式供用户选择，以适应不同的使用场景。

#### Scenario: 直接插入模式
- **WHEN** 用户选择"直接插入模式"并按下全局快捷键
- **THEN** 系统立即开始录制，转录完成后自动将文字插入到当前活动的文本框

#### Scenario: 预览确认模式
- **WHEN** 用户选择"预览确认模式"并按下全局快捷键
- **THEN** 系统显示录制悬浮球，转录完成后在悬浮球中显示文字，等待用户选择操作（插入/复制/清空）

#### Scenario: 默认模式
- **WHEN** 用户首次启动应用且未配置操作模式
- **THEN** 系统默认使用"预览确认模式"

### Requirement: Mode Setting Persistence
系统 SHALL 持久化保存用户选择的操作模式。

#### Scenario: 保存模式选择
- **WHEN** 用户在设置页面修改操作模式
- **THEN** 系统将选择保存到本地存储（localStorage 和数据库）

#### Scenario: 恢复模式选择
- **WHEN** 应用启动或重启
- **THEN** 系统从本地存储恢复用户之前选择的操作模式

### Requirement: Direct Insert Mode Behavior
在直接插入模式下，系统 SHALL 在转录完成后立即插入文字，无需用户额外操作。

#### Scenario: 录制和自动插入
- **WHEN** 用户在直接插入模式下完成录制
- **THEN** 系统转录完成后自动调用文本插入功能，将文字插入到当前活动的应用

#### Scenario: 插入失败处理
- **WHEN** 在直接插入模式下，文本插入失败（如缺少辅助功能权限）
- **THEN** 系统降级为将文字复制到剪贴板，并显示通知提示用户手动粘贴

#### Scenario: 最小化UI反馈
- **WHEN** 用户在直接插入模式下录制
- **THEN** 录制窗口仅显示最小化的状态指示（录制中图标和时长），不显示转录文本

### Requirement: Preview & Confirm Mode Behavior
在预览确认模式下，系统 SHALL 在悬浮球中显示转录文本，等待用户操作。

#### Scenario: 显示转录预览
- **WHEN** 用户在预览确认模式下完成录制和转录
- **THEN** 系统在悬浮球中显示转录的文字内容

#### Scenario: 用户选择插入
- **WHEN** 用户在预览悬浮球中点击"插入"按钮
- **THEN** 系统将文字插入到当前活动的应用，并关闭悬浮球

#### Scenario: 用户选择复制
- **WHEN** 用户在预览悬浮球中点击"复制"按钮
- **THEN** 系统将文字复制到剪贴板，显示"已复制"提示，但不关闭悬浮球

#### Scenario: 用户选择清空
- **WHEN** 用户在预览悬浮球中点击"清空"按钮
- **THEN** 系统清除悬浮球中的文字内容，关闭悬浮球

### Requirement: Simplified Recording States
系统 SHALL 简化录制状态，移除"处理中"状态，只保留必要的状态。

#### Scenario: 直接插入模式的状态流转
- **WHEN** 用户在直接插入模式下使用功能
- **THEN** 状态流转为：空闲 → 录制中 → 转录中 → 已插入 → 空闲

#### Scenario: 预览确认模式的状态流转
- **WHEN** 用户在预览确认模式下使用功能
- **THEN** 状态流转为：空闲 → 录制中 → 转录中 → 显示预览 → (用户操作) → 空闲

#### Scenario: 无"处理中"等待状态
- **WHEN** 录制结束后开始转录
- **THEN** 系统不显示单独的"处理中"悬浮球状态，而是直接过渡到转录完成状态

### Requirement: Settings UI for Mode Selection
设置页面 SHALL 提供清晰的操作模式选择界面。

#### Scenario: 显示模式选项
- **WHEN** 用户打开系统设置页面
- **THEN** 系统显示"操作模式"设置项，包含两个选项：直接插入模式和预览确认模式

#### Scenario: 模式说明
- **WHEN** 用户查看操作模式设置
- **THEN** 每个模式都显示清晰的说明文字，描述其使用场景和行为

#### Scenario: 切换模式
- **WHEN** 用户选择不同的操作模式
- **THEN** 系统立即保存选择，并显示"已切换到 XX 模式"的确认提示

#### Scenario: 模式图示（可选）
- **WHEN** 用户查看操作模式设置
- **THEN** 系统可选地显示每种模式的图示或动画演示，帮助用户理解

### Requirement: Mode-Aware Recording Window
录制窗口 SHALL 根据当前操作模式调整其行为和显示。

#### Scenario: 直接插入模式的窗口
- **WHEN** 用户在直接插入模式下触发录制
- **THEN** 录制窗口仅显示录制状态图标、时长和停止按钮，不显示文字内容区域

#### Scenario: 预览确认模式的窗口
- **WHEN** 用户在预览确认模式下触发录制
- **THEN** 录制窗口显示录制状态、时长、文字内容区域和操作按钮（插入/复制/清空）
