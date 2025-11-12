/// Python 环境管理模块
/// 负责检测、初始化和管理嵌入式 Python 环境

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub mod bundled;
pub mod installer;

/// 环境检查缓存
static ENV_CACHE: Mutex<Option<EnvCheckCache>> = Mutex::new(None);

/// Python依赖安装进度事件
#[derive(Debug, Clone, Serialize)]
pub struct PythonInstallProgress {
    pub step: String,
    pub progress: u32,
    pub message: String,
}

#[derive(Debug, Clone)]
struct EnvCheckCache {
    python_exists: bool,
    dependencies_installed: bool,
    timestamp: std::time::Instant,
}

impl EnvCheckCache {
    fn is_valid(&self) -> bool {
        // 缓存5分钟
        self.timestamp.elapsed().as_secs() < 300
    }
}

/// 检查模式
#[derive(Debug, Clone, Copy)]
pub enum CheckMode {
    /// 快速检查：仅验证文件存在性
    Quick,
    /// 完整检查：运行Python验证依赖
    Full,
}

/// Python 环境信息
#[derive(Debug, Clone)]
pub struct PythonEnv {
    pub python_path: PathBuf,
    pub version: String,
    pub is_embedded: bool,
    pub is_venv: bool,
    pub venv_path: Option<PathBuf>,
}

/// 快速检查Python环境健康状态（用于启动时）
/// 仅检查文件存在性，不启动Python进程
pub fn quick_check_python_health(app: &AppHandle) -> Result<bool, String> {
    use tauri::Manager;

    // 检查缓存
    if let Ok(cache) = ENV_CACHE.lock() {
        if let Some(cached) = cache.as_ref() {
            if cached.is_valid() {
                return Ok(cached.python_exists && cached.dependencies_installed);
            }
        }
    }

    // 检查嵌入式Python是否存在
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let python_dir = app_data_dir.join("python");

    #[cfg(target_os = "macos")]
    let python_path = python_dir.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let python_path = python_dir.join("python.exe");

    if !python_path.exists() {
        return Ok(false);
    }

    // 快速检查：验证site-packages目录存在（说明依赖可能已安装）
    #[cfg(target_os = "macos")]
    let site_packages = python_dir.join("lib").join("python3.11").join("site-packages");

    #[cfg(target_os = "windows")]
    let site_packages = python_dir.join("Lib").join("site-packages");

    let deps_exist = site_packages.exists() &&
                    site_packages.join("torch").exists() &&
                    site_packages.join("funasr").exists();

    // 更新缓存
    if let Ok(mut cache) = ENV_CACHE.lock() {
        *cache = Some(EnvCheckCache {
            python_exists: true,
            dependencies_installed: deps_exist,
            timestamp: std::time::Instant::now(),
        });
    }

    Ok(deps_exist)
}

/// 清除环境检查缓存
pub fn clear_env_cache() {
    if let Ok(mut cache) = ENV_CACHE.lock() {
        *cache = None;
    }
}

/// 检测 Python 环境
pub fn detect_python(app: &AppHandle) -> Result<PythonEnv, String> {
    // 1. 优先检查应用数据目录中的 Python (可能是打包后复制的)
    if let Ok(embedded_env) = detect_embedded_python(app) {
        return Ok(embedded_env);
    }

    // 2. 检查打包的 Python (仅开发模式会走到这里,生产模式应该已经复制过了)
    if bundled::is_bundled_python_available(app) {
        use tracing::warn;
        warn!("⚠️ Found bundled Python but not copied yet. This should be done during app setup.");
    }

    // 3. 检查系统 Python (兜底方案)
    detect_system_python()
}

/// 检测嵌入式 Python
fn detect_embedded_python(app: &AppHandle) -> Result<PythonEnv, String> {
    use tauri::Manager;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let python_dir = app_data_dir.join("python");

    #[cfg(target_os = "macos")]
    let python_path = python_dir.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let python_path = python_dir.join("python.exe");

    if !python_path.exists() {
        return Err("Embedded Python not found".to_string());
    }

    // 获取版本
    let output = Command::new(&python_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to get Python version: {}", e))?;

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(PythonEnv {
        python_path,
        version,
        is_embedded: true,
        is_venv: false,
        venv_path: None,
    })
}

/// 检测系统 Python
fn detect_system_python() -> Result<PythonEnv, String> {
    let python_cmd = if cfg!(target_os = "windows") {
        "python"
    } else {
        "python3"
    };

    let output = Command::new(python_cmd)
        .arg("--version")
        .output()
        .map_err(|_| "Python not found in system".to_string())?;

    if !output.status.success() {
        return Err("Failed to execute Python".to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    // 检查版本是否 >= 3.8
    if !is_version_compatible(&version) {
        return Err(format!("Python version {} is not compatible. Requires Python 3.8+", version));
    }

    Ok(PythonEnv {
        python_path: PathBuf::from(python_cmd),
        version,
        is_embedded: false,
        is_venv: false,
        venv_path: None,
    })
}

/// 检查 Python 版本是否兼容
fn is_version_compatible(version: &str) -> bool {
    // 提取版本号，如 "Python 3.11.5" -> "3.11.5"
    let version_str = version
        .split_whitespace()
        .nth(1)
        .unwrap_or("");

    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    let major = parts[0].parse::<u32>().unwrap_or(0);
    let minor = parts[1].parse::<u32>().unwrap_or(0);

    major == 3 && minor >= 8
}

/// 检查 FunASR 是否已安装
pub fn is_funasr_installed(python_env: &PythonEnv) -> Result<bool, String> {
    let output = Command::new(&python_env.python_path)
        .args(&["-c", "import funasr; print(funasr.__version__)"])
        .output()
        .map_err(|e| format!("Failed to check FunASR: {}", e))?;

    Ok(output.status.success())
}

/// 检查 ModelScope 是否已安装
pub fn is_modelscope_installed(python_env: &PythonEnv) -> Result<bool, String> {
    let output = Command::new(&python_env.python_path)
        .args(&["-c", "import modelscope; print(modelscope.__version__)"])
        .output()
        .map_err(|e| format!("Failed to check ModelScope: {}", e))?;

    Ok(output.status.success())
}

/// 检查 PyTorch 是否已安装
pub fn is_torch_installed(python_env: &PythonEnv) -> Result<bool, String> {
    let output = Command::new(&python_env.python_path)
        .args(&["-c", "import torch; print(torch.__version__)"])
        .output()
        .map_err(|e| format!("Failed to check PyTorch: {}", e))?;

    Ok(output.status.success())
}

/// 检查 torchaudio 是否已安装
pub fn is_torchaudio_installed(python_env: &PythonEnv) -> Result<bool, String> {
    let output = Command::new(&python_env.python_path)
        .args(&["-c", "import torchaudio; print(torchaudio.__version__)"])
        .output()
        .map_err(|e| format!("Failed to check torchaudio: {}", e))?;

    Ok(output.status.success())
}

/// 安装 FunASR（带进度反馈）
pub async fn install_funasr(python_env: &PythonEnv) -> Result<(), String> {
    install_funasr_with_progress(python_env, None).await
}

/// 安装 FunASR（带进度反馈）
pub async fn install_funasr_with_progress(
    python_env: &PythonEnv,
    app: Option<&AppHandle>,
) -> Result<(), String> {
    use tracing::info;

    info!("🐍 Installing FunASR and dependencies (torch, torchaudio, modelscope)...");

    // 定义安装步骤
    let packages = vec![
        ("torch", 25, "PyTorch 深度学习框架"),
        ("torchaudio", 50, "PyTorch 音频处理库"),
        ("modelscope", 75, "模型下载管理工具"),
        ("funasr", 100, "FunASR 语音识别框架"),
    ];

    // 开始安装
    if let Some(app_handle) = app {
        let _ = app_handle.emit(
            "python-install-progress",
            PythonInstallProgress {
                step: "开始安装".to_string(),
                progress: 0,
                message: "准备安装 Python 依赖...".to_string(),
            },
        );
    }

    for (package, progress, description) in packages {
        info!("📦 Installing {}: {}", package, description);

        if let Some(app_handle) = app {
            let _ = app_handle.emit(
                "python-install-progress",
                PythonInstallProgress {
                    step: format!("安装 {}", package),
                    progress,
                    message: format!("正在安装 {}...", description),
                },
            );
        }

        // 单独安装每个包，便于跟踪进度
        let mut cmd = Command::new(&python_env.python_path);
        cmd.args(&[
            "-m",
            "pip",
            "install",
            package,
            "-i",
            "https://mirror.sjtu.edu.cn/pypi/web/simple",
        ]);

        // 如果是系统 Python，添加 --break-system-packages 参数
        if !python_env.is_embedded && !python_env.is_venv {
            cmd.arg("--break-system-packages");
        }

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to install {}: {}", package, e))?;

        if !status.success() {
            return Err(format!("{} installation failed", package));
        }

        info!("✅ {} installed successfully", package);
    }

    // 完成
    if let Some(app_handle) = app {
        let _ = app_handle.emit(
            "python-install-progress",
            PythonInstallProgress {
                step: "完成".to_string(),
                progress: 100,
                message: "所有依赖安装完成！".to_string(),
            },
        );
    }

    info!("✅ FunASR and dependencies installed successfully");
    Ok(())
}

/// 获取或初始化 Python 环境（带缓存优化）
pub async fn ensure_python_env(app: &AppHandle) -> Result<PythonEnv, String> {
    ensure_python_env_with_mode(app, CheckMode::Full).await
}

/// 获取或初始化 Python 环境（指定检查模式）
pub async fn ensure_python_env_with_mode(
    app: &AppHandle,
    mode: CheckMode,
) -> Result<PythonEnv, String> {
    use tracing::info;

    // 1. 优先尝试使用应用数据目录中的 Python
    if let Ok(python_env) = detect_embedded_python(app) {
        info!("🐍 Found Python in app data: {}", python_env.version);

        // 根据检查模式决定是否验证依赖
        match mode {
            CheckMode::Quick => {
                // 快速模式：检查缓存或文件存在性
                if quick_check_python_health(app)? {
                    info!("✅ Quick check passed, dependencies likely installed");
                    return Ok(python_env);
                } else {
                    // 快速检查失败，可能需要完整检查
                    info!("⚠️ Quick check failed, performing full check...");
                }
            }
            CheckMode::Full => {
                // 完整模式：验证所有依赖
            }
        }

        // 检查所有依赖是否已安装
        let torch_installed = is_torch_installed(&python_env).unwrap_or(false);
        let torchaudio_installed = is_torchaudio_installed(&python_env).unwrap_or(false);
        let modelscope_installed = is_modelscope_installed(&python_env).unwrap_or(false);
        let funasr_installed = is_funasr_installed(&python_env).unwrap_or(false);

        // 如果有任何依赖缺失，重新安装
        if !torch_installed || !torchaudio_installed || !modelscope_installed || !funasr_installed {
            if !torch_installed {
                info!("📦 PyTorch not installed, installing all dependencies...");
            } else if !torchaudio_installed {
                info!("📦 torchaudio not installed, installing all dependencies...");
            } else if !modelscope_installed {
                info!("📦 ModelScope not installed, installing all dependencies...");
            } else {
                info!("📦 FunASR not installed, installing all dependencies...");
            }
            // 清除缓存，因为要重新安装
            clear_env_cache();
            install_funasr_with_progress(&python_env, Some(app)).await?;

            // 更新缓存
            if let Ok(mut cache) = ENV_CACHE.lock() {
                *cache = Some(EnvCheckCache {
                    python_exists: true,
                    dependencies_installed: true,
                    timestamp: std::time::Instant::now(),
                });
            }
        } else {
            info!("✅ All dependencies installed (torch, torchaudio, funasr, modelscope)");
        }

        return Ok(python_env);
    }

    // 2. 检查是否有打包的 Python (生产模式)
    if bundled::is_bundled_python_available(app) {
        info!("📦 Found bundled Python, setting up...");
        let python_dir = bundled::setup_bundled_python(app).await?;

        // 重新检测 Python 环境
        if let Ok(python_env) = detect_embedded_python(app) {
            info!("✅ Bundled Python setup complete: {}", python_env.version);

            // 打包的 Python 应该已经包含依赖,但仍然检查一次
            let all_installed = is_torch_installed(&python_env).unwrap_or(false)
                && is_torchaudio_installed(&python_env).unwrap_or(false)
                && is_modelscope_installed(&python_env).unwrap_or(false)
                && is_funasr_installed(&python_env).unwrap_or(false);

            if !all_installed {
                info!("⚠️ Bundled Python missing some dependencies, installing...");
                install_funasr_with_progress(&python_env, Some(app)).await?;
            }

            // 更新缓存
            if let Ok(mut cache) = ENV_CACHE.lock() {
                *cache = Some(EnvCheckCache {
                    python_exists: true,
                    dependencies_installed: true,
                    timestamp: std::time::Instant::now(),
                });
            }

            return Ok(python_env);
        }
    }

    // 3. 如果没有打包的 Python，下载并安装 (开发模式兜底)
    info!("📥 No bundled Python found, downloading...");
    let python_env = download_and_setup_embedded_python(app).await?;
    info!("🐍 Embedded Python ready: {} (embedded: true)", python_env.version);

    // 4. 安装 FunASR 和 ModelScope
    info!("📦 Installing FunASR and ModelScope in embedded Python...");
    install_funasr_with_progress(&python_env, Some(app)).await?;
    info!("✅ FunASR and ModelScope installed in embedded Python");

    // 更新缓存
    if let Ok(mut cache) = ENV_CACHE.lock() {
        *cache = Some(EnvCheckCache {
            python_exists: true,
            dependencies_installed: true,
            timestamp: std::time::Instant::now(),
        });
    }

    Ok(python_env)
}

/// 下载并设置嵌入式 Python
async fn download_and_setup_embedded_python(app: &AppHandle) -> Result<PythonEnv, String> {
    use tracing::info;

    // 下载嵌入式 Python
    let python_dir = installer::download_embedded_python(app).await?;

    // 安装 pip
    installer::install_pip(&python_dir).await?;

    // 返回 Python 环境信息
    let python_path = if cfg!(target_os = "macos") {
        python_dir.join("bin").join("python3")
    } else {
        python_dir.join("python.exe")
    };

    // 获取版本
    let output = Command::new(&python_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to get Python version: {}", e))?;

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    info!("✅ Embedded Python setup complete: {}", version);

    Ok(PythonEnv {
        python_path,
        version,
        is_embedded: true,
        is_venv: false,
        venv_path: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_check() {
        assert!(is_version_compatible("Python 3.8.0"));
        assert!(is_version_compatible("Python 3.11.5"));
        assert!(!is_version_compatible("Python 3.7.9"));
        assert!(!is_version_compatible("Python 2.7.18"));
    }
}
