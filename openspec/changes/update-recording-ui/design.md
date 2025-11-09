# Design Document: 更新录音窗口UI

## Context

当前录音窗口采用简单的圆形悬浮球设计，通过长按快捷键触发，录音后自动隐藏。这种设计存在以下问题：
1. 无法直观显示识别的文字内容
2. 长按交互不够直观和方便
3. 自动隐藏可能导致用户来不及查看结果
4. 缺少文本操作功能（复制、插入）

用户提供了新的 UI 设计稿，要求实现两种状态的窗口：录音中和结果展示。

## Goals / Non-Goals

### Goals
- 实现胶囊型录音窗口，包含麦克风图标、声波动画、关闭按钮
- 实现结果展示窗口，包含文本显示和操作按钮（复制、插入、清空）
- 将长按快捷键改为单次按键切换录音状态
- 移除自动隐藏逻辑，改为手动关闭
- UI 完全复刻用户提供的设计稿

### Non-Goals
- 不实现输入框光标定位（后续版本）
- 不实现 VAD 自动停止（后续版本）
- 不实现窗口拖拽功能（使用 Tauri 默认行为）

## Decisions

### Decision 1: 单次按键 vs 长按切换

**选择**: 单次按键切换录音状态

**理由**:
- 更符合用户习惯（类似录音笔的开始/停止按钮）
- 长按需要用户持续按住键盘，体验不够友好
- 单次按键可以明确区分"开始"和"停止"两个动作

**实现**:
- 在 Rust 后端维护录音状态（idle, recording, processing）
- 快捷键事件根据当前状态决定行为：
  - `idle` → `recording`: 显示窗口并开始录音
  - `recording` → `processing`: 停止录音并开始识别
  - `processing`: 忽略快捷键（等待识别完成）

**替代方案**:
- **长按持续录音**: 需要用户持续按住，体验较差
- **双击快捷键**: 容易误触，不够可靠

---

### Decision 2: 窗口状态切换动画

**选择**: 使用 CSS 过渡动画 + Tauri 窗口 resize

**理由**:
- CSS 动画性能好，流畅度高
- Tauri 支持动态调整窗口尺寸
- 可以实现平滑的尺寸和透明度过渡

**实现**:
```typescript
// 状态 1: 录音中 (300x60)
// 状态 2: 结果展示 (300-400 x 60-200, 自适应)
const windowSize = {
  recording: { width: 300, height: 60 },
  result: { width: 400, height: Math.min(200, 60 + textLines * 20) }
};
```

**替代方案**:
- **创建两个独立窗口**: 切换时关闭一个打开另一个，体验不够流畅
- **固定窗口尺寸**: 文本较长时会被截断

---

### Decision 3: 声波动画实现

**选择**: 使用 Canvas + Web Audio API

**理由**:
- Canvas 性能优秀，适合实时绘图
- Web Audio API 可以获取音频频域数据
- 可以实现流畅的声波动画

**实现**:
```typescript
const AudioWaveform = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const analyzerRef = useRef<AnalyserNode | null>(null);

  useEffect(() => {
    // 获取音频流
    // 创建 AnalyserNode
    // 绘制声波
  }, []);
};
```

**替代方案**:
- **CSS 动画**: 无法反映真实音频，只能是假动画
- **SVG**: 性能不如 Canvas
- **后端生成波形**: 延迟高，实时性差

---

### Decision 4: 文本插入实现

**选择**: 使用 macOS Accessibility API 模拟键盘输入

**理由**:
- 兼容性好，支持所有应用
- 可以插入到任意输入框
- Tauri 已有相关依赖（`cocoa`, `objc`）

**实现**:
```rust
// src-tauri/src/commands/text_insertion.rs
#[tauri::command]
pub fn insert_text(text: String) -> Result<(), String> {
    // 使用 CGEventCreateKeyboardEvent 模拟键盘输入
    // 逐字符输入文本
}
```

**替代方案**:
- **剪贴板 + Cmd+V**: 会覆盖用户剪贴板内容
- **AppleScript**: 权限要求更高，兼容性不如 Accessibility API

---

### Decision 5: 窗口定位策略

**选择**: 智能象限定位（鼠标附近，避免边缘）

**理由**:
- 窗口出现在鼠标附近，用户无需移动视线
- 根据鼠标位置（上下左右象限）调整窗口位置，避免超出屏幕
- 保持一定间距（20px），避免遮挡鼠标

**实现**:
```rust
// 判断鼠标在屏幕的哪个象限
let is_upper_half = cursor_y < screen_height / 2;
let is_left_half = cursor_x < screen_width / 2;

// 根据象限决定窗口位置
let x = if is_left_half {
    cursor_x + spacing  // 鼠标右侧
} else {
    cursor_x - window_size - spacing  // 鼠标左侧
};
```

**替代方案**:
- **固定屏幕中心**: 用户视线需要移动，体验较差
- **输入框上方**: 一期不实现输入框检测

## Risks / Trade-offs

### Risk 1: 声波动画性能

- **风险**: Canvas 绘制可能导致 UI 卡顿
- **影响**: 影响录音体验
- **缓解**:
  - 使用 `requestAnimationFrame` 限制绘制频率
  - 降低采样点数量（64 个点足够）
  - 使用 Web Worker 处理音频数据（如果需要）

### Risk 2: 文本插入兼容性

- **风险**: 某些应用可能不支持 Accessibility API 输入
- **影响**: 文本无法插入到特定应用
- **缓解**:
  - 提供复制功能作为备选方案
  - 显示插入失败提示，建议用户手动粘贴
  - 后续版本支持更多插入方式（剪贴板、AppleScript）

### Risk 3: 窗口尺寸调整闪烁

- **风险**: 状态切换时窗口 resize 可能出现闪烁
- **影响**: 视觉体验不流畅
- **缓解**:
  - 使用 CSS `transition` 平滑过渡
  - 确保窗口背景透明，减少视觉跳动
  - 测试不同 macOS 版本的兼容性

## Migration Plan

### 步骤 1: 向后兼容
- 保留现有的 `recordingStore` 接口
- 添加新的 `transcribedText` 状态和 actions

### 步骤 2: 渐进式重构
- 先重构 UI 组件（RecordingFloat.tsx）
- 再更新后端逻辑（shortcut.rs）
- 最后删除无用代码（accessibility.rs 的输入框检测）

### 步骤 3: 数据迁移
- 无需数据库迁移（设置表已存在）

### 步骤 4: Rollback Plan
- 如果出现严重 bug，可以回退到圆形悬浮球版本
- Git tag 记录当前版本作为回退点

## Open Questions

1. **声波颜色渐变**: 用户设计稿中的渐变色具体是什么颜色？蓝色到紫色的色值是多少？
   - → 待用户确认设计稿中的准确色值

2. **文本字体**: 识别文字使用什么字体和字号？
   - → 暂定使用系统默认字体，16px

3. **按钮图标**: 复制、插入、清空按钮使用什么图标库？
   - → 使用 `lucide-react` 图标库（项目已有依赖）

4. **空结果提示**: 未识别到内容时的提示文案？
   - → "未识别到内容，请重试"
