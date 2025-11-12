# Implementation Tasks

## 1. 准备工作

- [ ] 1.1 确认所需的 Tauri commands 都已实现
  - `get_setting`, `set_setting` - 设置读写
  - `get_recent_transcriptions` - 获取转录历史
  - `list_audio_devices` - 获取麦克风列表
  - `download_model`, `delete_model` - 模型管理
  - `set_auto_launch`, `get_auto_launch` - 开机自启
- [ ] 1.2 确认 settingsStore 和 historyStore 的功能完整性
- [ ] 1.3 准备图标资源(导航图标: 📱 📝 ⚙️,设置图标: ⚙️ 💻 🤖)

## 2. 主窗口基础架构

- [ ] 2.1 重写 `App.tsx` - 从演示界面改为主窗口布局
- [ ] 2.2 创建 `MainWindow.tsx` - 主窗口容器组件
- [ ] 2.3 创建 `Sidebar.tsx` - 左侧导航栏组件
- [ ] 2.4 实现导航项组件和状态管理(useState/Zustand)
- [ ] 2.5 实现页面切换逻辑

## 3. 首页 (HomePage)

- [ ] 3.1 创建 `src/windows/main/HomePage.tsx`
- [ ] 3.2 实现快捷键提示卡片组件
  - 卡片容器和样式
  - 标题和说明文字
  - 可选的[查看使用方法]按钮
- [ ] 3.3 实现转录历史列表组件
  - "今天"标题
  - TranscriptionList 组件
  - TranscriptionItem 组件(时间戳+内容)
- [ ] 3.4 连接 historyStore 加载今日历史
- [ ] 3.5 实现列表滚动和样式

## 4. 笔记页面 (NotesPage)

- [ ] 4.1 创建 `src/windows/main/NotesPage.tsx`
- [ ] 4.2 创建 EmptyState 组件
  - 垂直水平居中布局
  - 64px 📝 图标
  - "此功能正在开发中,敬请期待"文字
- [ ] 4.3 应用淡灰色调样式

## 5. 设置弹窗基础架构

- [ ] 5.1 创建 `src/windows/main/SettingsDialog.tsx` - 模态对话框组件
- [ ] 5.2 实现弹窗打开/关闭逻辑
- [ ] 5.3 实现背景遮罩和点击关闭
- [ ] 5.4 实现 Escape 键关闭
- [ ] 5.5 创建弹窗头部(标题 + 关闭按钮)
- [ ] 5.6 创建弹窗主体(左侧导航 + 右侧内容)
- [ ] 5.7 创建弹窗底部(版本号显示)
- [ ] 5.8 实现设置标签页切换逻辑

## 6. 通用设置面板 (GeneralSettings)

- [ ] 6.1 创建 `settings/GeneralSettings.tsx` 组件
- [ ] 6.2 创建 SettingItem 通用组件(标签+值+按钮)
- [ ] 6.3 实现键盘快捷键设置项
  - 显示当前快捷键值
  - [更改]按钮
  - 可选[了解更多]链接
- [ ] 6.4 实现麦克风设置项
  - 显示当前麦克风
  - [更改]按钮(暂时占位,二期实现)
- [ ] 6.5 实现语言设置项
  - 显示当前语言
  - [更改]按钮打开语言选择弹窗
- [ ] 6.6 连接 settingsStore 实现数据绑定

## 7. 语言选择弹窗 (LanguageSelector)

- [ ] 7.1 创建 `settings/LanguageSelector.tsx` 弹窗组件
- [ ] 7.2 实现弹窗头部(标题 + Auto-detect开关)
- [ ] 7.3 实现搜索框(一期可占位不实现搜索)
- [ ] 7.4 实现左侧语言卡片网格
  - 中文(简体)卡片 🇨🇳
  - 英语卡片 🇺🇸
- [ ] 7.5 实现右侧"Selected"列表
- [ ] 7.6 实现语言选择/移除逻辑
- [ ] 7.7 实现[保存并关闭]按钮
- [ ] 7.8 连接 settingsStore 保存语言选择

## 8. 系统设置面板 (SystemSettings)

- [ ] 8.1 创建 `settings/SystemSettings.tsx` 组件
- [ ] 8.2 创建 Toggle 开关通用组件
  - 开启/关闭状态
  - 绿色/灰色背景
  - 白色滑块
  - 200ms 动画
- [ ] 8.3 创建 ToggleItem 组件(中文+英文标签+Toggle)
- [ ] 8.4 实现"App settings"分组标题
- [ ] 8.5 实现开机自动启动 Toggle
  - 中英文标签
  - Toggle 开关
  - 连接 set_auto_launch 命令
- [ ] 8.6 实现在Dock中显示 Toggle
  - 中英文标签
  - Toggle 开关
  - 连接 Tauri Dock API
- [ ] 8.7 连接 settingsStore 实现数据绑定

## 9. 模型设置面板 (ModelSettings)

- [ ] 9.1 创建 `settings/ModelSettings.tsx` 组件
- [ ] 9.2 创建 RadioGroup 单选组组件
- [ ] 9.3 创建 ModelOption 组件
  - Radio 按钮
  - 模型名称+大小+描述
  - "推荐"标签(Base模型)
  - 下载状态指示器
- [ ] 9.4 实现四个模型选项
  - Base (74MB, 快速, 一般精度) 推荐
  - Small (244MB, 较快, 较高精度)
  - Medium (769MB, 较慢, 高精度)
  - Large (1.5GB, 慢, 最高精度)
- [ ] 9.5 实现模型下载状态显示
  - ✓ 已下载(绿色)
  - [下载]按钮(灰色)
  - 进度条(下载中)
  - ✗ 下载失败(红色) + [重试]
- [ ] 9.6 实现模型下载逻辑
  - 调用 download_model 命令
  - 监听下载进度事件
  - 显示进度条和百分比
  - 错误处理和重试
- [ ] 9.7 实现"已下载的模型"列表
  - 显示已下载模型名称和大小
  - [删除]按钮
  - 删除确认对话框
- [ ] 9.8 实现模型删除逻辑
  - 调用 delete_model 命令
  - 更新状态和 UI
  - Toast 反馈
- [ ] 9.9 实现模型选择逻辑
  - Radio 选择
  - 保存到 settingsStore
  - 未下载模型提示

## 10. 状态管理

- [ ] 10.1 扩展 settingsStore
  - 添加 microphone 字段
  - 添加 model 字段
  - 添加 autoLaunch 字段
  - 添加 showInDock 字段
- [ ] 10.2 扩展 historyStore
  - 添加 todayTranscriptions 计算属性
  - 添加 loadTodayHistory 方法
- [ ] 10.3 可能新增 uiStore
  - currentPage 状态
  - settingsTab 状态
  - isSettingsOpen 状态
  - isLanguageSelectorOpen 状态

## 11. 通用 UI 组件

- [ ] 11.1 创建 Toggle.tsx - Toggle 开关组件
- [ ] 11.2 创建 RadioGroup.tsx - 单选组组件
- [ ] 11.3 创建 ProgressBar.tsx - 进度条组件
- [ ] 11.4 扩展现有 Modal 组件(支持多层弹窗)
- [ ] 11.5 确保 Button、Input 组件可复用

## 12. Tauri Commands 实现

- [ ] 12.1 实现 get_recent_transcriptions 命令
- [ ] 12.2 实现 list_audio_devices 命令(可选,二期)
- [ ] 12.3 实现 download_model 命令
  - 从 Hugging Face 下载模型
  - 发送下载进度事件
  - 断点续传支持
- [ ] 12.4 实现 delete_model 命令
- [ ] 12.5 实现 set_auto_launch 命令(macOS)
- [ ] 12.6 实现 get_auto_launch 命令
- [ ] 12.7 实现 set_dock_visibility 命令(可选)

## 13. 样式和主题

- [ ] 13.1 定义卡片样式(白色/浅灰背景,8-12px圆角,阴影)
- [ ] 13.2 定义 Toggle 样式(绿色/灰色,44x24px)
- [ ] 13.3 定义按钮样式(主要/次要,6-8px圆角)
- [ ] 13.4 定义排版样式(标题加粗,行高1.5-1.6)
- [ ] 13.5 确保颜色对比度符合 WCAG AA
- [ ] 13.6 添加过渡动画(200ms)

## 14. 错误处理和反馈

- [ ] 14.1 为所有 Tauri 命令调用添加 try-catch
- [ ] 14.2 实现设置保存失败的回滚逻辑
- [ ] 14.3 实现模型下载失败的错误提示
- [ ] 14.4 实现 Toast 反馈
  - 成功: "设置已保存"
  - 失败: "保存失败: {error}"
  - 模型下载: "模型下载完成" / "下载失败"
  - 模型删除: "模型已删除"
- [ ] 14.5 为设置加载失败添加重试按钮

## 15. 性能优化

- [ ] 15.1 为设置面板组件添加 React.memo
- [ ] 15.2 为 TranscriptionItem 添加 React.memo
- [ ] 15.3 使用 lazy + Suspense 懒加载设置面板
- [ ] 15.4 优化图标资源
- [ ] 15.5 测试页面加载时间(<300ms)
- [ ] 15.6 测试标签切换性能(<50ms)

## 16. 可访问性

- [ ] 16.1 为所有交互元素添加 aria-label
- [ ] 16.2 为 Toggle 开关添加 aria-checked
- [ ] 16.3 为弹窗添加 aria-modal
- [ ] 16.4 实现 Tab 键盘导航
- [ ] 16.5 实现 Enter 键激活按钮
- [ ] 16.6 实现 Escape 键关闭弹窗
- [ ] 16.7 实现焦点管理(打开/关闭弹窗)
- [ ] 16.8 测试屏幕阅读器兼容性

## 17. 测试

- [ ] 17.1 编写 HomePage 组件测试
- [ ] 17.2 编写 NotesPage 组件测试
- [ ] 17.3 编写 GeneralSettings 组件测试
- [ ] 17.4 编写 SystemSettings 组件测试
- [ ] 17.5 编写 ModelSettings 组件测试
- [ ] 17.6 编写 LanguageSelector 组件测试
- [ ] 17.7 编写 Toggle 组件测试
- [ ] 17.8 编写 RadioGroup 组件测试
- [ ] 17.9 测试设置保存和加载
- [ ] 17.10 测试错误处理场景
- [ ] 17.11 测试性能指标

## 18. 集成和调试

- [ ] 18.1 集成主窗口到应用
- [ ] 18.2 从托盘菜单可以打开主窗口
- [ ] 18.3 测试主窗口在不同屏幕尺寸下的表现
- [ ] 18.4 测试设置弹窗在主窗口中的表现
- [ ] 18.5 测试多层弹窗(设置弹窗 + 语言选择弹窗)
- [ ] 18.6 测试所有导航和页面切换
- [ ] 18.7 测试所有设置项的保存和读取
- [ ] 18.8 测试模型下载和删除流程
- [ ] 18.9 修复发现的所有 bug

## 19. 文档和清理

- [ ] 19.1 为所有新组件添加 JSDoc 注释
- [ ] 19.2 删除临时的调试日志
- [ ] 19.3 更新 README.md,添加主窗口和设置说明
- [ ] 19.4 添加使用截图
- [ ] 19.5 更新开发文档

## 20. 验收和发布准备

- [ ] 20.1 用户验收测试(UAT)
- [ ] 20.2 性能基准测试
- [ ] 20.3 可访问性测试
- [ ] 20.4 跨版本 macOS 测试
- [ ] 20.5 准备发布说明