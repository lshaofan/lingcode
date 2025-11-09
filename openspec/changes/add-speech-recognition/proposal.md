# Proposal: 语音识别功能 (Whisper 集成)

## Why

语音识别是「聆码」的核心引擎，将录制的音频转换为文字。选择 Whisper 是因为其开源、多语言支持和可本地部署的特性，完美契合项目的隐私保护理念。

### 用户需求
- 用户需要高准确度的中文语音识别
- 用户期望实时或准实时的转录速度（< 2秒）
- 用户希望支持中英混说场景
- 用户需要能够查看和复制转录结果

### 技术必要性
- 需要集成 Whisper 模型推理引擎
- 需要管理多个模型文件（tiny/base/small/medium）
- 需要优化推理性能（CPU/GPU）
- 需要处理转录结果的后处理（标点、大小写）

## What Changes

### 新增功能
1. **Whisper 模型管理**
   - 模型下载和验证
   - 多模型切换（tiny, base, small, medium）
   - 模型存储和缓存管理
   - 模型更新检测

2. **语音识别引擎**
   - 集成 `whisper.cpp` 或 `faster-whisper`
   - 音频预处理（重采样、标准化）
   - 实时/批量转录
   - 多语言自动检测
   - 置信度评分

3. **转录结果处理**
   - 文本标准化（去除噪音）
   - 标点符号优化
   - 中英文大小写处理
   - 时间戳生成（可选）

4. **转录历史管理**
   - 转录结果存储到数据库
   - 转录历史查询和搜索
   - 转录结果编辑和删除
   - 导出转录历史

5. **性能优化**
   - 模型量化（INT8）
   - 批量推理
   - GPU 加速（可选）
   - 推理缓存

### 涉及的技术栈
- **Rust**: `whisper-rs` 或 FFI 绑定到 `whisper.cpp`
- **C/C++**: `whisper.cpp` 高性能推理引擎
- **Python** (可选): `faster-whisper` 作为备选方案
- **模型**: OpenAI Whisper 模型（GGML 格式）

## Impact

### 受影响的 Specs
- `whisper-integration` (新增) - Whisper 模型集成
- `model-management` (新增) - 模型下载和管理
- `data-storage` (修改) - 新增转录历史表
- `audio-recording` (修改) - 音频数据传递给识别引擎

### 受影响的代码
- `src-tauri/src/whisper/` (新增) - Whisper 推理模块
- `src-tauri/src/commands/transcription.rs` (新增) - 转录 Commands
- `src-tauri/src/db/transcriptions.rs` (新增) - 转录历史数据访问
- `src/stores/transcriptionStore.ts` (新增) - 转录状态管理
- `src/windows/history/` (新增) - 历史记录窗口

### 依赖
- **模型文件**:
  - `ggml-tiny.bin` (75MB)
  - `ggml-base.bin` (142MB)
  - `ggml-small.bin` (466MB)
  - `ggml-medium.bin` (1.5GB)
- **Rust Crates**:
  - `whisper-rs` ^0.10 或自定义 FFI
  - `reqwest` ^0.11 (模型下载)
  - `sha256` ^1.0 (校验和验证)
- **系统库**:
  - Core ML (macOS GPU 加速，可选)

### 性能影响
- 内存占用: +200MB - 2GB（取决于模型大小）
- 推理时间: 1-5秒（取决于音频长度和模型）
- 磁盘占用: +150MB - 1.5GB（模型文件）

## Risks & Mitigation

### 风险 1: Whisper 中文准确度不如专门模型
- **影响**: 用户可能对转录质量不满意
- **缓解**:
  - 提供多个模型选择（small/medium 准确度更高）
  - 支持后续 fine-tune 中文模型
  - 允许用户手动修正转录结果
  - 提供置信度指标，低置信度时提示用户

### 风险 2: 模型文件体积大
- **影响**: 首次下载耗时长，占用存储空间
- **缓解**:
  - 默认使用 tiny 或 base 模型（< 150MB）
  - 提供按需下载更大模型的选项
  - 实现模型下载进度显示
  - 允许用户删除不需要的模型

### 风险 3: 推理速度慢
- **影响**: 用户等待时间长，影响体验
- **缓解**:
  - 使用 `whisper.cpp` 优化推理性能
  - 支持 GPU 加速（Core ML on macOS）
  - 实现模型量化（INT8）
  - 显示转录进度条

### 风险 4: 跨平台兼容性
- **影响**: 不同平台 Whisper 集成方式不同
- **缓解**:
  - 使用 `whisper.cpp` 作为跨平台基础
  - 抽象推理接口，平台特定实现
  - 提供纯 CPU 后备方案

## Timeline

- **Week 1**: Whisper 模型下载和管理
- **Week 2**: whisper.cpp 集成和基础推理
- **Week 3**: 转录流程和结果处理
- **Week 4**: 性能优化和 GPU 加速
- **Week 5**: 测试和准确度调优
