/// FunASR 命令模块
/// 提供 FunASR 相关的 Tauri commands

use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::funasr::FunASRServer;

// Re-export prewarm_funasr_cmd from funasr module
pub use crate::funasr::prewarm_funasr_cmd;

/// FunASR 引擎状态
pub struct FunASRState {
    current_model: Arc<Mutex<Option<String>>>,
    server: Arc<Mutex<Option<FunASRServer>>>,
    python_env_checked: Arc<Mutex<bool>>,  // 标记是否已检查过Python环境
}

impl FunASRState {
    pub fn new() -> Self {
        Self {
            current_model: Arc::new(Mutex::new(None)),
            server: Arc::new(Mutex::new(None)),
            python_env_checked: Arc::new(Mutex::new(false)),
        }
    }

    /// 获取或创建服务器实例
    pub async fn get_or_create_server(&self, app: &AppHandle) -> Result<(), String> {
        let mut server_guard = self.server.lock().await;

        if server_guard.is_none() {
            // 只在首次创建时检查Python环境（包括依赖检查）
            // 后续调用不再重复检查，因为服务器一旦启动就说明环境正常
            let python_env = {
                let mut env_checked = self.python_env_checked.lock().await;
                if !*env_checked {
                    use tracing::info;
                    info!("🔍 [FunASR] First-time Python environment check...");
                    let env = crate::python::ensure_python_env(app).await?;
                    *env_checked = true;
                    env
                } else {
                    use tracing::info;
                    info!("⚡ [FunASR] Skipping Python env check (already verified)");
                    // 快速获取Python路径，不做依赖检查
                    crate::python::detect_python(app)?
                }
            };

            // 创建服务器实例
            let server = FunASRServer::new(app, python_env.python_path)?;

            *server_guard = Some(server);
        }

        Ok(())
    }
}

/// 初始化 FunASR 引擎
#[tauri::command]
pub async fn initialize_funasr(
    app: AppHandle,
    model_name: String,
    state: State<'_, FunASRState>,
) -> Result<(), String> {
    use tracing::info;

    info!("🎯 [FunASR] Initializing FunASR engine with model: {}", model_name);

    // 保存当前模型
    *state.current_model.lock().await = Some(model_name.clone());

    // 预先确保服务器创建（内部会在首次创建时检查Python环境）
    state.get_or_create_server(&app).await?;

    info!("✅ [FunASR] Engine initialized with model: {}", model_name);
    Ok(())
}

/// 使用 FunASR 转录最后一次录音
#[tauri::command]
pub async fn transcribe_last_recording_funasr(
    app: AppHandle,
    language: Option<String>,
    state: State<'_, FunASRState>,
) -> Result<String, String> {
    use super::audio::LAST_RECORDING;
    use tracing::info;

    info!("🎯 [FunASR] transcribe_last_recording_funasr called, language: {:?}", language);

    // 获取最后一次录音
    let audio_data = {
        let last_recording = LAST_RECORDING.lock();
        last_recording
            .as_ref()
            .ok_or("No recording available. Please record audio first.".to_string())?
            .clone()
    };

    // 获取实际采样率
    let actual_sample_rate = {
        let recorder = super::audio::AUDIO_RECORDER.lock();
        recorder.actual_sample_rate()
    };

    info!("🎯 [FunASR] Audio data available: {} samples at {}Hz", audio_data.len(), actual_sample_rate);

    // 检查音频长度（至少 0.5 秒）
    let min_samples = (actual_sample_rate as f32 * 0.5) as usize;
    if audio_data.len() < min_samples {
        return Err(format!("录音太短：{:.2}秒。请录制更长的音频（至少0.5秒）。",
            audio_data.len() as f32 / actual_sample_rate as f32));
    }

    // 获取当前模型
    let model_name = {
        let current_model = state.current_model.lock().await;
        current_model
            .as_ref()
            .ok_or("FunASR engine not initialized. Please download a model first.".to_string())?
            .clone()
    };

    // 确保服务器已启动（内部会在首次创建时检查Python环境，之后不再重复检查）
    state.get_or_create_server(&app).await?;

    // 保存音频到临时文件
    let temp_dir = std::env::temp_dir();
    let temp_audio_path = temp_dir.join(format!("funasr_temp_{}.wav", chrono::Utc::now().timestamp()));

    // 转换采样率到 16kHz（FunASR 需要）
    let resampled_audio = if actual_sample_rate != 16000 {
        let resampled = resample_audio(&audio_data, actual_sample_rate, 16000);
        info!("🎯 [FunASR] Resampled audio: {} samples ({}Hz) -> {} samples (16kHz)",
            audio_data.len(), actual_sample_rate, resampled.len());
        resampled
    } else {
        info!("🎯 [FunASR] Audio already at 16kHz, no resampling needed");
        audio_data.clone()
    };

    // 将 i16 PCM 数据保存为 WAV 文件（16kHz）
    save_audio_to_wav_16k(&resampled_audio, &temp_audio_path)?;

    info!("🎯 [FunASR] Saved audio to temporary file: {:?}", temp_audio_path);

    // 使用服务器进行转录
    let audio_path_str = temp_audio_path
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();

    let text = {
        let server_guard = state.server.lock().await;
        let server = server_guard
            .as_ref()
            .ok_or("FunASR server not initialized")?;

        info!("🎯 [FunASR] Calling server.transcribe...");
        let result = server.transcribe(
            &audio_path_str,
            &model_name,
            language.as_deref(),
        ).await;

        info!("🎯 [FunASR] Server.transcribe returned: {:?}", result);
        result?
    };

    info!("🎯 [FunASR] Text received, length: {}, content: '{}'", text.len(), text);

    // 删除临时文件
    let _ = std::fs::remove_file(&temp_audio_path);

    info!("✅ [FunASR] Transcription complete: '{}'", text);

    // 🚀 首次成功转录后，标记不再是首次启动，并触发后台预热（如果尚未预热）
    // 这样下次应用启动时就能享受到预热的好处
    {
        use crate::config::ConfigManager;
        use crate::db::Database;
        use std::sync::Arc;
        use tauri::Manager;

        if let Some(db) = app.try_state::<Arc<Database>>() {
            let config_manager = ConfigManager::new(db.connection());

            // 如果仍是首次启动，标记为已使用
            if let Ok(true) = config_manager.is_first_launch() {
                info!("🎉 First successful transcription, marking as no longer first launch");
                let _ = config_manager.mark_first_launch_complete();

                // 如果启用了预热，不做额外处理（服务器已经在第一次转录时启动了）
                // 下次应用启动时会自动预热
                if let Ok(true) = config_manager.is_prewarming_enabled() {
                    info!("💡 Prewarming enabled, server will be prewarmed on next app launch");
                }
            }
        }
    }

    Ok(text)
}

/// 下载 FunASR 模型
#[tauri::command]
pub async fn download_funasr_model(
    app: AppHandle,
    model_name: String,
    _state: State<'_, FunASRState>,
) -> Result<String, String> {
    use tracing::info;

    info!("📥 [FunASR] Downloading model: {}", model_name);

    // 确保 Python 环境可用
    let python_env = crate::python::ensure_python_env(&app).await?;

    // 下载模型
    let model_dir = crate::funasr::download_funasr_model(
        &app,
        &python_env.python_path,
        &model_name,
    )
    .await?;

    info!("✅ [FunASR] Model downloaded to: {}", model_dir);
    Ok(model_dir)
}

/// 获取当前 FunASR 模型
#[tauri::command]
pub async fn get_current_funasr_model(state: State<'_, FunASRState>) -> Result<Option<String>, String> {
    let current_model = state.current_model.lock().await;
    Ok(current_model.clone())
}

/// 使用 FunASR 转录音频数据（接收前端传来的音频）
#[tauri::command]
pub async fn transcribe_audio_funasr(
    app: AppHandle,
    audio_data: Vec<i16>,
    language: Option<String>,
    state: State<'_, FunASRState>,
) -> Result<String, String> {
    use tracing::info;

    info!("🎯 [FunASR] transcribe_audio_funasr called with {} samples, language: {:?}",
        audio_data.len(), language);

    // 前端传来的音频已经是 16kHz 单声道 PCM16 格式
    let actual_sample_rate = 16000;

    info!("🎯 [FunASR] Audio data: {} samples at {}Hz", audio_data.len(), actual_sample_rate);

    // 检查音频长度（至少 0.5 秒）
    let min_samples = (actual_sample_rate as f32 * 0.5) as usize;
    if audio_data.len() < min_samples {
        return Err(format!("录音太短：{:.2}秒。请录制更长的音频（至少0.5秒）。",
            audio_data.len() as f32 / actual_sample_rate as f32));
    }

    // 获取当前模型
    let model_name = {
        let current_model = state.current_model.lock().await;
        current_model
            .as_ref()
            .ok_or("FunASR engine not initialized. Please download a model first.".to_string())?
            .clone()
    };

    // 确保服务器已启动
    state.get_or_create_server(&app).await?;

    // 保存音频到临时文件
    let temp_dir = std::env::temp_dir();
    let temp_audio_path = temp_dir.join(format!("funasr_temp_{}.wav", chrono::Utc::now().timestamp()));

    // 音频已经是 16kHz，直接保存
    save_audio_to_wav_16k(&audio_data, &temp_audio_path)?;

    info!("🎯 [FunASR] Saved audio to temporary file: {:?}", temp_audio_path);

    // 使用服务器进行转录
    let audio_path_str = temp_audio_path
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();

    let text = {
        let server_guard = state.server.lock().await;
        let server = server_guard
            .as_ref()
            .ok_or("FunASR server not initialized")?;

        info!("🎯 [FunASR] Calling server.transcribe...");
        let result = server.transcribe(
            &audio_path_str,
            &model_name,
            language.as_deref(),
        ).await;

        info!("🎯 [FunASR] Server.transcribe returned: {:?}", result);
        result?
    };

    info!("🎯 [FunASR] Text received, length: {}, content: '{}'", text.len(), text);

    // 删除临时文件
    let _ = std::fs::remove_file(&temp_audio_path);

    info!("✅ [FunASR] Transcription complete: '{}'", text);

    // 🚀 首次成功转录后，标记不再是首次启动
    {
        use crate::config::ConfigManager;
        use crate::db::Database;
        use std::sync::Arc;
        use tauri::Manager;

        if let Some(db) = app.try_state::<Arc<Database>>() {
            let config_manager = ConfigManager::new(db.connection());

            if let Ok(true) = config_manager.is_first_launch() {
                info!("🎉 First successful transcription, marking as no longer first launch");
                let _ = config_manager.mark_first_launch_complete();

                if let Ok(true) = config_manager.is_prewarming_enabled() {
                    info!("💡 Prewarming enabled, server will be prewarmed on next app launch");
                }
            }
        }
    }

    Ok(text)
}

// 辅助函数

/// 简单的线性插值重采样（48kHz -> 16kHz）
fn resample_audio(audio_data: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate {
        return audio_data.to_vec();
    }

    let ratio = from_rate as f32 / to_rate as f32;
    let new_len = (audio_data.len() as f32 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = (i as f32 * ratio) as usize;
        if src_idx < audio_data.len() {
            resampled.push(audio_data[src_idx]);
        }
    }

    resampled
}

/// 将 i16 PCM 数据保存为 WAV 文件（16kHz）
fn save_audio_to_wav_16k(audio_data: &[i16], path: &std::path::Path) -> Result<(), String> {
    use hound::{WavSpec, WavWriter};

    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,  // FunASR 需要 16kHz
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;

    for &sample in audio_data {
        writer
            .write_sample(sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    Ok(())
}
