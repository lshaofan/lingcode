# Python 环境打包方案

## 目标
最小化应用体积,同时确保 FunASR 功能开箱即用。

## 方案设计

### 1. 打包内容 (~400MB)

#### Python 运行时 (精简版)
- Python 3.11.10 (macOS: python-build-standalone)
- 移除不必要的模块:
  - test/
  - tkinter/
  - idlelib/
  - ensurepip/
  - __pycache__/

#### 依赖包
- torch (CPU only)
- torchaudio
- funasr
- modelscope (模型下载器)

#### 不打包内容
- ❌ 模型文件 (用户按需下载)
- ❌ Jupyter/IPython
- ❌ 开发工具

### 2. 目录结构

```
应用包内:
MyApp.app/
└── Contents/
    └── Resources/
        └── python/          # 打包的 Python 环境 (只读)
            ├── bin/
            ├── lib/
            └── site-packages/

首次运行后:
~/Library/Application Support/com.lingcode.app/
├── python/              # 从应用包复制过来 (可写)
│   ├── bin/
│   ├── lib/
│   └── site-packages/
└── models/              # 模型下载目录
    ├── funasr/
    │   ├── paraformer-zh/
    │   └── sensevoice-small/
    └── whisper/
        ├── base.bin
        └── small.bin
```

### 3. 实现步骤

#### Step 1: 准备精简 Python 环境

```bash
# 开发机上准备环境
cd /tmp
mkdir python-bundle
cd python-bundle

# 下载 Python
curl -L -o python.tar.gz \
  "https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz"

tar -xzf python.tar.gz

# 安装依赖
./bin/python3 -m pip install \
  torch torchaudio funasr modelscope \
  --index-url https://mirror.sjtu.edu.cn/pypi/web/simple

# 精简环境
rm -rf lib/python3.11/test
rm -rf lib/python3.11/tkinter
rm -rf lib/python3.11/idlelib
rm -rf lib/python3.11/ensurepip
find . -type d -name "__pycache__" -exec rm -rf {} +
find . -name "*.pyc" -delete

# 压缩
tar -czf python-bundle.tar.gz python/
```

#### Step 2: 修改打包配置

```json
// tauri.conf.json
{
  "bundle": {
    "resources": [
      "python-bundle/**/*",
      "scripts/**/*.py"
    ]
  }
}
```

#### Step 3: 修改代码逻辑

见下面的代码实现。

### 4. 用户体验流程

#### 首次启动:
1. 应用启动 (无需等待)
2. 后台复制 Python 环境 (1-2秒)
3. 预热成功 ✅

#### 首次使用 FunASR:
1. 用户选择 FunASR 模型
2. 提示下载模型 (~300MB)
3. 显示下载进度
4. 下载完成,可立即使用

### 5. 打包大小对比

| 方案 | 应用体积 | 首次启动 | 首次使用 | 离线可用 |
|------|---------|---------|---------|---------|
| 当前 (动态下载) | 50MB | 快 | 慢 (5-10分钟) | ❌ |
| 完整打包 | 1.2GB | 快 | 快 | ✅ |
| **最小打包 (推荐)** | **400MB** | **快** | **中等 (1-2分钟)** | **部分** |

### 6. 优势

- ✅ 应用体积适中
- ✅ 无需等待 Python 下载
- ✅ 预热功能始终可用
- ✅ 模型按需下载
- ✅ 更新模型无需更新应用
- ✅ 支持离线使用核心功能

### 7. 后续优化

- 支持增量更新 Python 依赖
- 智能检测已安装的依赖
- 模型缓存共享机制
