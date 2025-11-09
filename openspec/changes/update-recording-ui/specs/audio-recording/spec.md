# Spec Delta: Audio Recording

## MODIFIED Requirements

### Requirement: 实时音频可视化

系统 MUST 在录音窗口显示实时声波动画。

#### Scenario: 声波动画显示

- **GIVEN** 正在录音
- **WHEN** 接收到新的音频数据块
- **THEN** 应该显示横向声波动画
- **AND** 声波振幅应该反映实时音量
- **AND** 更新频率应不低于 30 FPS

#### Scenario: 静音时的声波

- **WHEN** 录音但无声音输入
- **THEN** 声波应该显示为平直的基线
- **AND** 基线应该有轻微的呼吸动画

## ADDED Requirements

### Requirement: 录音窗口状态

系统 MUST 支持两种窗口状态的切换。

#### Scenario: 录音中状态

- **WHEN** 用户按下快捷键开始录音
- **THEN** 窗口应该显示为胶囊型（约 300x60px）
- **AND** 左侧显示麦克风图标
- **AND** 中间显示实时声波动画
- **AND** 右侧显示关闭按钮（×）

#### Scenario: 结果展示状态

- **WHEN** 录音停止且识别出文字
- **THEN** 窗口应该扩展显示文字内容
- **AND** 顶部显示识别的文本（只读）
- **AND** 底部显示操作按钮栏
- **AND** 窗口尺寸自适应文本长度（最大 400x200px）

#### Scenario: 空结果状态

- **WHEN** 录音停止但未识别出文字
- **THEN** 窗口应该显示提示信息："未识别到内容"
- **AND** 仅显示关闭按钮

---

### Requirement: 文本操作功能

系统 MUST 提供文本复制、插入、清空功能。

#### Scenario: 复制文本

- **GIVEN** 窗口显示识别结果
- **WHEN** 用户点击复制按钮
- **THEN** 应该将文本复制到系统剪贴板
- **AND** 按钮应该显示短暂的"已复制"反馈

#### Scenario: 插入文本

- **GIVEN** 窗口显示识别结果
- **WHEN** 用户点击插入按钮
- **THEN** 应该将文本插入到当前活动窗口的光标位置
- **AND** 插入成功后关闭录音窗口
- **AND** 插入失败时显示错误提示

#### Scenario: 清空文本

- **GIVEN** 窗口显示识别结果
- **WHEN** 用户点击清空按钮
- **THEN** 应该清空显示的文本
- **AND** 窗口恢复到录音中状态
- **AND** 准备好接受新的录音

---

### Requirement: 手动关闭窗口

系统 MUST 支持用户手动关闭录音窗口。

#### Scenario: 点击关闭按钮

- **GIVEN** 录音窗口显示中
- **WHEN** 用户点击右上角 × 按钮
- **THEN** 应该隐藏录音窗口
- **AND** 停止当前录音（如果正在录音）
- **AND** 清空窗口状态

#### Scenario: 按 Esc 键关闭

- **GIVEN** 录音窗口获得焦点
- **WHEN** 用户按下 Esc 键
- **THEN** 应该隐藏录音窗口
- **AND** 停止当前录音

---

### Requirement: UI 样式规范

系统 MUST 遵循以下 UI 设计规范。

#### Scenario: 窗口外观

- **WHEN** 显示录音窗口
- **THEN** 窗口应该采用半透明背景（毛玻璃效果）
- **AND** 圆角应该为 24px（胶囊型）
- **AND** 阴影应该为 `shadow-xl`
- **AND** 窗口应该始终置顶

#### Scenario: 图标和颜色

- **WHEN** 显示 UI 元素
- **THEN** 麦克风图标应该为白色
- **AND** 声波应该为渐变色（蓝色到紫色）
- **AND** 关闭按钮应该为半透明白色
- **AND** 文本应该为白色，背景深灰色

#### Scenario: 响应式布局

- **WHEN** 文本内容较长
- **THEN** 窗口高度应该自适应（最小 60px，最大 200px）
- **AND** 文本应该支持滚动
- **AND** 窗口宽度保持固定（300-400px）

## REMOVED Requirements

### Requirement: VAD 语音活动检测

**Reason**: 一期简化实现，去除自动停止功能，改为手动控制。
**Migration**: VAD 功能移至后续版本，当前版本仅支持手动停止录音。
