# Spec: Settings UI

## Capability
settings-ui

## Overview
设置页面用户界面,提供应用程序的配置和管理功能,包括通用设置、语音识别配置、快捷键自定义、历史记录查看和应用信息展示。

## ADDED Requirements

### Requirement: Layout Structure
设置页面 SHALL 使用左右分栏布局结构。

#### Scenario: Desktop Layout
**Given** 用户打开主窗口
**When** 窗口宽度大于 768px
**Then** 页面应显示为左侧导航+右侧内容的两栏布局
**And** 左侧导航宽度应为 200px
**And** 右侧内容区域应占据剩余宽度

#### Scenario: Narrow Window
**Given** 用户调整窗口大小
**When** 窗口宽度小于 768px
**Then** 导航应收起为图标模式或顶部标签栏
**And** 内容区域应占据全宽

### Requirement: Navigation Tabs
设置页面 SHALL 提供五个主要导航标签。

#### Scenario: Tab List
**Given** 用户在设置页面
**When** 查看左侧导航
**Then** 应显示以下标签按钮:
- "通用设置" (General)
- "语音识别" (Voice)
- "快捷键" (Shortcuts)
- "历史记录" (History)
- "关于" (About)
**And** 当前激活的标签应有视觉高亮
**And** 每个标签应有对应的图标

#### Scenario: Tab Switching
**Given** 用户在某个设置标签页
**When** 点击另一个标签按钮
**Then** 右侧内容区域应切换到对应的设置面板
**And** 标签切换应在 50ms 内完成
**And** 新面板应有淡入动画效果

### Requirement: General Settings Panel
通用设置面板 SHALL 提供基础应用配置选项。

#### Scenario: Language Selection
**Given** 用户在通用设置面板
**When** 查看语言设置项
**Then** 应显示一个下拉选择框
**And** 默认值应为"中文"
**And** 其他语言选项应暂时禁用(灰色显示)

#### Scenario: Theme Selection
**Given** 用户在通用设置面板
**When** 查看主题设置项
**Then** 应显示三个单选按钮: "浅色"、"深色"、"自动"
**And** 用户选择主题后应立即应用(无需重启)
**And** "自动"模式应跟随系统主题

#### Scenario: Auto Start Toggle
**Given** 用户在通用设置面板
**When** 切换"开机自动启动"开关
**Then** 设置应保存到数据库
**And** 应调用系统API注册/取消自动启动
**And** 操作成功后应显示 Toast 提示

#### Scenario: Notification Toggle
**Given** 用户在通用设置面板
**When** 切换"显示通知"开关
**Then** 设置应立即保存
**And** 后续的录音完成等事件应根据此设置决定是否显示通知

### Requirement: Voice Settings Panel
语音设置面板 SHALL 提供 Whisper 模型配置选项。

#### Scenario: Model Selection
**Given** 用户在语音设置面板
**When** 查看模型选择区域
**Then** 应显示四个模型选项:
- Base (74MB, 快速, 一般精度) - 标记为"推荐"
- Small (244MB, 较快, 较高精度)
- Medium (769MB, 较慢, 高精度)
- Large (1.5GB, 慢, 最高精度)
**And** 每个选项应显示文件大小和性能特点
**And** 已下载的模型应有✓标记

#### Scenario: Model Download
**Given** 用户选择了一个未下载的模型
**When** 点击该模型的下载按钮
**Then** 应开始下载模型文件
**And** 应显示下载进度条和百分比
**And** 下载完成后应自动切换到该模型
**And** 下载失败应显示错误提示和重试按钮

### Requirement: Shortcut Settings Panel
快捷键设置面板 SHALL 提供全局快捷键自定义功能。

#### Scenario: Shortcut Display
**Given** 用户在快捷键设置面板
**When** 查看录音快捷键设置项
**Then** 应显示当前的快捷键组合(如 "Cmd+Shift+S")
**And** 快捷键应显示在一个可点击的输入框中

#### Scenario: Shortcut Recording
**Given** 用户点击了快捷键输入框
**When** 输入框进入"等待按键"状态
**Then** 输入框应显示"请按下快捷键..."提示
**And** 应监听键盘按键事件
**And** 当用户按下有效的快捷键组合时,应显示该组合
**And** 应自动退出"等待按键"状态

#### Scenario: Shortcut Validation
**Given** 用户录制了新的快捷键
**When** 尝试保存快捷键
**Then** 应验证快捷键格式是否合法
**And** 应检测是否与系统快捷键冲突
**And** 如果冲突,应显示警告信息并阻止保存
**And** 如果合法,应保存到数据库并立即生效

### Requirement: History Panel
历史记录面板 SHALL 提供转录记录的查看和管理功能。

#### Scenario: History List Display
**Given** 用户在历史记录面板
**When** 面板加载完成
**Then** 应显示最近的转录记录列表(按时间倒序)
**And** 每条记录应显示: 时间、文本预览(前50字)、音频时长
**And** 列表应支持滚动加载更多(虚拟滚动)
**And** 每页应显示 20 条记录

#### Scenario: Search History
**Given** 用户在历史记录面板
**When** 在搜索框中输入关键词
**Then** 列表应实时过滤显示包含关键词的记录
**And** 搜索应有 300ms 防抖
**And** 搜索结果应高亮显示匹配的关键词

#### Scenario: View Record Details
**Given** 用户在历史记录列表
**When** 点击某条记录
**Then** 应展开显示该记录的完整转录文本
**And** 应显示详细信息: 完整时间戳、音频时长、使用的模型
**And** 再次点击应收起详情

#### Scenario: Copy Record
**Given** 用户查看某条历史记录
**When** 点击"复制"按钮
**Then** 转录文本应复制到剪贴板
**And** 应显示"已复制"的 Toast 提示

#### Scenario: Delete Single Record
**Given** 用户查看某条历史记录
**When** 点击"删除"按钮
**Then** 应弹出确认对话框
**And** 用户确认后应从数据库删除该记录
**And** 列表应更新移除该记录
**And** 应显示"已删除"的 Toast 提示

#### Scenario: Batch Delete
**Given** 用户选中了多条历史记录
**When** 点击"删除选中"按钮
**Then** 应弹出确认对话框,显示选中的记录数量
**And** 用户确认后应批量删除所有选中记录
**And** 列表应更新
**And** 应显示"已删除 N 条记录"的 Toast 提示

#### Scenario: Clear All History
**Given** 用户在历史记录面板
**When** 点击"清空所有"按钮
**Then** 应弹出二次确认对话框,警告此操作不可撤销
**And** 用户确认后应删除所有历史记录
**And** 列表应显示为空状态
**And** 应显示"所有记录已清空"的 Toast 提示

### Requirement: About Panel
关于面板 SHALL 显示应用程序的基本信息。

#### Scenario: Application Info
**Given** 用户在关于面板
**When** 查看面板内容
**Then** 应显示应用图标
**And** 应显示应用名称"聆码 Lingcode"
**And** 应显示当前版本号(如 "v1.0.0")
**And** 应显示应用描述"跨应用语音听写工具"

#### Scenario: External Links
**Given** 用户在关于面板
**When** 点击 GitHub 链接
**Then** 应在默认浏览器中打开项目 GitHub 页面
**And** 链接应有悬停效果提示

#### Scenario: License Information
**Given** 用户在关于面板
**When** 查看许可证信息
**Then** 应显示开源许可证类型(MIT / Apache 2.0)
**And** 应提供许可证详情的链接

### Requirement: Save and Reset Actions
设置页面 SHALL 提供保存和重置功能。

#### Scenario: Save Settings
**Given** 用户修改了任何设置项
**When** 点击底部的"保存"按钮
**Then** 所有修改的设置应保存到数据库
**And** 应用应立即应用新设置
**And** 应显示"设置已保存"的 Toast 提示
**And** 保存过程应在 200ms 内完成

#### Scenario: Reset to Default
**Given** 用户在设置页面
**When** 点击"重置为默认"按钮
**Then** 应弹出确认对话框
**And** 用户确认后应将所有设置恢复为默认值
**And** UI 应更新显示默认值
**And** 应显示"设置已重置"的 Toast 提示

### Requirement: Error Handling
设置页面 SHALL 妥善处理各种错误情况。

#### Scenario: Settings Load Failure
**Given** 应用启动时加载设置
**When** 数据库读取失败
**Then** 应使用默认设置值
**And** 应显示错误 Toast 提示用户
**And** 应提供"重新加载"按钮

#### Scenario: Settings Save Failure
**Given** 用户修改了设置并点击保存
**When** 数据库写入失败
**Then** 应保持原有设置值(回滚变更)
**And** 应显示错误 Toast 提示
**And** 应提供"重试"按钮

#### Scenario: Network Error
**Given** 用户尝试下载模型
**When** 网络连接失败
**Then** 应停止下载并显示错误信息
**And** 应提供"重试下载"按钮
**And** 已下载的部分应保留用于断点续传

### Requirement: Performance
设置页面 SHALL 满足性能要求以提供流畅体验。

#### Scenario: Initial Load Time
**Given** 用户打开主窗口
**When** 设置页面首次加载
**Then** 页面应在 300ms 内完成渲染
**And** 设置数据应在 200ms 内加载完成

#### Scenario: Tab Switch Performance
**Given** 用户在设置页面
**When** 切换到不同的设置标签
**Then** 标签切换应在 50ms 内完成
**And** 新面板的内容应立即显示

#### Scenario: Search Performance
**Given** 用户在历史记录面板搜索
**When** 输入搜索关键词
**Then** 搜索结果应在 100ms 内返回
**And** 应使用 300ms 防抖优化输入体验

### Requirement: Accessibility
设置页面 SHALL 符合可访问性标准。

#### Scenario: Keyboard Navigation
**Given** 用户使用键盘操作
**When** 按下 Tab 键
**Then** 焦点应在设置项之间按逻辑顺序移动
**And** 当前焦点元素应有明显的视觉指示
**And** Enter 键应激活按钮和切换开关

#### Scenario: Screen Reader Support
**Given** 用户使用屏幕阅读器
**When** 导航到任何设置项
**Then** 应读出设置项的标签和当前值
**And** 交互元素应有适当的 ARIA 标签
**And** 状态变化应有语音反馈

### Requirement: Theme Support
设置页面 SHALL 支持浅色和深色主题。

#### Scenario: Light Theme
**Given** 用户选择浅色主题
**When** 主题应用后
**Then** 所有 UI 元素应使用浅色配色方案
**And** 文本对比度应满足 WCAG AA 标准
**And** 背景应为浅色系

#### Scenario: Dark Theme
**Given** 用户选择深色主题
**When** 主题应用后
**Then** 所有 UI 元素应使用深色配色方案
**And** 文本对比度应满足 WCAG AA 标准
**And** 背景应为深色系,减少眼睛疲劳

#### Scenario: Auto Theme
**Given** 用户选择自动主题
**When** 系统主题发生变化
**Then** 应用主题应自动跟随系统主题
**And** 主题切换应平滑过渡(200ms)
