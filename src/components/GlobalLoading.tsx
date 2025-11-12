import React from 'react'

interface GlobalLoadingProps {
  message?: string
  progress?: number
  showProgress?: boolean
}

export const GlobalLoading: React.FC<GlobalLoadingProps> = ({
  message = '正在加载...',
  progress,
  showProgress = false,
}) => {
  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-white">
      <div className="text-center">
        {/* Logo 或应用图标 */}
        <div className="mb-6 flex justify-center">
          <div className="w-16 h-16 bg-blue-500 rounded-2xl flex items-center justify-center">
            <svg
              className="w-10 h-10 text-white"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
              />
            </svg>
          </div>
        </div>

        {/* 加载动画 */}
        <div className="mb-4 flex justify-center">
          <div className="relative w-12 h-12">
            <div className="absolute inset-0 border-4 border-gray-200 rounded-full"></div>
            <div className="absolute inset-0 border-4 border-blue-500 rounded-full border-t-transparent animate-spin"></div>
          </div>
        </div>

        {/* 加载消息 */}
        <p className="text-lg font-medium text-gray-900 mb-2">{message}</p>

        {/* 进度条（可选） */}
        {showProgress && progress !== undefined && (
          <div className="mt-4 w-64 mx-auto">
            <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500 transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
            <p className="text-sm text-gray-500 mt-2">{progress}%</p>
          </div>
        )}

        {/* 提示文本 */}
        <p className="text-sm text-gray-500 mt-4">请稍候，正在准备环境...</p>
      </div>
    </div>
  )
}
