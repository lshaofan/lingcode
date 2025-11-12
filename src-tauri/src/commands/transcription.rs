/// 转录命令模块
/// 提供音频转录相关的 Tauri commands

use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::whisper::{convert_i16_to_f32, WhisperEngine};

/// Whisper 引擎状态
pub struct WhisperState {
    engine: Arc<Mutex<Option<WhisperEngine>>>,
    current_model: Arc<Mutex<Option<String>>>,
}

impl WhisperState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            current_model: Arc::new(Mutex::new(None)),
        }
    }
}

/// 初始化 Whisper 引擎
#[tauri::command]
pub async fn initialize_whisper(
    app: AppHandle,
    model_name: String,
    state: State<'_, WhisperState>,
) -> Result<(), String> {
    use tracing::info;

    info!("🎯 [Whisper] Initializing Whisper engine with model: {}", model_name);

    // 获取模型路径
    let models_dir = get_models_dir(&app)?;
    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

    info!("🎯 [Whisper] Looking for model at: {:?}", model_path);

    if !model_path.exists() {
        info!("🎯 [Whisper] Model not found, checking models directory: {:?}", models_dir);

        // 列出 models 目录中的所有文件
        if models_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&models_dir) {
                info!("🎯 [Whisper] Files in models directory:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        info!("  - {:?}", entry.file_name());
                    }
                }
            }
        } else {
            info!("🎯 [Whisper] Models directory does not exist!");
        }

        return Err(format!("Model '{}' not found at {}. Please download it first.", model_name, model_path.display()));
    }

    // 创建 Whisper 引擎
    let engine = WhisperEngine::new(&model_path)
        .map_err(|e| format!("Failed to initialize Whisper engine: {}", e))?;

    // 保存到状态
    *state.engine.lock() = Some(engine);
    *state.current_model.lock() = Some(model_name);

    Ok(())
}

/// 转录音频
#[tauri::command]
pub async fn transcribe_audio(
    audio_data: Vec<i16>,
    language: Option<String>,
    state: State<'_, WhisperState>,
) -> Result<String, String> {
    use tracing::info;

    info!("🎯 [Transcription] transcribe_audio called, language: {:?}", language);

    // 如果是中文相关的语言代码，统一使用 "zh"
    let normalized_language = language.map(|lang| {
        if lang.starts_with("zh") || lang == "chinese" || lang == "Chinese" {
            info!("🎯 [Transcription] Normalizing language '{}' to 'zh'", lang);
            "zh".to_string()
        } else {
            lang
        }
    });

    // 检查引擎是否已初始化
    let engine = state.engine.lock();
    let engine = engine.as_ref().ok_or_else(|| {
        "Whisper engine not initialized. Please call initialize_whisper first.".to_string()
    })?;

    // 转换音频格式 (i16 -> f32)
    let audio_f32 = convert_i16_to_f32(&audio_data);

    // 执行转录
    let text = engine
        .transcribe(&audio_f32, normalized_language.as_deref())
        .map_err(|e| format!("Transcription failed: {}", e))?;

    Ok(text)
}

/// 转录最后一次录音
/// 从全局 LAST_RECORDING 中获取录音数据并转录
#[tauri::command]
pub async fn transcribe_last_recording(
    language: Option<String>,
    state: State<'_, WhisperState>,
) -> Result<String, String> {
    use tracing::info;

    info!("🎯 [Transcription] transcribe_last_recording called, language: {:?}", language);

    // 如果是中文相关的语言代码，统一使用 "zh"
    let normalized_language = language.map(|lang| {
        if lang.starts_with("zh") || lang == "chinese" || lang == "Chinese" {
            info!("🎯 [Transcription] Normalizing language '{}' to 'zh'", lang);
            "zh".to_string()
        } else {
            lang
        }
    });

    info!("🎯 [Transcription] Using normalized language: {:?}", normalized_language);

    // 获取最后一次录音 - 使用全局静态变量
    use super::audio::{LAST_RECORDING};
    let last_recording = LAST_RECORDING.lock();
    let audio_data = last_recording
        .as_ref()
        .ok_or("No recording available. Please record audio first.".to_string())?;

    info!("🎯 [Transcription] Audio data available: {} samples at 48kHz", audio_data.len());

    // 检查引擎是否已初始化
    let engine = state.engine.lock();
    let engine = engine.as_ref().ok_or_else(|| {
        info!("🎯 [Transcription] Whisper engine not initialized!");
        "Whisper engine not initialized. Please download a model first.".to_string()
    })?;

    // 转换音频格式 (i16 -> f32)
    let mut audio_f32 = convert_i16_to_f32(audio_data);

    // 重采样从 48kHz 到 16kHz（Whisper 需要 16kHz）
    use crate::whisper::resample_48khz_to_16khz;
    audio_f32 = resample_48khz_to_16khz(&audio_f32);
    info!("🎯 [Transcription] Resampled to 16kHz: {} samples", audio_f32.len());

    // 执行转录
    let text = engine
        .transcribe(&audio_f32, normalized_language.as_deref())
        .map_err(|e| format!("Transcription failed: {}", e))?;

    // 🔑 验证转录结果是否有效
    // 检测 Whisper 的"幻觉"输出（静音时经常输出的无意义内容）
    if is_invalid_transcription(&text, &audio_f32) {
        info!("🎯 [Transcription] Invalid transcription detected (hallucination or silence): '{}'", text);
        return Err("转录结果无效：可能是静音或噪音".to_string());
    }

    info!("🎯 [Transcription] Valid transcription: '{}'", text);
    Ok(text)
}

/// 转录音频（带时间戳）
#[tauri::command]
pub async fn transcribe_audio_with_timestamps(
    audio_data: Vec<i16>,
    language: Option<String>,
    state: State<'_, WhisperState>,
) -> Result<Vec<TranscriptionSegmentDTO>, String> {
    use tracing::info;

    info!("🎯 [Transcription] transcribe_audio_with_timestamps called, language: {:?}", language);

    // 如果是中文相关的语言代码，统一使用 "zh"
    let normalized_language = language.map(|lang| {
        if lang.starts_with("zh") || lang == "chinese" || lang == "Chinese" {
            info!("🎯 [Transcription] Normalizing language '{}' to 'zh'", lang);
            "zh".to_string()
        } else {
            lang
        }
    });

    // 检查引擎是否已初始化
    let engine = state.engine.lock();
    let engine = engine.as_ref().ok_or_else(|| {
        "Whisper engine not initialized. Please call initialize_whisper first.".to_string()
    })?;

    // 转换音频格式 (i16 -> f32)
    let audio_f32 = convert_i16_to_f32(&audio_data);

    // 执行转录
    let segments = engine
        .transcribe_with_timestamps(&audio_f32, normalized_language.as_deref())
        .map_err(|e| format!("Transcription failed: {}", e))?;

    // 转换为 DTO
    let segments_dto: Vec<TranscriptionSegmentDTO> = segments
        .into_iter()
        .map(|s| TranscriptionSegmentDTO {
            text: s.text,
            start_ms: s.start_ms,
            end_ms: s.end_ms,
        })
        .collect();

    Ok(segments_dto)
}

/// 获取当前使用的模型名称
#[tauri::command]
pub fn get_current_model(state: State<'_, WhisperState>) -> Result<Option<String>, String> {
    let model = state.current_model.lock();
    Ok(model.clone())
}

/// 转录段落 DTO
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TranscriptionSegmentDTO {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

// 辅助函数

fn get_models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    Ok(app_data_dir.join("models"))
}

/// 检测转录结果是否无效（Whisper 幻觉或静音）
///
/// 常见的无效情况：
/// 1. 音频太短（少于 0.3 秒）
/// 2. 包含 Whisper 常见的幻觉词汇（"字幕"、"J Chong" 等）
/// 3. 文本太短或只有标点符号
fn is_invalid_transcription(text: &str, audio_f32: &[f32]) -> bool {
    // 1. 检查音频长度（16kHz 采样率）
    let duration_seconds = audio_f32.len() as f32 / 16000.0;
    if duration_seconds < 0.3 {
        return true;
    }

    // 2. 去除空白字符
    let trimmed = text.trim();

    // 3. 检查是否为空或太短
    if trimmed.is_empty() || trimmed.len() < 2 {
        return true;
    }

    // 4. 检查是否只包含标点符号和括号
    let has_meaningful_char = trimmed.chars().any(|c| {
        !c.is_whitespace() && !".,!?;:()[]{}\"'，。！？；：（）【】「」『』".contains(c)
    });

    if !has_meaningful_char {
        return true;
    }

    // 5. 检查常见的 Whisper 幻觉模式（不区分大小写）
    let lower_text = text.to_lowercase();
    let hallucination_patterns = [
        "字幕",
        "subtitle",
        "j chong",
        "jchong",
        "thanks for watching",
        "请不吝点赞",
        "订阅",
        "subscribe",
        "翻译",
        "translation",
        "(music)",
        "[music]",
        "(laughs)",
        "[laughs]",
        "(applause)",
        "[applause]",
    ];

    for pattern in &hallucination_patterns {
        if lower_text.contains(pattern) {
            return true;
        }
    }

    // 6. 检查是否只是单个括号内容（如 "(字幕:xxx)"）
    if trimmed.starts_with('(') && trimmed.ends_with(')') && !trimmed[1..trimmed.len()-1].contains('(') {
        return true;
    }

    // 所有检查通过，认为是有效的转录
    false
}
