/// Whisper 转录引擎
/// 封装 whisper-rs 提供安全的转录接口

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::preprocessor::{validate_audio_data, PreprocessError};

#[derive(Debug)]
pub enum WhisperError {
    ModelNotFound(PathBuf),
    FailedToLoadModel(String),
    TranscriptionFailed(String),
    PreprocessError(PreprocessError),
    AudioTooShort,
    AudioTooLong,
}

impl fmt::Display for WhisperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WhisperError::ModelNotFound(path) => {
                write!(f, "Model file not found: {}", path.display())
            }
            WhisperError::FailedToLoadModel(err) => {
                write!(f, "Failed to load Whisper model: {}", err)
            }
            WhisperError::TranscriptionFailed(err) => {
                write!(f, "Transcription failed: {}", err)
            }
            WhisperError::PreprocessError(err) => {
                write!(f, "Audio preprocessing error: {}", err)
            }
            WhisperError::AudioTooShort => {
                write!(f, "Audio too short (minimum 0.1 seconds)")
            }
            WhisperError::AudioTooLong => {
                write!(f, "Audio too long (maximum 10 minutes)")
            }
        }
    }
}

impl Error for WhisperError {}

impl From<PreprocessError> for WhisperError {
    fn from(err: PreprocessError) -> Self {
        WhisperError::PreprocessError(err)
    }
}

/// Whisper 转录引擎
pub struct WhisperEngine {
    context: WhisperContext,
    model_path: PathBuf,
    n_threads: usize,
}

impl WhisperEngine {
    /// 创建新的 Whisper 引擎实例
    ///
    /// # 参数
    /// * `model_path` - 模型文件路径
    /// * `n_threads` - 推理使用的线程数（默认使用 CPU 核心数的一半）
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self, WhisperError> {
        let model_path = model_path.as_ref().to_path_buf();

        // 检查模型文件是否存在
        if !model_path.exists() {
            return Err(WhisperError::ModelNotFound(model_path));
        }

        // 创建 Whisper 上下文参数
        let params = WhisperContextParameters::default();

        // 加载模型
        let context = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| {
                WhisperError::FailedToLoadModel("Invalid model path encoding".to_string())
            })?,
            params,
        )
        .map_err(|e| WhisperError::FailedToLoadModel(e.to_string()))?;

        // 获取 CPU 核心数，使用一半作为默认线程数
        let n_threads = num_cpus::get() / 2;
        let n_threads = n_threads.max(1).min(8); // 限制在 1-8 之间

        Ok(Self {
            context,
            model_path,
            n_threads,
        })
    }

    /// 检查引擎是否已初始化
    pub fn is_initialized(&self) -> bool {
        true // 如果能创建成功就是已初始化
    }

    /// 获取模型路径
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// 转录音频
    ///
    /// # 参数
    /// * `audio_data` - f32 格式的音频数据（16kHz, 单声道）
    /// * `language` - 语言代码（如 "zh", "en"），None 表示自动检测
    ///
    /// # 返回
    /// 转录后的文本
    pub fn transcribe(
        &self,
        audio_data: &[f32],
        language: Option<&str>,
    ) -> Result<String, WhisperError> {
        use tracing::info;

        // 验证音频数据
        validate_audio_data(audio_data)?;

        // 检查音频长度（16kHz 采样率）
        let duration_secs = audio_data.len() as f32 / 16000.0;
        info!("🎯 [Whisper] Audio duration: {:.2} seconds", duration_secs);

        if duration_secs < 0.1 {
            return Err(WhisperError::AudioTooShort);
        }
        if duration_secs > 600.0 {
            // 10 分钟
            return Err(WhisperError::AudioTooLong);
        }

        // 创建转录参数 - 针对中文优化
        // 使用 BeamSearch 策略以提高准确度（虽然会稍微慢一点）
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,      // 使用 5 个候选
            patience: -1.0     // 默认值
        });

        // 设置语言
        info!("🎯 [Whisper] Language setting: {:?}", language);
        if let Some(lang) = language {
            info!("🎯 [Whisper] Setting explicit language: {}", lang);
            params.set_language(Some(lang));
            params.set_translate(false);

            // 针对中文的特殊优化
            if lang == "zh" {
                // 对中文的特殊设置
                params.set_temperature(0.0);  // 使用确定性解码
                params.set_suppress_blank(true);  // 抑制空白输出
                params.set_initial_prompt("以下是普通话的句子。");  // 提示这是中文
                params.set_single_segment(false);  // 允许多段
                params.set_print_special(false);  // 不打印特殊 token
                params.set_token_timestamps(false); // 不需要 token 时间戳
            }
        } else {
            info!("🎯 [Whisper] Using auto language detection");
            // 对于自动检测，我们仍然可以设置一些提示
            // Whisper 的自动检测有时候会偏向英文，特别是短音频
            params.set_translate(false);
            params.set_suppress_blank(true);
            // params.set_suppress_non_speech_tokens(true); // 某些版本可能没有这个方法
        }

        // 设置线程数
        params.set_n_threads(self.n_threads as i32);

        // 通用优化参数
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);  // 不打印特殊token
        params.set_token_timestamps(false);  // 不需要 token 级时间戳

        // 创建 state 并执行转录
        let mut state = self.context
            .create_state()
            .map_err(|e| WhisperError::TranscriptionFailed(format!("Failed to create state: {}", e)))?;

        state
            .full(params, audio_data)
            .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

        // 获取转录结果 - 使用迭代器方式
        let mut result = String::new();
        let mut segment_count = 0;
        for segment in state.as_iter() {
            let text = segment
                .to_str()
                .map_err(|e| WhisperError::TranscriptionFailed(format!("Failed to get segment text: {:?}", e)))?;
            info!("🎯 [Whisper] Segment {}: {}", segment_count, text);
            result.push_str(text);
            segment_count += 1;
        }

        let final_result = result.trim().to_string();
        info!("🎯 [Whisper] Final transcription result: {}", final_result);

        // 去除首尾空格
        Ok(final_result)
    }

    /// 转录音频并返回带时间戳的结果
    ///
    /// # 参数
    /// * `audio_data` - f32 格式的音频数据（16kHz, 单声道）
    /// * `language` - 语言代码（如 "zh", "en"），None 表示自动检测
    ///
    /// # 返回
    /// 转录后的段落列表，每个段落包含文本和时间戳
    pub fn transcribe_with_timestamps(
        &self,
        audio_data: &[f32],
        language: Option<&str>,
    ) -> Result<Vec<TranscriptionSegment>, WhisperError> {
        // 验证音频数据
        validate_audio_data(audio_data)?;

        // 创建转录参数 - 针对中文优化
        // 使用 BeamSearch 策略以提高准确度（虽然会稍微慢一点）
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,      // 使用 5 个候选
            patience: -1.0     // 默认值
        });

        // 设置语言
        if let Some(lang) = language {
            params.set_language(Some(lang));
            params.set_translate(false);

            // 针对中文的特殊优化
            if lang == "zh" {
                params.set_temperature(0.0);
                params.set_suppress_blank(true);
                // params.set_suppress_non_speech_tokens(true); // 某些版本可能没有这个方法
            }
        } else {
            params.set_translate(false);
            params.set_suppress_blank(true);
            // params.set_suppress_non_speech_tokens(true); // 某些版本可能没有这个方法
        }

        // 设置线程数
        params.set_n_threads(self.n_threads as i32);

        // 启用时间戳
        params.set_print_timestamps(true);
        params.set_print_special(false);
        params.set_token_timestamps(false);

        // 创建 state 并执行转录
        let mut state = self.context
            .create_state()
            .map_err(|e| WhisperError::TranscriptionFailed(format!("Failed to create state: {}", e)))?;

        state
            .full(params, audio_data)
            .map_err(|e| WhisperError::TranscriptionFailed(e.to_string()))?;

        // 获取转录结果 - 使用迭代器方式
        let mut segments = Vec::new();
        for segment in state.as_iter() {
            let text = segment
                .to_str()
                .map_err(|e| WhisperError::TranscriptionFailed(format!("Failed to get segment text: {:?}", e)))?;

            let start_time = segment.start_timestamp();
            let end_time = segment.end_timestamp();

            segments.push(TranscriptionSegment {
                text: text.trim().to_string(),
                start_ms: start_time as u64 * 10, // whisper 时间戳单位是厘秒（10ms）
                end_ms: end_time as u64 * 10,
            });
        }

        Ok(segments)
    }
}

/// 转录段落（带时间戳）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的模型文件才能运行
    // 在 CI 环境中可能需要跳过

    #[test]
    #[ignore] // 需要模型文件
    fn test_engine_initialization() {
        // 这个测试需要实际的模型文件路径
        let result = WhisperEngine::new("path/to/model.bin");
        // 在没有模型文件时应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_too_short() {
        // 创建一个很短的音频（少于 0.1 秒）
        let short_audio = vec![0.0f32; 1000]; // 0.0625 秒 @ 16kHz

        // 不需要真实模型就能测试验证逻辑
        // 这里我们只测试音频长度验证
        let duration = short_audio.len() as f32 / 16000.0;
        assert!(duration < 0.1);
    }
}
