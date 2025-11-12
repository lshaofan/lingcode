import { useEffect, useState } from 'react'
import { X, AlertCircle, AlertTriangle, Info } from 'lucide-react'

export type InlineToastType = 'error' | 'warning' | 'info'

interface InlineToastProps {
  type: InlineToastType
  message: string
  dismissible: boolean
  onDismiss: () => void
  duration?: number // 自动消失时长（毫秒），如果为 0 则不自动消失
}

export function InlineToast({
  type,
  message,
  dismissible,
  onDismiss,
  duration = 0,
}: InlineToastProps) {
  const [isVisible, setIsVisible] = useState(true)

  useEffect(() => {
    if (duration > 0) {
      const timer = setTimeout(() => {
        setIsVisible(false)
        setTimeout(onDismiss, 300) // 等待动画完成
      }, duration)

      return () => clearTimeout(timer)
    }
  }, [duration, onDismiss])

  if (!isVisible) {
    return null
  }

  // 根据类型设置样式和图标（简化版，适配 Tooltip 样式）
  const styles = {
    error: {
      text: 'text-red-400',
      icon: AlertCircle,
    },
    warning: {
      text: 'text-yellow-400',
      icon: AlertTriangle,
    },
    info: {
      text: 'text-blue-400',
      icon: Info,
    },
  }

  const style = styles[type]
  const Icon = style.icon

  return (
    <div className="flex items-start gap-2">
      <Icon className={`w-4 h-4 mt-0.5 flex-shrink-0 ${style.text}`} />
      <p className="text-sm flex-1 text-white/90">{message}</p>
      {dismissible && (
        <button
          onClick={() => {
            setIsVisible(false)
            setTimeout(onDismiss, 300)
          }}
          className="flex-shrink-0 text-white/70 hover:text-white/90 transition-colors"
          aria-label="关闭"
        >
          <X className="w-4 h-4" />
        </button>
      )}
    </div>
  )
}
