# model-management Specification

## Purpose
定义 Whisper 模型文件的下载、存储、验证和管理规范。

## ADDED Requirements

### Requirement: Model Download
系统 SHALL 提供从远程服务器下载模型文件的功能。

#### Scenario: Download model successfully
**GIVEN** 网络连接正常
**AND** 模型 URL 可访问
**WHEN** 用户选择下载 "base" 模型
**THEN** 应从 Hugging Face Mirror 下载模型文件
**AND** 应显示下载进度 (0-100%)
**AND** 下载速度应显示 (MB/s)
**AND** 下载完成后应显示 "下载完成" 提示

#### Scenario: Download with progress tracking
**GIVEN** 正在下载模型
**WHEN** 下载进度更新
**THEN** 应通过 Tauri Event 发送进度事件
**AND** 事件数据应包含: { progress: number, downloaded: number, total: number, speed: number }

#### Scenario: Resume interrupted download
**GIVEN** 模型下载中断
**AND** 部分文件已下载
**WHEN** 用户重新下载
**THEN** 应支持断点续传
**AND** 从已下载的位置继续

#### Scenario: Download failure with network error
**GIVEN** 网络连接中断
**WHEN** 正在下载模型
**THEN** 应停止下载
**AND** 保留已下载的部分文件
**AND** 显示 "网络错误，请稍后重试" 提示
**AND** 提供 [重试] 按钮

### Requirement: Model Verification
系统 SHALL 验证下载的模型文件完整性。

#### Scenario: Verify with SHA256 checksum
**GIVEN** 模型文件下载完成
**WHEN** 执行验证
**THEN** 应计算文件 SHA256 哈希值
**AND** 应与预期哈希值比对
**AND** 哈希匹配时标记为 "验证成功"

#### Scenario: Model file corrupted
**GIVEN** 模型文件已损坏
**WHEN** 执行 SHA256 验证
**THEN** 应返回 "Model file corrupted" 错误
**AND** 自动删除损坏的文件
**AND** 提示用户重新下载

### Requirement: Model Storage
系统 SHALL 在指定位置存储模型文件。

#### Scenario: Store model in app data directory
**GIVEN** 模型下载完成
**WHEN** 保存文件
**THEN** 应存储在 ~/Library/Application Support/com.lingcode.app/models/
**AND** 文件名应为 ggml-{model_name}.bin
**AND** 目录不存在时应自动创建

#### Scenario: Check available disk space
**GIVEN** 准备下载大型模型 (medium, 1.5GB)
**WHEN** 检查磁盘空间
**THEN** 应验证可用空间 > 模型大小 * 1.5
**AND** 空间不足时显示警告
**AND** 提示用户释放空间或选择小型模型

### Requirement: Model List Management
系统 SHALL 提供模型列表查询和管理功能。

#### Scenario: List all available models
**GIVEN** 应用启动
**WHEN** 调用 list_models()
**THEN** 应返回模型列表包含:
- tiny (75MB, 快速, 一般精度)
- base (142MB, 平衡, 推荐)
- small (466MB, 较高精度)
- medium (1.5GB, 最高精度)
**AND** 每个模型应标注下载状态 (已下载/未下载)

#### Scenario: Check if model is downloaded
**GIVEN** 某个模型已下载
**WHEN** 调用 is_model_downloaded("base")
**THEN** 应返回 true
**AND** 文件应存在且完整

#### Scenario: Get model file size
**GIVEN** 查询模型信息
**WHEN** 调用 get_model_info("base")
**THEN** 应返回:
- name: "base"
- size: 142000000 (bytes)
- size_mb: 142
- downloaded: true/false
- file_path: "/path/to/ggml-base.bin"

### Requirement: Model Deletion
系统 SHALL 允许用户删除已下载的模型。

#### Scenario: Delete model successfully
**GIVEN** 模型 "small" 已下载
**WHEN** 用户点击删除按钮
**THEN** 应弹出确认对话框 "确定要删除 small 模型吗?"
**AND** 用户确认后删除文件
**AND** 释放磁盘空间
**AND** 显示 "模型已删除" 提示

#### Scenario: Cannot delete active model
**GIVEN** 当前使用 "base" 模型
**WHEN** 用户尝试删除 "base" 模型
**THEN** 应显示警告 "当前正在使用该模型，无法删除"
**AND** 删除操作应被阻止

### Requirement: Model Switching
系统 SHALL 支持运行时切换不同模型。

#### Scenario: Switch to another model
**GIVEN** 当前使用 "base" 模型
**AND** "small" 模型已下载
**WHEN** 用户选择切换到 "small"
**THEN** 应卸载 "base" 模型
**AND** 加载 "small" 模型
**AND** 下次转录使用新模型

#### Scenario: Switch to undownloaded model
**GIVEN** "medium" 模型未下载
**WHEN** 用户尝试切换到 "medium"
**THEN** 应提示 "模型未下载，是否立即下载?"
**AND** 提供 [下载] 和 [取消] 按钮

### Requirement: Model Update Check
系统 SHALL 检查模型更新（可选功能）。

#### Scenario: Check for model updates
**GIVEN** 本地模型版本可能过时
**WHEN** 调用 check_model_updates()
**THEN** 应请求远程模型列表
**AND** 比对本地和远程 SHA256
**AND** 发现更新时提示用户

#### Scenario: No update available
**GIVEN** 本地模型是最新版本
**WHEN** 检查更新
**THEN** 应显示 "所有模型已是最新版本"

### Requirement: Download Cancellation
系统 SHALL 允许用户取消正在进行的下载。

#### Scenario: Cancel download mid-way
**GIVEN** 正在下载模型
**AND** 已下载 50%
**WHEN** 用户点击 [取消下载]
**THEN** 应立即停止下载
**AND** 删除未完成的文件
**AND** 释放网络和磁盘资源
**AND** 显示 "下载已取消"

### Requirement: Multiple Model Support
系统 SHALL 允许同时管理多个模型文件。

#### Scenario: Download multiple models concurrently
**GIVEN** 用户同时下载 base 和 small
**WHEN** 两个下载任务并行
**THEN** 应显示两个独立的进度条
**AND** 每个下载应独立管理

#### Scenario: Disk space management
**GIVEN** 已下载多个模型
**WHEN** 查看存储占用
**THEN** 应显示总占用空间
**AND** 显示每个模型占用大小
**AND** 提供批量删除选项

### Requirement: Model Mirror Source
系统 SHALL 支持从不同镜像源下载模型。

#### Scenario: Use Hugging Face Mirror
**GIVEN** 在中国大陆地区
**WHEN** 下载模型
**THEN** 应默认使用 hf-mirror.com 镜像
**AND** 下载速度应 > 1MB/s

#### Scenario: Fallback to official source
**GIVEN** 镜像源不可用
**WHEN** 下载失败
**THEN** 应自动切换到官方 huggingface.co
**AND** 显示 "正在切换下载源..."

### Requirement: Error Handling
系统 SHALL 妥善处理模型管理中的各种错误。

#### Scenario: Disk write permission denied
**GIVEN** 应用无磁盘写入权限
**WHEN** 尝试保存模型文件
**THEN** 应返回 "Permission denied" 错误
**AND** 提示用户检查权限设置

#### Scenario: Download URL invalid
**GIVEN** 模型 URL 无效或已失效
**WHEN** 尝试下载
**THEN** 应返回 "Invalid URL" 错误
**AND** 提供手动输入 URL 的选项（高级）

#### Scenario: File system full
**GIVEN** 磁盘空间不足
**WHEN** 下载到 90% 时空间耗尽
**THEN** 应立即停止下载
**AND** 删除未完成文件
**AND** 显示 "磁盘空间不足" 错误
**AND** 建议删除其他文件或选择小型模型

### Requirement: Logging and Monitoring
系统 SHALL 记录模型管理操作日志。

#### Scenario: Log download activity
**GIVEN** 下载模型
**WHEN** 下载完成或失败
**THEN** 应记录日志包含:
- 时间戳
- 模型名称
- 下载状态 (成功/失败)
- 下载时长
- 错误信息（如果失败）

#### Scenario: Monitor download metrics
**GIVEN** 正在下载
**WHEN** 监控下载指标
**THEN** 应记录:
- 平均下载速度
- 总耗时
- 重试次数
- 网络状态
