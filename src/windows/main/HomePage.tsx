import React, { useMemo } from 'react'
import { useHistoryStore, useSettingsStore } from '../../stores'
import { format, isToday, parseISO } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import { getShortcutDisplayParts } from '../../utils/shortcutFormatter'

export const HomePage: React.FC = () => {
  const { transcriptions } = useHistoryStore()
  const { settings } = useSettingsStore()

  // 筛选今天的转录记录
  const todayTranscriptions = useMemo(() => {
    return transcriptions.filter((t) => {
      try {
        const date = parseISO(t.created_at)
        return isToday(date)
      } catch {
        return false
      }
    })
  }, [transcriptions])

  // 格式化时间
  const formatTime = (dateString: string) => {
    try {
      const date = parseISO(dateString)
      return format(date, 'p', { locale: zhCN })
    } catch {
      return dateString
    }
  }

  // 获取快捷键显示部分
  const shortcutParts = useMemo(() => {
    return getShortcutDisplayParts(settings.shortcut || 'Cmd+Shift+S')
  }, [settings.shortcut])

  return (
    <div className="h-full flex flex-col max-w-4xl mx-auto p-8">
      {/* 快捷键提示卡片 - 固定不滚动 */}
      <div className="flex-shrink-0 mb-8 p-6 bg-gradient-to-br from-green-50 to-blue-50 rounded-xl shadow-sm">
        <h2 className="text-xl font-bold text-gray-900 mb-3 flex items-center gap-2 flex-wrap">
          按住
          {shortcutParts.map((part, index) => (
            <React.Fragment key={index}>
              {index > 0 && <span>+</span>}
              <kbd className="px-2 py-1 text-sm font-semibold text-gray-800 bg-white border border-gray-300 rounded">
                {part.symbol} {part.name}
              </kbd>
            </React.Fragment>
          ))}
          在任何应用中听写
        </h2>
        <p className="text-gray-700 leading-relaxed">
          聆码可在您的所有应用中使用。在电子邮件、消息、文档或其他任何地方尝试使用。
        </p>
      </div>

      {/* 今日转录历史 */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* 固定的标题 */}
        <h3 className="text-lg font-semibold text-gray-700 mb-4 flex-shrink-0">今天</h3>

        {/* 可滚动的列表区域 */}
        <div className="flex-1 overflow-y-auto min-h-0">
          {todayTranscriptions.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <div className="text-4xl mb-3">📝</div>
              <p>今天还没有转录记录</p>
              <p className="text-sm mt-2">使用全局快捷键开始语音输入</p>
            </div>
          ) : (
            <div className="space-y-3 pb-4">
              {todayTranscriptions.map((item, index) => (
                <div
                  key={item.id || index}
                  className="p-4 bg-white border border-gray-200 rounded-lg hover:border-gray-300 hover:shadow-sm transition-all"
                >
                  <div className="text-xs text-gray-500 mb-2">{formatTime(item.created_at)}</div>
                  <p className="text-gray-900 leading-relaxed whitespace-pre-wrap">{item.text}</p>
                  {item.app_context && (
                    <div className="mt-2 text-xs text-gray-400">来自: {item.app_context}</div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
