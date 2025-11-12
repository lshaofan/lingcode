# Tasks: add-voice-input-system

## Phase 1: Whisper 转录引擎集成 (Week 1-2)

### 1.1 基础转录功能（直接插入模式需求）

#### Task 1.1.1: whisper.cpp FFI 绑定
- [x] 在 `src-tauri/Cargo.toml` 添加 whisper-rs 依赖 (使用 whisper-rs v0.15 库)
- [x] ~~创建 `src-tauri/src/whisper/ffi.rs` 模块~~ (通过 whisper-rs 库提供)
- [x] ~~使用 bindgen 生成 whisper.cpp 的 Rust 绑定~~ (通过 whisper-rs 库提供)
- [x] 封装为安全的 Rust 接口 (WhisperEngine 封装)
- [x] 实现模型加载 (`WhisperContext::new_with_params`)
- [x] 实现转录功能 (`WhisperState::full`)
- [x] 实现资源管理 (Rust RAII 自动清理)
- [x] 验证 FFI 调用安全性

**Acceptance**:
- ✅ FFI 函数调用不会导致 segfault
- ✅ 可成功加载模型文件

#### Task 1.1.2: WhisperEngine 核心结构
- [x] 创建 `src-tauri/src/whisper/engine.rs`
- [x] 定义 `WhisperEngine` 结构体
  ```rust
  pub struct WhisperEngine {
      context: WhisperContext,
      model_path: PathBuf,
      n_threads: usize,
  }
  ```
- [x] 实现 `WhisperEngine::new(model_path)` 构造函数
- [x] ~~实现 `Drop` trait~~ (Rust RAII 自动清理)
- [x] 添加 `is_initialized()` 检查方法
- [x] 添加错误处理（ModelNotFound, FailedToLoad, TranscriptionFailed）

**Acceptance**:
- ✅ 引擎可成功初始化并自动清理资源
- ✅ 错误情况返回清晰的错误信息

#### Task 1.1.3: 音频预处理模块
- [x] 创建 `src-tauri/src/whisper/preprocessor.rs`
- [x] 实现 `convert_i16_to_f32(samples: &[i16]) -> Vec<f32>`
  - 转换公式：`f32 = i16 / 32768.0`
- [x] 实现 `validate_sample_rate(rate: u32) -> Result<()>`
  - 验证必须为 16kHz
- [x] 实现 `validate_channels(channels: u16) -> Result<()>`
  - 验证必须为单声道
- [x] 实现 `normalize_audio(samples: &mut [f32])`
  - 音量归一化处理
- [x] 添加预处理单元测试

**Acceptance**:
- ✅ i16 → f32 转换精度误差 < 0.001
- ✅ 非 16kHz 音频返回错误
- ✅ 立体声音频返回错误

#### Task 1.1.4: 基础转录接口
- [x] 在 `src-tauri/src/whisper/engine.rs` 实现 `transcribe()` 方法
  ```rust
  pub fn transcribe(
      &self,
      audio_data: &[f32],
      language: Option<&str>,
  ) -> Result<String>
  ```
- [x] ~~调用 whisper_full() 执行推理~~ (使用 WhisperState::full())
- [x] 提取转录文本结果 (full_get_segment_text())
- [x] 实现语言参数传递（zh, en, auto）
- [x] 添加空音频检测 (validate_audio_data)
- [x] 添加音频长度校验（最小 0.1s，最大 10 分钟）
- [x] 实现文本后处理（去除首尾空格）
- [x] 额外实现 transcribe_with_timestamps() 方法（带时间戳的转录结果）

**实现说明**:
- 使用正确的 whisper-rs API: 先创建 WhisperState，再调用 full()
- 音频长度限制：0.1s - 600s (10分钟)
- 集成了音频预处理模块进行验证

**Acceptance**:
- ✅ 能成功转录音频（通过前端集成）
- ✅ 语言参数正确传递
- ✅ 音频验证和长度校验已实现

#### Task 1.1.5: Tauri Command 封装
- [x] 创建 `src-tauri/src/commands/transcription.rs`
- [x] 实现多个 Tauri command:
  ```rust
  #[tauri::command]
  async fn initialize_whisper(
      app: AppHandle,
      model_name: String,
      state: State<'_, WhisperState>,
  ) -> Result<(), String>

  #[tauri::command]
  async fn transcribe_audio(
      audio_data: Vec<i16>,
      language: Option<String>,
      state: State<'_, WhisperState>,
  ) -> Result<String, String>

  #[tauri::command]
  async fn transcribe_last_recording(
      language: Option<String>,
      state: State<'_, WhisperState>,
  ) -> Result<String, String>

  #[tauri::command]
  async fn transcribe_audio_with_timestamps(...) -> Result<Vec<TranscriptionSegment>, String>

  #[tauri::command]
  async fn get_current_model(...) -> Result<Option<String>, String>
  ```
- [x] 在 `lib.rs` 注册所有 commands
- [x] 添加 WhisperState 管理 WhisperEngine 实例
- [x] 实现异步执行（使用 async fn）
- [x] 添加错误转换（使用 .map_err(|e| e.to_string())）
- [x] 集成全局静态 LAST_RECORDING 以避免 Send/Sync 问题

**实现说明**:
- WhisperState 使用 Arc<Mutex<Option<WhisperEngine>>> 管理引擎实例
- transcribe_last_recording 直接从全局静态变量读取录音数据
- 所有命令都是异步的，不会阻塞主线程
- 已在 lib.rs 的 setup 中初始化 WhisperState

**Acceptance**:
- ✅ 前端可通过 `invoke('initialize_whisper')` 初始化引擎
- ✅ 前端可通过 `invoke('transcribe_last_recording')` 转录录音
- ✅ 转录过程不阻塞 UI (async 执行)

### 1.2 转录进度反馈

#### Task 1.2.1: 进度回调机制
- [ ] 在 whisper FFI 中绑定进度回调函数
  ```rust
  type ProgressCallback = extern "C" fn(
      ctx: *mut whisper_context,
      user_data: *mut c_void,
      progress: i32,
  );
  ```
- [ ] 在 `WhisperEngine` 添加进度回调字段
- [ ] 实现 Rust 闭包 → C 回调的转换
- [ ] 传递 `user_data` 指针（Tauri AppHandle）

**Acceptance**:
- C 回调能成功触发 Rust 函数

#### Task 1.2.2: Tauri Event 发送
- [ ] 在进度回调中发出 Tauri event
  ```rust
  app_handle.emit_all("transcription-progress", json!({
      "progress": 0.65,
      "stage": "processing"
  }))
  ```
- [ ] 定义进度事件 payload 结构
  ```typescript
  interface TranscriptionProgress {
      progress: number;  // 0.0 - 1.0
      stage: 'loading' | 'processing' | 'postprocessing';
  }
  ```

**Acceptance**:
- 前端能监听到进度事件
- 进度值单调递增

#### Task 1.2.3: 前端进度显示
- [ ] 在 `RecordingFloat.tsx` 添加进度状态
  ```typescript
  const [transcriptionProgress, setTranscriptionProgress] = useState(0);
  ```
- [ ] 监听 `transcription-progress` 事件
- [ ] 在直接插入模式 UI 显示进度条
  - 当 `status === 'processing'` 时显示
  - 使用线性进度条组件
- [ ] 显示进度百分比文本 "转录中... 65%"

**Acceptance**:
- 直接插入模式下能看到实时进度
- 进度从 0% 平滑过渡到 100%

---

## Phase 2: 实时流式转录系统 (Week 3-4)

### 2.1 智能停顿检测（VAD 增强版）

#### Task 2.1.1: 基础 VAD 模块
- [ ] 创建 `src-tauri/src/audio/vad.rs`
- [ ] 定义 `VoiceActivityDetector` trait
  ```rust
  pub trait VoiceActivityDetector {
      fn is_speech(&mut self, samples: &[f32]) -> bool;
      fn reset(&mut self);
  }
  ```
- [ ] 实现简单 RMS 能量检测
  ```rust
  pub struct SimpleVAD {
      threshold: f32,  // 默认 0.02
  }
  ```
- [ ] 计算音频 RMS 能量值
- [ ] 与阈值比较判断是否为语音

**Acceptance**:
- 能检测出明显的语音段
- 能过滤环境噪音

#### Task 2.1.2: AdaptiveVAD 结构
- [ ] 在 `vad.rs` 实现 `AdaptiveVAD` 结构体
  ```rust
  pub struct AdaptiveVAD {
      silence_threshold: f32,
      min_pause_duration: Duration,
      max_pause_duration: Duration,
      speech_density_history: VecDeque<f32>,
      last_speech_time: Instant,
  }
  ```
- [ ] 实现 `new()` 构造函数（默认 0.5s 停顿）
- [ ] 添加 `speech_density_history` 维护（5 秒滑动窗口）

**Acceptance**:
- 结构体正确初始化
- 历史窗口正确滚动

#### Task 2.1.3: 语音密度计算
- [ ] 实现 `calculate_speech_density(samples: &[f32]) -> f32`
  - 统计语音帧占比
  - 返回 0.0 - 1.0 范围值
- [ ] 实现 `update_density_history(density: f32)`
  - 添加新密度值
  - 保持窗口大小为 5 秒（80 个 chunk，假设 62.5ms/chunk）
- [ ] 实现 `average_speech_rate() -> f32`
  - 计算历史平均语速

**Acceptance**:
- 快速说话时密度 > 0.7
- 慢速说话时密度 < 0.4

#### Task 2.1.4: 动态阈值调整
- [ ] 实现 `adjust_threshold(&mut self)`
  ```rust
  let avg_rate = self.average_speech_rate();
  let base_pause = 300.0; // 毫秒
  let adjustment = (1.0 - avg_rate) * 500.0;
  self.min_pause_duration = Duration::from_millis(
      (base_pause + adjustment) as u64
  );
  ```
- [ ] 限制阈值范围 [0.3s, 0.8s]
- [ ] 每秒调用一次调整函数

**Acceptance**:
- 快速语速时停顿阈值约 0.3s
- 慢速语速时停顿阈值约 0.8s

#### Task 2.1.5: 停顿检测逻辑
- [ ] 实现 `detect_pause(&mut self, samples: &[f32]) -> PauseDetection`
  ```rust
  pub enum PauseDetection {
      Speaking,
      ShortPause,      // < min_pause_duration
      SentenceEnd,     // >= min_pause_duration
      SessionTimeout,  // >= max_pause_duration (30s)
  }
  ```
- [ ] 跟踪 `last_speech_time`
- [ ] 计算静音持续时间
- [ ] 返回对应的停顿状态

**Acceptance**:
- 0.5s 停顿返回 `SentenceEnd`
- 连续说话返回 `Speaking`
- 30s 无声返回 `SessionTimeout`

#### Task 2.1.6: VAD 单元测试
- [ ] 测试：连续语音不触发停顿
- [ ] 测试：0.5s 静音触发 SentenceEnd
- [ ] 测试：30s 静音触发 SessionTimeout
- [ ] 测试：快速语速调整阈值到 0.3s
- [ ] 测试：慢速语速调整阈值到 0.8s
- [ ] 测试：环境噪音不误判为语音

**Acceptance**:
- 所有测试通过
- 准确率 > 95%

### 2.2 流式转录管道

#### Task 2.2.1: 音频分块器
- [ ] 创建 `src-tauri/src/audio/chunker.rs`
- [ ] 实现 `AudioChunker` 结构体
  ```rust
  pub struct AudioChunker {
      chunk_size: usize,  // 1000 samples = 62.5ms
      buffer: Vec<f32>,
  }
  ```
- [ ] 实现 `push(&mut self, samples: Vec<f32>) -> Vec<Vec<f32>>`
  - 接收新采样数据
  - 缓冲不足一个 chunk 时暂存
  - 返回完整的 chunk 列表
- [ ] 实现 `flush(&mut self) -> Option<Vec<f32>>`
  - 返回剩余不完整 chunk

**Acceptance**:
- 能正确切分 16kHz 音频为 62.5ms chunk
- buffer 不会无限增长

#### Task 2.2.2: 转录任务队列
- [ ] 创建 `src-tauri/src/whisper/streaming.rs`
- [ ] 实现 `TranscriptionQueue` 结构
  ```rust
  pub struct TranscriptionQueue {
      pending: Arc<Mutex<VecDeque<AudioChunk>>>,
      workers: Vec<JoinHandle<()>>,
  }
  ```
- [ ] 实现 `push_chunk(chunk: AudioChunk)`
- [ ] 实现多线程 worker 池（2-4 个线程）
- [ ] worker 从队列取任务并执行转录

**Acceptance**:
- 支持并行处理多个 chunk
- 队列不会阻塞录音线程

#### Task 2.2.3: Chunk 转录执行
- [ ] 实现 `transcribe_chunk(engine: &WhisperEngine, chunk: Vec<f32>) -> Result<String>`
- [ ] 调用 `engine.transcribe()` 处理单个 chunk
- [ ] 添加超时保护（每个 chunk 最多 5 秒）
- [ ] 处理空转录结果（返回空字符串，不报错）
- [ ] 记录转录耗时指标

**Acceptance**:
- 单个 chunk 转录时间 < 200ms（base 模型 + Core ML）
- 超时时返回错误

#### Task 2.2.4: 异步流式 Pipeline
- [ ] 创建 `StreamingTranscription` 结构
  ```rust
  pub struct StreamingTranscription {
      audio_rx: mpsc::Receiver<Vec<f32>>,
      text_tx: mpsc::Sender<String>,
      vad: AdaptiveVAD,
      queue: TranscriptionQueue,
  }
  ```
- [ ] 实现 `start()` 方法启动管道
  - 异步循环接收音频数据
  - VAD 检测停顿点
  - 累积音频 chunk 直到停顿
  - 发送到转录队列
- [ ] 实现 `stop()` 方法停止管道
  - 处理完队列中所有任务
  - 释放资源

**Acceptance**:
- 管道能持续运行 5 分钟以上
- 停止时无资源泄漏

#### Task 2.2.5: Tauri Event 流式发送
- [ ] 在转录完成后发送 `transcription-chunk` 事件
  ```rust
  app_handle.emit_all("transcription-chunk", text_chunk);
  ```
- [ ] 按时间顺序发送（保证顺序性）
- [ ] 处理事件发送失败（前端已关闭）

**Acceptance**:
- 前端能按顺序收到文本块
- chunk 不会丢失或乱序

#### Task 2.2.6: 预览模式启动流程
- [ ] 实现 `start_streaming_transcription` Tauri command
  ```rust
  #[tauri::command]
  async fn start_streaming_transcription(
      language: Option<String>,
      state: State<'_, AppState>,
  ) -> Result<(), String>
  ```
- [ ] 启动音频录制（连续模式）
- [ ] 创建 StreamingTranscription 实例
- [ ] 连接音频流 → 转录管道
- [ ] 保存实例到 AppState（用于停止）

**Acceptance**:
- 前端调用后立即开始实时转录
- 音频延迟 < 500ms

#### Task 2.2.7: 前端流式接收
- [ ] 在 `recordingStore.ts` 添加流式文本状态
  ```typescript
  streamedText: string;
  appendTranscriptionChunk: (chunk: string) => void;
  ```
- [ ] 实现 `appendTranscriptionChunk`
  ```typescript
  appendTranscriptionChunk: (chunk) => {
      set((state) => ({
          streamedText: state.streamedText + ' ' + chunk.trim()
      }));
  }
  ```
- [ ] 在 `RecordingFloat.tsx` 监听事件
  ```typescript
  useEffect(() => {
      const unlisten = listen<string>('transcription-chunk', (event) => {
          appendTranscriptionChunk(event.payload);
      });
      return () => { unlisten.then(fn => fn()); };
  }, []);
  ```

**Acceptance**:
- 预览窗口实时显示累积文本
- 文本自动追加，不闪烁

### 2.3 预览模式结束逻辑

#### Task 2.3.1: 快捷键结束
- [ ] 在 `shortcut.rs` 预览模式按键事件中
- [ ] 检测到再次按下快捷键时
- [ ] 调用 `stop_streaming_transcription()` command
- [ ] 保持悬浮窗显示（等待用户操作）

**Acceptance**:
- 按快捷键立即停止录制
- 已转录文本保留

#### Task 2.3.2: 按钮结束
- [ ] 在 `RecordingFloat.tsx` 预览模式 UI
- [ ] 添加 "停止" 按钮
- [ ] 点击时调用 `invoke('stop_streaming_transcription')`
- [ ] 更新状态为 'idle'

**Acceptance**:
- 点击按钮停止录制
- 状态正确更新

#### Task 2.3.3: 30 秒超时结束
- [ ] 在 `StreamingTranscription` 管道中
- [ ] VAD 检测到 `SessionTimeout` 时
- [ ] 自动停止录制
- [ ] 发送 `recording-timeout` 事件
  ```rust
  app_handle.emit_all("recording-timeout", ());
  ```
- [ ] 前端监听事件并更新状态

**Acceptance**:
- 30s 无声后自动停止
- 前端显示 "已自动停止"

#### Task 2.3.4: 取消按钮
- [ ] 在预览模式 UI 添加 "取消" 按钮
- [ ] 点击时：
  - 调用 `stop_streaming_transcription()`
  - 清空 `streamedText`
  - 关闭悬浮窗
- [ ] 不保存转录结果

**Acceptance**:
- 点击取消清空文本
- 窗口正确关闭

#### Task 2.3.5: 停止转录 Command
- [ ] 实现 `stop_streaming_transcription` Tauri command
  ```rust
  #[tauri::command]
  async fn stop_streaming_transcription(
      state: State<'_, AppState>,
  ) -> Result<(), String>
  ```
- [ ] 停止音频录制
- [ ] 调用 `StreamingTranscription::stop()`
- [ ] 等待队列清空（最多 5 秒）
- [ ] 释放资源
- [ ] 从 AppState 移除实例

**Acceptance**:
- 停止后不再收到事件
- 资源正确释放

---

## Phase 3: 跨应用文本插入 (Week 5)

### 3.1 macOS 辅助功能权限

#### Task 3.1.1: 权限检查模块
- [ ] 创建 `src-tauri/src/accessibility/mod.rs`
- [ ] 使用 `cocoa` crate 调用 macOS API
- [ ] 实现 `check_accessibility_permission() -> bool`
  ```rust
  use cocoa::appkit::NSWorkspace;
  use cocoa::foundation::NSString;

  pub fn check_accessibility_permission() -> bool {
      unsafe {
          let options = /* AXIsProcessTrusted() */;
          // 调用 macOS Accessibility API
      }
  }
  ```
- [ ] 添加 Tauri command 封装
  ```rust
  #[tauri::command]
  fn is_accessibility_granted() -> bool
  ```

**Acceptance**:
- 能正确检测权限状态
- macOS 10.14+ 兼容

#### Task 3.1.2: 权限申请引导
- [ ] 实现 `request_accessibility_permission()` 函数
  ```rust
  pub fn request_accessibility_permission() -> Result<()> {
      // 打开系统设置 → 隐私与安全性 → 辅助功能
      open::that("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")?;
      Ok(())
  }
  ```
- [ ] 添加 Tauri command 封装
  ```rust
  #[tauri::command]
  fn open_accessibility_settings() -> Result<(), String>
  ```

**Acceptance**:
- 调用后打开正确的系统设置页面
- 应用在列表中高亮

#### Task 3.1.3: 前端权限提示 UI
- [ ] 创建 `src/components/AccessibilityPrompt.tsx`
- [ ] 检测权限状态
  ```typescript
  const [hasPermission, setHasPermission] = useState(false);

  useEffect(() => {
      invoke<boolean>('is_accessibility_granted').then(setHasPermission);
  }, []);
  ```
- [ ] 未授权时显示提示卡片
  - 说明：需要辅助功能权限以实现自动文本插入
  - 按钮：[打开系统设置]
- [ ] 点击按钮调用 `invoke('open_accessibility_settings')`

**Acceptance**:
- 首次启动时显示权限提示
- 授权后提示消失

### 3.2 文本插入引擎

#### Task 3.2.1: 剪贴板插入方案
- [ ] 创建 `src-tauri/src/insertion/clipboard.rs`
- [ ] 使用 `arboard` crate 操作剪贴板
- [ ] 实现 `insert_via_clipboard(text: &str) -> Result<()>`
  ```rust
  use arboard::Clipboard;

  pub fn insert_via_clipboard(text: &str) -> Result<()> {
      let mut clipboard = Clipboard::new()?;

      // 1. 备份原剪贴板内容
      let backup = clipboard.get_text().ok();

      // 2. 写入转录文本
      clipboard.set_text(text)?;

      // 3. 模拟 Cmd+V 粘贴
      simulate_paste_keystroke()?;

      // 4. 延迟 100ms 后恢复剪贴板
      thread::sleep(Duration::from_millis(100));
      if let Some(original) = backup {
          clipboard.set_text(original)?;
      }

      Ok(())
  }
  ```
- [ ] 处理剪贴板权限错误

**Acceptance**:
- 文本成功插入到活动窗口
- 原剪贴板内容正确恢复
- 延迟 < 200ms

#### Task 3.2.2: 键盘模拟模块
- [ ] 创建 `src-tauri/src/insertion/keyboard.rs`
- [ ] 使用 `enigo` crate 模拟按键
- [ ] 实现 `simulate_paste_keystroke() -> Result<()>`
  ```rust
  use enigo::{Enigo, Key, KeyboardControllable};

  pub fn simulate_paste_keystroke() -> Result<()> {
      let mut enigo = Enigo::new();

      // macOS: Cmd + V
      enigo.key_down(Key::Meta);
      enigo.key_click(Key::Layout('v'));
      enigo.key_up(Key::Meta);

      Ok(())
  }
  ```
- [ ] 添加平台检测（仅 macOS）

**Acceptance**:
- 能正确触发粘贴操作
- 不影响其他按键状态

#### Task 3.2.3: Accessibility API 插入
- [ ] 创建 `src-tauri/src/insertion/accessibility.rs`
- [ ] 使用 macOS AXUIElement API
- [ ] 实现 `insert_via_accessibility(text: &str) -> Result<()>`
  ```rust
  pub fn insert_via_accessibility(text: &str) -> Result<()> {
      // 1. 获取系统焦点元素
      let focused_element = get_focused_ui_element()?;

      // 2. 检查是否支持文本插入
      if !supports_text_insertion(&focused_element) {
          return Err("Element does not support text".into());
      }

      // 3. 插入文本
      set_text_value(&focused_element, text)?;

      Ok(())
  }
  ```
- [ ] 处理不支持的元素类型

**Acceptance**:
- 能直接插入到文本框
- 不依赖剪贴板

#### Task 3.2.4: 插入策略选择器
- [ ] 创建 `src-tauri/src/insertion/mod.rs`
- [ ] 实现 `InsertionStrategy` 枚举
  ```rust
  pub enum InsertionStrategy {
      Clipboard,
      Accessibility,
      Fallback,
  }
  ```
- [ ] 实现 `insert_text(text: &str, strategy: InsertionStrategy) -> Result<()>`
  ```rust
  pub fn insert_text(text: &str, strategy: InsertionStrategy) -> Result<()> {
      match strategy {
          Clipboard => insert_via_clipboard(text),
          Accessibility => {
              insert_via_accessibility(text)
                  .or_else(|_| insert_via_clipboard(text))  // 降级
          },
          Fallback => {
              // 尝试所有方法
              insert_via_accessibility(text)
                  .or_else(|_| insert_via_clipboard(text))
                  .or_else(|_| Err("All insertion methods failed".into()))
          }
      }
  }
  ```

**Acceptance**:
- 策略正确选择
- 失败时能自动降级

#### Task 3.2.5: 插入 Tauri Command
- [ ] 创建 `src-tauri/src/commands/insertion.rs`
- [ ] 实现 `insert_transcribed_text` command
  ```rust
  #[tauri::command]
  async fn insert_transcribed_text(
      text: String,
      state: State<'_, AppState>,
  ) -> Result<(), String> {
      let strategy = state.insertion_strategy.lock().unwrap();
      insert_text(&text, *strategy)
          .map_err(|e| e.to_string())
  }
  ```
- [ ] 在 `main.rs` 注册 command
- [ ] 添加错误日志记录

**Acceptance**:
- 前端可调用插入功能
- 错误信息清晰返回

#### Task 3.2.6: 直接插入模式集成
- [ ] 在 `recordingStore.ts` 的 `stopRecording()` 中
- [ ] 转录完成后自动调用插入
  ```typescript
  async stopRecording() {
      set({ status: 'processing' });

      const audioData = await invoke<number[]>('stop_audio_recording');
      const text = await invoke<string>('transcribe_audio', { audioData });

      // 自动插入（直接插入模式）
      if (operationMode === 'direct') {
          try {
              await invoke('insert_transcribed_text', { text });
              set({ status: 'success' });
              // 2 秒后自动关闭
              setTimeout(() => { hideWindow(); }, 2000);
          } catch (error) {
              // 降级：复制到剪贴板
              await writeText(text);
              set({
                  status: 'fallback',
                  message: '已复制到剪贴板，请手动粘贴'
              });
          }
      }
  }
  ```

**Acceptance**:
- 直接插入模式自动插入文本
- 失败时自动复制到剪贴板

#### Task 3.2.7: 预览模式手动插入
- [ ] 在 `RecordingFloat.tsx` 预览模式 UI
- [ ] 添加 "插入" 按钮
- [ ] 点击时调用插入功能
  ```typescript
  const handleInsert = async () => {
      const text = recordingStore.streamedText;
      try {
          await invoke('insert_transcribed_text', { text });
          recordingStore.hideWindow();
      } catch (error) {
          // 显示错误提示
          toast.error('插入失败，已复制到剪贴板');
          await writeText(text);
      }
  };
  ```

**Acceptance**:
- 点击按钮插入文本到目标应用
- 失败时显示友好提示

#### Task 3.2.8: 应用兼容性测试
- [ ] 测试浏览器：Chrome, Safari, Firefox
  - 地址栏、搜索框、文本框
- [ ] 测试编辑器：VSCode, Sublime Text, Xcode
  - 代码编辑器、终端
- [ ] 测试通讯工具：微信、QQ、Slack
  - 消息输入框
- [ ] 测试 Office：Word, Excel, Pages
  - 文档编辑区
- [ ] 记录不兼容的应用（黑名单）

**Acceptance**:
- 80% 主流应用兼容
- 黑名单应用提示用户

---

## Phase 4: 集成测试与优化 (Week 6)

### 4.1 端到端测试

#### Task 4.1.1: 直接插入模式 E2E 测试
- [ ] 创建 `tests/e2e/direct_insert_mode.rs`
- [ ] 测试场景：按住快捷键 → 说话 3 秒 → 松开 → 验证文本插入
- [ ] 使用模拟音频文件（3 秒中文）
- [ ] 验证：
  - 转录文本准确率 > 85%
  - 总延迟 < 2 秒
  - 文本成功插入到测试窗口

**Acceptance**:
- 测试通过率 100%
- 无崩溃或内存泄漏

#### Task 4.1.2: 预览模式 E2E 测试
- [ ] 创建 `tests/e2e/preview_mode.rs`
- [ ] 测试场景：按快捷键 → 连续说话 15 秒（3 段话） → 停止 → 验证流式显示
- [ ] 验证：
  - 3 段文本正确累积显示
  - 停顿检测准确（每段间隔 0.5-1s）
  - 手动插入成功

**Acceptance**:
- 流式文本无乱序
- 停顿检测准确率 > 90%

#### Task 4.1.3: 长时间录制测试
- [ ] 测试预览模式录制 5 分钟
- [ ] 验证：
  - 无内存泄漏
  - 转录质量稳定
  - 30s 超时正确触发
- [ ] 监控内存占用（应 < 200MB）

**Acceptance**:
- 长时间运行稳定
- 资源占用合理

#### Task 4.1.4: 错误恢复测试
- [ ] 测试：模型文件损坏
  - 验证错误提示清晰
- [ ] 测试：网络断开（模型下载中）
  - 验证断点续传
- [ ] 测试：权限被撤销
  - 验证降级到剪贴板
- [ ] 测试：音频设备拔出
  - 验证优雅停止

**Acceptance**:
- 所有错误场景正确处理
- 无应用崩溃

### 4.2 性能优化

#### Task 4.2.1: Core ML 加速启用
- [ ] 在 whisper.cpp 编译时启用 Core ML 支持
  - 添加编译标志 `-DWHISPER_COREML=1`
- [ ] 在 `WhisperEngine::new()` 中
  - 检测 macOS 版本 (>= 12.0)
  - 设置 Core ML 上下文参数
- [ ] 对比测试：CPU vs Core ML
  - 记录推理速度提升比例

**Acceptance**:
- Core ML 推理速度比 CPU 快 3-5 倍
- base 模型 1 秒音频转录 < 150ms

#### Task 4.2.2: 多线程优化
- [ ] 调整 whisper n_threads 参数
  - 根据 CPU 核心数自动设置（num_cpus / 2）
- [ ] 测试不同线程数性能
  - 2, 4, 8 线程对比
- [ ] 选择最优配置

**Acceptance**:
- 多核 CPU 转录速度提升 2-3 倍

#### Task 4.2.3: 内存占用优化
- [ ] 实现音频数据即用即释放
  - 转录完成立即 drop Vec<f32>
- [ ] 限制转录队列大小（最多 10 个 chunk）
- [ ] 使用 Arc 共享 WhisperEngine 实例（避免多次加载）
- [ ] 监控内存占用（Instruments / Activity Monitor）

**Acceptance**:
- 持续运行 30 分钟内存稳定
- 峰值内存 < 300MB

#### Task 4.2.4: 启动速度优化
- [ ] 延迟加载 Whisper 模型
  - 首次转录时才加载
- [ ] 预热转录引擎
  - 启动时用 0.5s 静音音频预热
- [ ] 测量冷启动 vs 热启动时间

**Acceptance**:
- 应用启动时间 < 1 秒
- 首次转录延迟 < 500ms

### 4.3 用户体验优化

#### Task 4.3.1: 加载状态提示
- [ ] 在 `RecordingFloat.tsx` 添加加载动画
  - 模型加载时显示 spinner
  - 显示文本 "正在加载模型..."
- [ ] 在转录过程中显示脉动动画
  - 直接插入模式：线性进度条
  - 预览模式：波形动画

**Acceptance**:
- 用户始终知道系统状态
- 无"卡死"假象

#### Task 4.3.2: 快捷键冲突检测
- [ ] 检测常见快捷键冲突
  - Cmd+Space (Spotlight)
  - Ctrl+Space (输入法切换)
- [ ] 提示用户选择不冲突的快捷键
- [ ] 在设置页面显示冲突警告

**Acceptance**:
- 冲突时显示清晰警告
- 用户可自定义快捷键

#### Task 4.3.3: 音频质量提示
- [ ] 检测麦克风输入音量
  - 过低时显示 "音量过低，请提高麦克风音量"
  - 过高时显示 "音量过大，可能失真"
- [ ] 在预览模式显示实时音量指示器
  - 波形或柱状图

**Acceptance**:
- 用户能及时调整音量
- 转录质量提升

#### Task 4.3.4: 错误提示优化
- [ ] 统一错误提示样式（Toast 通知）
- [ ] 错误信息本地化（中文）
  - "Model not found" → "模型文件未找到，请下载"
  - "Permission denied" → "无辅助功能权限，请授权"
- [ ] 提供解决方案链接
  - 点击打开帮助文档或设置页面

**Acceptance**:
- 错误提示友好易懂
- 用户知道如何解决

### 4.4 文档与发布

#### Task 4.4.1: 更新 CHANGELOG
- [ ] 记录新功能：
  - ✨ 双模式语音输入（直接插入 & 预览模式）
  - ✨ 实时流式转录
  - ✨ 智能停顿检测（自适应 0.3-0.8s）
  - ✨ 跨应用文本插入
  - ✨ Whisper 模型管理
- [ ] 记录技术改进：
  - ⚡️ Core ML GPU 加速
  - ⚡️ 多线程并行转录
  - 🐛 修复若干 Bug

**Acceptance**:
- CHANGELOG 完整清晰

#### Task 4.4.2: 更新用户文档
- [ ] 编写使用指南
  - 如何授权辅助功能权限
  - 如何下载 Whisper 模型
  - 直接插入模式使用方法
  - 预览模式使用方法
- [ ] 添加常见问题 FAQ
  - 转录不准确？
  - 无法插入文本？
  - 如何更换模型？

**Acceptance**:
- 新用户能快速上手

#### Task 4.4.3: 验证 OpenSpec 规范
- [ ] 运行 `openspec validate`
- [ ] 修复所有验证错误
- [ ] 确保 specs 与实现一致

**Acceptance**:
- 验证通过无错误

#### Task 4.4.4: 归档旧提案
- [ ] 归档 `add-audio-recording` 提案
  - 运行 `openspec archive add-audio-recording`
  - 创建对应的 spec 文件
- [ ] 归档 `add-speech-recognition` 提案
- [ ] 归档 `add-text-insertion` 提案
- [ ] 验证归档后项目状态

**Acceptance**:
- 旧提案正确归档
- 新提案作为唯一活跃提案

#### Task 4.4.5: 提交和合并
- [ ] 提交所有代码变更
  ```bash
  git add .
  git commit -m "feat: 实现语音输入系统（双模式 + 实时转录）"
  ```
- [ ] 创建 Pull Request
- [ ] Code Review
- [ ] 合并到主分支

**Acceptance**:
- 代码审查通过
- CI 测试全部通过

---

## 总结

**总任务数**: 105 个
**预计工期**: 6 周
**关键里程碑**:
- Week 2: 直接插入模式可用
- Week 4: 预览模式流式转录可用
- Week 5: 跨应用插入完成
- Week 6: 完整测试与发布

**技术风险**:
1. Core ML 加速可能需要额外调试
2. 跨应用插入兼容性可能低于预期
3. AdaptiveVAD 算法需要大量测试调优

**缓解措施**:
- 预留 buffer 时间（每阶段 +2 天）
- 早期进行兼容性测试
- 提供降级方案（CPU 推理、剪贴板插入）

---

## Phase 1 实施状态 (已完成)

### 完成时间
2025-11-10

### 完成的任务
✅ **Task 1.1.1**: whisper.cpp FFI 绑定 (使用 whisper-rs v0.15 库)
✅ **Task 1.1.2**: WhisperEngine 核心结构
✅ **Task 1.1.3**: 音频预处理模块
✅ **Task 1.1.4**: 基础转录接口
✅ **Task 1.1.5**: Tauri Command 封装

### 实现亮点
1. **正确使用 whisper-rs API**: 采用 WhisperContext + WhisperState 模式，避免了 API 误用
2. **解决 Send/Sync 问题**: 使用全局静态变量 (once_cell::Lazy) 管理音频录制器，避免 cpal::Stream 在 macOS 上的 Send trait 限制
3. **完善的音频预处理**: 实现 i16→f32 转换、采样率验证、音量归一化等完整流程
4. **前端深度集成**:
   - ModelSettings.tsx 自动初始化 Whisper 引擎
   - recordingStore.ts 集成真实转录 API
   - 支持语言设置和自动检测
5. **额外实现**: transcribe_with_timestamps() 方法（为 Phase 2 做准备）

### 技术要点
- **依赖**: whisper-rs v0.15, num_cpus v1.16, once_cell v1.19
- **架构**: WhisperEngine 直接持有 WhisperContext（不使用 Arc/Mutex）
- **线程优化**: 自动设置转录线程数为 CPU 核心数的一半（限制在 1-8 之间）
- **错误处理**: 完整的错误类型定义和转换（WhisperError → String）
- **资源管理**: 使用 Rust RAII 自动清理资源

### 未完成的任务
⏸️ **Task 1.2.x**: 转录进度反馈（推迟到后续 Phase 实现）

### 下一步
- 测试 Phase 1 功能的完整性
- 进入 Phase 2: 实时流式转录系统
