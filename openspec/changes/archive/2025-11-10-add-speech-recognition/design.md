# Design: 语音识别功能技术设计

## Context

Whisper 是 OpenAI 开源的多语言语音识别模型，通过 Transformer 架构实现高准确度的转录。选择 `whisper.cpp` 作为推理引擎是因为其高性能的 C++ 实现，支持 CPU 和 GPU 加速，非常适合桌面应用。

## Goals / Non-Goals

### Goals
- 集成 whisper.cpp 实现本地语音识别
- 支持多个模型大小（tiny, base, small, medium）
- 中文转录准确率 > 85%
- 转录延迟 < 3 秒（1 分钟音频）
- 支持中英文混合识别
- 提供模型管理界面

### Non-Goals
- 不实现在线语音识别（完全本地）
- 不支持实时流式转录（批量处理）
- 不支持 Whisper large 模型（太大，2GB+）
- 不实现自定义 fine-tune（一期）

## Architecture

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                   用户界面层                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 转录进度     │  │ 历史记录      │  │ 模型管理     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              状态管理层 (Zustand)                        │
│  transcriptionStore: { status, result, progress }       │
└─────────────────────────────────────────────────────────┐
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                Tauri Commands (Rust)                    │
│  - transcribe_audio()                                   │
│  - download_model()                                     │
│  - list_models()                                        │
│  - switch_model()                                       │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│           Whisper 推理引擎 (whisper.cpp FFI)            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 模型加载      │─▶│ 音频预处理    │─▶│ 批量推理     │ │
│  │ (GGML)       │  │ (16kHz PCM)  │  │ (多线程)     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                          │                              │
│                          ▼                              │
│                   ┌──────────────┐                      │
│                   │ 后处理        │                      │
│                   │ (标点/大小写) │                      │
│                   └──────────────┘                      │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
                  转录结果文本
              (存储到 SQLite)
```

### 数据流

```
1. 录音完成，获得音频数据 (Vec<i16>)
   ↓
2. 调用 transcribe_audio(audio_data) Command
   ↓
3. Rust 后端音频预处理
   ↓
4. 加载 Whisper 模型（如果未加载）
   ↓
5. 调用 whisper.cpp 推理
   ↓
6. 通过 Tauri Event 发送进度更新
   ↓
7. 获得转录文本
   ↓
8. 后处理（标点、大小写）
   ↓
9. 返回结果到前端
   ↓
10. 保存到数据库
   ↓
11. 触发文本插入
```

## Decisions

### 决策 1: Whisper 实现方式 - whisper.cpp vs faster-whisper

**决策**: 使用 `whisper.cpp`

**原因**:
- **性能**: C++ 实现比 Python 快 4-10 倍
- **依赖**: 不需要 Python 环境和 ONNX 运行时
- **体积**: 编译后体积小，易集成
- **跨平台**: 支持 macOS/Windows/Linux
- **GPU**: 支持 Core ML (macOS) 和 CUDA

**权衡**:
- faster-whisper 基于 CTranslate2，也很快，但需要 Python
- whisper-rs 可用，但不如直接用 whisper.cpp FFI

### 决策 2: 模型格式 - GGML vs ONNX

**决策**: 使用 **GGML 格式**

**原因**:
- **官方**: whisper.cpp 原生支持
- **量化**: 支持 INT8/INT4 量化，减小模型体积
- **兼容**: 转换工具成熟

**模型文件**:
```
models/
├── ggml-tiny.bin     (75 MB)   - 快速，准确度中等
├── ggml-base.bin     (142 MB)  - 平衡速度和准确度（推荐）
├── ggml-small.bin    (466 MB)  - 高准确度
└── ggml-medium.bin   (1.5 GB)  - 最高准确度
```

### 决策 3: 模型下载方式 - 内置 vs 在线下载

**决策**: **在线下载** + 可选内置 tiny

**原因**:
- **灵活**: 用户按需下载
- **体积**: 不增加安装包大小
- **更新**: 模型更新不需要发布新版本

**实现**:
- 首次启动提示下载 base 模型
- 提供内置 tiny 模型作为后备
- 从 Hugging Face Mirror 下载（国内加速）

### 决策 4: 推理策略 - 批量 vs 流式

**决策**: **批量推理**

**原因**:
- **简单**: 实现复杂度低
- **准确**: Whisper 模型设计为批量处理
- **适用**: 录音时长通常 < 5 分钟

**权衡**:
- 流式推理延迟更低，但 Whisper 不是为流式设计
- 可在二期考虑 whisper-streaming

### 决策 5: GPU 加速 - Core ML vs Metal

**决策**: **Core ML** (macOS)

**原因**:
- **官方**: whisper.cpp 支持 Core ML
- **兼容**: 所有 M1/M2 Mac 支持
- **简单**: 编译时启用即可

**性能提升**:
- CPU: base 模型 ~10 秒/分钟音频
- Core ML: base 模型 ~2-3 秒/分钟音频

## Implementation Details

### Whisper.cpp 集成

#### 1. 编译 whisper.cpp 静态库

```bash
# 下载 whisper.cpp
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp

# 编译为静态库（启用 Core ML）
make WHISPER_COREML=1

# 生成 .a 静态库
ar rcs libwhisper.a *.o
```

#### 2. Rust FFI 绑定

```rust
// src-tauri/src/whisper/ffi.rs

#[repr(C)]
pub struct WhisperContext {
    _private: [u8; 0],
}

#[link(name = "whisper", kind = "static")]
extern "C" {
    pub fn whisper_init_from_file(path: *const c_char) -> *mut WhisperContext;
    pub fn whisper_free(ctx: *mut WhisperContext);
    pub fn whisper_full(
        ctx: *mut WhisperContext,
        params: WhisperFullParams,
        samples: *const f32,
        n_samples: c_int,
    ) -> c_int;
    pub fn whisper_full_get_segment_text(
        ctx: *mut WhisperContext,
        i_segment: c_int,
    ) -> *const c_char;
    pub fn whisper_full_n_segments(ctx: *mut WhisperContext) -> c_int;
}

#[repr(C)]
pub struct WhisperFullParams {
    pub n_threads: c_int,
    pub language: *const c_char,
    pub translate: bool,
    // ... 其他参数
}
```

#### 3. Whisper Engine 封装

```rust
// src-tauri/src/whisper/engine.rs

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use crate::whisper::ffi::*;

pub struct WhisperEngine {
    ctx: *mut WhisperContext,
    model_path: String,
}

impl WhisperEngine {
    pub fn new(model_path: &str) -> Result<Self, String> {
        let path = CString::new(model_path).unwrap();
        let ctx = unsafe { whisper_init_from_file(path.as_ptr()) };

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
        audio_data: &[i16],
        language: Option<&str>,
    ) -> Result<String, String> {
        // 转换 i16 到 f32 (-1.0 到 1.0)
        let samples: Vec<f32> = audio_data
            .iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();

        let lang = language.unwrap_or("zh");
        let lang_cstr = CString::new(lang).unwrap();

        let params = WhisperFullParams {
            n_threads: 4,
            language: lang_cstr.as_ptr(),
            translate: false,
            // 其他默认参数...
        };

        let result = unsafe {
            whisper_full(
                self.ctx,
                params,
                samples.as_ptr(),
                samples.len() as c_int,
            )
        };

        if result != 0 {
            return Err("Transcription failed".to_string());
        }

        // 获取转录结果
        let n_segments = unsafe { whisper_full_n_segments(self.ctx) };
        let mut text = String::new();

        for i in 0..n_segments {
            let segment_text = unsafe {
                let ptr = whisper_full_get_segment_text(self.ctx, i);
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            };
            text.push_str(&segment_text);
        }

        Ok(text.trim().to_string())
    }
}

impl Drop for WhisperEngine {
    fn drop(&mut self) {
        unsafe { whisper_free(self.ctx) };
    }
}
```

#### 4. Tauri Command

```rust
// src-tauri/src/commands/transcription.rs

use tauri::State;
use std::sync::Mutex;
use crate::whisper::engine::WhisperEngine;

#[tauri::command]
pub async fn transcribe_audio(
    audio_data: Vec<i16>,
    language: Option<String>,
    engine: State<'_, Mutex<Option<WhisperEngine>>>,
    window: tauri::Window,
) -> Result<String, String> {
    let mut engine = engine.lock().unwrap();

    // 如果引擎未初始化，加载默认模型
    if engine.is_none() {
        let model_path = get_model_path("base")?;
        *engine = Some(WhisperEngine::new(&model_path)?);
    }

    let whisper = engine.as_mut().unwrap();

    // 发送进度事件
    let _ = window.emit("transcription-progress", 0.1);

    // 转录
    let result = whisper.transcribe(
        &audio_data,
        language.as_deref(),
    )?;

    let _ = window.emit("transcription-progress", 1.0);

    // 保存到数据库
    save_transcription(&result)?;

    Ok(result)
}

fn get_model_path(model_name: &str) -> Result<String, String> {
    let app_data = std::env::var("HOME")
        .unwrap_or_else(|_| "/tmp".to_string());

    let model_path = format!(
        "{}/Library/Application Support/com.lingcode.app/models/ggml-{}.bin",
        app_data, model_name
    );

    if !std::path::Path::new(&model_path).exists() {
        return Err(format!("Model not found: {}", model_name));
    }

    Ok(model_path)
}
```

### 模型管理

```rust
// src-tauri/src/whisper/model_manager.rs

use reqwest;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use sha2::{Sha256, Digest};

pub struct ModelManager {
    models_dir: String,
}

impl ModelManager {
    pub fn new() -> Self {
        let models_dir = format!(
            "{}/Library/Application Support/com.lingcode.app/models",
            std::env::var("HOME").unwrap()
        );
        std::fs::create_dir_all(&models_dir).unwrap();

        Self { models_dir }
    }

    pub async fn download_model(
        &self,
        model_name: &str,
        on_progress: impl Fn(f64),
    ) -> Result<(), String> {
        let url = get_model_url(model_name);
        let file_path = format!("{}/ggml-{}.bin", self.models_dir, model_name);

        let response = reqwest::get(&url)
            .await
            .map_err(|e| e.to_string())?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut file = File::create(&file_path).map_err(|e| e.to_string())?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;

            downloaded += chunk.len() as u64;
            let progress = downloaded as f64 / total_size as f64;
            on_progress(progress);
        }

        // 验证 SHA256
        self.verify_model(model_name)?;

        Ok(())
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "tiny".to_string(),
                size: 75_000_000,
                downloaded: self.is_downloaded("tiny"),
            },
            ModelInfo {
                name: "base".to_string(),
                size: 142_000_000,
                downloaded: self.is_downloaded("base"),
            },
            ModelInfo {
                name: "small".to_string(),
                size: 466_000_000,
                downloaded: self.is_downloaded("small"),
            },
            ModelInfo {
                name: "medium".to_string(),
                size: 1_500_000_000,
                downloaded: self.is_downloaded("medium"),
            },
        ]
    }

    fn is_downloaded(&self, model_name: &str) -> bool {
        let path = format!("{}/ggml-{}.bin", self.models_dir, model_name);
        Path::new(&path).exists()
    }

    fn verify_model(&self, model_name: &str) -> Result<(), String> {
        let path = format!("{}/ggml-{}.bin", self.models_dir, model_name);
        let mut file = File::open(&path).map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;

        let hash = format!("{:x}", hasher.finalize());
        let expected = get_model_sha256(model_name);

        if hash != expected {
            return Err("Model file corrupted".to_string());
        }

        Ok(())
    }
}

pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub downloaded: bool,
}

fn get_model_url(model_name: &str) -> String {
    // 使用 Hugging Face Mirror (国内加速)
    format!(
        "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_name
    )
}

fn get_model_sha256(model_name: &str) -> String {
    // 模型文件的 SHA256 校验和
    match model_name {
        "tiny" => "abc123...".to_string(),
        "base" => "def456...".to_string(),
        // ... 其他模型
        _ => String::new(),
    }
}
```

### 前端集成

```tsx
// src/stores/transcriptionStore.ts

import { create } from 'zustand'

interface TranscriptionStore {
  status: 'idle' | 'transcribing' | 'completed' | 'error'
  result: string | null
  progress: number
  error: string | null

  setStatus: (status: TranscriptionStore['status']) => void
  setResult: (result: string) => void
  setProgress: (progress: number) => void
  setError: (error: string) => void
  reset: () => void
}

export const useTranscriptionStore = create<TranscriptionStore>((set) => ({
  status: 'idle',
  result: null,
  progress: 0,
  error: null,

  setStatus: (status) => set({ status }),
  setResult: (result) => set({ result, status: 'completed' }),
  setProgress: (progress) => set({ progress }),
  setError: (error) => set({ error, status: 'error' }),
  reset: () => set({
    status: 'idle',
    result: null,
    progress: 0,
    error: null,
  }),
}))
```

```tsx
// src/windows/recording/RecordingWindow.tsx (扩展)

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useTranscriptionStore } from '@/stores/transcriptionStore'

export function RecordingWindow() {
  const { setStatus, setProgress, setResult, setError } = useTranscriptionStore()

  useEffect(() => {
    const unlisten = listen<number>('transcription-progress', (event) => {
      setProgress(event.payload)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const handleStop = async () => {
    try {
      setStatus('transcribing')

      const audioData = await invoke<number[]>('stop_recording')
      const text = await invoke<string>('transcribe_audio', {
        audioData,
        language: 'zh',
      })

      setResult(text)

      // 触发文本插入
      await invoke('insert_text', { text })
    } catch (error) {
      setError(String(error))
    }
  }

  return (
    // ... UI
  )
}
```

## Performance Optimization

### 1. 模型量化

```bash
# 使用 whisper.cpp 工具量化模型
./quantize ggml-base.bin ggml-base-q8_0.bin q8_0

# INT8 量化后体积减少 50%，速度提升 20-30%
```

### 2. 多线程推理

```rust
let params = WhisperFullParams {
    n_threads: num_cpus::get() as c_int, // 使用所有 CPU 核心
    // ...
};
```

### 3. 推理缓存

```rust
// 缓存常用短语的转录结果
use lru::LruCache;

struct TranscriptionCache {
    cache: LruCache<Vec<u8>, String>,
}

// 使用音频数据哈希作为 key
```

## Testing Strategy

### 准确度测试数据集

```
tests/audio/
├── zh-CN/           # 中文测试集
│   ├── short.wav    # 短句 (< 10秒)
│   ├── medium.wav   # 中句 (10-30秒)
│   └── long.wav     # 长句 (30-60秒)
├── en-US/           # 英文测试集
└── mixed/           # 中英混合
```

### 性能基准

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_transcribe(c: &mut Criterion) {
        let mut engine = WhisperEngine::new("models/ggml-base.bin").unwrap();
        let audio = load_test_audio("tests/audio/zh-CN/medium.wav");

        c.bench_function("transcribe_30s", |b| {
            b.iter(|| {
                engine.transcribe(black_box(&audio), Some("zh"))
            })
        });
    }

    criterion_group!(benches, bench_transcribe);
    criterion_main!(benches);
}
```

## Security & Privacy

- ✅ 所有转录完全本地进行
- ✅ 模型文件 SHA256 验证
- ✅ 音频数据仅内存缓存
- ✅ 用户可删除转录历史
- ✅ 不上传任何音频或文本数据
