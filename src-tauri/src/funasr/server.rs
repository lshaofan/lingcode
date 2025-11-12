/// FunASR 常驻服务器管理
/// 保持 Python 进程运行，模型只加载一次，大幅提升性能

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct Request {
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC 响应
#[derive(Debug, Deserialize)]
struct Response {
    success: bool,
    #[serde(default)]
    text: String,
    #[serde(default)]
    error: String,
    #[serde(default)]
    message: String,
}

/// FunASR 服务器实例
pub struct FunASRServer {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    python_path: PathBuf,
    script_path: PathBuf,
}

impl FunASRServer {
    /// 创建新的服务器实例
    pub fn new(app: &AppHandle, python_path: PathBuf) -> Result<Self, String> {
        let script_path = get_server_script_path(app)?;

        Ok(Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            python_path,
            script_path,
        })
    }

    /// 检查服务器是否存活
    pub async fn is_alive(&self) -> bool {
        let mut process_guard = self.process.lock().await;

        if let Some(child) = process_guard.as_mut() {
            match child.try_wait() {
                Ok(None) => true,  // 进程还在运行
                Ok(Some(status)) => {
                    warn!("⚠️  FunASR server exited with status: {:?}", status);
                    false
                }
                Err(e) => {
                    warn!("⚠️  Failed to check FunASR server status: {}", e);
                    false
                }
            }
        } else {
            false  // 进程不存在
        }
    }

    /// 启动服务器（带超时和重试）
    pub async fn start(&self) -> Result<(), String> {
        // 先释放 process_guard 的锁，避免死锁
        drop(self.process.lock().await);

        // 使用 is_alive 检查（内部会加锁）
        if self.is_alive().await {
            info!("🔄 FunASR server already running");
            return Ok(());
        }

        info!("🚀 Starting FunASR server...");
        info!("   Python: {:?}", self.python_path);
        info!("   Script: {:?}", self.script_path);

        // 启动 Python 进程
        let mut child = Command::new(&self.python_path)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to start FunASR server: {}", e))?;

        // 获取 stdin 和 stdout
        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to get stdin")?;

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to get stdout")?;

        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        *self.process.lock().await = Some(child);

        info!("✅ FunASR server process started");

        // 发送 ping 测试连接（带超时 30 秒）
        match timeout(
            Duration::from_secs(30),
            self.send_request("ping", serde_json::json!({}))
        ).await {
            Ok(Ok(_)) => {
                info!("✅ FunASR server responding to ping");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("❌ FunASR server ping failed: {}", e);
                self.force_stop().await;
                Err(format!("Server initialization failed: {}", e))
            }
            Err(_) => {
                error!("❌ FunASR server ping timeout (30s)");
                self.force_stop().await;
                Err("Server initialization timeout. Model may be too large or Python environment is slow.".to_string())
            }
        }
    }

    /// 强制停止服务器（清理资源）
    async fn force_stop(&self) {
        warn!("🛑 Force stopping FunASR server...");

        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        *self.stdin.lock().await = None;
        *self.stdout.lock().await = None;
    }

    /// 停止服务器
    pub async fn stop(&self) -> Result<(), String> {
        info!("🛑 Stopping FunASR server...");

        // 发送 shutdown 命令
        let _ = self.send_request("shutdown", serde_json::json!({})).await;

        // 等待进程退出
        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            let _ = child.wait();
        }

        *self.stdin.lock().await = None;
        *self.stdout.lock().await = None;

        info!("👋 FunASR server stopped");
        Ok(())
    }

    /// 发送请求并等待响应（带超时）
    async fn send_request(&self, method: &str, params: serde_json::Value) -> Result<Response, String> {
        let request = Request {
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // 发送请求
        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard
                .as_mut()
                .ok_or("Server not running")?;

            writeln!(stdin, "{}", request_json)
                .map_err(|e| format!("Failed to write request: {}", e))?;

            stdin
                .flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }

        // 读取响应（带超时：ping 用 30s，transcribe 用 60s）
        let timeout_duration = if method == "ping" {
            Duration::from_secs(30)
        } else {
            Duration::from_secs(60)
        };

        let response_line = {
            let stdout_arc = self.stdout.clone();

            match timeout(timeout_duration, tokio::task::spawn_blocking(move || {
                let mut stdout_guard = stdout_arc.blocking_lock();
                let stdout = stdout_guard
                    .as_mut()
                    .ok_or("Server not running")?;

                let mut line = String::new();
                stdout
                    .read_line(&mut line)
                    .map_err(|e| format!("Failed to read response: {}", e))?;

                Ok::<String, String>(line)
            })).await {
                Ok(Ok(Ok(line))) => line,
                Ok(Ok(Err(e))) => return Err(e),
                Ok(Err(e)) => return Err(format!("Task error: {}", e)),
                Err(_) => {
                    error!("❌ Request timeout after {:?} for method: {}", timeout_duration, method);
                    return Err(format!("Request timeout after {:?}", timeout_duration));
                }
            }
        };

        // 解析响应
        info!("📨 Received response line: {}", response_line.trim());
        let response: Response = serde_json::from_str(&response_line)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        info!("📊 Parsed response - success: {}, text length: {}",
            response.success, response.text.len());

        Ok(response)
    }

    /// 转录音频（带自动重试）
    pub async fn transcribe(
        &self,
        audio_path: &str,
        model_name: &str,
        language: Option<&str>,
    ) -> Result<String, String> {
        const MAX_RETRIES: u32 = 2;

        for attempt in 1..=MAX_RETRIES {
            // 健康检查：如果服务器挂了，尝试重启
            if !self.is_alive().await {
                warn!("⚠️  Server not alive, attempting to restart (attempt {}/{})", attempt, MAX_RETRIES);

                // 清理旧进程
                self.force_stop().await;

                // 尝试重新启动
                match self.start().await {
                    Ok(_) => info!("✅ Server restarted successfully"),
                    Err(e) => {
                        if attempt == MAX_RETRIES {
                            return Err(format!("Failed to restart server after {} attempts: {}", MAX_RETRIES, e));
                        }
                        warn!("⚠️  Restart failed, will retry: {}", e);
                        continue;
                    }
                }
            } else {
                // 服务器正常，确保启动
                self.start().await?;
            }

            info!("🎤 Transcribing audio (attempt {}/{}): {}", attempt, MAX_RETRIES, audio_path);
            info!("   Model: {}", model_name);
            info!("   Language: {:?}", language);

            let mut params = serde_json::json!({
                "audio_path": audio_path,
                "model_name": model_name,
            });

            if let Some(lang) = language {
                params["language"] = serde_json::json!(lang);
            }

            match self.send_request("transcribe", params).await {
                Ok(response) => {
                    if !response.success {
                        return Err(response.error);
                    }

                    info!("✅ Transcription complete");
                    return Ok(response.text);
                }
                Err(e) => {
                    error!("❌ Transcription failed (attempt {}/{}): {}", attempt, MAX_RETRIES, e);

                    if attempt == MAX_RETRIES {
                        return Err(format!("Transcription failed after {} attempts: {}", MAX_RETRIES, e));
                    }

                    // 强制停止服务器，下次循环会重启
                    self.force_stop().await;

                    // 等待一小段时间再重试
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        Err("Transcription failed: max retries exceeded".to_string())
    }
}

// 注意：Drop trait 不能是异步的，所以我们不在这里清理
// 服务器进程会在程序退出时自动终止

/// 获取服务器脚本路径
fn get_server_script_path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current dir: {}", e))?;

        let script_path = if current_dir.ends_with("src-tauri") {
            current_dir.join("scripts").join("funasr_server.py")
        } else {
            current_dir
                .join("src-tauri")
                .join("scripts")
                .join("funasr_server.py")
        };

        if !script_path.exists() {
            return Err(format!("Server script not found: {:?}", script_path));
        }

        return Ok(script_path);
    }

    #[cfg(not(debug_assertions))]
    {
        // TODO: 生产模式
        Err("Production mode not yet implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request {
            method: "transcribe".to_string(),
            params: serde_json::json!({
                "audio_path": "/tmp/test.wav",
                "model_name": "paraformer-zh",
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("transcribe"));
        assert!(json.contains("/tmp/test.wav"));
    }
}
