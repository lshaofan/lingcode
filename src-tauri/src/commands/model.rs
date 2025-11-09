use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// Whisper 模型信息
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: String,
    pub size_bytes: u64,
    pub speed: String,
    pub accuracy: String,
    pub is_recommended: bool,
    pub is_downloaded: bool,
    pub download_url: String,
}

/// 获取所有可用的 Whisper 模型
#[tauri::command]
pub fn get_available_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let models_dir = get_models_dir(&app)?;

    let models = vec![
        ModelInfo {
            name: "base".to_string(),
            size: "74MB".to_string(),
            size_bytes: 74 * 1024 * 1024,
            speed: "快速".to_string(),
            accuracy: "一般精度".to_string(),
            is_recommended: true,
            is_downloaded: check_model_downloaded(&models_dir, "base"),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
        },
        ModelInfo {
            name: "small".to_string(),
            size: "244MB".to_string(),
            size_bytes: 244 * 1024 * 1024,
            speed: "较快".to_string(),
            accuracy: "较高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "small"),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
        },
        ModelInfo {
            name: "medium".to_string(),
            size: "769MB".to_string(),
            size_bytes: 769 * 1024 * 1024,
            speed: "较慢".to_string(),
            accuracy: "高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "medium"),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
        },
        ModelInfo {
            name: "large".to_string(),
            size: "1.5GB".to_string(),
            size_bytes: 1536 * 1024 * 1024,
            speed: "慢".to_string(),
            accuracy: "最高精度".to_string(),
            is_recommended: false,
            is_downloaded: check_model_downloaded(&models_dir, "large"),
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large.bin".to_string(),
        },
    ];

    Ok(models)
}

/// 下载模型
#[tauri::command]
pub async fn download_model(app: AppHandle, model_name: String) -> Result<(), String> {
    let models_dir = get_models_dir(&app)?;
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

    // 如果模型已存在,先删除
    if model_path.exists() {
        std::fs::remove_file(&model_path)
            .map_err(|e| format!("Failed to remove existing model: {}", e))?;
    }

    // 获取下载 URL
    let download_url = match model_name.as_str() {
        "base" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        "small" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        "medium" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        "large" => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large.bin",
        _ => return Err("Invalid model name".to_string()),
    };

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
                model_name: model_name.clone(),
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

#[derive(serde::Serialize, Clone)]
struct DownloadProgress {
    model_name: String,
    progress: u32,
    downloaded: u64,
    total: u64,
}