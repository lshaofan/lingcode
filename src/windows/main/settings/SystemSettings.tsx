import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../../stores'
import { Toggle, RadioGroup, type RadioOption } from '../../../components'
import { useToast } from '../../../components'

export const SystemSettings: React.FC = () => {
  const { settings, updateSetting } = useSettingsStore()
  const toast = useToast()
  const [loading, setLoading] = useState(false)

  // 加载开机自启状态
  useEffect(() => {
    const loadAutoLaunchStatusAsync = async () => {
      await loadAutoLaunchStatus()
    }
    void loadAutoLaunchStatusAsync()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const loadAutoLaunchStatus = async () => {
    try {
      const status = await invoke<boolean>('get_auto_launch')
      if (status !== settings.autoStart) {
        await updateSetting('autoStart', status)
      }
    } catch (error) {
      console.error('Failed to load auto launch status:', error)
    }
  }

  const handleAutoStartChange = async (enabled: boolean) => {
    setLoading(true)
    try {
      await invoke('set_auto_launch', { enable: enabled })
      await updateSetting('autoStart', enabled)
      toast.success(enabled ? '已启用开机自启动' : '已禁用开机自启动')
    } catch (error) {
      toast.error(`设置失败: ${String(error)}`)
      console.error('Failed to set auto launch:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleShowInDockChange = async (enabled: boolean) => {
    setLoading(true)
    try {
      await updateSetting('showInDock', enabled)
      toast.success(enabled ? '应用将在 Dock 中显示' : '应用将仅在托盘显示')
    } catch (error) {
      toast.error(`设置失败: ${String(error)}`)
      console.error('Failed to set show in dock:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleOperationModeChange = async (value: string) => {
    const mode = value as 'direct' | 'preview'
    setLoading(true)
    try {
      await updateSetting('operationMode', mode)
      // 通知其他窗口设置已更新
      await emit('settings-updated', { key: 'operationMode', value: mode })
      console.log('[SystemSettings] Emitted settings-updated event for operationMode:', mode)
      const modeText = mode === 'direct' ? '直接插入模式' : '预览确认模式'
      toast.success(`已切换到${modeText}`)
    } catch (error) {
      toast.error(`切换模式失败: ${String(error)}`)
      console.error('Failed to change operation mode:', error)
    } finally {
      setLoading(false)
    }
  }

  const operationModeOptions: RadioOption[] = [
    {
      value: 'direct',
      label: '直接插入模式',
      description: '录制完成后自动插入文字到当前应用，适合快速输入场景',
    },
    {
      value: 'preview',
      label: '预览确认模式（推荐）',
      description: '录制完成后先预览文字，由您决定是否插入，更加安全可控',
    },
  ]

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-semibold text-gray-900">系统设置</h3>

      {/* 操作模式 */}
      <div>
        <h4 className="text-sm font-medium text-gray-700 mb-3">操作模式</h4>
        <RadioGroup
          name="operationMode"
          value={settings.operationMode}
          onChange={(value) => void handleOperationModeChange(value)}
          options={operationModeOptions.map((opt) => ({ ...opt, disabled: loading }))}
        />
      </div>

      <div>
        <h4 className="text-sm font-medium text-gray-500 mb-3">App settings</h4>

        <div className="space-y-4">
          {/* 开机自动启动 */}
          <div className="p-4 bg-gray-50 rounded-lg flex items-center justify-between">
            <div className="flex-1">
              <div className="font-medium text-gray-900">开机自动启动</div>
              <div className="text-sm text-gray-500 mt-1">Launch app at login</div>
            </div>
            <Toggle
              checked={settings.autoStart}
              onChange={(enabled) => void handleAutoStartChange(enabled)}
              disabled={loading}
            />
          </div>

          {/* 在 Dock 中显示 */}
          <div className="p-4 bg-gray-50 rounded-lg flex items-center justify-between">
            <div className="flex-1">
              <div className="font-medium text-gray-900">在 Dock 中显示</div>
              <div className="text-sm text-gray-500 mt-1">Show app in dock</div>
            </div>
            <Toggle
              checked={settings.showInDock}
              onChange={(enabled) => void handleShowInDockChange(enabled)}
              disabled={loading}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
