/// Python 嵌入式环境安装器
/// 负责下载和安装嵌入式 Python 环境

use std::path::PathBuf;
use tauri::AppHandle;

/// 嵌入式 Python 下载信息
#[derive(Debug)]
pub struct EmbeddedPythonInfo {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
}

/// 获取嵌入式 Python 信息
pub fn get_embedded_python_info() -> EmbeddedPythonInfo {
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 python-build-standalone 项目（版本 3.11.10）
        // 使用 ghproxy 镜像加速中国用户访问
        EmbeddedPythonInfo {
            version: "3.11.10".to_string(),
            download_url: "https://ghproxy.com/https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz".to_string(),
            sha256: "a5fc05c5ca825e714ce86ee77501c4bdc5cf0396a160925a1a538e6469a2504b".to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 python.org 官方 embeddable 版本
        EmbeddedPythonInfo {
            version: "3.11.10".to_string(),
            download_url: "https://www.python.org/ftp/python/3.11.10/python-3.11.10-embed-amd64.zip".to_string(),
            sha256: "".to_string(),
        }
    }
}

/// 下载并解压嵌入式 Python
pub async fn download_embedded_python(app: &AppHandle) -> Result<PathBuf, String> {
    use tracing::info;
    use tauri::Manager;
    use std::process::Command;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let python_dir = app_data_dir.join("python");
    std::fs::create_dir_all(&python_dir)
        .map_err(|e| format!("Failed to create python directory: {}", e))?;

    let info = get_embedded_python_info();
    info!("📥 Downloading embedded Python {}...", info.version);

    // 下载文件
    let client = reqwest::Client::new();
    let response = client
        .get(&info.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download Python: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    // 保存到临时文件
    let temp_file = python_dir.join("python_temp.tar.gz");
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    tokio::fs::write(&temp_file, bytes)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    info!("📦 Extracting embedded Python...");

    // 解压 tar.gz 文件（macOS/Linux）
    #[cfg(not(target_os = "windows"))]
    {
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&temp_file)
            .arg("-C")
            .arg(&python_dir)
            .arg("--strip-components=1") // 去掉顶层目录
            .status()
            .map_err(|e| format!("Failed to extract Python: {}", e))?;

        if !status.success() {
            return Err("Failed to extract Python archive".to_string());
        }
    }

    // 解压 zip 文件（Windows）
    #[cfg(target_os = "windows")]
    {
        // 使用 PowerShell 解压
        let status = Command::new("powershell")
            .arg("-Command")
            .arg(format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", temp_file.display(), python_dir.display()))
            .status()
            .map_err(|e| format!("Failed to extract Python: {}", e))?;

        if !status.success() {
            return Err("Failed to extract Python archive".to_string());
        }
    }

    // 删除临时文件
    let _ = std::fs::remove_file(&temp_file);

    info!("✅ Embedded Python extracted to: {:?}", python_dir);
    Ok(python_dir)
}

/// 安装 pip 到嵌入式 Python（使用中国镜像源）
pub async fn install_pip(python_dir: &PathBuf) -> Result<(), String> {
    use tracing::info;

    info!("📦 Installing pip...");

    #[cfg(target_os = "macos")]
    let python_path = python_dir.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let python_path = python_dir.join("python.exe");

    // 下载 get-pip.py（使用阿里云镜像加速中国用户访问）
    let client = reqwest::Client::new();
    let response = client
        .get("https://mirrors.aliyun.com/pypi/get-pip.py")
        .send()
        .await
        .map_err(|e| format!("Failed to download get-pip.py: {}", e))?;

    let get_pip_script = response
        .text()
        .await
        .map_err(|e| format!("Failed to read get-pip.py: {}", e))?;

    let get_pip_path = python_dir.join("get-pip.py");
    tokio::fs::write(&get_pip_path, get_pip_script)
        .await
        .map_err(|e| format!("Failed to write get-pip.py: {}", e))?;

    // 执行安装（使用国内 PyPI 镜像）
    let status = std::process::Command::new(&python_path)
        .arg(&get_pip_path)
        .arg("--index-url")
        .arg("https://mirror.sjtu.edu.cn/pypi/web/simple")  // 上海交通大学 PyPI 镜像
        .status()
        .map_err(|e| format!("Failed to install pip: {}", e))?;

    if !status.success() {
        return Err("pip installation failed".to_string());
    }

    info!("✅ pip installed successfully");
    Ok(())
}
