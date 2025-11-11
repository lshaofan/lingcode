import React, { useEffect } from 'react'
import { useDownloadStore } from '../stores'

export const GlobalDownloadModal: React.FC = () => {
  const { status, modelName, progress, message, error, resetDownload } = useDownloadStore()

  const isVisible = status !== 'idle'

  // 当下载完成或失败时，3秒后自动关闭
  useEffect(() => {
    if (status === 'completed' || status === 'error') {
      const timer = setTimeout(() => {
        resetDownload()
      }, 3000)
      return () => clearTimeout(timer)
    }
  }, [status, resetDownload])

  if (!isVisible) return null

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black bg-opacity-60">
      <div className="relative w-full max-w-md bg-white rounded-2xl shadow-2xl p-8">
        {/* 标题 */}
        <div className="text-center mb-6">
          <h3 className="text-2xl font-bold text-gray-900">
            {status === 'downloading' && '正在下载模型'}
            {status === 'completed' && '下载完成'}
            {status === 'error' && '下载失败'}
          </h3>
          {modelName && (
            <p className="text-sm text-gray-600 mt-2">模型: {modelName.toUpperCase()}</p>
          )}
        </div>

        {/* 进度区域 */}
        <div className="space-y-4">
          {status === 'downloading' && (
            <>
              {/* 进度条 */}
              <div className="relative h-3 bg-gray-200 rounded-full overflow-hidden">
                {progress > 0 && progress < 100 ? (
                  // 有进度时显示确定的进度条
                  <div
                    className="absolute left-0 top-0 h-full bg-gradient-to-r from-green-400 to-green-600 transition-all duration-300 ease-out"
                    style={{ width: `${progress}%` }}
                  >
                    <div className="absolute inset-0 bg-white opacity-20 animate-pulse" />
                  </div>
                ) : (
                  // 无进度或刚开始时显示不确定的加载动画
                  <div className="absolute inset-0 bg-gradient-to-r from-green-400 to-green-600 animate-pulse">
                    <div
                      className="h-full bg-gradient-to-r from-transparent via-white to-transparent opacity-30 animate-shimmer"
                      style={{
                        animation: 'shimmer 2s infinite',
                        backgroundSize: '200% 100%',
                      }}
                    />
                  </div>
                )}
              </div>

              {/* 进度文字 */}
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-600">{message || '正在下载...'}</span>
                {progress > 0 && <span className="font-semibold text-green-600">{progress}%</span>}
              </div>

              {/* 加载动画 */}
              <div className="flex justify-center items-center space-x-2 mt-4">
                <div
                  className="w-2 h-2 bg-green-500 rounded-full animate-bounce"
                  style={{ animationDelay: '0ms' }}
                />
                <div
                  className="w-2 h-2 bg-green-500 rounded-full animate-bounce"
                  style={{ animationDelay: '150ms' }}
                />
                <div
                  className="w-2 h-2 bg-green-500 rounded-full animate-bounce"
                  style={{ animationDelay: '300ms' }}
                />
              </div>

              {/* 提示文字 */}
              <p className="text-xs text-center text-gray-500 mt-4">
                请勿关闭窗口，下载过程可能需要几分钟...
              </p>
            </>
          )}

          {status === 'completed' && (
            <div className="text-center">
              {/* 成功图标 */}
              <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
                <svg
                  className="w-8 h-8 text-green-600"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={3}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <p className="text-gray-700">{message}</p>
              <p className="text-xs text-gray-500 mt-2">3秒后自动关闭...</p>
            </div>
          )}

          {status === 'error' && (
            <div className="text-center">
              {/* 错误图标 */}
              <div className="mx-auto w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4">
                <svg
                  className="w-8 h-8 text-red-600"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </div>
              <p className="text-red-700 font-medium">{error}</p>
              <p className="text-xs text-gray-500 mt-2">3秒后自动关闭...</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
