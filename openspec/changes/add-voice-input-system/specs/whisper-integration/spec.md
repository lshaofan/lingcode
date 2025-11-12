# whisper-integration Specification

## Purpose
定义 Whisper 语音识别引擎的集成规范，包括模型加载、音频预处理、推理执行和结果后处理。

## ADDED Requirements

### Requirement: Whisper Engine Initialization
系统 SHALL 提供 Whisper 推理引擎初始化功能。

#### Scenario: Load model successfully
**GIVEN** 模型文件存在于指定路径
**WHEN** 调用 WhisperEngine::new(model_path)
**THEN** 引擎应成功初始化
**AND** 返回可用的 WhisperEngine 实例

#### Scenario: Model file not found
**GIVEN** 模型文件不存在
**WHEN** 调用 WhisperEngine::new(model_path)
**THEN** 应返回 "Model not found" 错误
**AND** 错误信息应包含文件路径

#### Scenario: Model file corrupted
**GIVEN** 模型文件已损坏
**WHEN** 尝试加载模型
**THEN** 应返回 "Failed to load model" 错误

### Requirement: Audio Preprocessing
系统 SHALL 对输入音频进行预处理以满足 Whisper 模型输入要求。

#### Scenario: Convert i16 to f32
**GIVEN** 音频数据为 Vec<i16> 格式
**WHEN** 调用 preprocess_audio()
**THEN** 应转换为 Vec<f32> 范围 [-1.0, 1.0]
**AND** 转换公式应为 f32 = i16 / 32768.0

#### Scenario: Validate sample rate
**GIVEN** 音频采样率不是 16kHz
**WHEN** 调用 transcribe()
**THEN** 应返回 "Sample rate must be 16kHz" 错误

#### Scenario: Validate mono channel
**GIVEN** 音频不是单声道
**WHEN** 调用 transcribe()
**THEN** 应返回 "Audio must be mono" 错误

### Requirement: Transcription Execution
系统 SHALL 执行音频转录并返回文本结果。

#### Scenario: Transcribe Chinese audio
**GIVEN** 引擎已初始化且模型已加载
**AND** 提供有效的中文音频数据
**WHEN** 调用 transcribe(audio_data, language: "zh")
**THEN** 应返回中文转录文本
**AND** 转录准确率应 > 85%

#### Scenario: Transcribe English audio
**GIVEN** 引擎已初始化
**AND** 提供英文音频数据
**WHEN** 调用 transcribe(audio_data, language: "en")
**THEN** 应返回英文转录文本

#### Scenario: Auto-detect language
**GIVEN** 引擎已初始化
**AND** 未指定语言参数
**WHEN** 调用 transcribe(audio_data, language: None)
**THEN** Whisper 应自动检测语言
**AND** 返回检测到的语言代码

#### Scenario: Report progress
**GIVEN** 正在执行转录
**WHEN** 推理进度更新
**THEN** 应通过回调函数报告进度 (0.0 - 1.0)
**AND** 进度应单调递增

### Requirement: Text Postprocessing
系统 SHALL 对转录结果进行后处理优化。

#### Scenario: Add Chinese punctuation
**GIVEN** 转录结果是中文文本
**WHEN** 调用 postprocess_text(text, "zh")
**THEN** 应添加中文标点符号（。、，等）

#### Scenario: Capitalize English sentences
**GIVEN** 转录结果是英文文本
**WHEN** 调用 postprocess_text(text, "en")
**THEN** 句子首字母应大写

#### Scenario: Trim whitespace
**GIVEN** 转录结果包含多余空格
**WHEN** 调用 postprocess_text()
**THEN** 应移除首尾空格和多余的中间空格

### Requirement: Performance Optimization
系统 SHALL 支持性能优化选项。

#### Scenario: Use Core ML acceleration on macOS
**GIVEN** 系统是 macOS 且支持 Core ML
**WHEN** 初始化引擎时启用 Core ML
**THEN** 推理应使用 GPU 加速
**AND** 推理速度应比纯 CPU 快 3-5 倍

#### Scenario: Multi-threaded inference
**GIVEN** 系统有多个 CPU 核心
**WHEN** 设置 n_threads 参数
**THEN** Whisper 应使用指定数量的线程
**AND** 推理速度应随线程数增加而提升

#### Scenario: Model quantization
**GIVEN** 使用量化模型 (INT8)
**WHEN** 执行转录
**THEN** 内存占用应减少约 50%
**AND** 推理速度应提升 20-30%
**AND** 准确率下降应 < 2%

### Requirement: Error Handling
系统 SHALL 妥善处理各种错误情况。

#### Scenario: Empty audio data
**GIVEN** 音频数据为空
**WHEN** 调用 transcribe()
**THEN** 应返回 "Audio data is empty" 错误

#### Scenario: Audio too short
**GIVEN** 音频时长 < 0.1 秒
**WHEN** 调用 transcribe()
**THEN** 应返回 "Audio too short" 错误

#### Scenario: Audio too long
**GIVEN** 音频时长 > 10 分钟
**WHEN** 调用 transcribe()
**THEN** 应返回警告并建议分段处理

#### Scenario: Inference timeout
**GIVEN** 推理时间超过 30 秒
**WHEN** 仍未完成
**THEN** 应超时并返回错误
**AND** 释放所有资源

### Requirement: Memory Management
系统 SHALL 有效管理内存资源。

#### Scenario: Release audio data after transcription
**GIVEN** 转录完成
**WHEN** 返回结果
**THEN** 应立即释放音频数据内存

#### Scenario: Model memory footprint
**GIVEN** 加载 base 模型
**WHEN** 引擎运行
**THEN** 内存占用应 < 500MB

#### Scenario: Multiple transcription requests
**GIVEN** 连续多次转录请求
**WHEN** 完成 10 次转录
**THEN** 内存占用应保持稳定
**AND** 不应有内存泄漏

### Requirement: Streaming Transcription
系统 SHALL 支持实时流式转录（预览模式核心功能）。

#### Scenario: Start streaming transcription
**GIVEN** 用户启动预览模式
**WHEN** 调用 start_streaming_transcription()
**THEN** 引擎应启动流式转录管道
**AND** 持续监听音频 chunk 事件
**AND** 返回 StreamingHandle 用于停止

#### Scenario: Process audio chunks
**GIVEN** 流式转录已启动
**WHEN** 接收到 audio-chunk 事件
**THEN** 应将 chunk 加入转录队列
**AND** 异步执行转录
**AND** 不阻塞音频录制线程

#### Scenario: Emit transcription chunks
**GIVEN** Chunk 转录完成
**WHEN** 得到文本结果
**THEN** 应立即发送 transcription-chunk 事件
**AND** 事件 payload 为文本字符串
**AND** 按时间顺序发送（不乱序）

#### Scenario: Stop streaming transcription
**GIVEN** 流式转录正在运行
**WHEN** 调用 stop_streaming_transcription()
**THEN** 应停止接收新 chunk
**AND** 等待队列中的 chunk 处理完成（最多 5 秒）
**AND** 释放所有资源
**AND** 关闭事件监听

#### Scenario: Streaming transcription timeout
**GIVEN** 流式转录运行超过 10 分钟
**WHEN** 达到最大时长
**THEN** 应自动停止转录
**AND** 发送 transcription-timeout 事件

### Requirement: Adaptive Voice Activity Detection (AdaptiveVAD)
系统 SHALL 提供智能停顿检测，自动分割语句。

#### Scenario: Initialize AdaptiveVAD
**GIVEN** 启动流式转录
**WHEN** 创建 AdaptiveVAD 实例
**THEN** 应初始化默认停顿阈值为 0.5 秒
**AND** 设置阈值范围 [0.3s, 0.8s]
**AND** 设置最大超时为 30 秒

#### Scenario: Detect speech
**GIVEN** AdaptiveVAD 正在运行
**WHEN** 音频 RMS 能量 > 阈值 (0.02)
**THEN** 应标记为 "Speaking" 状态
**AND** 更新 last_speech_time

#### Scenario: Detect short pause
**GIVEN** 连续 0.2 秒静音
**WHEN** 静音时长 < min_pause_duration
**THEN** 应返回 ShortPause
**AND** 不触发转录切割

#### Scenario: Detect sentence end
**GIVEN** 连续 0.5 秒静音
**WHEN** 静音时长 >= min_pause_duration
**THEN** 应返回 SentenceEnd
**AND** 触发当前累积 chunk 的转录
**AND** 清空音频缓冲

#### Scenario: Detect session timeout
**GIVEN** 连续 30 秒静音
**WHEN** 静音时长 >= max_pause_duration
**THEN** 应返回 SessionTimeout
**AND** 自动停止流式转录
**AND** 发送 recording-timeout 事件

#### Scenario: Calculate speech density
**GIVEN** 最近 5 秒音频历史
**WHEN** 调用 calculate_speech_density()
**THEN** 应统计语音帧占比
**AND** 返回 0.0 - 1.0 范围值
**AND** 快速语速返回 > 0.7
**AND** 慢速语速返回 < 0.4

#### Scenario: Adjust pause threshold
**GIVEN** 语音密度历史数据
**WHEN** 调用 adjust_threshold()
**THEN** 应计算平均语速 avg_rate
**AND** 快速语速 (rate > 0.7) → 阈值调整到 0.3s
**AND** 慢速语速 (rate < 0.4) → 阈值调整到 0.8s
**AND** 中等语速 → 阈值保持 0.5s

#### Scenario: Maintain speech density history
**GIVEN** AdaptiveVAD 运行中
**WHEN** 每秒更新一次语音密度
**THEN** 应维护 5 秒滑动窗口
**AND** 窗口满时移除最旧数据
**AND** 添加新密度值

#### Scenario: VAD performance
**GIVEN** 实时音频流输入
**WHEN** 执行 VAD 检测
**THEN** 单次检测耗时应 < 5ms
**AND** CPU 占用应 < 3%
**AND** 不影响音频录制

### Requirement: Transcription Queue Management
系统 SHALL 管理转录任务队列，支持并行处理。

#### Scenario: Create transcription queue
**GIVEN** 启动流式转录
**WHEN** 初始化队列
**THEN** 应创建固定大小线程池（2-4 线程）
**AND** 初始化任务队列 (VecDeque)
**AND** 启动 worker 线程

#### Scenario: Push chunk to queue
**GIVEN** VAD 检测到 SentenceEnd
**WHEN** 有累积的音频 chunk
**THEN** 应将 chunk 加入队列
**AND** 包含元数据（时间戳、chunk_id）
**AND** 如果队列满（> 10 个），等待或丢弃最旧的

#### Scenario: Worker process chunk
**GIVEN** 队列中有待处理 chunk
**WHEN** worker 线程空闲
**THEN** 应从队列取出 chunk
**AND** 调用 engine.transcribe(chunk)
**AND** 处理完成后发送 transcription-chunk 事件
**AND** 释放 chunk 内存

#### Scenario: Parallel transcription
**GIVEN** 队列中有多个 chunk
**WHEN** 多个 worker 同时工作
**THEN** 应并行处理多个 chunk
**AND** 按 chunk_id 顺序发送结果（可能需要排序缓冲）
**AND** 总吞吐量提升 2-3 倍

#### Scenario: Queue overflow handling
**GIVEN** 转录速度 < 音频生成速度
**WHEN** 队列达到上限
**THEN** 应记录警告日志
**AND** 丢弃最旧的 chunk 或限流
**AND** 前端显示 "转录延迟" 提示

### Requirement: Real-time Text Feedback
系统 SHALL 提供实时文本反馈和状态更新。

#### Scenario: Send text chunk event
**GIVEN** Chunk 转录完成
**WHEN** 得到文本结果 "你好世界"
**THEN** 应发送 Tauri Event
**AND** 事件名称为 transcription-chunk
**AND** payload 为字符串 "你好世界"
**AND** 延迟应 < 100ms

#### Scenario: Frontend append text
**GIVEN** 前端监听 transcription-chunk
**WHEN** 收到事件
**THEN** 应将文本追加到显示区域
**AND** 使用 " " 空格分隔多个 chunk
**AND** 自动滚动到最新文本

#### Scenario: Empty transcription handling
**GIVEN** Chunk 转录结果为空
**WHEN** Whisper 返回空字符串
**THEN** 不应发送事件
**AND** 不记录为错误
**AND** 继续处理下一个 chunk

#### Scenario: Transcription latency
**GIVEN** 从音频录制到文本显示
**WHEN** 测量端到端延迟
**THEN** 平均延迟应 < 500ms
**AND** P95 延迟应 < 1 秒
**AND** 使用 base 模型 + Core ML
