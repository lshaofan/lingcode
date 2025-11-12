# Spec: Audio Recording

## ADDED Requirements

### Requirement: 音频捕获

系统 MUST 能够捕获系统麦克风音频，并 SHALL 以 16kHz, 16-bit PCM, Mono 格式处理。

#### Scenario: 开始录音

- **GIVEN** 用户已授权麦克风权限
- **WHEN** 用户按下全局快捷键或点击录音按钮
- **THEN** 系统应该开始捕获麦克风音频
- **AND** 录音窗口应该显示并置顶
- **AND** 音频数据应该实时缓冲到内存

#### Scenario: 停止录音

- **WHEN** 用户点击停止按钮或 VAD 检测到静音
- **THEN** 系统应该停止音频捕获
- **AND** 返回完整的音频数据（i16 数组）
- **AND** 清空音频缓冲区

#### Scenario: 取消录音

- **WHEN** 用户点击取消按钮
- **THEN** 系统应该停止音频捕获
- **AND** 丢弃所有音频数据
- **AND** 关闭录音窗口

---

### Requirement: 音频格式

系统 MUST 将音频转换为 Whisper 要求的格式。

#### Scenario: 采样率转换

- **GIVEN** 麦克风原始采样率为 44.1kHz 或 48kHz
- **WHEN** 捕获音频数据
- **THEN** 系统应该将采样率转换为 16kHz
- **AND** 使用高质量重采样算法（线性插值或更好）

#### Scenario: 格式转换

- **GIVEN** 麦克风输出为 f32 格式
- **WHEN** 处理音频数据
- **THEN** 系统应该转换为 i16 格式
- **AND** 范围为 -32768 到 32767

#### Scenario: 单声道处理

- **GIVEN** 麦克风为立体声
- **WHEN** 捕获音频
- **THEN** 系统应该混合为单声道
- **AND** 使用 (L + R) / 2 算法

---

### Requirement: 实时音频可视化

系统 MUST 在录音窗口显示实时波形。

#### Scenario: 波形更新

- **GIVEN** 正在录音
- **WHEN** 接收到新的音频数据块
- **THEN** 波形图应该更新显示
- **AND** 更新频率应不低于 30 FPS
- **AND** 波形应该显示最近 2 秒的音频

#### Scenario: 音量指示

- **WHEN** 录音音量过低
- **THEN** 应该显示警告提示
- **AND** 建议用户调整麦克风音量

---

### Requirement: VAD 语音活动检测

系统 SHALL 提供可选的 VAD 自动停止功能。

#### Scenario: 检测静音

- **GIVEN** VAD 功能已启用
- **WHEN** 检测到连续 2 秒静音
- **THEN** 系统应该自动停止录音
- **AND** 触发转录流程

#### Scenario: VAD 配置

- **WHEN** 用户打开设置
- **THEN** 应该能够配置 VAD 开关
- **AND** 应该能够配置静音阈值 (dB)
- **AND** 应该能够配置静音时长 (1-5 秒)

---

### Requirement: 录音时长限制

系统 MUST 限制单次录音最大时长，防止内存溢出。

#### Scenario: 超时自动停止

- **GIVEN** 最大录音时长为 5 分钟
- **WHEN** 录音时长达到 5 分钟
- **THEN** 系统应该自动停止录音
- **AND** 显示提示："录音已达最大时长"

#### Scenario: 时长配置

- **WHEN** 用户在设置中配置最大时长
- **THEN** 应该支持 1-10 分钟范围
- **AND** 默认值为 5 分钟

---

### Requirement: 连续录制模式

系统 MUST 支持长时间连续录制（预览模式需求）。

#### Scenario: 启动连续录制

- **GIVEN** 用户选择预览模式
- **WHEN** 用户按下快捷键
- **THEN** 系统应该启动连续录制模式
- **AND** 录制过程不自动停止
- **AND** 持续输出音频数据流

#### Scenario: 实时音频流输出

- **GIVEN** 正在连续录制
- **WHEN** 捕获到新的音频数据
- **THEN** 系统应该实时发送音频 chunk (62.5ms)
- **AND** 通过 Tauri Event 发送到前端或转录模块
- **AND** 不等待录制结束

#### Scenario: 音频分块

- **GIVEN** 连续录制模式
- **WHEN** 音频缓冲区达到 1000 samples (16kHz, 62.5ms)
- **THEN** 系统应该切出一个 chunk
- **AND** 发送 `audio-chunk` 事件携带 Vec<i16> 数据
- **AND** 继续录制下一个 chunk

#### Scenario: 连续录制内存管理

- **GIVEN** 连续录制超过 5 分钟
- **WHEN** 监控内存占用
- **THEN** 内存增长应该稳定（不累积所有音频）
- **AND** 处理完的 chunk 应该立即释放
- **AND** 总内存占用 < 150MB

---

### Requirement: 性能要求

系统 MUST 满足以下性能指标。

#### Scenario: 录音延迟

- **WHEN** 用户触发录音
- **THEN** 录音窗口应该在 100ms 内显示
- **AND** 音频捕获应该在 50ms 内开始

#### Scenario: 内存占用

- **GIVEN** 录音时长为 5 分钟
- **WHEN** 查看内存占用
- **THEN** 音频缓冲区应该 < 50MB
- **AND** 总内存增量应该 < 100MB

#### Scenario: CPU 占用

- **GIVEN** 正在录音
- **WHEN** 查看 CPU 使用率
- **THEN** 音频处理 CPU 占用应该 < 10%

---

### Requirement: 错误处理

系统 MUST 妥善处理各种错误情况。

#### Scenario: 麦克风未找到

- **GIVEN** 系统没有麦克风设备
- **WHEN** 用户尝试录音
- **THEN** 应该显示错误提示："未检测到麦克风"
- **AND** 提供检查硬件的建议

#### Scenario: 音频流中断

- **GIVEN** 正在录音
- **WHEN** 麦克风被拔出或禁用
- **THEN** 应该捕获错误
- **AND** 显示提示："录音中断"
- **AND** 保存已录制的音频

#### Scenario: 缓冲区溢出

- **WHEN** 音频缓冲区接近限制
- **THEN** 应该发出警告
- **AND** 自动停止录音

---

### Requirement: 多设备支持

系统 SHALL 支持多个音频输入设备。

#### Scenario: 设备枚举

- **WHEN** 用户打开设置
- **THEN** 应该列出所有可用的麦克风设备
- **AND** 显示设备名称和类型

#### Scenario: 设备切换

- **WHEN** 用户选择不同的麦克风
- **THEN** 系统应该使用新设备录音
- **AND** 保存设备偏好到数据库

#### Scenario: 设备热插拔

- **GIVEN** 正在使用外接麦克风
- **WHEN** 麦克风被拔出
- **THEN** 系统应该自动切换到默认设备
- **AND** 显示切换通知
