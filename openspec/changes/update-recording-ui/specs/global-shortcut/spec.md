# Spec Delta: Global Shortcut

## MODIFIED Requirements

### Requirement: 全局快捷键注册

系统 MUST 支持注册系统级全局快捷键。

#### Scenario: 注册默认快捷键

- **GIVEN** 应用首次启动
- **WHEN** 应用初始化完成
- **THEN** 应该注册默认快捷键 `Cmd+Shift+S`
- **AND** 快捷键在任何应用下都能触发

#### Scenario: 快捷键切换录音状态

- **GIVEN** 快捷键已成功注册
- **WHEN** 用户按下 `Cmd+Shift+S`（首次）
- **THEN** 应该显示录音窗口并开始录音
- **WHEN** 用户再次按下 `Cmd+Shift+S`
- **THEN** 应该停止录音并显示结果

#### Scenario: 注册失败处理

- **GIVEN** 快捷键已被其他应用占用
- **WHEN** 尝试注册快捷键
- **THEN** 应该捕获错误
- **AND** 显示提示："快捷键冲突，请更换"
- **AND** 提供快捷键设置入口

---

### Requirement: 多快捷键支持

系统 SHALL 支持配置多个快捷键。

#### Scenario: 录音快捷键

- **WHEN** 配置录音快捷键
- **THEN** 默认为 `Cmd+Shift+S`
- **AND** 可配置为任意组合
- **AND** 单次按下切换录音状态（开始/停止）

#### Scenario: 关闭窗口快捷键

- **WHEN** 录音窗口显示中
- **THEN** 按下 `Esc` 应该关闭窗口
- **AND** 如果正在录音，应该停止录音

#### Scenario: 快捷键优先级

- **GIVEN** 多个快捷键配置
- **WHEN** 快捷键冲突
- **THEN** 应该按优先级处理
- **AND** 全局快捷键优先于应用内快捷键

## REMOVED Requirements

### Requirement: 停止录音快捷键

**Reason**: 统一为单个快捷键切换模式，不再需要单独的停止快捷键。
**Migration**: 原"按下原录音快捷键停止"的行为已整合到"快捷键切换录音状态"需求中。
