import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Sidebar } from './Sidebar'
import { HomePage } from './HomePage'
import { NotesPage } from './NotesPage'
import { SettingsDialog } from './SettingsDialog'
import { useUIStore } from '../../stores'
import { useSettingsStore } from '../../stores'
import { useHistoryStore } from '../../stores'
import { useLoadingStore } from '../../stores'
import { Button, GlobalLoading, GlobalDownloadModal } from '../../components'

interface PermissionStatus {
  microphone: 'granted' | 'denied' | 'not_determined' | 'restricted' | 'checking'
  accessibility: boolean | null
}

export const MainWindow: React.FC = () => {
  const { currentPage } = useUIStore()
  const { loadSettings, settings } = useSettingsStore()
  const { loadRecent } = useHistoryStore()
  const { openSettings, setSettingsTab } = useUIStore()
  const {
    isInitializing,
    addInitializationTask,
    updateInitializationTask,
    completeInitialization,
    initializationProgress,
  } = useLoadingStore()

  const [showPermissionAlert, setShowPermissionAlert] = useState(false)
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus>({
    microphone: 'checking',
    accessibility: null,
  })
  const [initMessage, setInitMessage] = useState('正在初始化...')
  const initStartedRef = React.useRef(false)

  useEffect(() => {
    // 防止 React StrictMode 导致的重复初始化
    if (initStartedRef.current) {
      console.log('[MainWindow] ⚠️  初始化已经开始，跳过重复调用')
      return
    }
    initStartedRef.current = true

    // 全局初始化流程
    const initializeApp = async () => {
      console.log('[MainWindow] 🚀 开始应用初始化')

      // 任务 1: 加载设置
      const task1Id = 'load-settings'
      addInitializationTask({
        id: task1Id,
        message: '加载应用设置',
        status: 'running',
      })
      setInitMessage('加载应用设置...')

      try {
        await loadSettings()
        updateInitializationTask(task1Id, { status: 'completed' })
      } catch (error) {
        console.error('Failed to load settings:', error)
        updateInitializationTask(task1Id, {
          status: 'failed',
          error: String(error),
        })
      }

      // 任务 2: 加载历史记录
      const task2Id = 'load-history'
      addInitializationTask({
        id: task2Id,
        message: '加载历史记录',
        status: 'running',
      })
      setInitMessage('加载历史记录...')

      try {
        await loadRecent(50)
        updateInitializationTask(task2Id, { status: 'completed' })
      } catch (error) {
        console.error('Failed to load history:', error)
        updateInitializationTask(task2Id, {
          status: 'failed',
          error: String(error),
        })
      }

      // 任务 3: 检查权限
      const task3Id = 'check-permissions'
      addInitializationTask({
        id: task3Id,
        message: '检查系统权限',
        status: 'running',
      })
      setInitMessage('检查系统权限...')

      try {
        const micStatus = await invoke<string>('check_microphone_permission')
        const accessibilityStatus = await invoke<boolean>('check_accessibility_permission_cmd')

        setPermissionStatus({
          microphone: micStatus as 'granted' | 'denied' | 'not_determined' | 'restricted',
          accessibility: accessibilityStatus,
        })

        if (micStatus !== 'granted' || !accessibilityStatus) {
          console.log('[MainWindow] ⚠️  权限不完整，将在初始化后显示提示')
          setTimeout(() => setShowPermissionAlert(true), 1000)
        }

        updateInitializationTask(task3Id, { status: 'completed' })
      } catch (error) {
        console.error('Failed to check permissions:', error)
        updateInitializationTask(task3Id, {
          status: 'failed',
          error: String(error),
        })
      }

      // 等待设置加载完成后再进行模型初始化
      await new Promise((resolve) => setTimeout(resolve, 100))

      // 任务 4: 初始化音频系统
      const task4Id = 'init-audio'
      addInitializationTask({
        id: task4Id,
        message: '初始化音频系统',
        status: 'running',
      })
      setInitMessage('初始化音频系统...')

      try {
        await invoke('initialize_audio_system')
        updateInitializationTask(task4Id, { status: 'completed' })
      } catch (error) {
        console.error('Failed to initialize audio system:', error)
        updateInitializationTask(task4Id, {
          status: 'failed',
          error: String(error),
        })
      }

      // 任务 5: 初始化并预热模型
      const currentModel = settings.model || 'base'
      const task5Id = 'init-model'
      addInitializationTask({
        id: task5Id,
        message: `初始化模型 (${currentModel})`,
        status: 'running',
      })
      setInitMessage(`初始化模型 ${currentModel}...`)

      try {
        // 判断模型引擎类型
        const funasrModels = ['paraformer-zh', 'paraformer-large', 'sensevoice-small']
        const isFunASR = funasrModels.includes(currentModel)

        if (isFunASR) {
          // FunASR 模型：尝试初始化并预热
          console.log('[MainWindow] 初始化 FunASR 模型...')

          try {
            // 初始化 FunASR 引擎
            await invoke('initialize_funasr', { modelName: currentModel })
            console.log('[MainWindow] ✅ FunASR 引擎初始化成功')

            // 预热模型
            console.log('[MainWindow] 开始预热 FunASR 模型...')
            await invoke('prewarm_funasr_cmd')
            console.log('[MainWindow] ✅ FunASR 模型预热完成')
          } catch (error) {
            console.error('[MainWindow] FunASR 初始化失败:', error)
            console.log('[MainWindow] ℹ️  FunASR 初始化失败不影响应用启动，首次使用时会自动初始化')
            // FunASR 失败不阻塞应用启动，首次使用时会自动初始化
          }
        } else {
          // Whisper 模型：直接初始化
          await invoke('initialize_whisper', { modelName: currentModel })
          console.log('[MainWindow] ✅ Whisper 模型初始化完成')
        }

        updateInitializationTask(task5Id, { status: 'completed' })
      } catch (error) {
        console.error('Failed to initialize model:', error)
        updateInitializationTask(task5Id, {
          status: 'failed',
          error: String(error),
        })
      }

      // 完成初始化
      console.log('[MainWindow] ✅ 应用初始化完成')
      setTimeout(() => {
        completeInitialization()
      }, 500)
    }

    void initializeApp()
  }, []) // 只在挂载时执行一次

  const handleOpenEnvironmentSettings = () => {
    setShowPermissionAlert(false)
    setSettingsTab('environment')
    openSettings()
  }

  const renderContent = () => {
    switch (currentPage) {
      case 'home':
        return <HomePage />
      case 'notes':
        return <NotesPage />
      default:
        return <HomePage />
    }
  }

  // 显示全局 Loading
  if (isInitializing) {
    return (
      <GlobalLoading message={initMessage} progress={initializationProgress} showProgress={true} />
    )
  }

  return (
    <div className="h-screen flex overflow-hidden bg-white">
      <Sidebar />
      <main className="flex-1 overflow-y-auto">{renderContent()}</main>
      <SettingsDialog />

      {/* 全局下载遮罩 */}
      <GlobalDownloadModal />

      {/* 权限提示对话框 */}
      {showPermissionAlert && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="absolute inset-0 bg-black bg-opacity-50"
            onClick={() => setShowPermissionAlert(false)}
          />
          <div className="relative bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
            <div className="flex items-start gap-4">
              <div className="flex-shrink-0 w-12 h-12 rounded-full bg-yellow-100 flex items-center justify-center text-2xl">
                ⚠️
              </div>
              <div className="flex-1">
                <h3 className="text-lg font-semibold text-gray-900 mb-2">需要授权权限</h3>
                <p className="text-sm text-gray-600 mb-4">聆码需要以下权限才能正常工作：</p>
                <ul className="space-y-2 mb-4">
                  {permissionStatus.microphone !== 'granted' && (
                    <li className="flex items-center gap-2 text-sm">
                      <span className="text-red-500">❌</span>
                      <span className="text-gray-700">
                        <strong>麦克风权限</strong> - 录制语音需要
                      </span>
                    </li>
                  )}
                  {!permissionStatus.accessibility && (
                    <li className="flex items-center gap-2 text-sm">
                      <span className="text-red-500">❌</span>
                      <span className="text-gray-700">
                        <strong>辅助功能权限</strong> - 插入文本需要
                      </span>
                    </li>
                  )}
                </ul>
                <div className="flex items-center gap-2 justify-end">
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => setShowPermissionAlert(false)}
                  >
                    稍后设置
                  </Button>
                  <Button variant="primary" size="sm" onClick={handleOpenEnvironmentSettings}>
                    前往设置
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
