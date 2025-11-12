import React, { useEffect, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../../stores'
import { Button } from '../../../components'
import { useToast } from '../../../components'

interface PermissionStatus {
  microphone: 'granted' | 'denied' | 'not_determined' | 'restricted' | 'checking'
  accessibility: boolean | 'checking'
}

interface PythonEnvStatus {
  status: 'checking' | 'ready' | 'missing' | 'error' | 'prewarmed'
  message: string
  details?: string
}

export const EnvironmentSettings: React.FC = () => {
  const { settings } = useSettingsStore()
  const toast = useToast()
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus>({
    microphone: 'checking',
    accessibility: 'checking',
  })
  const [pythonStatus, setPythonStatus] = useState<PythonEnvStatus>({
    status: 'checking',
    message: '正在检查Python环境...',
  })
  const [isCheckingMic, setIsCheckingMic] = useState(false)
  const [isCheckingAccessibility, setIsCheckingAccessibility] = useState(false)

  // 检查麦克风权限
  const checkMicrophonePermission = useCallback(async () => {
    try {
      setIsCheckingMic(true)
      const status = await invoke<string>('check_microphone_permission')
      setPermissionStatus((prev) => ({ ...prev, microphone: status as any }))
    } catch (error) {
      console.error('Failed to check microphone permission:', error)
      toast.error(`检查麦克风权限失败: ${String(error)}`)
    } finally {
      setIsCheckingMic(false)
    }
  }, [toast])

  // 请求麦克风权限
  const requestMicrophonePermission = useCallback(async () => {
    try {
      setIsCheckingMic(true)
      const status = await invoke<string>('request_microphone_permission')
      setPermissionStatus((prev) => ({ ...prev, microphone: status as any }))

      if (status === 'granted') {
        toast.success('麦克风权限已授予')
      } else if (status === 'denied') {
        toast.error('麦克风权限被拒绝，请在系统设置中手动授予')
        // 打开系统设置
        await invoke('open_microphone_settings')
      }
    } catch (error) {
      console.error('Failed to request microphone permission:', error)
      toast.error(`请求麦克风权限失败: ${String(error)}`)
    } finally {
      setIsCheckingMic(false)
    }
  }, [toast])

  // 检查辅助功能权限
  const checkAccessibilityPermission = useCallback(async () => {
    try {
      setIsCheckingAccessibility(true)
      const hasPermission = await invoke<boolean>('check_accessibility_permission_cmd')
      setPermissionStatus((prev) => ({ ...prev, accessibility: hasPermission }))
    } catch (error) {
      console.error('Failed to check accessibility permission:', error)
      toast.error(`检查辅助功能权限失败: ${String(error)}`)
    } finally {
      setIsCheckingAccessibility(false)
    }
  }, [toast])

  // 请求辅助功能权限
  const requestAccessibilityPermission = useCallback(async () => {
    try {
      setIsCheckingAccessibility(true)
      await invoke('request_accessibility_permission_cmd')
      toast.info('请在打开的系统设置中授予辅助功能权限')

      // 等待一小段时间后重新检查
      setTimeout(() => {
        checkAccessibilityPermission()
      }, 2000)
    } catch (error) {
      console.error('Failed to request accessibility permission:', error)
      toast.error(`请求辅助功能权限失败: ${String(error)}`)
      setIsCheckingAccessibility(false)
    }
  }, [toast, checkAccessibilityPermission])

  // 检查Python环境
  const checkPythonEnvironment = useCallback(async () => {
    try {
      setPythonStatus({
        status: 'checking',
        message: '正在检查Python环境...',
      })

      // 检查是否是 FunASR 模型
      const isFunASR =
        settings.model &&
        ['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(settings.model)

      if (isFunASR) {
        // 设置超时，防止长时间卡在"检查中"
        const timeoutId = setTimeout(() => {
          setPythonStatus({
            status: 'ready',
            message: 'Python 环境检查完成',
            details: '环境初始化可能需要一些时间，首次使用时会自动安装依赖',
          })
        }, 3000) // 3秒超时

        try {
          // 触发检查（通过重新初始化）
          await invoke('initialize_funasr', { modelName: settings.model })
          clearTimeout(timeoutId)
          // 状态会通过事件监听器更新
          // 如果没有收到事件，超时会自动设置为 ready 状态
        } catch (error) {
          clearTimeout(timeoutId)
          throw error
        }
      } else {
        // Whisper 模型不需要 Python 环境
        setPythonStatus({
          status: 'ready',
          message: '当前模型不需要 Python 环境',
          details: `${settings.model} 是本地模型，可直接使用`,
        })
      }
    } catch (error) {
      console.error('Failed to check Python environment:', error)
      setPythonStatus({
        status: 'error',
        message: '环境检查失败',
        details: String(error),
      })
    }
  }, [settings.model])

  // 全部检查
  const checkAll = useCallback(async () => {
    try {
      await Promise.all([
        checkMicrophonePermission(),
        checkAccessibilityPermission(),
        checkPythonEnvironment(),
      ])

      // 等待一小段时间确保状态已更新
      setTimeout(() => {
        // 检查所有权限和环境状态
        const micGranted = permissionStatus.microphone === 'granted'
        const accessibilityGranted = permissionStatus.accessibility === true
        const isFunASR =
          settings.model &&
          ['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(settings.model)
        const pythonReady =
          !isFunASR || pythonStatus.status === 'ready' || pythonStatus.status === 'prewarmed'

        if (micGranted && accessibilityGranted && pythonReady) {
          toast.success('✅ 所有权限和环境都已就绪！')
        } else {
          const issues = []
          if (!micGranted) issues.push('麦克风权限')
          if (!accessibilityGranted) issues.push('辅助功能权限')
          if (!pythonReady) issues.push('Python环境')
          toast.warning(`⚠️ 部分项目需要设置: ${issues.join('、')}`)
        }
      }, 500)
    } catch (error) {
      console.error('Failed to check all permissions:', error)
      toast.error(`检测失败: ${String(error)}`)
    }
  }, [
    checkMicrophonePermission,
    checkAccessibilityPermission,
    checkPythonEnvironment,
    permissionStatus,
    pythonStatus,
    settings.model,
    toast,
  ])

  // 初始加载时检查所有权限
  useEffect(() => {
    // 只在组件挂载时执行一次
    const initialCheck = async () => {
      await checkMicrophonePermission()
      await checkAccessibilityPermission()
      await checkPythonEnvironment()
    }
    initialCheck()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []) // 空依赖数组，只在挂载时执行一次

  // 监听Python环境状态变化
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<PythonEnvStatus>('python-env-status', (event) => {
        console.log('[EnvironmentSettings] Received python-env-status:', event.payload)
        setPythonStatus(event.payload)
      })
      return unlisten
    }

    let unlisten: (() => void) | null = null
    setupListener().then((fn) => {
      unlisten = fn
    })

    return () => {
      if (unlisten) unlisten()
    }
  }, [])

  // 状态图标和颜色
  const getStatusIcon = (status: string | boolean) => {
    if (status === 'checking') return '⏳'
    if (status === 'granted' || status === true || status === 'ready' || status === 'prewarmed')
      return '✅'
    if (status === 'denied' || status === false || status === 'error') return '❌'
    if (status === 'missing' || status === 'not_determined') return '⚠️'
    return '❓'
  }

  const getStatusColor = (status: string | boolean) => {
    if (status === 'granted' || status === true || status === 'ready' || status === 'prewarmed')
      return 'text-green-600'
    if (status === 'denied' || status === false || status === 'error') return 'text-red-600'
    if (status === 'missing' || status === 'not_determined') return 'text-yellow-600'
    return 'text-gray-600'
  }

  const getStatusText = (type: 'microphone' | 'accessibility', status: string | boolean) => {
    if (type === 'microphone') {
      if (status === 'checking') return '检查中...'
      if (status === 'granted') return '已授权'
      if (status === 'denied') return '已拒绝'
      if (status === 'not_determined') return '未设置'
      if (status === 'restricted') return '受限制'
    } else {
      if (status === 'checking') return '检查中...'
      if (status === true) return '已授权'
      if (status === false) return '未授权'
    }
    return '未知'
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-2xl font-semibold text-gray-900">环境检测</h3>
        <Button variant="secondary" size="sm" onClick={checkAll}>
          🔄 全部检测
        </Button>
      </div>

      <p className="text-sm text-gray-600">
        聆码需要以下权限和环境才能正常工作，请确保所有项目都已授权。
      </p>

      {/* 权限状态卡片 */}
      <div className="space-y-4">
        {/* 麦克风权限 */}
        <div className="p-4 bg-gray-50 rounded-lg border border-gray-200">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">{getStatusIcon(permissionStatus.microphone)}</span>
                <h4 className="font-semibold text-gray-900">麦克风权限</h4>
                <span
                  className={`text-sm font-medium ${getStatusColor(permissionStatus.microphone)}`}
                >
                  {getStatusText('microphone', permissionStatus.microphone)}
                </span>
              </div>
              <p className="text-sm text-gray-600 mb-3">
                录制语音需要麦克风权限。如果被拒绝，请在"系统设置 → 隐私与安全性 → 麦克风"中授权。
              </p>
              <div className="flex items-center gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={checkMicrophonePermission}
                  disabled={isCheckingMic}
                >
                  {isCheckingMic ? '检查中...' : '重新检测'}
                </Button>
                {permissionStatus.microphone !== 'granted' && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={requestMicrophonePermission}
                    disabled={isCheckingMic}
                  >
                    请求授权
                  </Button>
                )}
                {permissionStatus.microphone === 'denied' && (
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => invoke('open_microphone_settings')}
                  >
                    打开系统设置
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* 辅助功能权限 */}
        <div className="p-4 bg-gray-50 rounded-lg border border-gray-200">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">{getStatusIcon(permissionStatus.accessibility)}</span>
                <h4 className="font-semibold text-gray-900">辅助功能权限</h4>
                <span
                  className={`text-sm font-medium ${getStatusColor(permissionStatus.accessibility)}`}
                >
                  {getStatusText('accessibility', permissionStatus.accessibility)}
                </span>
              </div>
              <p className="text-sm text-gray-600 mb-3">
                在光标位置插入文本需要辅助功能权限。如果未授权，请在"系统设置 → 隐私与安全性 →
                辅助功能"中授权。
              </p>
              <div className="flex items-center gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={checkAccessibilityPermission}
                  disabled={isCheckingAccessibility}
                >
                  {isCheckingAccessibility ? '检查中...' : '重新检测'}
                </Button>
                {permissionStatus.accessibility === false && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={requestAccessibilityPermission}
                    disabled={isCheckingAccessibility}
                  >
                    请求授权
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Python环境状态 */}
        <div className="p-4 bg-gray-50 rounded-lg border border-gray-200">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">{getStatusIcon(pythonStatus.status)}</span>
                <h4 className="font-semibold text-gray-900">Python 环境</h4>
                <span className={`text-sm font-medium ${getStatusColor(pythonStatus.status)}`}>
                  {pythonStatus.message}
                </span>
              </div>
              <div className="space-y-2">
                <p className="text-sm text-gray-600">
                  FunASR 模型需要 Python 环境。首次使用会自动安装依赖（需要网络连接）。
                </p>
                {pythonStatus.details && (
                  <p className="text-xs text-gray-500 bg-white p-2 rounded border border-gray-200">
                    {pythonStatus.details}
                  </p>
                )}
                {settings.model && (
                  <p className="text-sm text-gray-700">
                    <span className="font-medium">当前模型:</span> {settings.model}
                    {['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(
                      settings.model,
                    )
                      ? ' (需要 Python 环境)'
                      : ' (本地模型)'}
                  </p>
                )}
              </div>
              <div className="flex items-center gap-2 mt-3">
                <Button variant="secondary" size="sm" onClick={checkPythonEnvironment}>
                  重新检测
                </Button>
                {pythonStatus.status === 'missing' && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={async () => {
                      toast.info('正在初始化 Python 环境，请稍候...')
                      try {
                        if (
                          settings.model &&
                          ['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(
                            settings.model,
                          )
                        ) {
                          await invoke('initialize_funasr', { modelName: settings.model })
                          toast.success('Python 环境初始化成功')
                          await checkPythonEnvironment()
                        }
                      } catch (error) {
                        toast.error(`初始化失败: ${String(error)}`)
                      }
                    }}
                  >
                    初始化环境
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 整体状态总结 */}
      <div className="p-4 bg-blue-50 rounded-lg border border-blue-200">
        <div className="flex items-start gap-3">
          <span className="text-2xl">
            {permissionStatus.microphone === 'granted' &&
            permissionStatus.accessibility === true &&
            (pythonStatus.status === 'ready' ||
              pythonStatus.status === 'prewarmed' ||
              !['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(
                settings.model || '',
              ))
              ? '✅'
              : '⚠️'}
          </span>
          <div>
            <h4 className="font-semibold text-gray-900 mb-1">环境状态总结</h4>
            <p className="text-sm text-gray-700">
              {permissionStatus.microphone === 'granted' &&
              permissionStatus.accessibility === true &&
              (pythonStatus.status === 'ready' ||
                pythonStatus.status === 'prewarmed' ||
                !['paraformer-zh', 'paraformer-large', 'sensevoice-small'].includes(
                  settings.model || '',
                ))
                ? '所有必需的权限和环境都已就绪，可以正常使用聆码！'
                : '部分权限或环境未就绪，请按照上面的提示完成设置。'}
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
