import { useState, useEffect, useCallback } from 'react'

export interface ShortcutRecorderResult {
  recording: boolean
  shortcut: string
  startRecording: () => void
  stopRecording: () => void
  clearShortcut: () => void
}

// 修饰键映射
const MODIFIER_KEYS = {
  Meta: 'Cmd',
  Control: 'Ctrl',
  Alt: 'Opt',
  Shift: 'Shift',
}

// 特殊键映射
const SPECIAL_KEYS: Record<string, string> = {
  ' ': 'Space',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Escape: 'Esc',
  Delete: 'Delete',
  Backspace: 'Backspace',
  Tab: 'Tab',
  Enter: 'Enter',
  CapsLock: 'CapsLock',
}

// 判断是否为修饰键
function isModifierKey(key: string): boolean {
  return key in MODIFIER_KEYS
}

// 从物理按键代码格式化键名(e.code)
function formatKeyFromCode(code: string): string | null {
  // 处理字母键: KeyA -> A, KeyB -> B, etc.
  if (/^Key[A-Z]$/.test(code)) {
    return code.substring(3) // 移除 "Key" 前缀
  }

  // 处理数字键: Digit0 -> 0, Digit1 -> 1, etc.
  if (/^Digit\d$/.test(code)) {
    return code.substring(5) // 移除 "Digit" 前缀
  }

  // 处理功能键: F1, F2, etc.
  if (/^F\d{1,2}$/.test(code)) {
    return code
  }

  // 处理特殊键
  const codeToKeyMap: Record<string, string> = {
    Space: 'Space',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Escape: 'Esc',
    Delete: 'Delete',
    Backspace: 'Backspace',
    Tab: 'Tab',
    Enter: 'Enter',
    CapsLock: 'CapsLock',
  }

  return codeToKeyMap[code] || null
}

// 格式化键名(e.key,作为备用)
function formatKey(key: string): string {
  // 如果是修饰键,返回映射后的名称
  if (key in MODIFIER_KEYS) {
    return MODIFIER_KEYS[key as keyof typeof MODIFIER_KEYS]
  }

  // 如果是特殊键,返回映射后的名称
  const specialKey = SPECIAL_KEYS[key]
  if (specialKey) {
    return specialKey
  }

  // 如果是单个字母或数字,转为大写
  if (key.length === 1) {
    return key.toUpperCase()
  }

  // 处理功能键 F1-F12
  if (/^F\d{1,2}$/.test(key)) {
    return key
  }

  return key
}

/**
 * 快捷键录制 Hook
 * 用于在设置页面录制用户按下的快捷键组合
 */
export function useShortcutRecorder(): ShortcutRecorderResult {
  const [recording, setRecording] = useState(false)
  const [shortcut, setShortcut] = useState('')
  const [lastRecordTime, setLastRecordTime] = useState(0)

  // 开始录制
  const startRecording = useCallback(() => {
    setRecording(true)
    setShortcut('')
    setLastRecordTime(0)
  }, [])

  // 停止录制
  const stopRecording = useCallback(() => {
    setRecording(false)
    setLastRecordTime(0)
  }, [])

  // 清空快捷键
  const clearShortcut = useCallback(() => {
    setShortcut('')
  }, [])

  // 监听键盘事件
  useEffect(() => {
    if (!recording) {
      return
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault()
      e.stopPropagation()

      // 🔑 关键修复:使用 e.code 而不是 e.key
      // e.key 会被修饰键影响(例如 Opt+Shift+A 可能变成 À)
      // e.code 返回物理按键代码(例如 "KeyA", "KeyB" 等)
      const code = e.code
      const key = e.key
      const now = Date.now()

      // 防抖:避免重复触发
      if (now - lastRecordTime < 100) {
        return
      }

      // 构建快捷键字符串
      const keys: string[] = []

      // macOS 优先使用 Cmd
      if (e.metaKey) {
        keys.push('Cmd')
      }

      // 添加 Ctrl
      if (e.ctrlKey) {
        keys.push('Ctrl')
      }

      // 添加 Alt/Option
      if (e.altKey) {
        keys.push('Opt')
      }

      // 添加 Shift
      if (e.shiftKey) {
        keys.push('Shift')
      }

      // 如果不是单独的修饰键,添加主键
      // 优先使用 code 来确定主键,避免被修饰键影响
      if (!isModifierKey(key)) {
        const mainKey = formatKeyFromCode(code) || formatKey(key)
        keys.push(mainKey)
      }

      // 生成快捷键字符串
      const shortcutString = keys.join('+')

      // 验证快捷键是否有效
      // 必须包含至少一个修饰键和一个普通键
      if (e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) {
        // 如果包含非修饰键,生成快捷键
        if (!isModifierKey(key) && keys.length >= 2) {
          setShortcut(shortcutString)
          setLastRecordTime(now)

          // 立即完成录制
          setTimeout(() => {
            setRecording(false)
          }, 100)
        } else if (isModifierKey(key) && keys.length >= 1) {
          // 如果只按了修饰键,临时显示(用于预览)
          setShortcut(shortcutString)
          setLastRecordTime(now)
        }
      }
    }

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault()
      e.stopPropagation()
      // KeyUp 事件用于清理,但我们不需要在这里做任何操作
      // 录制在 KeyDown 时完成
    }

    // 添加事件监听
    window.addEventListener('keydown', handleKeyDown, true)
    window.addEventListener('keyup', handleKeyUp, true)

    return () => {
      window.removeEventListener('keydown', handleKeyDown, true)
      window.removeEventListener('keyup', handleKeyUp, true)
    }
  }, [recording, lastRecordTime, shortcut])

  return {
    recording,
    shortcut,
    startRecording,
    stopRecording,
    clearShortcut,
  }
}
