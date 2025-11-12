import React, { useEffect } from 'react'
import { useUIStore } from '../../stores'
import { GeneralSettings } from './settings/GeneralSettings'
import { SystemSettings } from './settings/SystemSettings'
import { ModelSettings } from './settings/ModelSettings'
import { EnvironmentSettings } from './settings/EnvironmentSettings'

type SettingsTab = 'general' | 'system' | 'model' | 'environment'

interface TabItem {
  id: SettingsTab
  icon: string
  label: string
}

const tabs: TabItem[] = [
  { id: 'environment', icon: '🔍', label: '环境检测' },
  { id: 'general', icon: '⚙️', label: '通用设置' },
  { id: 'system', icon: '💻', label: '系统设置' },
  { id: 'model', icon: '🤖', label: '模型设置' },
]

export const SettingsDialog: React.FC = () => {
  const { isSettingsOpen, closeSettings, settingsTab, setSettingsTab } = useUIStore()

  // ESC 键关闭
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isSettingsOpen) {
        closeSettings()
      }
    }

    window.addEventListener('keydown', handleEscape)
    return () => window.removeEventListener('keydown', handleEscape)
  }, [isSettingsOpen, closeSettings])

  const renderContent = () => {
    switch (settingsTab) {
      case 'environment':
        return <EnvironmentSettings />
      case 'general':
        return <GeneralSettings />
      case 'system':
        return <SystemSettings />
      case 'model':
        return <ModelSettings />
      default:
        return <EnvironmentSettings />
    }
  }

  if (!isSettingsOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-12">
      {/* 背景遮罩 */}
      <div className="absolute inset-0 bg-black bg-opacity-50" onClick={closeSettings} />

      {/* 弹窗主体 - 使用 max-w/max-h 让其自动适应窗口大小，同时留出边距 */}
      <div className="relative w-full h-full max-w-[850px] max-h-[650px] bg-white rounded-xl shadow-2xl flex flex-col overflow-hidden">
        {/* 顶部栏 */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">设置</h2>
          <button
            onClick={closeSettings}
            className="text-gray-400 hover:text-gray-600 transition-colors"
            aria-label="关闭设置"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* 主体内容 */}
        <div className="flex flex-1 overflow-hidden">
          {/* 左侧导航 */}
          <nav className="w-52 bg-gray-50 border-r border-gray-200 p-4 space-y-1">
            {tabs.map((tab) => {
              const isActive = tab.id === settingsTab
              return (
                <button
                  key={tab.id}
                  onClick={() => setSettingsTab(tab.id)}
                  className={`
                    w-full flex items-center gap-3 px-4 py-3 rounded-lg
                    text-left transition-colors duration-200
                    ${
                      isActive
                        ? 'bg-white text-green-600 shadow-sm'
                        : 'text-gray-700 hover:bg-white hover:text-gray-900'
                    }
                  `}
                >
                  <span className="text-xl">{tab.icon}</span>
                  <span className="font-medium">{tab.label}</span>
                </button>
              )
            })}
          </nav>

          {/* 右侧内容 */}
          <div className="flex-1 overflow-y-auto p-6">{renderContent()}</div>
        </div>

        {/* 底部栏 */}
        <div className="px-6 py-3 border-t border-gray-200 bg-gray-50">
          <p className="text-sm text-gray-500">聆码 v1.0.0</p>
        </div>
      </div>
    </div>
  )
}
