# settings-ui Specification

## Purpose
TBD - created by archiving change add-settings-page. Update Purpose after archive.
## Requirements
### Requirement: Main Window Layout
主窗口 SHALL 采用左侧导航+右侧内容的布局结构。

#### Scenario: Window Structure
**Given** 用户打开主窗口
**When** 窗口加载完成
**Then** 应显示左侧导航栏(宽度 200px)
**And** 右侧内容区域占据剩余宽度
**And** 窗口最小尺寸应为 900x600px

#### Scenario: Navigation Menu
**Given** 用户查看左侧导航
**When** 应用启动完成
**Then** 应显示三个导航项:
- 📱 首页
- 📝 笔记
- ⚙️ 设置
**And** 默认选中"首页"
**And** 选中项应有视觉高亮

### Requirement: Home Page
首页 SHALL 显示快捷键提示和今日转录历史。

#### Scenario: Shortcut Tip Card
**Given** 用户在首页
**When** 页面加载完成
**Then** 应显示快捷键提示卡片
**And** 卡片标题应为"按住 ^ Ctrl + ⌥ Opt 在任何应用中听写"
**And** 卡片应包含使用说明文字
**And** 可选显示[查看使用方法]按钮

#### Scenario: Today's Transcription List
**Given** 用户在首页
**When** 页面加载完成
**Then** 应显示"今天"标题
**And** 应显示今日所有转录记录
**And** 每条记录应包含时间戳和文本内容
**And** 记录应按时间倒序排列

#### Scenario: Transcription Item Display
**Given** 转录历史列表中有记录
**When** 用户查看某条记录
**Then** 应显示12px灰色时间戳(如"09:21 PM")
**And** 应显示14px黑色转录文本
**And** 文本应支持多行显示,行高1.5
**And** 记录应有白色卡片背景和8px圆角

### Requirement: Notes Page Placeholder
笔记页面 SHALL 在一期显示开发中占位状态。

#### Scenario: Empty State Display
**Given** 用户点击笔记导航
**When** 笔记页面加载
**Then** 应显示垂直水平居中的空状态
**And** 应显示64px大小的📝图标
**And** 应显示"此功能正在开发中,敬请期待"提示文字
**And** 文字应为16px灰色

### Requirement: Settings Dialog Trigger
点击设置导航 SHALL 打开模态设置弹窗。

#### Scenario: Open Settings
**Given** 用户在主窗口任意页面
**When** 点击"设置"导航项
**Then** 应显示模态设置弹窗
**And** 应显示半透明黑色背景遮罩
**And** 弹窗应居中显示
**And** 弹窗尺寸应为800x600px

#### Scenario: Close Settings
**Given** 设置弹窗已打开
**When** 用户点击[×]按钮或按下Escape键
**Then** 设置弹窗应关闭
**And** 背景遮罩应移除
**And** 焦点应返回主窗口

### Requirement: Settings Dialog Structure
设置弹窗 SHALL 包含左侧导航和右侧内容面板。

#### Scenario: Dialog Layout
**Given** 设置弹窗已打开
**When** 用户查看弹窗
**Then** 顶部应显示"设置"标题和[×]关闭按钮
**And** 左侧应显示200px宽的设置导航
**And** 右侧应显示对应的设置内容
**And** 底部应显示版本号(如"聆码 v1.0.0")

#### Scenario: Settings Navigation
**Given** 用户在设置弹窗
**When** 查看左侧导航
**Then** 应显示三个导航项:
- ⚙️ 通用设置
- 💻 系统设置
- 🤖 模型设置
**And** 默认选中"通用设置"
**And** 选中项应有视觉高亮

### Requirement: General Settings Panel
通用设置面板 SHALL 提供快捷键、麦克风和语言配置。

#### Scenario: Keyboard Shortcut Setting
**Given** 用户在通用设置面板
**When** 查看键盘快捷键设置项
**Then** 应显示"键盘快捷键"标签
**And** 应显示当前快捷键值(如"按住 ^ Ctrl + ⌥ Opt 并说话")
**And** 应显示[更改]按钮
**And** 可选显示[了解更多 →]链接

#### Scenario: Microphone Setting
**Given** 用户在通用设置面板
**When** 查看麦克风设置项
**Then** 应显示"麦克风"标签
**And** 应显示当前麦克风值(如"自动检测 (CF2000)")
**And** 应显示[更改]按钮

#### Scenario: Language Setting
**Given** 用户在通用设置面板
**When** 查看语言设置项
**Then** 应显示"语言"标签
**And** 应显示当前语言值(如"中文(简体)")
**And** 应显示[更改]按钮

#### Scenario: Open Language Selector
**Given** 用户在通用设置面板
**When** 点击语言设置的[更改]按钮
**Then** 应打开语言选择弹窗
**And** 弹窗应覆盖在设置弹窗之上

### Requirement: Language Selector Dialog
语言选择弹窗 SHALL 提供语言选择和Auto-detect开关。

#### Scenario: Language Selector Layout
**Given** 语言选择弹窗已打开
**When** 用户查看弹窗
**Then** 顶部应显示"Languages"标题和Auto-detect开关
**And** 应显示搜索框(占位符"Search for any languages")
**And** 左侧应显示可选语言卡片网格
**And** 右侧应显示"Selected"区域
**And** 底部应显示[保存并关闭]按钮

#### Scenario: Available Languages
**Given** 用户在语言选择弹窗
**When** 查看可选语言列表
**Then** 应显示以下语言选项:
- 🇨🇳 Mandarin (Simplified) / 中文(简体)
- 🇺🇸 English / 英语
**And** 每个选项应为卡片样式
**And** 卡片应显示国旗图标和语言名称

#### Scenario: Select Language
**Given** 用户在语言选择弹窗
**When** 点击某个语言卡片
**Then** 该语言应添加到右侧"Selected"列表
**And** 列表中应显示该语言和[—]移除按钮

#### Scenario: Save Language Selection
**Given** 用户已选择语言
**When** 点击[保存并关闭]按钮
**Then** 选择的语言应保存到设置
**And** 语言选择弹窗应关闭
**And** 通用设置面板的语言值应更新

### Requirement: System Settings Panel
系统设置面板 SHALL 提供开机自启和Dock显示配置。

#### Scenario: App Settings Section
**Given** 用户在系统设置面板
**When** 查看App settings分组
**Then** 应显示"App settings"标题
**And** 应包含两个Toggle开关设置项

#### Scenario: Auto Launch Setting
**Given** 用户在系统设置面板
**When** 查看开机自动启动设置
**Then** 应显示"开机自动启动"中文标签
**And** 应显示"Launch app at login"英文副标签
**And** 应显示Toggle开关
**And** 默认状态应为关闭(灰色)

#### Scenario: Toggle Auto Launch
**Given** 用户在系统设置面板
**When** 点击开机自启Toggle开关
**Then** 开关状态应切换(开启/关闭)
**And** 开启时应显示绿色背景
**And** 设置应立即保存
**And** 应调用系统API更新登录项

#### Scenario: Dock Visibility Setting
**Given** 用户在系统设置面板
**When** 查看Dock显示设置
**Then** 应显示"在 Dock 中显示"中文标签
**And** 应显示"Show app in dock"英文副标签
**And** 应显示Toggle开关
**And** 默认状态应为开启(绿色)

#### Scenario: Toggle Dock Visibility
**Given** 用户在系统设置面板
**When** 点击Dock显示Toggle开关
**Then** 开关状态应切换
**And** 设置应立即保存
**And** 应调用Tauri API更新Dock显示状态

### Requirement: Model Settings Panel
模型设置面板 SHALL 提供Whisper模型选择和管理功能。

#### Scenario: Model Selection Section
**Given** 用户在模型设置面板
**When** 查看Whisper模型选择区域
**Then** 应显示"Whisper 模型选择"标题
**And** 应显示四个模型选项(Radio Group)

#### Scenario: Model Options Display
**Given** 用户在模型设置面板
**When** 查看模型选项列表
**Then** 应显示以下四个选项:
- ○ Base (74MB, 快速, 一般精度) 推荐
- ○ Small (244MB, 较快, 较高精度)
- ○ Medium (769MB, 较慢, 高精度)
- ○ Large (1.5GB, 慢, 最高精度)
**And** 每个选项应显示Radio按钮、名称、大小和性能描述
**And** Base选项应显示"推荐"标签

#### Scenario: Model Download Status - Downloaded
**Given** 某个模型已下载
**When** 用户查看该模型选项
**Then** 应显示绿色✓标记
**And** 应显示"已下载"文字

#### Scenario: Model Download Status - Not Downloaded
**Given** 某个模型未下载
**When** 用户查看该模型选项
**Then** 应显示灰色[下载]按钮
**And** 鼠标悬停时按钮应变深灰

#### Scenario: Download Model
**Given** 某个模型未下载
**When** 用户点击[下载]按钮
**Then** 按钮应替换为进度条
**And** 应显示下载百分比(如"45%")
**And** 应显示[取消]按钮
**And** 下载完成后应显示✓和"已下载"

#### Scenario: Download Failed
**Given** 模型下载过程中发生错误
**When** 下载失败
**Then** 应显示红色✗标记
**And** 应显示"下载失败"文字
**And** 应显示[重试]按钮
**And** 应显示错误Toast提示

#### Scenario: Select Model
**Given** 用户在模型设置面板
**When** 点击某个模型的Radio按钮
**Then** 该模型应被选中
**And** 其他模型应取消选中
**And** 选择应立即保存到设置
**And** 如果模型未下载,应提示需要先下载

#### Scenario: Downloaded Models List
**Given** 用户在模型设置面板
**When** 查看"已下载的模型"区域
**Then** 应显示所有已下载模型的列表
**And** 每个模型应显示名称和文件大小
**And** 每个模型应有[删除]按钮

#### Scenario: Delete Model
**Given** 某个模型已下载
**When** 用户点击该模型的[删除]按钮
**Then** 应弹出确认对话框
**And** 用户确认后应删除模型文件
**And** 该模型应从列表中移除
**And** 该模型选项状态应变为"未下载"
**And** 应显示"模型已删除"Toast提示

### Requirement: Settings Persistence
所有设置变更 SHALL 立即保存到数据库。

#### Scenario: Save Setting
**Given** 用户修改了任何设置项
**When** 设置值发生变化
**Then** 应调用Tauri命令保存到SQLite
**And** 保存成功后应更新Zustand store
**And** 保存成功应显示"设置已保存"Toast
**And** 保存失败应回滚值并显示错误Toast

### Requirement: Performance
主窗口和设置系统 SHALL 满足性能要求。

#### Scenario: Window Initial Load
**Given** 用户打开主窗口
**When** 窗口首次加载
**Then** 页面应在300ms内完成渲染
**And** 首页数据应在200ms内加载完成

#### Scenario: Settings Dialog Open
**Given** 用户点击设置导航
**When** 打开设置弹窗
**Then** 弹窗应在100ms内显示
**And** 弹窗动画应流畅(60fps)

#### Scenario: Settings Tab Switch
**Given** 用户在设置弹窗
**When** 切换设置标签
**Then** 标签切换应在50ms内完成
**And** 面板切换应有淡入淡出动画

### Requirement: Accessibility
主窗口和设置 SHALL 支持键盘导航和屏幕阅读器。

#### Scenario: Keyboard Navigation
**Given** 用户使用键盘操作
**When** 按下Tab键
**Then** 焦点应在可交互元素间移动
**And** 当前焦点应有明显的视觉指示
**And** 按下Enter应激活按钮和Toggle
**And** 按下Escape应关闭弹窗

#### Scenario: ARIA Labels
**Given** 用户使用屏幕阅读器
**When** 导航到任何交互元素
**Then** 元素应有适当的aria-label
**And** Toggle开关应有aria-checked属性
**And** 弹窗应有aria-modal属性
**And** 状态变化应有语音反馈

### Requirement: Error Handling
系统 SHALL 妥善处理各种错误情况。

#### Scenario: Settings Load Failure
**Given** 应用启动
**When** 从数据库加载设置失败
**Then** 应使用默认设置值
**And** 应显示错误Toast
**And** 应提供[重新加载]按钮

#### Scenario: Settings Save Failure
**Given** 用户修改设置
**When** 保存到数据库失败
**Then** 设置值应回滚到原值
**And** 应显示错误Toast:"保存失败: {error}"
**And** 应提供[重试]选项

#### Scenario: Model Download Error
**Given** 用户下载模型
**When** 网络连接失败
**Then** 下载应停止
**And** 应显示错误Toast
**And** 应保留已下载的部分(用于断点续传)
**And** 应显示[重试下载]按钮

### Requirement: UI Styling
界面 SHALL 遵循简洁现代的设计风格。

#### Scenario: Card Style
**Given** 页面包含卡片元素
**When** 用户查看卡片
**Then** 卡片应有白色或浅灰背景
**And** 圆角应为8-12px
**And** 应有轻微阴影
**And** 内边距应为16-24px

#### Scenario: Toggle Switch Style
**Given** 页面包含Toggle开关
**When** 用户查看开关
**Then** 关闭状态应为灰色背景
**And** 开启状态应为绿色背景(#10B981)
**And** 滑块应为白色
**And** 尺寸应为44x24px
**And** 状态切换应有200ms动画

#### Scenario: Button Style
**Given** 页面包含按钮
**When** 用户查看或交互
**Then** 主要按钮应有明显的背景色
**And** 次要按钮应有灰色背景
**And** 鼠标悬停应有颜色变化
**And** 圆角应为6-8px

#### Scenario: Typography
**Given** 页面包含文本
**When** 用户查看文本
**Then** 标题应使用加粗字体
**And** 正文应使用常规字体
**And** 行高应为1.5-1.6
**And** 颜色对比度应满足WCAG AA标准

