# Python 依赖说明文档

本文档详细说明了 lingcode 应用中 FunASR 语音识别功能所需的 Python 环境和依赖。

## 📋 依赖清单

### 核心依赖

#### 1. Python 运行时
- **版本要求**: Python 3.8 或更高（推荐 3.11）
- **安装位置**:
  - 生产环境: `{app_data_dir}/python/` (自动下载)
  - 开发环境: 系统 Python 3

#### 2. PyTorch 生态
```
torch >= 2.0.0          # PyTorch 深度学习框架 (~200MB)
torchaudio >= 2.0.0     # PyTorch 音频处理库 (~20MB)
```

#### 3. FunASR 相关
```
funasr >= 1.0.0         # 阿里达摩院语音识别框架 (~10MB)
modelscope >= 1.9.0     # 模型下载和管理工具 (~50MB)
```

### 隐式依赖（自动安装）
以下依赖会作为上述包的传递依赖自动安装：
```
numpy                   # 数值计算
onnxruntime            # ONNX 推理引擎（或 onnxruntime-gpu）
kaldifeat              # Kaldi 特征提取
librosa                # 音频分析
soundfile              # 音频文件读写
```

## 💾 存储空间要求

### Python 环境
- **嵌入式 Python**: ~80MB
- **依赖包总计**: ~380MB
- **模型文件**:
  - paraformer-zh: ~220MB (推荐)
  - paraformer-large: ~380MB
  - sensevoice-small: ~160MB

**总计**: 约 680MB - 1.2GB (取决于选择的模型)

## 📦 安装方式

### 自动安装（推荐）
应用会根据用户配置自动处理Python环境：

1. **首次使用FunASR时**：
   - 检测系统Python或下载嵌入式Python
   - 自动安装所有依赖包
   - 显示详细安装进度

2. **已有环境**：
   - 快速检查环境健康状态
   - 可选后台预热，提升识别速度

### 手动安装（离线场景）
如需离线安装，请按以下步骤操作：

```bash
# 1. 确保Python 3.8+已安装
python3 --version

# 2. 安装依赖（使用国内镜像）
python3 -m pip install torch torchaudio funasr modelscope \
    -i https://mirror.sjtu.edu.cn/pypi/web/simple

# 3. 下载模型（可选，应用内也可下载）
python3 -c "from modelscope import snapshot_download; \
    snapshot_download('damo/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch')"
```

## 🚀 性能优化

### 预热机制
- **启用条件**: 数据库配置中 `enable_prewarming=true`
- **效果**: 首次识别从 5-10秒 降至 <1秒
- **内存占用**: 约 2GB (模型常驻内存)

### 缓存策略
- 环境检查结果缓存 5 分钟
- 避免重复的Python进程启动和依赖验证

## 🔧 开发环境配置

### 开发时测试
```bash
# 在 src-tauri 目录下
cd src-tauri

# 测试Python脚本
python3 scripts/funasr_transcribe.py check --model paraformer-zh

# 测试识别（需要音频文件）
python3 scripts/funasr_transcribe.py transcribe \
    --audio /path/to/audio.wav \
    --model paraformer-zh
```

### 环境变量
无特殊环境变量要求。应用会自动配置：
- `MODELSCOPE_CACHE`: 模型缓存目录 (默认 `~/.cache/modelscope`)
- Python 搜索路径会自动调整以使用嵌入式Python

## 📝 故障排除

### 常见问题

**1. 依赖安装失败**
```
解决方案：
- 检查网络连接
- 尝试切换镜像源
- 查看日志文件获取详细错误
```

**2. 模型加载缓慢**
```
原因：首次加载需要初始化PyTorch和模型权重
解决方案：启用预热机制，或等待首次初始化完成
```

**3. 内存不足**
```
症状：应用崩溃或识别失败
要求：至少 4GB 可用内存
解决方案：关闭其他应用程序释放内存
```

## 🔄 更新说明

### 依赖更新
依赖包会在以下情况自动检查和更新：
- 应用版本更新
- 用户手动触发重新安装
- 依赖验证失败

### 模型更新
- 模型文件需要用户手动下载新版本
- 应用不会自动更新已下载的模型

## 🔒 安全性

### 依赖来源
- **PyPI镜像**: https://mirror.sjtu.edu.cn/pypi/web/simple
- **模型仓库**: https://www.modelscope.cn

### 隔离措施
- 使用虚拟环境避免污染系统Python（生产环境）
- 依赖签名验证（通过 pip 自动处理）

## 📚 参考资料

- [FunASR 官方文档](https://github.com/alibaba-damo-academy/FunASR)
- [ModelScope 文档](https://modelscope.cn/docs)
- [PyTorch 音频文档](https://pytorch.org/audio)

---

**最后更新**: 2025-01-11
**维护者**: Claude Code
