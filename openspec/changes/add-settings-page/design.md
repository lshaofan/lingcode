# Design: 设置页面架构

## UI 布局设计

### 整体布局
```
┌─────────────────────────────────────────────────────────┐
│  聆码 Lingcode               v1.0.0    [最小化] [×]     │
├─────────────┬───────────────────────────────────────────┤
│             │                                           │
│  通用设置   │   ┌─ 通用设置 ─────────────────────────┐ │
│  语音识别   │   │                                    │ │
│  快捷键     │   │  语言:  [中文 ▼]                  │ │
│  历史记录   │   │  主题:  ○ 浅色 ● 深色 ○ 自动     │ │
│  关于       │   │  [ ] 开机自动启动                  │ │
│             │   │  [✓] 显示通知                      │ │
│             │   │                                    │ │
│             │   └────────────────────────────────────┘ │
│             │                                           │
├─────────────┴───────────────────────────────────────────┤
│                           [保存]  [重置为默认]          │
└─────────────────────────────────────────────────────────┘
```

### 组件层次结构
```
SettingsPage (src/windows/main/SettingsPage.tsx)
├── SettingsSidebar (侧边栏导航)
│   ├── SettingsTab × 5
│   └── Version信息
└── SettingsContent (内容区域)
    ├── GeneralSettings (通用设置)
    ├── VoiceSettings (语音设置)
    ├── ShortcutSettings (快捷键设置)
    ├── HistorySettings (历史记录)
    └── AboutSettings (关于)
```

## 状态管理策略

### Zustand Stores
使用现有的 stores,无需新增:
- `settingsStore`: 管理所有设置项
- `historyStore`: 管理转录历史记录

### 数据流
```
UI 组件 → Zustand Action → Tauri Command → SQLite → 返回结果 → 更新 Store → 触发 UI 重新渲染
```

## 各设置区域详细设计

### 1. 通用设置 (GeneralSettings)
**组件:** `src/windows/main/settings/GeneralSettings.tsx`

**字段:**
- 语言: Select 下拉框(默认中文,暂时禁用其他选项)
- 主题: Radio Group (light/dark/auto)
- 自动启动: Toggle Switch
- 通知: Toggle Switch

**交互:**
- 所有设置变更立即保存到 store
- 主题变更立即应用(不需要重启)

### 2. 语音识别设置 (VoiceSettings)
**组件:** `src/windows/main/settings/VoiceSettings.tsx`

**字段:**
- 模型选择: Radio Group
  - Base (74MB, 快速, 一般精度) ✓ 推荐
  - Small (244MB, 较快, 较高精度)
  - Medium (769MB, 较慢, 高精度)
  - Large (1.5GB, 慢, 最高精度)
- 模型状态显示:
  - 已下载: ✓ 绿色标记
  - 未下载: [下载] 按钮
  - 下载中: 进度条

**交互:**
- 选择模型后自动保存
- 如果模型未下载,显示下载按钮和预计大小
- 下载过程显示进度条

### 3. 快捷键设置 (ShortcutSettings)
**组件:** `src/windows/main/settings/ShortcutSettings.tsx`

**字段:**
- 录音快捷键: 可编辑的快捷键输入框
  - 默认: Cmd+Shift+S
  - 点击输入框后,监听键盘按键
  - 显示当前快捷键组合
- 冲突检测: 检测系统快捷键冲突并警告

**交互:**
- 点击输入框进入"等待按键"状态
- 按下键盘组合后,验证是否合法
- 保存前检测冲突
- 保存后立即生效(无需重启)

### 4. 历史记录 (HistorySettings)
**组件:** `src/windows/main/settings/HistorySettings.tsx`

**字段:**
- 搜索框: 搜索转录文本内容
- 记录列表: 时间倒序显示
  - 显示: 时间、文本预览(前50字)、时长
  - 操作: [复制] [删除]
- 批量操作: [全选] [删除选中] [清空所有]
- 分页: 每页20条

**交互:**
- 搜索实时过滤
- 点击记录展开查看完整文本
- 删除操作需要确认
- "清空所有"需要二次确认

### 5. 关于 (AboutSettings)
**组件:** `src/windows/main/settings/AboutSettings.tsx`

**内容:**
- 应用图标
- 应用名称: 聆码 Lingcode
- 版本号: v1.0.0
- 描述: 跨应用语音听写工具
- GitHub 链接: 可点击打开浏览器
- 开源协议: MIT / Apache 2.0
- 检查更新按钮(未来实现)

## 技术实现细节

### 主题切换实现
- 使用 CSS 变量定义颜色主题
- `data-theme` 属性控制当前主题
- `auto` 模式监听系统主题变化(使用 `prefers-color-scheme` media query)

### 快捷键录制实现
```typescript
const [isRecording, setIsRecording] = useState(false);
const [keys, setKeys] = useState<string[]>([]);

const handleKeyDown = (e: KeyboardEvent) => {
  if (!isRecording) return;
  e.preventDefault();

  const modifiers = [];
  if (e.metaKey) modifiers.push('Cmd');
  if (e.ctrlKey) modifiers.push('Ctrl');
  if (e.shiftKey) modifiers.push('Shift');
  if (e.altKey) modifiers.push('Alt');

  if (!['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) {
    modifiers.push(e.key.toUpperCase());
    setKeys(modifiers);
    setIsRecording(false);
    // 保存快捷键
    saveShortcut(modifiers.join('+'));
  }
};
```

### 历史记录虚拟滚动
由于历史记录可能很多,使用虚拟滚动优化性能:
- 使用 `react-window` 库
- 只渲染可见区域的记录
- 懒加载更多数据

## 错误处理

### 错误场景
1. 设置保存失败
2. 模型下载失败
3. 快捷键冲突
4. 历史记录加载失败

### 处理策略
- 使用 Toast 组件显示错误信息
- 设置保存失败后回滚到之前的值
- 网络错误时提供重试按钮
- 数据加载失败显示错误状态和重新加载按钮

## 性能优化

### 优化策略
1. **防抖 (Debounce)**: 搜索输入、设置变更
2. **虚拟滚动**: 历史记录列表
3. **React.memo**: 设置项组件
4. **懒加载**: 各设置标签页按需加载

### 预期性能指标
- 设置页面初始加载: < 300ms
- 标签页切换: < 50ms
- 搜索响应: < 100ms (防抖后)
- 设置保存: < 200ms

## 可访问性 (Accessibility)

- 键盘导航: Tab 键在设置项间切换
- ARIA 标签: 为所有交互元素添加标签
- 焦点管理: 切换标签页时聚焦到内容区域
- 屏幕阅读器友好: 状态变化有语音提示

## 未来扩展

### 可能的增强功能
- 设置导入/导出
- 云同步设置(可选)
- 高级设置折叠面板
- 快捷操作搜索(Cmd+K)
- 多语言支持(英文、日文等)
