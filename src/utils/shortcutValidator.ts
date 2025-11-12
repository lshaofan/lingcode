/**
 * 快捷键验证和冲突检测工具
 */

export interface ValidationResult {
  isValid: boolean
  error?: string
  warning?: string
}

// macOS 系统级快捷键列表
const MACOS_SYSTEM_SHORTCUTS = [
  'Cmd+Q', // 退出应用
  'Cmd+W', // 关闭窗口
  'Cmd+H', // 隐藏窗口
  'Cmd+M', // 最小化窗口
  'Cmd+Opt+H', // 隐藏其他窗口
  'Cmd+Tab', // 切换应用
  'Cmd+Space', // Spotlight
  'Cmd+Ctrl+Space', // 表情符号
  'Cmd+Shift+3', // 截图全屏
  'Cmd+Shift+4', // 截图选区
  'Cmd+Shift+5', // 截图工具
  'Cmd+C', // 复制
  'Cmd+V', // 粘贴
  'Cmd+X', // 剪切
  'Cmd+Z', // 撤销
  'Cmd+Shift+Z', // 重做
  'Cmd+A', // 全选
  'Cmd+S', // 保存
  'Cmd+P', // 打印
  'Cmd+N', // 新建
  'Cmd+T', // 新标签页
  'Cmd+F', // 查找
  'Cmd+,', // 偏好设置
]

// Windows 系统级快捷键列表
const WINDOWS_SYSTEM_SHORTCUTS = [
  'Ctrl+Alt+Delete', // 任务管理器
  'Ctrl+Shift+Esc', // 任务管理器
  'Ctrl+C', // 复制
  'Ctrl+V', // 粘贴
  'Ctrl+X', // 剪切
  'Ctrl+Z', // 撤销
  'Ctrl+Y', // 重做
  'Ctrl+A', // 全选
  'Ctrl+S', // 保存
  'Ctrl+P', // 打印
  'Ctrl+N', // 新建
  'Ctrl+T', // 新标签页
  'Ctrl+F', // 查找
  'Ctrl+W', // 关闭窗口
]

// 常见应用快捷键(可能冲突)
const COMMON_APP_SHORTCUTS = [
  'Cmd+Shift+A', // Spotlight
  'Cmd+Shift+C', // Chrome 开发者工具
  'Cmd+Opt+I', // Chrome 开发者工具
  'Cmd+Shift+T', // 重新打开关闭的标签页
  'Cmd+Shift+N', // 新无痕窗口
  'Cmd+Shift+P', // VSCode 命令面板
]

/**
 * 检测快捷键是否与系统快捷键冲突
 */
export function validateShortcut(shortcut: string): ValidationResult {
  if (!shortcut || shortcut.trim() === '') {
    return {
      isValid: false,
      error: '快捷键不能为空',
    }
  }

  // 分割快捷键
  const parts = shortcut.split('+').map((s) => s.trim())

  // 至少需要一个修饰键
  const hasModifier =
    parts.includes('Cmd') ||
    parts.includes('Ctrl') ||
    parts.includes('Alt') ||
    parts.includes('Opt') ||
    parts.includes('Shift')

  if (!hasModifier) {
    return {
      isValid: false,
      error: '快捷键必须包含至少一个修饰键(Cmd/Ctrl/Alt/Shift)',
    }
  }

  // 检测是否为纯修饰键组合
  const modifierKeys = ['Cmd', 'Ctrl', 'Alt', 'Opt', 'Shift']
  const hasNonModifier = parts.some((part) => !modifierKeys.includes(part))

  // 如果是纯修饰键组合,至少需要2个修饰键
  if (!hasNonModifier && parts.length < 2) {
    return {
      isValid: false,
      error: '快捷键至少需要两个修饰键或一个修饰键加一个普通键',
    }
  }

  // 检测 macOS 系统快捷键冲突
  if (navigator.platform.toLowerCase().includes('mac')) {
    if (MACOS_SYSTEM_SHORTCUTS.includes(shortcut)) {
      return {
        isValid: false,
        error: `快捷键 "${shortcut}" 与 macOS 系统快捷键冲突,无法使用`,
      }
    }

    // 检测常见应用快捷键
    if (COMMON_APP_SHORTCUTS.includes(shortcut)) {
      return {
        isValid: true,
        warning: `快捷键 "${shortcut}" 可能与其他应用冲突,建议使用其他组合`,
      }
    }
  }

  // 检测 Windows 系统快捷键冲突
  if (navigator.platform.toLowerCase().includes('win')) {
    if (WINDOWS_SYSTEM_SHORTCUTS.includes(shortcut)) {
      return {
        isValid: false,
        error: `快捷键 "${shortcut}" 与 Windows 系统快捷键冲突,无法使用`,
      }
    }
  }

  // 推荐的快捷键模式
  const hasGoodPattern =
    (parts.includes('Cmd') || parts.includes('Ctrl')) && parts.includes('Shift')

  if (!hasGoodPattern && parts.length === 2) {
    return {
      isValid: true,
      warning: '建议使用包含 Cmd/Ctrl + Shift 的组合键,以减少与其他应用的冲突',
    }
  }

  return {
    isValid: true,
  }
}

/**
 * 格式化验证错误/警告消息
 */
export function getValidationMessage(result: ValidationResult): string {
  if (result.error) {
    return `❌ ${result.error}`
  }
  if (result.warning) {
    return `⚠️ ${result.warning}`
  }
  return '✅ 快捷键可用'
}
