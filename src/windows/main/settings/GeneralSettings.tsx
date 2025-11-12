import React, { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../../../stores'
import { Button, RadioGroup, RadioOption } from '../../../components'
import { useToast } from '../../../components'
import { getShortcutDisplayParts } from '../../../utils/shortcutFormatter'
import { useShortcutRecorder } from '../../../hooks'
import { validateShortcut, getValidationMessage } from '../../../utils/shortcutValidator'

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
  const [showVerificationDialog, setShowVerificationDialog] = useState(false)
  const [newShortcut, setNewShortcut] = useState('')
  const {
    recording,
    shortcut: recordedShortcut,
    startRecording,
    stopRecording,
  } = useShortcutRecorder()

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
    void loadAudioDevices()
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
    startRecording()
  }

  // 验证录制的快捷键
  const validationResult = useMemo(() => {
    if (!recordedShortcut) {
      return null
    }
    return validateShortcut(recordedShortcut)
  }, [recordedShortcut])

  // 当录制完成时
  useEffect(() => {
    if (recordedShortcut && !recording) {
      // 录制完成,更新显示
      console.log('[GeneralSettings] Recorded shortcut:', recordedShortcut)
      console.log('[GeneralSettings] Validation result:', validationResult)
    }
  }, [recordedShortcut, recording, validationResult])

  // 保存快捷键
  const handleSaveShortcut = async () => {
    if (!recordedShortcut) {
      toast.error('请先录制快捷键')
      return
    }

    // 验证快捷键
    const validation = validateShortcut(recordedShortcut)
    if (!validation.isValid) {
      toast.error(validation.error || '快捷键无效')
      return
    }

    // 显示警告(如果有)
    if (validation.warning) {
      toast.warning(validation.warning)
    }

    try {
      // 先注销旧的快捷键
      await invoke('unregister_shortcuts')

      // 更新设置
      await updateSetting('shortcut', recordedShortcut)

      // 注册新的快捷键
      await invoke('register_shortcuts_cmd')

      // 保存新快捷键并显示验证对话框
      setNewShortcut(recordedShortcut)
      setShowShortcutDialog(false)
      stopRecording()

      // 延迟一下再显示验证对话框,确保快捷键已注册
      setTimeout(() => {
        setShowVerificationDialog(true)
      }, 300)
    } catch (error) {
      toast.error(`更新快捷键失败: ${String(error)}`)
      console.error('Failed to update shortcut:', error)
    }
  }

  // 完成验证
  const handleVerificationComplete = () => {
    setShowVerificationDialog(false)
    toast.success('快捷键验证完成,可以正常使用了!')
  }

  // 取消录制
  const handleCancelRecording = () => {
    setShowShortcutDialog(false)
    stopRecording()
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
            onChange={(value) => {
              void (async () => {
                try {
                  // 先更新设置
                  await updateSetting('microphone', value)
                  // 然后设置音频录制器使用的设备
                  await invoke('set_audio_device', { deviceId: value })
                  toast.success('麦克风已切换')
                } catch (error) {
                  toast.error(`切换麦克风失败: ${String(error)}`)
                }
              })()
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
          onChange={(lang) => void handleLanguageChange(lang)}
          options={languageOptions}
        />
        <p className="text-xs text-gray-500 mt-2">💡 提示：更改语言后需要重启应用才能完全生效</p>
      </div>

      {/* 快捷键录制对话框 */}
      {showShortcutDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="absolute inset-0 bg-black bg-opacity-50"
            onClick={handleCancelRecording}
          />
          <div className="relative bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">设置快捷键</h3>
            <div className="space-y-4">
              <div
                className={`p-4 bg-gray-50 rounded-lg border-2 ${recording ? 'border-blue-500 animate-pulse' : 'border-gray-300'}`}
              >
                <p className="text-sm text-gray-600 mb-2">
                  {recording ? '请按下您想要设置的快捷键组合...' : '录制的快捷键:'}
                </p>
                <div className="text-center">
                  <kbd className="px-4 py-2 text-lg font-semibold text-gray-800 bg-white border-2 border-gray-300 rounded-lg">
                    {recordedShortcut || settings.shortcut || '等待输入...'}
                  </kbd>
                </div>
              </div>

              {/* 显示当前快捷键 */}
              <div className="p-3 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-xs text-blue-800 mb-1">当前快捷键:</p>
                <code className="text-sm font-mono text-blue-900">{settings.shortcut}</code>
              </div>

              {/* 显示验证结果 */}
              {recordedShortcut && validationResult && (
                <div
                  className={`p-3 rounded-lg border ${
                    validationResult.error
                      ? 'bg-red-50 border-red-200'
                      : validationResult.warning
                        ? 'bg-yellow-50 border-yellow-200'
                        : 'bg-green-50 border-green-200'
                  }`}
                >
                  <p
                    className={`text-sm ${
                      validationResult.error
                        ? 'text-red-800'
                        : validationResult.warning
                          ? 'text-yellow-800'
                          : 'text-green-800'
                    }`}
                  >
                    {getValidationMessage(validationResult)}
                  </p>
                </div>
              )}

              <p className="text-xs text-gray-500">
                💡 提示:
                <br />
                • 快捷键格式: 修饰键 + 普通键(如 Cmd+Shift+S)
                <br />
                • 修饰键: Cmd、Ctrl、Alt、Shift
                <br />
                • 普通键: 字母、数字、功能键、方向键等
                <br />
                • ⚠️ 纯修饰键组合(如 Ctrl+Opt)由于 Tauri 框架限制暂不支持
                <br />• 建议使用 Cmd/Ctrl + Shift 的组合,减少与其他应用冲突
              </p>

              <div className="flex items-center gap-2 justify-end">
                <Button variant="secondary" size="sm" onClick={handleCancelRecording}>
                  取消
                </Button>
                {!recording && recordedShortcut && (
                  <Button variant="secondary" size="sm" onClick={startRecording}>
                    重新录制
                  </Button>
                )}
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => void handleSaveShortcut()}
                  disabled={
                    !recordedShortcut ||
                    recording ||
                    (validationResult !== null && !validationResult.isValid)
                  }
                >
                  {recording ? '录制中...' : '保存'}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 快捷键验证对话框 */}
      {showVerificationDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="absolute inset-0 bg-black bg-opacity-50" />
          <div className="relative bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">验证新快捷键</h3>
            <div className="space-y-4">
              <div className="p-4 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-sm text-blue-800 mb-2">新快捷键已设置:</p>
                <div className="text-center mb-3">
                  <kbd className="px-4 py-2 text-lg font-semibold text-gray-800 bg-white border-2 border-blue-400 rounded-lg">
                    {newShortcut}
                  </kbd>
                </div>
              </div>

              <div className="p-4 bg-green-50 rounded-lg border-2 border-green-300">
                <p className="text-sm text-green-800 mb-3 font-medium">✅ 快捷键已更新成功!</p>
                <div className="space-y-2 text-sm text-green-700">
                  <p>📝 请在外部应用中测试新快捷键:</p>
                  <ol className="list-decimal list-inside space-y-1 ml-2">
                    <li>打开浏览器、文本编辑器或其他应用</li>
                    <li>点击任意输入框使其获得焦点</li>
                    <li>
                      按下新设置的快捷键{' '}
                      <kbd className="px-1.5 py-0.5 bg-white border border-green-400 rounded">
                        {newShortcut}
                      </kbd>
                    </li>
                    <li>录制语音,转录的文字会自动插入到输入框中</li>
                  </ol>
                </div>
                <p className="text-xs text-green-700 mt-3">
                  💡
                  注意:由于技术限制,快捷键在本应用内的输入框中可能无法正常工作,请在外部应用中测试。
                </p>
              </div>

              <div className="p-3 bg-yellow-50 rounded-lg border border-yellow-200">
                <p className="text-xs text-yellow-800">
                  ⚠️ 提示: 如果快捷键无法触发录制窗口,请点击下方"重新设置"按钮
                </p>
              </div>

              <div className="flex items-center gap-2 justify-end">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => {
                    setShowVerificationDialog(false)
                    setShowShortcutDialog(true)
                    startRecording()
                  }}
                >
                  重新设置
                </Button>
                <Button variant="primary" size="sm" onClick={handleVerificationComplete}>
                  验证完成
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
