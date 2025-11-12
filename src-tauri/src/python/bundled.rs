/// 打包的 Python 环境管理
/// 负责从应用资源目录复制 Python 环境到可写目录

use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

/// 检查打包的 Python 是否存在
pub fn is_bundled_python_available(app: &AppHandle) -> bool {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled_python = resource_dir.join("python");

        #[cfg(target_os = "macos")]
        let python_exe = bundled_python.join("bin").join("python3");

        #[cfg(target_os = "windows")]
        let python_exe = bundled_python.join("python.exe");

        return python_exe.exists();
    }

    false
}

/// 获取打包的 Python 路径
pub fn get_bundled_python_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let bundled_python = resource_dir.join("python");

    #[cfg(target_os = "macos")]
    let python_exe = bundled_python.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let python_exe = bundled_python.join("python.exe");

    if !python_exe.exists() {
        return Err("Bundled Python not found".to_string());
    }

    Ok(bundled_python)
}

/// 将打包的 Python 复制到应用数据目录
/// 因为应用资源目录是只读的,需要复制到可写位置
pub async fn setup_bundled_python(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;

    info!("📦 Setting up bundled Python environment...");

    // 获取目标目录
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let target_python_dir = app_data_dir.join("python");

    // 检查是否已经复制过
    #[cfg(target_os = "macos")]
    let target_python_exe = target_python_dir.join("bin").join("python3");

    #[cfg(target_os = "windows")]
    let target_python_exe = target_python_dir.join("python.exe");

    if target_python_exe.exists() {
        info!("✅ Python environment already exists at: {:?}", target_python_dir);
        return Ok(target_python_dir);
    }

    // 获取打包的 Python 路径
    let bundled_python_dir = get_bundled_python_path(app)?;

    info!("📂 Copying Python from {:?} to {:?}", bundled_python_dir, target_python_dir);

    // 创建目标目录
    tokio::fs::create_dir_all(&target_python_dir)
        .await
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // 复制目录 (使用异步方式避免阻塞)
    copy_dir_recursive(&bundled_python_dir, &target_python_dir).await?;

    info!("✅ Python environment copied successfully");

    // 在 macOS 上设置可执行权限
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;

        let python_exe = target_python_dir.join("bin").join("python3");
        if let Ok(metadata) = tokio::fs::metadata(&python_exe).await {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = tokio::fs::set_permissions(&python_exe, permissions).await;
        }
    }

    Ok(target_python_dir)
}

/// 递归复制目录 (使用 Box::pin 避免无限大小的 future)
fn copy_dir_recursive<'a>(
    src: &'a PathBuf,
    dst: &'a PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        use tokio::fs;

        // 创建目标目录
        fs::create_dir_all(dst)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        // 读取源目录
        let mut entries = fs::read_dir(src)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        // 遍历条目
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            // 跳过 __pycache__ 和 .pyc 文件
            if file_name == "__pycache__" || file_name.to_string_lossy().ends_with(".pyc") {
                continue;
            }

            let metadata = entry
                .metadata()
                .await
                .map_err(|e| format!("Failed to get metadata: {}", e))?;

            if metadata.is_dir() {
                // 递归复制子目录
                copy_dir_recursive(&src_path, &dst_path).await?;
            } else {
                // 复制文件
                if let Err(e) = fs::copy(&src_path, &dst_path).await {
                    warn!("Failed to copy {:?}: {}", src_path, e);
                    // 继续复制其他文件,不中断整个过程
                }
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundled_python_detection() {
        // 测试逻辑...
    }
}
