import React, { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../../../stores'
import { Button, RadioGroup, RadioOption } from '../../../components'
import { useToast } from '../../../components'
import { getShortcutDisplayParts } from '../../../utils/shortcutFormatter'

interface AudioDevice {
  id: string
  name: string
}

export const GeneralSettings: React.FC = () => {
  const { settings, updateSetting } = useSettingsStore()
  const toast = useToast()
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([])
  const [loadingDevices, setLoadingDevices] = useState(false)
  const [showShortcutDialog, setShowShortcutDialog] = useState(false)
  const [recordingShortcut, setRecordingShortcut] = useState(false)

  // 获取快捷键显示部分
  const shortcutParts = useMemo(() => {
    return getShortcutDisplayParts(settings.shortcut || 'Cmd+Shift+S')
  }, [settings.shortcut])

  // 加载音频设备列表
  const loadAudioDevices = async () => {
    setLoadingDevices(true)
    try {
      const devices = await invoke<AudioDevice[]>('get_audio_devices')
      setAudioDevices(devices)
    } catch (error) {
      console.error('Failed to load audio devices:', error)
      toast.error(`加载麦克风列表失败: ${String(error)}`)
    } finally {
      setLoadingDevices(false)
    }
  }

  // 初始化时加载设备列表
  useEffect(() => {
    loadAudioDevices()
  }, [])

  // 处理语言更改
  const handleLanguageChange = async (language: string) => {
    try {
      await updateSetting('language', language)
      toast.success(`语言已切换为 ${language === 'zh' ? '中文' : '英语'}`)
    } catch (error) {
      toast.error(`切换语言失败: ${String(error)}`)
    }
  }

  // 处理快捷键录制
  const handleRecordShortcut = () => {
    setShowShortcutDialog(true)
    setRecordingShortcut(true)
  }

  // 语言选项
  const languageOptions: RadioOption[] = [
    {
      value: 'zh',
      label: '中文（简体）',
      description: '使用中文界面',
    },
    {
      value: 'en',
      label: 'English',
      description: 'Use English interface',
    },
  ]

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-semibold text-gray-900">通用设置</h3>

      {/* 键盘快捷键 */}
      <div>
        <h4 className="font-medium text-gray-900 mb-3">键盘快捷键</h4>
        <div className="p-4 bg-gray-50 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <p className="text-sm text-gray-600 flex items-center gap-1 flex-wrap mb-2">
                按住
                {shortcutParts.map((part, index) => (
                  <React.Fragment key={index}>
                    {index > 0 && <span>+</span>}
                    <kbd className="px-2 py-0.5 text-xs font-semibold text-gray-800 bg-white border border-gray-300 rounded">
                      {part.symbol} {part.name}
                    </kbd>
                  </React.Fragment>
                ))}
                并说话
              </p>
              <p className="text-xs text-gray-500">
                当前快捷键:{' '}
                <code className="px-1 py-0.5 bg-white rounded">{settings.shortcut}</code>
              </p>
            </div>
            <Button variant="secondary" size="sm" onClick={handleRecordShortcut}>
              更改
            </Button>
          </div>
        </div>
      </div>

      {/* 麦克风选择 */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h4 className="font-medium text-gray-900">麦克风</h4>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => void loadAudioDevices()}
            disabled={loadingDevices}
          >
            {loadingDevices ? '加载中...' : '🔄 刷新'}
          </Button>
        </div>
        {loadingDevices ? (
          <div className="text-center py-8 text-gray-500">加载中...</div>
        ) : audioDevices.length > 0 ? (
          <RadioGroup
            name="microphone"
            value={settings.microphone || 'auto'}
            onChange={async (value) => {
              try {
                // 先更新设置
                await updateSetting('microphone', value)
                // 然后设置音频录制器使用的设备
                await invoke('set_audio_device', { deviceId: value })
                toast.success('麦克风已切换')
              } catch (error) {
                toast.error(`切换麦克风失败: ${String(error)}`)
              }
            }}
            options={[
              {
                value: 'auto',
                label: '自动检测',
                description: '使用系统默认麦克风',
              },
              ...audioDevices.map((device) => ({
                value: device.id,
                label: device.name,
                description: `设备ID: ${device.id}`,
              })),
            ]}
          />
        ) : (
          <div className="p-4 bg-yellow-50 rounded-lg border border-yellow-200 text-sm text-yellow-800">
            ⚠️ 未检测到可用的麦克风设备
          </div>
        )}
      </div>

      {/* 语言选择 */}
      <div>
        <h4 className="font-medium text-gray-900 mb-3">语言 / Language</h4>
        <RadioGroup
          name="language"
          value={settings.language || 'zh'}
          onChange={handleLanguageChange}
          options={languageOptions}
        />
        <p className="text-xs text-gray-500 mt-2">💡 提示：更改语言后需要重启应用才能完全生效</p>
      </div>

      {/* 快捷键录制对话框 */}
      {showShortcutDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="absolute inset-0 bg-black bg-opacity-50"
            onClick={() => {
              setShowShortcutDialog(false)
              setRecordingShortcut(false)
            }}
          />
          <div className="relative bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">设置快捷键</h3>
            <div className="space-y-4">
              <div className="p-4 bg-gray-50 rounded-lg border-2 border-blue-500">
                <p className="text-sm text-gray-600 mb-2">
                  {recordingShortcut ? '请按下您想要设置的快捷键组合...' : '当前快捷键:'}
                </p>
                <div className="text-center">
                  <kbd className="px-4 py-2 text-lg font-semibold text-gray-800 bg-white border-2 border-gray-300 rounded-lg">
                    {settings.shortcut}
                  </kbd>
                </div>
              </div>
              <p className="text-xs text-gray-500">
                💡 建议使用包含 Cmd 或 Ctrl 的组合键，避免与其他应用冲突
              </p>
              <div className="flex items-center gap-2 justify-end">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => {
                    setShowShortcutDialog(false)
                    setRecordingShortcut(false)
                  }}
                >
                  取消
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => {
                    toast.info('快捷键录制功能即将推出')
                    setShowShortcutDialog(false)
                    setRecordingShortcut(false)
                  }}
                >
                  确定
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
