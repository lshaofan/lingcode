# Proposal: 语音输入完整系统（双模式 + 实时转录）

## Why

「聆码」的核心价值是提供**两种灵活的语音输入方式**，满足不同场景下的用户需求：

### 场景 1：快速输入（直接插入模式）
**用户故事**: 在聊天、文档编辑时，需要快速说几句话并立即插入
- 按住快捷键说话
- 松开后自动转录 + 插入
- 无需额外操作，一气呵成

**适用场景**: 微信聊天、回复邮件、填写表单

### 场景 2：长文本创作（预览模式）
**用户故事**: 口述长段内容（如会议纪要、语音笔记），需要边说边查看转录结果
- 按一次快捷键开始
- 实时看到转录文字出现在窗口
- 可以继续说，支持停顿换气
- 说完后手动确认插入或复制

**适用场景**: 会议记录、语音日记、长文章口述

### 为什么需要这个完整系统

当前已完成：
- ✅ 双模式 UI 框架（70%完成度）
- ✅ 悬浮窗管理系统
- ✅ 音频录制底层（Rust）
- ✅ 模型下载管理

核心缺失：
- ❌ **Whisper 转录引擎集成**（最关键）
- ❌ **实时流式转录**（预览模式核心）
- ❌ **智能停顿检测与音频切片**
- ❌ **跨应用文本插入**

补齐这 30% 的核心功能后，整个应用即可投入使用。

## What Changes

### 1. 双操作模式系统

#### 模式 A: 直接插入模式 (Direct Mode)

**交互流程**:
```
用户按住 Cmd+Shift+S
    ↓
[录音窗口弹出] - 窄窗口 (380x120px)
    ↓
[开始录制] - 显示"正在录制..."+ 红色脉冲动画
    ↓
用户说话...
    ↓
用户松开快捷键
    ↓
[停止录制]
    ↓
[窗口切换状态] - 显示"正在转录..."+ 蓝色旋转动画
    ↓
[Whisper 转录] - 1-3秒
    ↓
[自动插入] - 文本自动出现在当前应用光标位置
    ↓
[窗口关闭]
```

**关键特性**:
- **Press & Hold (按住说话)**: 类似对讲机，按住才录音
- **自动插入**: 无需手动操作，松手即插入
- **快速反馈**: 适合短句输入（< 30秒）
- **进度提示**: 长录音时显示转录进度条

**UI 设计**:
- 窄窄的悬浮球窗口（不遮挡视线）
- 录制状态：红色麦克风图标 + 脉冲环
- 转录状态：蓝色图标 + 旋转进度环

---

#### 模式 B: 预览模式 (Preview Mode)

**交互流程**:
```
用户按下 Cmd+Shift+S （首次）
    ↓
[录音窗口弹出] - 宽窗口 (880x200px)
    ↓
[开始实时录制] - 显示"正在录制..."
    ↓
用户说话: "你好"
    ↓
[智能停顿检测] - 检测到 0.5秒 静音
    ↓
[自动切片转录] - 转录"你好"
    ↓
[追加文本到窗口] - 显示: "你好"
    ↓
用户继续说: "我是聆码"
    ↓
[再次检测停顿]
    ↓
[追加转录] - 显示: "你好 我是聆码"
    ↓
... 循环往复，支持长时间录制 ...
    ↓
用户结束录制（4种方式）:
  - 再次按 Cmd+Shift+S (切换隐藏窗口并停止)
  - 点击窗口中的 [完成] 按钮
  - 长时间无声音（30秒）自动结束
  - 点击 [取消] 按钮
    ↓
[显示完整转录文本]
    ↓
用户操作:
  - 点击 [插入] → 插入到活动应用 → 关闭窗口
  - 点击 [复制] → 复制到剪贴板
  - 点击 [清空] → 清空文本重新开始
```

**关键特性**:
- **实时流式转录**: 边说边看到文字出现
- **智能停顿检测**: 自动识别句子边界（0.5秒停顿 → 切片转录）
- **自适应阈值**: 根据说话速度智能调整停顿判断（0.3-0.8秒）
- **文本追加**: 每次转录结果追加显示，形成完整段落
- **多种结束方式**: 灵活控制录制流程
- **支持长录制**: 会议记录、语音笔记等场景

**UI 设计**:
- 宽敞的文本显示区域（支持多行滚动）
- 实时追加新转录的文字
- 操作按钮组：清空、复制、插入

---

### 2. 核心技术实现

#### 2.1 Whisper 转录引擎集成

**基础转录**（直接插入模式需求）:
- 集成 whisper.cpp C++ 库
- Rust FFI 绑定
- 音频预处理（i16 → f32 转换）
- 批量推理接口
- 文本后处理（标点、大小写）

**Tauri Command**:
```rust
#[tauri::command]
async fn transcribe_audio(
    audio_data: Vec<i16>,
    language: String,
) -> Result<String, String>
```

---

#### 2.2 实时流式转录系统（预览模式核心）

**智能停顿检测算法**:
```rust
struct AdaptiveVAD {
    silence_threshold: f32,        // 静音阈值（动态）
    min_pause_duration: Duration,  // 最小停顿时长（智能）
    max_pause_duration: Duration,  // 最大停顿（30秒超时）
    speech_rate_history: Vec<f32>, // 说话速度历史
}

// 自适应调整
impl AdaptiveVAD {
    fn adjust_threshold(&mut self, recent_speech: &[f32]) {
        let speech_rate = calculate_speech_rate(recent_speech);

        // 说话快 → 停顿阈值短 (0.3s)
        // 说话慢 → 停顿阈值长 (0.8s)
        self.min_pause_duration = Duration::from_millis(
            (300.0 + speech_rate * 500.0) as u64
        );
    }
}
```

**流式转录管道**:
```
[音频录制线程] (持续运行)
    ↓ 每 100ms
[VAD 检测静音段]
    ↓ 检测到停顿 (0.5秒)
[音频切片提取] (Vec<i16>)
    ↓ 异步任务
[Whisper 转录] (tokio::spawn，不阻塞录音)
    ↓ ~1秒
[Tauri Event: transcription-chunk] (发送文本片段)
    ↓
[前端追加文本] (setTranscribedText append)
    ↓
[继续录音，等待下一次停顿...]
```

**Tauri Commands**:
```rust
// 启动流式录制（预览模式专用）
#[tauri::command]
async fn start_streaming_recording() -> Result<(), String>

// 普通录制（直接插入模式）
#[tauri::command]
async fn start_recording() -> Result<(), String>

// 停止录制
#[tauri::command]
async fn stop_recording() -> Result<Vec<i16>, String>
```

**Tauri Events**:
```typescript
// 前端监听实时转录片段
listen<string>('transcription-chunk', (event) => {
  const newText = event.payload;
  setTranscribedText(prev => prev + ' ' + newText);  // 追加
});
```

---

#### 2.3 跨应用文本插入

**macOS 辅助功能权限**:
- 检查 Accessibility 权限
- 引导用户授权
- 权限被拒时降级到剪贴板方案

**插入策略**:
- **主策略**: 剪贴板粘贴（兼容性最好）
  - 备份用户剪贴板
  - 写入转录文本
  - 模拟 Cmd+V
  - 恢复剪贴板
- **备选**: 键盘模拟输入
- **降级**: 仅复制到剪贴板，提示用户手动粘贴

**Tauri Command**:
```rust
#[tauri::command]
async fn insert_text(text: String) -> Result<(), String>
```

---

### 3. 状态管理集成

**recordingStore.ts 改造**:

```typescript
// 启动录制（根据模式选择）
startRecording: async () => {
  const mode = get().operationMode;

  if (mode === 'direct') {
    await invoke('start_recording');  // 常规录制
  } else {
    await invoke('start_streaming_recording');  // 流式录制
  }

  set({ state: 'recording' });
  // 启动计时器...
},

// 停止录制（根据模式处理）
stopRecording: async () => {
  const audioData = await invoke('stop_recording');
  const mode = get().operationMode;

  if (mode === 'direct') {
    // 直接插入模式：一次性转录 + 自动插入
    set({ state: 'processing' });

    const text = await invoke('transcribe_audio', {
      audioData,
      language: 'zh'
    });

    set({ transcription: text, transcribedText: text });

    // 自动插入
    await get().insertText();
    await window.hide();
  } else {
    // 预览模式：转录已实时完成，只需保存
    const text = get().transcribedText;
    // 保存到数据库...
    // 窗口保持打开，等待用户操作
  }
},

// 插入文本（移除占位符）
insertText: async () => {
  const text = get().transcribedText;
  if (!text) return;

  try {
    await invoke('insert_text', { text });
    toast.success('已插入');
    get().reset();
  } catch (error) {
    // 降级：复制到剪贴板
    await navigator.clipboard.writeText(text);
    toast.info('已复制到剪贴板，请手动粘贴');
  }
},
```

**监听流式转录事件**（在 RecordingFloat 组件中）:
```typescript
useEffect(() => {
  const unlisten = listen<string>('transcription-chunk', (event) => {
    const newChunk = event.payload;
    setTranscribedText(prev => prev + ' ' + newChunk);  // 追加
  });

  return () => unlisten.then(fn => fn());
}, []);
```

---

## Impact

### 受影响的 Specs

**新增 Specs**:
- `audio-recording` (已存在) - **扩展**: 增加流式录制模式
- `global-shortcut` (已存在)
- `whisper-integration` - **新增**: Whisper 转录引擎集成
- `model-management` - **新增**: 模型下载管理
- `text-insertion` - **新增**: 跨应用文本插入
- `adaptive-vad` - **新增**: 智能停顿检测规范

**修改 Specs**:
- `settings-ui` - 新增操作模式设置
- `data-storage` - 新增转录历史表

### 受影响的代码

**Rust 后端（新增）**:
```
src-tauri/src/
├── whisper/                       # 新增
│   ├── engine.rs                  # Whisper 引擎核心
│   ├── ffi.rs                     # whisper.cpp FFI 绑定
│   ├── model_manager.rs           # 模型管理
│   └── preprocessor.rs            # 音频预处理
├── audio/
│   ├── recorder.rs                # 修改：支持流式录制
│   ├── adaptive_vad.rs            # 新增：智能 VAD
│   └── chunk_processor.rs         # 新增：音频切片处理
├── insertion/                     # 新增
│   ├── engine.rs                  # 插入引擎
│   ├── clipboard.rs               # 剪贴板策略
│   └── accessibility.rs           # 权限管理
├── commands/
│   ├── transcription.rs           # 新增：转录 Commands
│   ├── insertion.rs               # 新增：插入 Commands
│   └── audio.rs                   # 修改：新增流式录制 API
└── shortcut.rs                    # 已存在，完整实现
```

**前端（修改）**:
```
src/
├── stores/
│   └── recordingStore.ts          # 修改：连接真实 API
├── windows/recording/
│   └── RecordingFloat.tsx         # 修改：监听 transcription-chunk 事件
└── hooks/
    └── useStreamingTranscription.ts  # 新增：流式转录 Hook
```

### 依赖清单

**Rust Crates**:
```toml
# 音频录制（已有）
cpal = "0.15"
hound = "3.5"

# 全局快捷键（已有）
tauri-plugin-global-shortcut = "2.0"

# Whisper 集成（新增）
libc = "0.2"        # FFI 绑定
tokio = "1"         # 异步运行时

# 文本插入（新增）
enigo = "0.2"       # 键盘模拟
clipboard = "0.5"   # 剪贴板操作
cocoa = "0.25"      # macOS API

# 模型下载（已有）
reqwest = "0.11"
sha2 = "0.10"
```

**外部依赖**:
- **whisper.cpp** - 需要编译为静态库（启用 Core ML）
- **Whisper 模型** - base (142MB, 推荐) / small / medium

**系统权限**:
- macOS 麦克风权限（已申请）
- macOS Accessibility 权限（需新增）

### 性能影响

| 指标 | 直接插入模式 | 预览模式（实时转录） |
|------|-------------|-------------------|
| **录音启动** | < 100ms | < 100ms |
| **转录延迟** | 1-3秒（松手后） | < 1秒（停顿后） |
| **内存占用** | +500MB（模型） | +500MB |
| **CPU 占用** | 15%（录制）+ 10%（转录） | 20%（录制+实时转录） |
| **适用时长** | < 1分钟 | 长时间（会议记录） |

---

## Risks & Mitigation

### 风险 1: 实时转录延迟过高
**影响**: 用户停顿后等待 > 2秒 才看到文字，体验差

**缓解**:
- 使用 base 模型（142MB，平衡速度和准确度）
- Core ML GPU 加速（M 系列芯片）
- 异步转录，不阻塞录音
- 目标: 停顿后 < 1秒 出结果

---

### 风险 2: 智能停顿检测不准确
**影响**:
- 误判：正常说话被切断
- 漏判：停顿很久才切片

**缓解**:
- 自适应阈值算法
- 初始保守值 0.5秒（不易误判）
- 根据用户说话节奏动态调整
- 最大静音保护 30秒（防止卡死）
- 后续可提供用户手动调节（高级设置）

---

### 风险 3: 长时间录制内存溢出
**影响**: 预览模式下录 30分钟，内存占用 > 2GB

**缓解**:
- 音频切片后立即释放内存
- 转录完成后删除音频数据
- 限制单次录制最大时长（30分钟）
- 监控内存占用，超阈值时警告

---

### 风险 4: whisper.cpp 编译困难
**影响**: 跨平台编译复杂，依赖多

**缓解**:
- 提供预编译库（macOS ARM64）
- 详细的编译文档
- Docker 编译环境
- CI/CD 自动化编译

---

### 风险 5: 文本插入兼容性问题
**影响**: 某些应用不支持自动输入

**缓解**:
- 剪贴板方案兼容性 99%
- 失败时自动降级
- 显示明确提示："已复制到剪贴板"
- 维护应用兼容性数据库

---

## Timeline

### Phase 1: 基础转录（Week 1-2）
- 编译 whisper.cpp
- Rust FFI 绑定
- 实现 `transcribe_audio` Command
- 直接插入模式跑通

### Phase 2: 实时流式转录（Week 3-4）
- 智能 VAD 实现
- 流式录制管道
- `transcription-chunk` 事件
- 预览模式跑通

### Phase 3: 文本插入（Week 5）
- 权限管理
- 插入引擎
- `insert_text` Command
- 端到端闭环

### Phase 4: 测试与优化（Week 6）
- 性能优化
- 错误处理
- 用户测试
- Bug 修复

**总计**: 6 周完整实现

---

## Success Metrics

### 功能完整性
- ✅ 直接插入模式可用（按住说话 → 松开插入）
- ✅ 预览模式可用（实时转录 + 手动操作）
- ✅ 转录准确率 > 90%（中文）
- ✅ 文本插入成功率 > 95%

### 性能指标
- 直接插入模式延迟 < 5秒（含录音时间）
- 预览模式实时转录延迟 < 1秒（停顿后）
- 内存占用 < 500MB
- CPU 占用 < 25%

### 用户体验
- 智能停顿检测准确率 > 90%
- 长时间录制（30分钟）无崩溃
- 错误提示清晰友好
- 支持常见应用（Chrome、VSCode、微信等）

---

## Future Enhancements

### Phase 2+ 优化
- [ ] 停顿阈值用户可调（高级设置）
- [ ] 预览模式实时编辑转录文本
- [ ] 语音指令（"删除上一句"）
- [ ] 转录历史全文搜索

### Phase 3+ 高级功能
- [ ] 多语言混合优化
- [ ] 自定义词库
- [ ] 语音翻译（中译英/英译中）
- [ ] 云端同步历史（可选）
