# Design: 语音输入系统技术设计（双模式 + 实时转录）

## Context

「聆码」实现了**双操作模式**的语音输入系统：
- **直接插入模式**: 按住说话，松开自动转录插入（快速输入）
- **预览模式**: 实时流式转录，边说边显示文字（长文本创作）

**当前状态** (70% 完成):
- ✅ 双模式 UI 组件完整实现
- ✅ 悬浮窗管理和智能定位
- ✅ 快捷键系统（按下/松开事件）
- ✅ 音频录制底层（Rust AudioRecorder）
- ✅ 模型下载管理系统

**本设计目标** (剩余 30%):
- 集成 Whisper 转录引擎
- 实现实时流式转录管道
- 智能停顿检测与音频切片
- 跨应用文本插入

## Goals / Non-Goals

### Goals
- 直接插入模式延迟 < 5秒（含录音）
- 预览模式实时转录延迟 < 1秒（停顿后）
- 智能停顿检测准确率 > 90%
- 转录准确率 > 90%（中文）
- 文本插入成功率 > 95%

### Non-Goals
- 不支持在线语音识别（完全本地）
- 不支持富文本插入（仅纯文本）
- 不支持多语言实时切换（一期固定中文）
- 不实现语音指令（二期）

## Overall Architecture

### 系统整体架构

```
┌──────────────────────────────────────────────────────────┐
│                     用户交互层                            │
│  ┌────────────┐         ┌────────────┐                  │
│  │ 全局快捷键  │──事件──▶│ 录音悬浮窗  │                  │
│  │Cmd+Shift+S │  通知   │ 双模式 UI   │                  │
│  └────────────┘         └────────────┘                  │
└──────────────────────────────────────────────────────────┘
                     │
                     ▼ Tauri Event
┌──────────────────────────────────────────────────────────┐
│              状态管理层 (Zustand)                         │
│  recordingStore: {                                       │
│    operationMode: 'direct' | 'preview',                 │
│    state: 'idle' | 'recording' | 'processing',          │
│    transcribedText: string,        // 实时累积          │
│    transcription: string           // 最终结果           │
│  }                                                       │
└──────────────────────────────────────────────────────────┘
                     │
                     ▼ Tauri Commands
┌──────────────────────────────────────────────────────────┐
│               Rust Commands Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ 录制 API      │  │ 转录 API      │  │ 插入 API      │  │
│  │              │  │              │  │              │  │
│  │start_        │  │transcribe_   │  │insert_text   │  │
│  │recording     │  │audio         │  │              │  │
│  │              │  │              │  │check_        │  │
│  │start_        │  │              │  │accessibility │  │
│  │streaming     │  │              │  │              │  │
│  │              │  │              │  │              │  │
│  │stop_         │  │              │  │              │  │
│  │recording     │  │              │  │              │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└──────────────────────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│              核心处理层 (Rust Modules)                    │
│                                                          │
│  [直接插入模式流程]                                       │
│  ┌───────────┐                                          │
│  │AudioRecorder│ → 录音缓冲 → 完整音频数据(Vec<i16>)      │
│  │(常规录制)   │    ↓                                    │
│  └───────────┘  WhisperEngine.transcribe()             │
│                    ↓                                    │
│                 文本结果 → InsertionEngine → 自动插入     │
│                                                          │
│  [预览模式流程]                                           │
│  ┌───────────┐                                          │
│  │AudioRecorder│ → 实时音频流                            │
│  │(流式模式)   │    ↓                                    │
│  └───────────┘  AdaptiveVAD（停顿检测）                  │
│                    ↓ 检测到停顿(0.5s)                    │
│                 音频切片提取                              │
│                    ↓ 异步任务(tokio::spawn)              │
│                 WhisperEngine.transcribe_chunk()        │
│                    ↓ Tauri Event                        │
│                 前端追加文本(transcription-chunk)         │
│                    ↓                                    │
│                 继续录音...循环往复                       │
│                    ↓ 用户结束                           │
│                 显示完整文本 → 用户手动插入              │
└──────────────────────────────────────────────────────────┘
```

## Key Technical Decisions

### 决策 1: 双模式架构 - 独立流程 vs 共享代码

**决策**: **共享底层，独立流程**

**架构设计**:
```rust
// 共享的音频录制底层
struct AudioRecorder {
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<i16>>>,
    // ...
}

// 模式 A: 直接插入（简单录制）
impl AudioRecorder {
    pub fn start_recording(&mut self) -> Result<(), String> {
        // 开始录制，累积到缓冲区
    }

    pub fn stop_recording(&mut self) -> Result<Vec<i16>, String> {
        // 停止并返回完整音频
    }
}

// 模式 B: 预览（流式录制）
impl AudioRecorder {
    pub fn start_streaming(
        &mut self,
        vad: Arc<Mutex<AdaptiveVAD>>,
        on_chunk: impl Fn(Vec<i16>) + Send + 'static,
    ) -> Result<(), String> {
        // 启动录制 + VAD 检测循环
        // 检测到停顿时调用 on_chunk 回调
    }
}
```

**理由**:
- 底层录制逻辑复杂，不重复实现
- 上层流程差异大，独立处理更清晰
- 便于测试和维护

---

### 决策 2: 实时转录 - 同步 vs 异步

**决策**: **完全异步管道**

**实现架构**:
```rust
// 录音线程（不阻塞）
tokio::spawn(async move {
    loop {
        // 检查 VAD
        if vad.detect_pause(&audio_buffer) {
            let chunk = extract_chunk(&audio_buffer);

            // 异步转录任务（不阻塞录音）
            let whisper = whisper_engine.clone();
            tokio::spawn(async move {
                let text = whisper.transcribe(&chunk).await;
                window.emit("transcription-chunk", text).unwrap();
            });
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
});
```

**理由**:
- 转录耗时 1-3秒，不能阻塞录音
- 用户可能说很快，需要并发处理多个切片
- 提升用户体验（边说边看到文字）

---

### 决策 3: 停顿检测 - 固定阈值 vs 智能自适应

**决策**: **智能自适应阈值**

**算法设计**:
```rust
struct AdaptiveVAD {
    silence_threshold: f32,        // 静音判断阈值（动态）
    min_pause_duration: Duration,  // 最小停顿时长（智能）
    max_pause_duration: Duration,  // 最大停顿（30秒超时保护）
    speech_rate_history: Vec<f32>, // 说话速度历史（近 5秒）
}

impl AdaptiveVAD {
    // 自适应调整停顿阈值
    fn adjust_threshold(&mut self, audio_samples: &[f32]) {
        // 1. 计算说话速度（语音密度）
        let speech_density = self.calculate_speech_density(audio_samples);
        self.speech_rate_history.push(speech_density);

        // 2. 计算平均说话速度
        let avg_rate: f32 = self.speech_rate_history.iter().sum::<f32>()
            / self.speech_rate_history.len() as f32;

        // 3. 动态调整停顿阈值
        // 说话快（高密度）→ 短停顿 0.3s
        // 说话慢（低密度）→ 长停顿 0.8s
        // 正常速度 → 0.5s
        self.min_pause_duration = Duration::from_millis(
            (300.0 + (1.0 - avg_rate) * 500.0) as u64
        );
    }

    // 检测是否为句子停顿
    fn is_sentence_boundary(
        &self,
        silence_duration: Duration,
    ) -> bool {
        silence_duration >= self.min_pause_duration
            && silence_duration < self.max_pause_duration
    }

    // 计算语音密度（0.0-1.0）
    fn calculate_speech_density(&self, samples: &[f32]) -> f32 {
        let energy: f32 = samples.iter()
            .map(|&s| s * s)
            .sum();

        let avg_energy = energy / samples.len() as f32;

        // 归一化到 0.0-1.0
        (avg_energy / 0.01).min(1.0)
    }
}
```

**理由**:
- 用户说话节奏差异大（有人快，有人慢）
- 固定阈值容易误判或漏判
- 自适应可提升 10-20% 准确率
- 初始值 0.5秒（保守，不易误判）

---

### 决策 4: 音频切片 - 精确切割 vs 重叠窗口

**决策**: **精确切割 + 最小上下文**

**实现**:
```rust
struct AudioChunkProcessor {
    buffer: VecDeque<i16>,
    last_speech_pos: usize,
}

impl AudioChunkProcessor {
    // 提取音频切片
    fn extract_chunk(&mut self, pause_pos: usize) -> Vec<i16> {
        // 从上次语音结束到当前停顿位置
        let start = self.last_speech_pos;
        let end = pause_pos;

        // 添加少量上下文（50ms 前后）
        let context_samples = 800;  // 16kHz * 0.05s
        let start_with_context = start.saturating_sub(context_samples);
        let end_with_context = (end + context_samples).min(self.buffer.len());

        let chunk: Vec<i16> = self.buffer
            .range(start_with_context..end_with_context)
            .copied()
            .collect();

        // 清理已处理的音频（释放内存）
        self.buffer.drain(0..end);
        self.last_speech_pos = 0;

        chunk
    }
}
```

**理由**:
- 精确切割避免重复转录
- 少量上下文提升识别准确度
- 及时释放内存，避免溢出

---

## Implementation Details

### 模块 1: Whisper 转录引擎集成

#### 1.1 编译 whisper.cpp

**脚本**:
```bash
#!/bin/bash
# scripts/build-whisper.sh

# 下载 whisper.cpp
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp

# 编译为静态库（启用 Core ML）
make WHISPER_COREML=1 libwhisper.a

# 安装到项目
cp libwhisper.a ../src-tauri/lib/
cp ggml.h whisper.h ../src-tauri/include/
```

#### 1.2 Rust FFI 绑定

```rust
// src-tauri/src/whisper/ffi.rs

use std::os::raw::{c_char, c_float, c_int};

#[repr(C)]
pub struct WhisperContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct WhisperFullParams {
    pub n_threads: c_int,
    pub language: *const c_char,
    pub translate: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    // ... 其他参数
}

#[link(name = "whisper", kind = "static")]
extern "C" {
    pub fn whisper_init_from_file(
        path_model: *const c_char
    ) -> *mut WhisperContext;

    pub fn whisper_free(ctx: *mut WhisperContext);

    pub fn whisper_full(
        ctx: *mut WhisperContext,
        params: WhisperFullParams,
        samples: *const c_float,
        n_samples: c_int,
    ) -> c_int;

    pub fn whisper_full_n_segments(
        ctx: *mut WhisperContext
    ) -> c_int;

    pub fn whisper_full_get_segment_text(
        ctx: *mut WhisperContext,
        i_segment: c_int,
    ) -> *const c_char;
}
```

#### 1.3 Whisper Engine 核心

```rust
// src-tauri/src/whisper/engine.rs

use std::ffi::{CStr, CString};
use crate::whisper::ffi::*;

pub struct WhisperEngine {
    ctx: *mut WhisperContext,
    model_path: String,
}

unsafe impl Send for WhisperEngine {}
unsafe impl Sync for WhisperEngine {}

impl WhisperEngine {
    pub fn new(model_path: &str) -> Result<Self, String> {
        let path_cstr = CString::new(model_path)
            .map_err(|e| e.to_string())?;

        let ctx = unsafe {
            whisper_init_from_file(path_cstr.as_ptr())
        };

        if ctx.is_null() {
            return Err("Failed to load Whisper model".to_string());
        }

        Ok(Self {
            ctx,
            model_path: model_path.to_string(),
        })
    }

    pub fn transcribe(
        &mut self,
        audio_i16: &[i16],
        language: &str,
    ) -> Result<String, String> {
        // 1. 转换 i16 → f32 (-1.0 to 1.0)
        let samples: Vec<f32> = audio_i16
            .iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();

        // 2. 设置参数
        let lang_cstr = CString::new(language)
            .map_err(|e| e.to_string())?;

        let params = WhisperFullParams {
            n_threads: num_cpus::get() as i32,
            language: lang_cstr.as_ptr(),
            translate: false,
            print_progress: false,
            print_realtime: false,
            // ... 其他默认参数
        };

        // 3. 执行推理
        let result = unsafe {
            whisper_full(
                self.ctx,
                params,
                samples.as_ptr(),
                samples.len() as i32,
            )
        };

        if result != 0 {
            return Err("Transcription failed".to_string());
        }

        // 4. 提取文本
        let n_segments = unsafe {
            whisper_full_n_segments(self.ctx)
        };

        let mut text = String::new();
        for i in 0..n_segments {
            let segment_text = unsafe {
                let ptr = whisper_full_get_segment_text(self.ctx, i);
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            };
            text.push_str(&segment_text);
        }

        // 5. 后处理
        let text = self.postprocess_text(&text, language);

        Ok(text.trim().to_string())
    }

    fn postprocess_text(&self, text: &str, language: &str) -> String {
        let mut result = text.to_string();

        if language == "zh" {
            // 中文标点优化
            result = result.replace("。 ", "。");
            result = result.replace("， ", "，");
        } else if language == "en" {
            // 英文首字母大写
            if let Some(first_char) = result.chars().next() {
                if first_char.is_lowercase() {
                    result = first_char.to_uppercase().to_string() + &result[1..];
                }
            }
        }

        result
    }
}

impl Drop for WhisperEngine {
    fn drop(&mut self) {
        unsafe {
            whisper_free(self.ctx);
        }
    }
}
```

---

### 模块 2: 实时流式转录管道

#### 2.1 智能 VAD 实现

```rust
// src-tauri/src/audio/adaptive_vad.rs

use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct AdaptiveVAD {
    silence_threshold: f32,
    min_pause_duration: Duration,
    max_pause_duration: Duration,
    speech_density_history: VecDeque<f32>,
    silence_start: Option<Instant>,
}

impl AdaptiveVAD {
    pub fn new() -> Self {
        Self {
            silence_threshold: 0.01,  // 能量阈值
            min_pause_duration: Duration::from_millis(500),  // 初始 0.5s
            max_pause_duration: Duration::from_secs(30),    // 超时保护
            speech_density_history: VecDeque::with_capacity(50),
            silence_start: None,
        }
    }

    // 检测当前是否为静音
    pub fn is_silence(&self, audio_chunk: &[f32]) -> bool {
        let energy = self.calculate_energy(audio_chunk);
        energy < self.silence_threshold
    }

    // 检测是否为句子停顿
    pub fn detect_pause(&mut self, audio_chunk: &[f32]) -> Option<Duration> {
        let is_silent = self.is_silence(audio_chunk);

        if is_silent {
            if self.silence_start.is_none() {
                self.silence_start = Some(Instant::now());
            }

            let silence_duration = self.silence_start.unwrap().elapsed();

            // 自适应调整阈值
            self.adjust_threshold(audio_chunk);

            // 判断是否达到停顿时长
            if silence_duration >= self.min_pause_duration {
                self.silence_start = None;  // 重置
                return Some(silence_duration);
            }
        } else {
            self.silence_start = None;  // 有声音，重置计时
        }

        None
    }

    // 计算音频能量
    fn calculate_energy(&self, samples: &[f32]) -> f32 {
        let energy: f32 = samples.iter()
            .map(|&s| s * s)
            .sum();

        energy / samples.len() as f32
    }

    // 自适应调整阈值
    fn adjust_threshold(&mut self, samples: &[f32]) {
        let energy = self.calculate_energy(samples);
        self.speech_density_history.push_back(energy);

        if self.speech_density_history.len() > 50 {
            self.speech_density_history.pop_front();
        }

        // 计算平均语音密度
        let avg_density: f32 = self.speech_density_history
            .iter()
            .sum::<f32>() / self.speech_density_history.len() as f32;

        // 动态调整停顿时长
        // 高密度（说话快）→ 短停顿 300ms
        // 低密度（说话慢）→ 长停顿 800ms
        let normalized_density = (avg_density / 0.02).min(1.0);
        let pause_ms = 300.0 + (1.0 - normalized_density) * 500.0;

        self.min_pause_duration = Duration::from_millis(pause_ms as u64);
    }
}
```

#### 2.2 流式录制 Command

```rust
// src-tauri/src/commands/audio.rs (扩展)

use tauri::{AppHandle, Manager, Window};
use tokio::sync::Mutex;
use std::sync::Arc;

#[tauri::command]
pub async fn start_streaming_recording(
    app: AppHandle,
    recorder: State<'_, Arc<Mutex<AudioRecorder>>>,
    whisper: State<'_, Arc<Mutex<WhisperEngine>>>,
) -> Result<(), String> {
    let mut recorder = recorder.lock().await;
    let whisper = whisper.clone();
    let window = app.get_webview_window("recording_float")
        .ok_or("Window not found")?;

    // 创建 VAD
    let vad = Arc::new(Mutex::new(AdaptiveVAD::new()));

    // 启动流式录制
    recorder.start_streaming(vad.clone(), move |audio_chunk: Vec<i16>| {
        let whisper = whisper.clone();
        let window = window.clone();

        // 异步转录任务
        tokio::spawn(async move {
            let mut whisper = whisper.lock().await;

            match whisper.transcribe(&audio_chunk, "zh") {
                Ok(text) => {
                    // 发送转录片段到前端
                    let _ = window.emit("transcription-chunk", text);
                }
                Err(e) => {
                    eprintln!("Transcription error: {}", e);
                }
            }
        });
    })?;

    Ok(())
}
```

---

### 模块 3: 跨应用文本插入

#### 3.1 剪贴板插入策略

```rust
// src-tauri/src/insertion/clipboard.rs

use clipboard::{ClipboardContext, ClipboardProvider};
use enigo::{Enigo, Key, KeyboardControllable};
use std::thread;
use std::time::Duration;

pub struct ClipboardInsertion;

impl ClipboardInsertion {
    pub fn insert(text: &str) -> Result<(), String> {
        // 1. 备份剪贴板
        let backup = Self::get_clipboard()?;

        // 2. 写入转录文本
        Self::set_clipboard(text)?;

        // 3. 等待剪贴板更新
        thread::sleep(Duration::from_millis(50));

        // 4. 模拟 Cmd+V (macOS) / Ctrl+V (其他)
        Self::simulate_paste()?;

        // 5. 延迟恢复剪贴板
        thread::sleep(Duration::from_millis(100));
        if !backup.is_empty() {
            Self::set_clipboard(&backup)?;
        }

        Ok(())
    }

    fn get_clipboard() -> Result<String, String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| e.to_string())?;

        ctx.get_contents()
            .unwrap_or_else(|_| String::new())
            .into()
    }

    fn set_clipboard(text: &str) -> Result<(), String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| e.to_string())?;

        ctx.set_contents(text.to_string())
            .map_err(|e| e.to_string())
    }

    fn simulate_paste() -> Result<(), String> {
        let mut enigo = Enigo::new();

        #[cfg(target_os = "macos")]
        {
            enigo.key_down(Key::Meta);  // Cmd
            enigo.key_click(Key::Layout('v'));
            enigo.key_up(Key::Meta);
        }

        #[cfg(not(target_os = "macos"))]
        {
            enigo.key_down(Key::Control);
            enigo.key_click(Key::Layout('v'));
            enigo.key_up(Key::Control);
        }

        Ok(())
    }
}
```

#### 3.2 插入 Command

```rust
// src-tauri/src/commands/insertion.rs

#[tauri::command]
pub async fn insert_text(text: String) -> Result<(), String> {
    ClipboardInsertion::insert(&text)
}

#[tauri::command]
pub async fn check_accessibility_permission() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        // macOS 辅助功能权限检查
        // 简化版本，实际需要更复杂的实现
        Ok(true)  // 暂时占位
    }

    #[cfg(not(target_os = "macos"))]
    Ok(true)
}
```

---

## Performance Targets

| 指标 | 目标值 | 测量方法 |
|------|-------|---------|
| 快捷键响应 | < 50ms | 按下到窗口显示 |
| 录音启动 | < 100ms | start_recording() 到首个音频数据 |
| VAD 检测 | < 100ms | 停顿发生到检测触发 |
| 转录延迟 (base) | < 1秒 | 停顿后到文本出现（预览模式） |
| 转录延迟 (Core ML) | < 0.5秒 | 使用 GPU 加速 |
| 文本插入 | < 100ms | insert_text() 到完成 |
| **端到端 (直接)** | **< 5秒** | 按住到插入（30秒录音） |
| **实时转录 (预览)** | **< 1秒** | 停顿到文本出现 |

## Testing Strategy

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_vad_threshold_adjustment() {
        let mut vad = AdaptiveVAD::new();

        // 模拟快速说话（高能量）
        let fast_speech = vec![0.5f32; 1600];  // 100ms @ 16kHz
        vad.adjust_threshold(&fast_speech);

        // 阈值应降低（检测更快停顿）
        assert!(vad.min_pause_duration < Duration::from_millis(500));

        // 模拟慢速说话（低能量）
        let slow_speech = vec![0.1f32; 1600];
        vad.adjust_threshold(&slow_speech);

        // 阈值应提高（容忍更长停顿）
        assert!(vad.min_pause_duration > Duration::from_millis(500));
    }

    #[test]
    fn test_audio_chunk_extraction() {
        let mut processor = AudioChunkProcessor::new();

        // 添加音频数据
        processor.buffer.extend(vec![1i16; 16000]);  // 1秒音频

        // 提取切片
        let chunk = processor.extract_chunk(8000);  // 0.5秒位置

        assert_eq!(chunk.len(), 8000 + 800 * 2);  // 加上上下文
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_full_streaming_workflow() {
    // 1. 加载测试音频（多段话，带停顿）
    let audio_file = "tests/fixtures/multi-sentence-zh.wav";

    // 2. 启动流式转录
    let mut chunks_received = Vec::new();

    start_streaming_recording_with_callback(audio_file, |text| {
        chunks_received.push(text);
    }).await;

    // 3. 验证收到多个切片
    assert!(chunks_received.len() >= 2);

    // 4. 验证文本内容
    let full_text = chunks_received.join(" ");
    assert!(full_text.contains("测试"));
}
```

---

## Security & Privacy

- ✅ 所有音频处理完全本地
- ✅ 模型文件 SHA256 验证
- ✅ 音频数据转录后立即释放
- ✅ 不上传任何音频或文本
- ✅ 剪贴板数据妥善备份恢复
- ✅ 用户可删除所有历史记录

---

## Future Optimizations

### Phase 2
- 停顿阈值用户可调（高级设置）
- 模型热切换（无需重启）
- 缓存常用短语

### Phase 3
- Whisper 量化模型（INT8）
- 流式 VAD 优化
- 预测性模型加载
