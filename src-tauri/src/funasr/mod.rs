/// FunASR 引擎模块
/// 通过 Python subprocess 调用 FunASR 进行语音识别

use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

pub mod engine;
pub mod prewarmer;
pub mod server;

pub use engine::FunASREngine;
pub use prewarmer::{prewarm_funasr, prewarm_funasr_cmd, quick_health_check, PythonEnvStatus};
pub use server::FunASRServer;

/// FunASR 转录结果
#[derive(Debug, serde::Deserialize)]
pub struct TranscriptionResult {
    pub success: bool,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub error: String,
}

/// 获取 Python 脚本路径
fn get_script_path(_app: &AppHandle) -> Result<PathBuf, String> {
    // 开发模式：使用 src-tauri/scripts 目录
    #[cfg(debug_assertions)]
    {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current dir: {}", e))?;

        // 尝试两种路径：当前目录/scripts 或 当前目录/src-tauri/scripts
        let script_path = if current_dir.ends_with("src-tauri") {
            current_dir.join("scripts").join("funasr_transcribe.py")
        } else {
            current_dir.join("src-tauri").join("scripts").join("funasr_transcribe.py")
        };

        if !script_path.exists() {
            return Err(format!("Script not found: {:?}", script_path));
        }

        return Ok(script_path);
    }

    // 生产模式：脚本应该打包在资源目录中（暂未实现）
    #[cfg(not(debug_assertions))]
    {
        // TODO: 生产模式需要打包 Python 脚本
        Err("Production mode not yet implemented".to_string())
    }
}

/// 调用 Python 脚本执行转录
pub async fn transcribe_with_python(
    app: &AppHandle,
    python_path: &PathBuf,
    audio_path: &str,
    model_name: &str,
    language: Option<&str>,
) -> Result<String, String> {
    use tracing::info;

    let script_path = get_script_path(app)?;

    info!("🐍 Calling FunASR Python script: {:?}", script_path);
    info!("   Model: {}, Audio: {}", model_name, audio_path);

    let mut cmd = Command::new(python_path);
    cmd.arg(&script_path)
        .arg("transcribe")
        .arg("--audio")
        .arg(audio_path)
        .arg("--model")
        .arg(model_name);

    if let Some(lang) = language {
        cmd.arg("--language").arg(lang);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute Python script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("🐍 Python output: {}", stdout);

    // 提取 JSON 结果（最后一行以 { 开头的内容）
    let json_line = stdout
        .lines()
        .filter(|line| line.trim().starts_with('{'))
        .last()
        .ok_or("No JSON result found in Python output")?;

    info!("🐍 Extracted JSON line: {}", json_line);

    // 解析 JSON 结果
    let result: TranscriptionResult = serde_json::from_str(json_line)
        .map_err(|e| format!("Failed to parse result: {}", e))?;

    if !result.success {
        return Err(result.error);
    }

    Ok(result.text)
}

/// 下载 FunASR 模型（带实时进度反馈）
pub async fn download_funasr_model(
    app: &AppHandle,
    python_path: &PathBuf,
    model_name: &str,
) -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use tauri::Emitter;
    use tracing::{info, warn};

    let script_path = get_script_path(app)?;

    info!("📥 Downloading FunASR model: {}", model_name);

    // 启动进程，捕获 stderr 和 stdout
    let mut child = Command::new(python_path)
        .arg(&script_path)
        .arg("download")
        .arg("--model")
        .arg(model_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute download script: {}", e))?;

    // 获取 stderr 用于进度监听
    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture stderr")?;

    let app_handle = app.clone();

    // 异步读取 stderr 并解析进度
    let progress_handle = tauri::async_runtime::spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                // 打印所有 stderr 输出到日志
                info!("[Python] {}", line_str);

                // 解析进度标记: PROGRESS:percentage:name:message
                if line_str.starts_with("PROGRESS:") {
                    let parts: Vec<&str> = line_str.splitn(4, ':').collect();
                    if parts.len() >= 4 {
                        if let Ok(progress) = parts[1].parse::<u32>() {
                            let name = parts[2];
                            let message = parts[3];

                            // 发送进度事件到前端
                            #[derive(serde::Serialize, Clone)]
                            struct ModelDownloadProgress {
                                progress: u32,
                                component: String,
                                message: String,
                            }

                            let _ = app_handle.emit(
                                "model-download-progress",
                                ModelDownloadProgress {
                                    progress,
                                    component: name.to_string(),
                                    message: message.to_string(),
                                },
                            );

                            info!("📊 下载进度: {}% - {} ({})", progress, name, message);
                        }
                    }
                }
            }
        }
    });

    // 等待进程结束
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    // 等待进度监听线程结束
    let _ = progress_handle.await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Model download failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 提取 JSON 结果（最后一行以 { 开头的内容）
    let json_line = stdout
        .lines()
        .filter(|line| line.trim().starts_with('{'))
        .last()
        .ok_or("No JSON result found in Python output")?;

    info!("🐍 Extracted JSON line: {}", json_line);

    // 解析结果
    #[derive(serde::Deserialize)]
    struct DownloadResult {
        success: bool,
        #[serde(default)]
        model_dir: String,
        #[serde(default)]
        error: String,
    }

    let result: DownloadResult = serde_json::from_str(json_line)
        .map_err(|e| format!("Failed to parse result: {}", e))?;

    if !result.success {
        return Err(result.error);
    }

    info!("✅ Model downloaded to: {}", result.model_dir);
    Ok(result.model_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_result_parsing() {
        let json = r#"{"success": true, "text": "测试文本"}"#;
        let result: TranscriptionResult = serde_json::from_str(json).unwrap();
        assert!(result.success);
        assert_eq!(result.text, "测试文本");
    }
}
