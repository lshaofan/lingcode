# Design: 语音录制功能技术设计

## Context

语音录制是「聆码」的核心入口，需要提供低延迟、高可靠的音频捕获能力。设计需要平衡用户体验、性能和跨平台兼容性。

## Goals / Non-Goals

### Goals
- 实现全局快捷键触发录音
- 提供实时音频可视化反馈
- 确保音频质量满足 Whisper 输入要求（16kHz, 16-bit PCM）
- 支持 VAD 自动停止录音
- 录音延迟 < 100ms

### Non-Goals
- 不支持多通道/立体声录音（仅单声道）
- 不提供音频后期处理（降噪、增益等）
- 不支持录音文件持久化（仅转录后删除）
- 不支持录音暂停/恢复（仅开始/停止）

## Architecture

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    用户交互层                            │
│  ┌──────────────┐         ┌──────────────┐             │
│  │ 全局快捷键    │         │ 录音悬浮窗    │             │
│  │ Cmd+Shift+S  │────────▶│ 波形 + 控制   │             │
│  └──────────────┘         └──────────────┘             │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                   状态管理层 (Zustand)                   │
│  recordingStore: { isRecording, audioData, duration }   │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                Tauri Commands (Rust)                    │
│  - start_recording()                                    │
│  - stop_recording()                                     │
│  - get_audio_devices()                                  │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              音频处理层 (Rust + cpal)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 音频捕获      │─▶│ 格式转换      │─▶│ VAD 检测     │ │
│  │ (cpal)       │  │ (16kHz PCM)  │  │ (webrtc-vad) │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
                 音频数据缓冲区
                  (发送到前端)
```

### 数据流

```
1. 用户按下快捷键
   ↓
2. global-shortcut 插件触发事件
   ↓
3. 前端调用 start_recording() Command
   ↓
4. Rust 后端启动音频流捕获
   ↓
5. 实时音频数据通过 Tauri Event 发送到前端
   ↓
6. 前端更新波形可视化
   ↓
7. 用户停止录音 OR VAD 检测到静音
   ↓
8. 调用 stop_recording()，返回完整音频数据
   ↓
9. 传递给语音识别模块
```

## Decisions

### 决策 1: 音频库选择 - cpal vs rodio

**决策**: 使用 `cpal` (Cross-Platform Audio Library)

**原因**:
- **底层控制**: cpal 提供低级别音频流访问，适合实时处理
- **跨平台**: 支持 macOS (Core Audio), Windows (WASAPI), Linux (ALSA)
- **性能**: 更低的延迟，适合实时录音
- **社区**: 活跃维护，Tauri 生态常用

**权衡**:
- rodio 更高级，但主要用于音频播放，不适合录音场景

### 决策 2: 快捷键方案 - tauri-plugin-global-shortcut vs rdev

**决策**: 使用 `tauri-plugin-global-shortcut`

**原因**:
- **官方插件**: Tauri 官方维护，兼容性好
- **简单 API**: 注册和监听更简单
- **安全性**: 通过 Tauri 权限系统管理

**权衡**:
- rdev 功能更强大，但需要额外权限和配置

### 决策 3: 音频格式 - WAV vs MP3 vs Opus

**决策**: 使用 **WAV (16kHz, 16-bit PCM, Mono)**

**原因**:
- **Whisper 标准**: Whisper 模型要求 16kHz 采样率
- **无损**: PCM 格式无损，保证音质
- **简单**: 不需要编解码，减少延迟
- **兼容性**: 所有平台原生支持

**权衡**:
- 文件大: 1 分钟约 2MB，但仅内存缓存，不持久化

### 决策 4: VAD 引擎 - webrtc-vad vs silero-vad

**决策**: 使用 `webrtc-vad` (WebRTC Voice Activity Detection)

**原因**:
- **轻量级**: C 库，Rust bindings 可用
- **实时性**: 低延迟，适合流式处理
- **成熟**: Google WebRTC 项目的一部分

**权衡**:
- silero-vad 更准确，但需要 ONNX 运行时，增加复杂度

### 决策 5: 录音窗口实现 - 独立窗口 vs 内嵌窗口

**决策**: **独立悬浮窗口**

**原因**:
- **置顶**: 录音时始终可见，不被遮挡
- **灵活**: 用户可拖动到任意位置
- **专注**: 不干扰其他应用

**实现细节**:
```json
{
  "label": "recording",
  "title": "录音中...",
  "width": 320,
  "height": 120,
  "resizable": false,
  "decorations": false,
  "alwaysOnTop": true,
  "transparent": true,
  "skipTaskbar": true
}
```

## Implementation Details

### Rust 音频录制模块

```rust
// src-tauri/src/audio/recorder.rs

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<i16>>>,
    is_recording: Arc<Mutex<bool>>,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self, window: tauri::Window) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("没有找到输入设备")?;

        let config = cpal::StreamConfig {
            channels: 1, // 单声道
            sample_rate: cpal::SampleRate(16000), // 16kHz
            buffer_size: cpal::BufferSize::Fixed(512),
        };

        let buffer = Arc::clone(&self.buffer);
        let is_recording = Arc::clone(&self.is_recording);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if *is_recording.lock().unwrap() {
                    // 转换 f32 到 i16
                    let samples: Vec<i16> = data.iter()
                        .map(|&s| (s * i16::MAX as f32) as i16)
                        .collect();

                    buffer.lock().unwrap().extend(samples.clone());

                    // 发送实时音频数据到前端（用于波形）
                    let _ = window.emit("audio-data", samples);
                }
            },
            move |err| eprintln!("录音错误: {}", err),
            None,
        ).map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;
        *is_recording.lock().unwrap() = true;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<i16>, String> {
        *self.is_recording.lock().unwrap() = false;
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        let audio_data = self.buffer.lock().unwrap().clone();
        self.buffer.lock().unwrap().clear();

        Ok(audio_data)
    }
}
```

### Tauri Commands

```rust
// src-tauri/src/commands/audio.rs

use tauri::State;
use std::sync::Mutex;
use crate::audio::recorder::AudioRecorder;

#[tauri::command]
pub async fn start_recording(
    recorder: State<'_, Mutex<AudioRecorder>>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut recorder = recorder.lock().unwrap();
    recorder.start(window)?;
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    recorder: State<'_, Mutex<AudioRecorder>>,
) -> Result<Vec<i16>, String> {
    let mut recorder = recorder.lock().unwrap();
    recorder.stop()
}

#[tauri::command]
pub async fn check_microphone_permission() -> Result<bool, String> {
    // macOS 特定权限检查
    #[cfg(target_os = "macos")]
    {
        // 使用 AVFoundation 检查权限
        // 简化版本，实际需要 Objective-C 桥接
        Ok(true) // 占位符
    }

    #[cfg(not(target_os = "macos"))]
    Ok(true)
}
```

### 前端录音窗口

```tsx
// src/windows/recording/RecordingWindow.tsx

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useRecordingStore } from '@/stores/recordingStore'
import { WaveformVisualizer } from './WaveformVisualizer'

export function RecordingWindow() {
  const [duration, setDuration] = useState(0)
  const { isRecording, setAudioData } = useRecordingStore()

  useEffect(() => {
    const unlisten = listen<number[]>('audio-data', (event) => {
      setAudioData(event.payload)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  useEffect(() => {
    if (!isRecording) return

    const interval = setInterval(() => {
      setDuration((prev) => prev + 1)
    }, 1000)

    return () => clearInterval(interval)
  }, [isRecording])

  const handleStop = async () => {
    try {
      const audioData = await invoke<number[]>('stop_recording')
      // 传递给语音识别模块
      console.log('录音完成', audioData.length, '采样点')
    } catch (error) {
      console.error('停止录音失败:', error)
    }
  }

  return (
    <div className="h-full bg-gray-900/95 backdrop-blur rounded-2xl p-4 flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-red-500 animate-pulse" />
          <span className="text-white text-sm">录音中</span>
        </div>
        <span className="text-gray-300 text-sm font-mono">
          {formatDuration(duration)}
        </span>
      </div>

      <WaveformVisualizer />

      <div className="mt-4 flex gap-2">
        <button
          onClick={handleStop}
          className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg"
        >
          完成
        </button>
        <button
          onClick={() => invoke('cancel_recording')}
          className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg"
        >
          取消
        </button>
      </div>
    </div>
  )
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
}
```

### 全局快捷键注册

```rust
// src-tauri/src/main.rs

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 注册默认快捷键
            let shortcut: Shortcut = "Cmd+Shift+S".parse().unwrap();

            app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut| {
                // 打开录音窗口
                if let Some(window) = app.get_webview_window("recording") {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    // 创建新窗口
                    let _ = tauri::WebviewWindowBuilder::new(
                        app,
                        "recording",
                        tauri::WebviewUrl::App("recording.html".into()),
                    )
                    .title("录音中...")
                    .inner_size(320.0, 120.0)
                    .resizable(false)
                    .decorations(false)
                    .always_on_top(true)
                    .transparent(true)
                    .skip_taskbar(true)
                    .build();
                }
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Testing Strategy

### 单元测试
- 音频格式转换测试
- VAD 检测准确性测试
- 缓冲区管理测试

### 集成测试
- 完整录音流程测试
- 快捷键触发测试
- 权限检查测试

### 手动测试清单
- [ ] 首次启动权限申请流程
- [ ] 快捷键冲突场景
- [ ] 长时间录音（5分钟+）内存占用
- [ ] 多次连续录音稳定性
- [ ] 不同麦克风设备兼容性

## Performance Targets

- 快捷键响应时间: < 50ms
- 录音启动延迟: < 100ms
- 音频处理延迟: < 50ms
- 内存占用: < 50MB (5分钟录音)
- CPU 占用: < 10% (录音时)

## Security & Privacy

- 音频数据仅存储在内存中
- 转录后立即清除音频缓冲区
- 不上传任何音频数据
- 权限申请透明化，用户完全知情
