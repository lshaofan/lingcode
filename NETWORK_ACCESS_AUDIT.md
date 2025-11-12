# 网络访问审计报告 - 中国用户可用性检查

## 📋 概述

本报告检查所有需要联网下载的地方,确保中国用户可以正常访问。

## ✅ 已使用中国镜像源的地方

### 1. **PyPI 包安装** ✅

**文件: `src-tauri/src/python/mod.rs:318-325`**
```rust
cmd.args(&[
    "-m",
    "pip",
    "install",
    package,
    "-i",
    "https://mirror.sjtu.edu.cn/pypi/web/simple",  // ✅ 上海交通大学镜像
]);
```
- ✅ 使用上海交通大学 PyPI 镜像
- ✅ 中国境内高速访问
- ✅ 稳定可靠

**文件: `src-tauri/src/python/installer.rs:154`**
```rust
.arg("https://mirror.sjtu.edu.cn/pypi/web/simple")
```
- ✅ pip 安装时也使用交大镜像

**文件: `scripts/prepare-python-bundle.sh:66-70`**
```bash
$PYTHON_BIN -m pip install --upgrade pip \
    -i https://mirror.sjtu.edu.cn/pypi/web/simple

$PYTHON_BIN -m pip install torch torchaudio \
    --index-url https://download.pytorch.org/whl/cpu
```
- ✅ pip 升级使用交大镜像
- ⚠️ PyTorch 使用官方源 (需要检查)

### 2. **Whisper 模型下载** ✅

**文件: `src-tauri/src/commands/model.rs:46-150`**
```rust
download_url: "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
```
- ✅ 使用 `hf-mirror.com` (HuggingFace 中国镜像)
- ✅ 中国境内可访问
- ✅ 所有 Whisper 模型 (base, small, medium, large) 都使用镜像

### 3. **FunASR/ModelScope** ✅

**文件: `src-tauri/scripts/funasr_transcribe.py:158`**
```python
os.environ["MODELSCOPE_ENDPOINT"] = "https://www.modelscope.cn"
```
- ✅ 使用阿里云 ModelScope 中国站
- ✅ 专为中国用户优化
- ✅ 高速下载

## ⚠️ 需要注意的地方

### 1. **Python 运行时下载** ⚠️

**文件: `src-tauri/src/python/installer.rs:22`**
```rust
download_url: "https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz"
```

**状态:** ⚠️ 使用 GitHub releases
- **问题:** GitHub 在中国访问较慢或不稳定
- **影响:** 仅在动态下载模式下影响 (打包模式不受影响)
- **解决方案:**
  1. **推荐:** 使用打包模式 (Python 已包含在应用中)
  2. **备选:** 使用 GitHub 镜像 (ghproxy.com)

**修改建议:**
```rust
// 原始URL (慢)
"https://github.com/indygreg/python-build-standalone/releases/..."

// 使用镜像 (快)
"https://ghproxy.com/https://github.com/indygreg/python-build-standalone/releases/..."
```

### 2. **get-pip.py 下载** ⚠️

**文件: `src-tauri/src/python/installer.rs:135`**
```rust
.get("https://bootstrap.pypa.io/get-pip.py")
```

**状态:** ⚠️ 使用国外源
- **问题:** bootstrap.pypa.io 在中国访问较慢
- **影响:** 仅在动态下载模式下影响
- **解决方案:** 使用镜像或打包模式

**修改建议:**
```rust
// 使用国内镜像
.get("https://mirrors.aliyun.com/pypi/get-pip.py")
```

### 3. **PyTorch 下载** ✅ (已使用官方 CDN)

**文件: `scripts/prepare-python-bundle.sh:69`**
```bash
--index-url https://download.pytorch.org/whl/cpu
```

**状态:** ✅ 官方 CDN,中国可访问
- PyTorch 官方使用全球 CDN
- 中国境内访问速度可接受
- 如需更快,可使用清华镜像:
  ```bash
  -i https://pypi.tuna.tsinghua.edu.cn/simple
  ```

## 🔍 详细检查清单

| 资源 | URL | 中国可用性 | 推荐方案 |
|------|-----|-----------|---------|
| **PyPI 包** | mirror.sjtu.edu.cn | ✅ 极快 | 保持不变 |
| **Whisper 模型** | hf-mirror.com | ✅ 快速 | 保持不变 |
| **FunASR 模型** | modelscope.cn | ✅ 极快 | 保持不变 |
| **Python 运行时** | github.com | ⚠️ 较慢 | 使用打包模式 |
| **get-pip.py** | bootstrap.pypa.io | ⚠️ 较慢 | 使用镜像/打包 |
| **PyTorch** | download.pytorch.org | ✅ 可用 | 可选清华镜像 |

## 💡 推荐配置

### 方案 A: 打包模式 (推荐) ⭐

**优势:**
- ✅ 完全离线,无需下载 Python/依赖
- ✅ 不受网络影响
- ✅ 用户体验最佳

**实现:**
```bash
# 运行一次即可
./scripts/prepare-python-bundle.sh

# 打包
npm run tauri build
```

### 方案 B: 动态下载模式 + 镜像优化

**如果必须使用动态下载,建议修改以下文件:**

#### 1. Python 运行时使用镜像

**修改: `src-tauri/src/python/installer.rs:22`**
```rust
// 使用 ghproxy 加速 GitHub
download_url: "https://ghproxy.com/https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz".to_string(),
```

#### 2. get-pip.py 使用镜像

**修改: `src-tauri/src/python/installer.rs:135`**
```rust
// 使用阿里云镜像
.get("https://mirrors.aliyun.com/pypi/get-pip.py")
```

#### 3. PyTorch 使用清华镜像 (可选)

**修改: `scripts/prepare-python-bundle.sh:69`**
```bash
echo "   Installing torch..."
$PYTHON_BIN -m pip install torch torchaudio \
    -i https://pypi.tuna.tsinghua.edu.cn/simple
```

## 📊 速度对比测试

### PyPI 镜像源速度 (中国境内)

```bash
# 测试命令
time pip install requests -i <MIRROR_URL>
```

| 镜像源 | 速度 | 稳定性 |
|--------|------|--------|
| 上海交大 (当前) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 清华大学 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 阿里云 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 官方 PyPI | ⭐⭐ | ⭐⭐⭐ |

### GitHub 加速方案速度

| 方案 | 速度 | 稳定性 |
|------|------|--------|
| ghproxy.com | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| fastgit.org | ⭐⭐⭐ | ⭐⭐⭐ |
| 直连 GitHub | ⭐ | ⭐⭐ |
| **打包模式** | **⭐⭐⭐⭐⭐** | **⭐⭐⭐⭐⭐** |

## ✅ 总结

### 当前状态: 🟢 良好

- ✅ 核心依赖 (PyPI) 已使用国内镜像
- ✅ Whisper 模型已使用 HF 镜像
- ✅ FunASR 已使用 ModelScope 中国站
- ⚠️ Python 运行时和 get-pip 仍使用国外源

### 建议措施:

**高优先级 (推荐):**
1. ✅ **使用打包模式** - 已实现,完全避免网络问题

**中优先级 (可选优化):**
2. ⚠️ 为动态下载模式添加 GitHub 镜像支持
3. ⚠️ 为 get-pip.py 添加镜像支持

**低优先级:**
4. PyTorch 使用清华镜像 (当前官方源已足够快)

### 用户体验评估:

| 模式 | 中国用户体验 | 评分 |
|------|------------|------|
| **打包模式** | 极佳,完全离线 | ⭐⭐⭐⭐⭐ |
| 动态下载 (当前) | 良好,部分较慢 | ⭐⭐⭐⭐ |
| 动态下载 (优化后) | 很好,全部快速 | ⭐⭐⭐⭐⭐ |

## 🔧 快速修复方案

如果用户反馈网络问题,可以立即应用以下补丁:

```bash
# 1. 创建补丁文件
cat > china-mirror.patch <<'EOF'
--- a/src-tauri/src/python/installer.rs
+++ b/src-tauri/src/python/installer.rs
@@ -19,7 +19,7 @@
         // macOS: 使用 python-build-standalone 项目（版本 3.11.10）
         EmbeddedPythonInfo {
             version: "3.11.10".to_string(),
-            download_url: "https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz".to_string(),
+            download_url: "https://ghproxy.com/https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz".to_string(),
             sha256: "a5fc05c5ca825e714ce86ee77501c4bdc5cf0396a160925a1a538e6469a2504b".to_string(),
         }
     }
@@ -132,7 +132,7 @@
     // 下载 get-pip.py（使用中国镜像）
     let client = reqwest::Client::new();
     let response = client
-        .get("https://bootstrap.pypa.io/get-pip.py")
+        .get("https://mirrors.aliyun.com/pypi/get-pip.py")
         .send()
         .await
         .map_err(|e| format!("Failed to download get-pip.py: {}", e))?;
EOF

# 2. 应用补丁
git apply china-mirror.patch
```

## 📞 用户支持

如果用户在中国遇到下载问题:

1. **首选方案:** 使用打包版本 (无需下载)
2. **临时方案:** 提供离线安装包
3. **技术支持:** 引导用户检查网络连接或使用 VPN

## 🔄 持续监控

建议定期检查以下镜像源的可用性:
- ✅ mirror.sjtu.edu.cn
- ✅ hf-mirror.com
- ✅ modelscope.cn
- ⚠️ ghproxy.com (如果使用)
