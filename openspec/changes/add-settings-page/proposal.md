# Proposal: 添加设置和管理页面

## Why

当前的主窗口只是一个简单的演示界面,没有实际的设置管理功能。用户需要一个专门的设置页面来:
- 配置语音识别模型选择(base/small/medium/large)
- 自定义全局快捷键
- 调整应用行为(自动启动、通知等)
- 选择界面主题(浅色/深色/跟随系统)
- 查看和管理转录历史记录

这是应用的核心管理界面,用户需要一个直观、易用的设置中心来控制所有功能。

## What Changes

### 添加设置页面 UI
创建一个完整的设置/管理页面,包含以下功能模块:

**1. 通用设置区域**
- 语言选择(当前仅中文,为将来扩展预留)
- 主题切换(浅色/深色/自动)
- 自动启动开关
- 通知开关

**2. 语音识别设置区域**
- Whisper 模型选择(base/small/medium/large)
- 模型信息展示(大小、精度说明)

**3. 快捷键设置区域**
- 全局快捷键自定义
- 快捷键冲突检测

**4. 转录历史区域**
- 显示最近的转录记录列表
- 搜索和筛选功能
- 删除单条/批量删除功能
- 查看详情

**5. 关于区域**
- 应用版本信息
- GitHub 链接
- 开源协议信息

### UI 组织结构
- 使用左侧标签页导航(General, Voice, Shortcuts, History, About)
- 右侧显示对应的设置内容
- 响应式布局,适配不同窗口大小
- 保存按钮在底部固定

### 数据持久化
- 所有设置通过 Zustand store 管理
- 通过 Tauri commands 读取/写入 SQLite 数据库
- 设置变更实时生效

## Impact

### 受影响的 Specs
- `settings-ui` (新增) - 设置页面 UI 规范

### 受影响的代码
- `src/App.tsx` (重写) - 从演示界面改为设置页面
- `src/windows/main/` (新增) - 创建设置页面组件
- `src/stores/settingsStore.ts` (修改) - 可能需要扩展状态
- `src/stores/historyStore.ts` (可能新增) - 历史记录状态管理

### 依赖
- 现有的 `settingsStore` 和 `historyStore`
- Tauri commands: `get_setting`, `set_setting`, `get_recent_transcriptions` 等
- 现有的 UI 组件库(Button, Input, Modal 等)

### UI 设计原则
- 简洁明了的布局
- 清晰的分组和视觉层次
- 即时反馈和错误提示
- 适配浅色/深色主题
