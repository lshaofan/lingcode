/// FunASR 预热模块
/// 负责在应用启动时后台预热FunASR服务器，减少首次识别延迟

use crate::commands::funasr::FunASRState;
use crate::python::quick_check_python_health;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// 预热状态事件
#[derive(Debug, Clone, Serialize)]
pub struct PythonEnvStatus {
    pub status: String,        // "checking" | "ready" | "missing" | "error"
    pub message: String,
    pub details: Option<String>,
}

/// Tauri 命令：预热 FunASR 模型
#[tauri::command]
pub async fn prewarm_funasr_cmd(app: AppHandle) -> Result<(), String> {
    use tracing::{info, warn};

    info!("🔥 Starting FunASR prewarming...");

    // 快速检查Python环境
    match quick_check_python_health(&app) {
        Ok(true) => {
            info!("✅ Python environment healthy");

            // 尝试启动FunASR服务器（预加载模型）
            if let Some(funasr_state) = app.try_state::<FunASRState>() {
                match funasr_state.get_or_create_server(&app).await {
                    Ok(_) => {
                        info!("✅ FunASR server prewarmed successfully");
                        Ok(())
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to prewarm FunASR: {}", e);
                        Err(format!("Failed to prewarm FunASR: {}", e))
                    }
                }
            } else {
                Err("FunASR state not initialized".to_string())
            }
        }
        Ok(false) => {
            warn!("⚠️  Python environment not healthy");
            Err("Python environment not healthy".to_string())
        }
        Err(e) => {
            warn!("⚠️  Failed to check Python environment: {}", e);
            Err(format!("Failed to check Python environment: {}", e))
        }
    }
}

/// 后台预热FunASR服务器
/// 这个函数会在应用启动后异步执行，不阻塞主线程
pub fn prewarm_funasr(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        use tracing::{info, warn};

        info!("🔥 Starting FunASR prewarming...");

        // 发送检查中状态
        let _ = app.emit(
            "python-env-status",
            PythonEnvStatus {
                status: "checking".to_string(),
                message: "正在检查Python环境...".to_string(),
                details: None,
            },
        );

        // 1. 快速检查Python环境
        match quick_check_python_health(&app) {
            Ok(true) => {
                info!("✅ Python environment healthy");

                // 发送环境就绪状态
                let _ = app.emit(
                    "python-env-status",
                    PythonEnvStatus {
                        status: "ready".to_string(),
                        message: "Python环境已就绪".to_string(),
                        details: Some("依赖已安装".to_string()),
                    },
                );

                // 2. 尝试启动FunASR服务器（预加载模型）
                info!("🚀 Attempting to start FunASR server for prewarming...");

                if let Some(funasr_state) = app.try_state::<FunASRState>() {
                    match funasr_state.get_or_create_server(&app).await {
                        Ok(_) => {
                            info!("✅ FunASR server prewarmed successfully");

                            // 发送预热完成状态
                            let _ = app.emit(
                                "python-env-status",
                                PythonEnvStatus {
                                    status: "prewarmed".to_string(),
                                    message: "FunASR已预热，识别速度更快".to_string(),
                                    details: Some("模型已加载到内存".to_string()),
                                },
                            );
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to start FunASR server for prewarming: {}", e);
                            // 预热失败不算错误，首次使用时会自动初始化
                        }
                    }
                } else {
                    warn!("⚠️ FunASR state not found");
                }
            }
            Ok(false) => {
                info!("ℹ️ Python environment not ready (dependencies missing)");

                // 发送环境缺失状态
                let _ = app.emit(
                    "python-env-status",
                    PythonEnvStatus {
                        status: "missing".to_string(),
                        message: "Python环境未安装".to_string(),
                        details: Some("使用FunASR前需要先安装依赖".to_string()),
                    },
                );
            }
            Err(e) => {
                warn!("⚠️ Failed to check Python environment: {}", e);

                // 发送错误状态
                let _ = app.emit(
                    "python-env-status",
                    PythonEnvStatus {
                        status: "error".to_string(),
                        message: "环境检查失败".to_string(),
                        details: Some(e),
                    },
                );
            }
        }
    });
}

/// 快速健康检查（不启动服务器）
/// 用于启动时快速显示环境状态
pub fn quick_health_check(app: &AppHandle) -> PythonEnvStatus {
    match quick_check_python_health(app) {
        Ok(true) => PythonEnvStatus {
            status: "ready".to_string(),
            message: "Python环境已就绪".to_string(),
            details: None,
        },
        Ok(false) => PythonEnvStatus {
            status: "missing".to_string(),
            message: "Python环境未安装".to_string(),
            details: None,
        },
        Err(e) => PythonEnvStatus {
            status: "error".to_string(),
            message: "环境检查失败".to_string(),
            details: Some(e),
        },
    }
}
