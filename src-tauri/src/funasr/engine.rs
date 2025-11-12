/// FunASR 引擎
/// 管理 FunASR 模型和转录状态

use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;

use crate::python::PythonEnv;

/// FunASR 引擎状态
pub struct FunASREngine {
    python_env: Arc<Mutex<Option<PythonEnv>>>,
    current_model: Arc<Mutex<Option<String>>>,
    app_handle: AppHandle,
}

impl FunASREngine {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            python_env: Arc::new(Mutex::new(None)),
            current_model: Arc::new(Mutex::new(None)),
            app_handle,
        }
    }

    /// 初始化 Python 环境
    pub async fn initialize(&self) -> Result<(), String> {
        use crate::python;
        use tracing::info;

        info!("🚀 Initializing FunASR engine...");

        // 确保 Python 环境可用
        let python_env = python::ensure_python_env(&self.app_handle).await?;

        *self.python_env.lock() = Some(python_env);

        info!("✅ FunASR engine initialized");
        Ok(())
    }

    /// 加载模型
    pub async fn load_model(&self, model_name: String) -> Result<(), String> {
        use tracing::info;

        info!("📦 Loading FunASR model: {}", model_name);

        // 检查 Python 环境
        let python_env = self.python_env.lock();
        if python_env.is_none() {
            return Err("Python environment not initialized".to_string());
        }

        // 更新当前模型
        *self.current_model.lock() = Some(model_name.clone());

        info!("✅ Model loaded: {}", model_name);
        Ok(())
    }

    /// 转录音频
    pub async fn transcribe(
        &self,
        audio_path: &str,
        language: Option<String>,
    ) -> Result<String, String> {
        use tracing::info;

        info!("🎤 Transcribing audio with FunASR...");

        // 检查 Python 环境
        let python_env = self.python_env.lock();
        let python_env = python_env
            .as_ref()
            .ok_or("Python environment not initialized")?;

        // 检查模型
        let current_model = self.current_model.lock();
        let model_name = current_model
            .as_ref()
            .ok_or("No model loaded")?;

        // 调用 Python 脚本
        let text = super::transcribe_with_python(
            &self.app_handle,
            &python_env.python_path,
            audio_path,
            model_name,
            language.as_deref(),
        )
        .await?;

        info!("✅ Transcription complete: {}", text);
        Ok(text)
    }

    /// 下载模型
    pub async fn download_model(&self, model_name: String) -> Result<String, String> {
        use tracing::info;

        info!("📥 Downloading model: {}", model_name);

        // 检查 Python 环境
        let python_env = self.python_env.lock();
        let python_env = python_env
            .as_ref()
            .ok_or("Python environment not initialized")?;

        // 调用 Python 脚本下载模型
        let model_dir = super::download_funasr_model(
            &self.app_handle,
            &python_env.python_path,
            &model_name,
        )
        .await?;

        info!("✅ Model downloaded: {}", model_dir);
        Ok(model_dir)
    }

    /// 获取当前模型
    pub fn get_current_model(&self) -> Option<String> {
        self.current_model.lock().clone()
    }
}
