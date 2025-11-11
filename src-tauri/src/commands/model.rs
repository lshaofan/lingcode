use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};

/// 模型引擎类型
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelEngine {
    Whisper,
    FunASR,
}

/// 模型信息
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub engine: ModelEngine,
    pub size: String,
    pub size_bytes: u64,
    pub speed: String,
    pub accuracy: String,
    pub is_recommended: bool,
    pub is_downloaded: bool,
    pub download_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 获取所有可用的模型（Whisper + FunASR）
#[tauri::command]
pub fn get_available_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let models_dir = get_models_dir(&app)?;

    let mut models = vec![];

    // Whisper 模型
    models.extend(vec![
        ModelInfo {
            name: "base".to_string(),
            engine: ModelEngine::Whisper,
            size: "74MB".to_string(),
            size_bytes: 74 * 1024 * 1024,
            speed: "快速".to_string(),
            accuracy: "一般精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "base"),
            download_url: "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
            description: Some("Whisper 基础模型，支持多语言".to_string()),
        },
        ModelInfo {
            name: "small".to_string(),
            engine: ModelEngine::Whisper,
            size: "244MB".to_string(),
            size_bytes: 244 * 1024 * 1024,
            speed: "较快".to_string(),
            accuracy: "较高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "small"),
            download_url: "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
            description: Some("Whisper 小型模型，平衡速度和精度".to_string()),
        },
        ModelInfo {
            name: "medium".to_string(),
            engine: ModelEngine::Whisper,
            size: "769MB".to_string(),
            size_bytes: 769 * 1024 * 1024,
            speed: "较慢".to_string(),
            accuracy: "高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "medium"),
            download_url: "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
            description: Some("Whisper 中型模型，高精度".to_string()),
        },
        ModelInfo {
            name: "large".to_string(),
            engine: ModelEngine::Whisper,
            size: "1.5GB".to_string(),
            size_bytes: 1536 * 1024 * 1024,
            speed: "慢".to_string(),
            accuracy: "最高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "large"),
            download_url: "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-large.bin".to_string(),
            description: Some("Whisper 大型模型，最高精度".to_string()),
        },
    ]);

    // FunASR 模型
    models.extend(vec![
        ModelInfo {
            name: "paraformer-zh".to_string(),
            engine: ModelEngine::FunASR,
            size: "~220MB".to_string(),
            size_bytes: 220 * 1024 * 1024,
            speed: "快速".to_string(),
            accuracy: "高精度（中文）".to_string(),
            is_recommended: true,
            is_downloaded: check_funasr_model_downloaded(&app, "paraformer-zh"),
            download_url: "modelscope://damo/speech_paraformer-large-vad-punc_asr_nat-zh-cn-16k-common-vocab8404-pytorch".to_string(),
            description: Some("阿里 FunASR 中文识别模型，专为中文优化".to_string()),
        },
        ModelInfo {
            name: "paraformer-large".to_string(),
            engine: ModelEngine::FunASR,
            size: "~380MB".to_string(),
            size_bytes: 380 * 1024 * 1024,
            speed: "较快".to_string(),
            accuracy: "极高精度（中文）".to_string(),
            is_recommended: false,
            is_downloaded: check_funasr_model_downloaded(&app, "paraformer-large"),
            download_url: "modelscope://iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch".to_string(),
            description: Some("FunASR 大型中文模型，更高精度".to_string()),
        },
        ModelInfo {
            name: "sensevoice-small".to_string(),
            engine: ModelEngine::FunASR,
            size: "~160MB".to_string(),
            size_bytes: 160 * 1024 * 1024,
            speed: "快速".to_string(),
            accuracy: "高精度（多语言+情感）".to_string(),
            is_recommended: false,
            is_downloaded: check_funasr_model_downloaded(&app, "sensevoice-small"),
            download_url: "modelscope://iic/SenseVoiceSmall".to_string(),
            description: Some("支持多语言和情感识别".to_string()),
        },
    ]);

    Ok(models)
}

/// 下载模型
#[tauri::command]
pub async fn download_model(app: AppHandle, model_name: String) -> Result<(), String> {
    use tracing::info;

    info!("📥 [Model] Downloading model: {}", model_name);

    // 先获取模型列表，确定模型类型
    let models = get_available_models(app.clone())?;
    let model_info = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("Invalid model name: {}", model_name))?;

    info!("📥 [Model] Model engine: {:?}", model_info.engine);

    // 根据模型引擎类型调用不同的下载逻辑
    match model_info.engine {
        ModelEngine::FunASR => {
            // 调用 FunASR 下载函数
            info!("📥 [Model] Calling FunASR download function");

            // 确保 Python 环境可用
            let python_env = crate::python::ensure_python_env(&app).await?;

            // 下载 FunASR 模型
            crate::funasr::download_funasr_model(&app, &python_env.python_path, &model_name).await?;

            Ok(())
        }
        ModelEngine::Whisper => {
            // Whisper 模型下载逻辑（保持原有逻辑）
            info!("📥 [Model] Downloading Whisper model");
            let models_dir = get_models_dir(&app)?;
            std::fs::create_dir_all(&models_dir)
                .map_err(|e| format!("Failed to create models directory: {}", e))?;

            let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

            // 如果模型已存在,先删除
            if model_path.exists() {
                std::fs::remove_file(&model_path)
                    .map_err(|e| format!("Failed to remove existing model: {}", e))?;
            }

            // 获取下载 URL（使用中国镜像站）
            let download_url = match model_name.as_str() {
                "base" => "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
                "small" => "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
                "medium" => "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
                "large" => "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-large.bin",
                _ => return Err("Invalid Whisper model name".to_string()),
            };

            download_whisper_model(&app, &model_name, download_url, &model_path).await
        }
    }
}

/// Whisper 模型下载逻辑（从原 download_model 函数中提取）
async fn download_whisper_model(
    app: &AppHandle,
    model_name: &str,
    download_url: &str,
    model_path: &PathBuf,
) -> Result<(), String> {
    // 使用 reqwest 下载文件
    let client = reqwest::Client::new();
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // 使用 tokio 的文件写入
    let mut file = tokio::fs::File::create(&model_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Failed to read chunk: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        // 发送进度事件
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = app.emit(
            "model-download-progress",
            DownloadProgress {
                model_name: model_name.to_string(),
                progress,
                downloaded,
                total: total_size,
            },
        );
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    Ok(())
}

/// 删除模型
#[tauri::command]
pub fn delete_model(app: AppHandle, model_name: String) -> Result<(), String> {
    let models_dir = get_models_dir(&app)?;
    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

    if !model_path.exists() {
        return Err("Model not found".to_string());
    }

    std::fs::remove_file(&model_path).map_err(|e| format!("Failed to delete model: {}", e))?;

    Ok(())
}

/// 获取已下载的模型列表
#[tauri::command]
pub fn get_downloaded_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let models = get_available_models(app)?;
    Ok(models.into_iter().filter(|m| m.is_downloaded).collect())
}

/// 获取模型目录路径（调试用）
#[tauri::command]
pub fn get_models_directory(app: AppHandle) -> Result<String, String> {
    let models_dir = get_models_dir(&app)?;
    Ok(models_dir.to_string_lossy().to_string())
}

// 辅助函数

fn get_models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    Ok(app_data_dir.join("models"))
}

fn check_model_downloaded(models_dir: &PathBuf, model_name: &str) -> bool {
    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));
    model_path.exists()
}

/// 检查 FunASR 模型是否已下载
fn check_funasr_model_downloaded(app: &AppHandle, model_name: &str) -> bool {
    use std::process::Command;
    use tracing::info;

    // 尝试获取 Python 环境（使用同步检测）
    let python_env = match crate::python::detect_python(app) {
        Ok(env) => env,
        Err(e) => {
            info!("Failed to detect Python env for checking FunASR model: {}", e);
            return false;
        }
    };

    // 调用 Python 脚本检查模型是否存在
    let script_path = match std::env::current_dir() {
        Ok(dir) => {
            // 开发模式：使用 src-tauri/scripts 目录
            #[cfg(debug_assertions)]
            {
                // 尝试两种路径：当前目录/scripts 或 当前目录/src-tauri/scripts
                let script_path = if dir.ends_with("src-tauri") {
                    dir.join("scripts").join("funasr_transcribe.py")
                } else {
                    dir.join("src-tauri").join("scripts").join("funasr_transcribe.py")
                };

                if !script_path.exists() {
                    info!("Script not found: {:?}", script_path);
                    return false;
                }
                script_path
            }
            #[cfg(not(debug_assertions))]
            {
                // 生产模式：暂未实现
                info!("Production mode not yet implemented");
                return false;
            }
        }
        Err(e) => {
            info!("Failed to get current dir: {}", e);
            return false;
        }
    };

    info!("🔍 Checking FunASR model '{}' with Python: {:?}", model_name, python_env.python_path);

    let output = Command::new(&python_env.python_path)
        .arg(&script_path)
        .arg("check")
        .arg("--model")
        .arg(model_name)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                info!("🐍 Python script output: {}", stdout);

                // 提取 JSON 结果（最后一行以 { 开头的内容）
                let json_line = stdout
                    .lines()
                    .filter(|line| line.trim().starts_with('{'))
                    .last();

                match json_line {
                    Some(line) => {
                        info!("🐍 Extracted JSON line: {}", line);
                        match serde_json::from_str::<serde_json::Value>(line) {
                            Ok(json) => {
                                let exists = json.get("exists").and_then(|v| v.as_bool()).unwrap_or(false);
                                info!("✅ FunASR model '{}' check result: exists = {}", model_name, exists);
                                exists
                            }
                            Err(e) => {
                                info!("❌ Failed to parse check result for '{}': {}", model_name, e);
                                false
                            }
                        }
                    }
                    None => {
                        info!("❌ No JSON result found in Python output for '{}'", model_name);
                        false
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!("❌ Failed to check FunASR model '{}': {}", model_name, stderr);
                false
            }
        }
        Err(e) => {
            info!("❌ Command failed when checking FunASR model '{}': {}", model_name, e);
            false
        }
    }
}

#[derive(serde::Serialize, Clone)]
struct DownloadProgress {
    model_name: String,
    progress: u32,
    downloaded: u64,
    total: u64,
}

/// 一键安装FunASR完整环境（Python + 依赖 + 模型）
/// 这是用户首次使用FunASR时的便捷命令
#[tauri::command]
pub async fn setup_funasr_environment(
    app: AppHandle,
    model_name: Option<String>,
    funasr_state: State<'_, crate::commands::funasr::FunASRState>,
) -> Result<(), String> {
    use crate::python::{ensure_python_env, install_funasr_with_progress};
    use tracing::info;

    let model = model_name.unwrap_or_else(|| "paraformer-zh".to_string());

    info!("🚀 Starting FunASR environment setup with model: {}", model);

    // 发送开始事件
    let _ = app.emit(
        "funasr-setup-status",
        SetupStatus {
            step: "开始".to_string(),
            progress: 0,
            message: "开始安装FunASR环境...".to_string(),
            is_error: false,
        },
    );

    // 步骤1: 确保Python环境（会自动安装依赖并显示进度）
    info!("📦 Step 1/2: Setting up Python environment and dependencies...");
    let _ = app.emit(
        "funasr-setup-status",
        SetupStatus {
            step: "Python环境".to_string(),
            progress: 10,
            message: "正在设置Python环境和依赖...".to_string(),
            is_error: false,
        },
    );

    match ensure_python_env(&app).await {
        Ok(python_env) => {
            info!("✅ Python environment ready: {}", python_env.version);

            let _ = app.emit(
                "funasr-setup-status",
                SetupStatus {
                    step: "Python环境".to_string(),
                    progress: 50,
                    message: "Python环境已就绪".to_string(),
                    is_error: false,
                },
            );
        }
        Err(e) => {
            let error_msg = format!("Python环境设置失败: {}", e);
            info!("❌ {}", error_msg);

            let _ = app.emit(
                "funasr-setup-status",
                SetupStatus {
                    step: "Python环境".to_string(),
                    progress: 0,
                    message: error_msg.clone(),
                    is_error: true,
                },
            );

            return Err(error_msg);
        }
    }

    // 步骤2: 下载FunASR模型
    info!("📥 Step 2/2: Downloading FunASR model '{}'...", model);
    let _ = app.emit(
        "funasr-setup-status",
        SetupStatus {
            step: "下载模型".to_string(),
            progress: 60,
            message: format!("正在下载 {} 模型...", model),
            is_error: false,
        },
    );

    // 使用现有的download_funasr_model命令
    match crate::commands::funasr::download_funasr_model(app.clone(), model.clone(), funasr_state).await {
        Ok(_) => {
            info!("✅ Model '{}' downloaded successfully", model);

            let _ = app.emit(
                "funasr-setup-status",
                SetupStatus {
                    step: "完成".to_string(),
                    progress: 100,
                    message: "FunASR环境安装完成！".to_string(),
                    is_error: false,
                },
            );

            Ok(())
        }
        Err(e) => {
            let error_msg = format!("模型下载失败: {}", e);
            info!("❌ {}", error_msg);

            let _ = app.emit(
                "funasr-setup-status",
                SetupStatus {
                    step: "下载模型".to_string(),
                    progress: 60,
                    message: error_msg.clone(),
                    is_error: true,
                },
            );

            Err(error_msg)
        }
    }
}

/// 环境设置状态事件
#[derive(serde::Serialize, Clone)]
struct SetupStatus {
    step: String,
    progress: u32,
    message: String,
    is_error: bool,
}