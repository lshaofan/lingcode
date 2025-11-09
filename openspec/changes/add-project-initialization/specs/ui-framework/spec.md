# Spec Delta: UI Framework

## ADDED Requirements

### Requirement: 基础 UI 组件库

系统 MUST 提供一套基础 UI 组件,包括 Button、Input、Modal、Toast、Tooltip 等,并 SHALL 使用 TailwindCSS 样式。

#### Scenario: Button 组件渲染

- **WHEN** 使用 `<Button variant="primary">保存</Button>`
- **THEN** 应该渲染一个蓝色主按钮
- **AND** 按钮应该支持 primary、secondary、ghost 变体

#### Scenario: Input 组件输入

- **WHEN** 用户在 `<Input type="text" />` 中输入文字
- **THEN** 输入内容应该实时更新
- **AND** 支持 placeholder、disabled、error 等状态

#### Scenario: Modal 组件显示

- **WHEN** 调用 `<Modal isOpen={true}>内容</Modal>`
- **THEN** 应该显示模态框遮罩和内容
- **AND** 点击遮罩或 ESC 键应该关闭模态框

#### Scenario: Toast 通知显示

- **WHEN** 调用 `toast.success("操作成功")`
- **THEN** 应该在屏幕右上角显示成功提示
- **AND** 3 秒后自动消失

---

### Requirement: 多窗口管理

系统 MUST 支持创建和管理多个窗口,包括主窗口、录音窗口、历史记录窗口。

#### Scenario: 主窗口显示

- **WHEN** 应用启动
- **THEN** 系统托盘应该显示应用图标
- **AND** 点击托盘菜单"设置"应该打开主窗口

#### Scenario: 录音窗口创建

- **WHEN** 用户触发录音快捷键
- **THEN** 应该创建半透明的悬浮录音窗口
- **AND** 窗口应该可以拖动
- **AND** 窗口应该始终置顶

#### Scenario: 历史记录窗口显示

- **WHEN** 用户点击托盘菜单"历史记录"
- **THEN** 应该打开历史记录窗口
- **AND** 窗口应该显示所有历史转录记录

#### Scenario: 窗口间通信

- **WHEN** 录音窗口完成转录
- **THEN** 应该通过 Tauri Event 发送事件
- **AND** 历史记录窗口应该收到事件并刷新列表

---

### Requirement: 系统托盘集成

系统 MUST 在系统托盘常驻,并 SHALL 提供快速访问菜单。

#### Scenario: 托盘图标显示

- **WHEN** 应用启动
- **THEN** 系统托盘应该显示应用图标
- **AND** 图标应该使用 macOS 原生样式

#### Scenario: 托盘菜单交互

- **WHEN** 用户点击托盘图标
- **THEN** 应该显示菜单,包含:设置、历史记录、关于、退出
- **AND** 点击菜单项应该执行对应操作

#### Scenario: 托盘图标状态

- **WHEN** 应用正在录音
- **THEN** 托盘图标应该显示录音状态(例如红色圆点)
- **AND** 录音停止后图标恢复正常

---

### Requirement: 状态管理

系统 MUST 使用 Zustand 管理全局状态,包括设置、录音状态、历史记录等。

#### Scenario: 设置状态持久化

- **WHEN** 用户修改设置(例如快捷键)
- **THEN** 设置应该保存到 Zustand store
- **AND** 设置应该同步到 SQLite 数据库
- **AND** 应用重启后设置应该恢复

#### Scenario: 录音状态共享

- **WHEN** 录音窗口开始录音
- **THEN** 录音状态应该更新到 store
- **AND** 主窗口和托盘图标应该反映录音状态

#### Scenario: 历史记录同步

- **WHEN** 新增一条转录记录
- **THEN** 历史记录 store 应该更新
- **AND** 历史记录窗口应该自动刷新列表

---

### Requirement: 响应式布局

系统 UI MUST 支持不同窗口尺寸,并 SHALL 保持良好的用户体验。

#### Scenario: 主窗口自适应

- **WHEN** 用户调整主窗口大小
- **THEN** 布局应该自适应窗口尺寸
- **AND** 最小宽度为 600px,最小高度为 400px

#### Scenario: 录音窗口固定尺寸

- **WHEN** 显示录音窗口
- **THEN** 窗口应该为固定尺寸 (300px × 150px)
- **AND** 窗口不可调整大小

#### Scenario: 历史记录窗口滚动

- **WHEN** 历史记录超过窗口高度
- **THEN** 应该显示垂直滚动条
- **AND** 滚动应该流畅无卡顿
