#!/bin/bash

# 准备 Python 打包环境
# 用于生产环境打包,创建一个精简的 Python 环境

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUNDLE_DIR="$PROJECT_ROOT/python-bundle"

echo "📦 Preparing Python bundle for production..."
echo "   Bundle directory: $BUNDLE_DIR"

# 清理旧的打包目录
if [ -d "$BUNDLE_DIR" ]; then
    echo "🗑️  Removing old bundle directory..."
    rm -rf "$BUNDLE_DIR"
fi

mkdir -p "$BUNDLE_DIR"
cd "$BUNDLE_DIR"

# 检测系统架构
ARCH=$(uname -m)
OS=$(uname -s)

echo "🔍 Detected system: $OS $ARCH"

if [ "$OS" == "Darwin" ]; then
    if [ "$ARCH" == "arm64" ]; then
        # macOS ARM64 (Apple Silicon)
        PYTHON_URL="https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-aarch64-apple-darwin-install_only.tar.gz"
        PYTHON_FILENAME="python-macos-arm64.tar.gz"
    else
        # macOS x86_64 (Intel)
        PYTHON_URL="https://github.com/indygreg/python-build-standalone/releases/download/20241016/cpython-3.11.10+20241016-x86_64-apple-darwin-install_only.tar.gz"
        PYTHON_FILENAME="python-macos-x64.tar.gz"
    fi
else
    echo "❌ Unsupported OS: $OS"
    exit 1
fi

# 下载 Python
echo "📥 Downloading Python from $PYTHON_URL..."
curl -L -o "$PYTHON_FILENAME" "$PYTHON_URL"

# 解压
echo "📦 Extracting Python..."
tar -xzf "$PYTHON_FILENAME"
rm "$PYTHON_FILENAME"

# 重命名目录
mv python python-temp || true
if [ -d "python-temp" ]; then
    mv python-temp python
fi

# 安装依赖
echo "📦 Installing Python dependencies..."
PYTHON_BIN="$BUNDLE_DIR/python/bin/python3"

$PYTHON_BIN -m pip install --upgrade pip -i https://mirror.sjtu.edu.cn/pypi/web/simple

echo "   Installing torch..."
# 优先使用清华镜像加速中国用户下载
$PYTHON_BIN -m pip install torch torchaudio \
    -i https://pypi.tuna.tsinghua.edu.cn/simple

echo "   Installing funasr..."
$PYTHON_BIN -m pip install funasr -i https://mirror.sjtu.edu.cn/pypi/web/simple

echo "   Installing modelscope..."
$PYTHON_BIN -m pip install modelscope -i https://mirror.sjtu.edu.cn/pypi/web/simple

# 精简环境
echo "🧹 Cleaning up Python environment..."

# 删除测试文件
rm -rf python/lib/python3.11/test
rm -rf python/lib/python3.11/tkinter
rm -rf python/lib/python3.11/idlelib
rm -rf python/lib/python3.11/ensurepip

# 删除所有 __pycache__ 和 .pyc 文件
find python -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
find python -name "*.pyc" -delete 2>/dev/null || true

# 删除不需要的文档和示例
find python -name "*.md" -delete 2>/dev/null || true
# 删除 .txt 文件,但保留 funasr 的 version.txt
find python -name "*.txt" ! -path "*/funasr/version.txt" -delete 2>/dev/null || true
find python -type d -name "examples" -exec rm -rf {} + 2>/dev/null || true
find python -type d -name "tests" -exec rm -rf {} + 2>/dev/null || true

# 计算大小
BUNDLE_SIZE=$(du -sh python | cut -f1)
echo "✅ Python bundle prepared successfully"
echo "   Size: $BUNDLE_SIZE"
echo "   Location: $BUNDLE_DIR/python"

# 创建 README
cat > python/README.txt <<EOF
This is a bundled Python 3.11 environment for FunASR.

Included packages:
- Python 3.11.10
- torch (CPU only)
- torchaudio
- funasr
- modelscope

This environment is copied to the application data directory on first run.
EOF

echo ""
echo "📝 Next steps:"
echo "   1. Verify the bundle works: $PYTHON_BIN --version"
echo "   2. Test FunASR import: $PYTHON_BIN -c 'import funasr; print(funasr.__version__)'"
echo "   3. Update tauri.conf.json to include this directory in resources"
echo "   4. Run: npm run tauri build"
echo ""
echo "⚠️  Note: The bundle will be copied to:"
echo "   ~/Library/Application Support/com.lingcode.app/python/"
echo "   on first application run."
