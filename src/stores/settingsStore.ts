import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'

export interface Settings {
  [key: string]: any // 添加索引签名
  language: string
  model:
    | 'base'
    | 'small'
    | 'medium'
    | 'large'
    | 'paraformer-zh'
    | 'paraformer-large'
    | 'sensevoice-small'
  shortcut: string
  microphone: string
  theme: 'light' | 'dark' | 'auto'
  autoStart: boolean
  showInDock: boolean
  notifications: boolean
  autoDetectLanguage: boolean
  operationMode: 'direct' | 'preview'
}

interface SettingsStore {
  settings: Settings
  loading: boolean
  error: string | null

  // Actions
  loadSettings: () => Promise<void>
  updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => Promise<void>
  resetSettings: () => Promise<void>
}

const defaultSettings: Settings = {
  language: 'zh',
  model: 'base',
  shortcut: 'Cmd+Shift+S',
  microphone: 'auto',
  theme: 'auto',
  autoStart: false,
  showInDock: true,
  notifications: true,
  autoDetectLanguage: false, // 默认关闭自动检测，强制使用中文
  operationMode: 'preview',
}

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      settings: defaultSettings,
      loading: false,
      error: null,

      loadSettings: async () => {
        set({ loading: true, error: null })
        try {
          const allSettings =
            await invoke<Array<{ key: string; value: string }>>('get_all_settings')

          const settings: Settings = { ...defaultSettings }
          allSettings.forEach(({ key, value }) => {
            if (key in settings) {
              try {
                const parsedValue = JSON.parse(value) as unknown
                // 使用类型安全的属性赋值
                const settingsRecord = settings as Record<string, unknown>
                if (key in settingsRecord) {
                  settingsRecord[key] = parsedValue
                }
              } catch {
                // 如果JSON解析失败，直接使用字符串值
                const settingsRecord = settings as Record<string, unknown>
                if (key in settingsRecord) {
                  settingsRecord[key] = value
                }
              }
            }
          })

          set({ settings, loading: false })
        } catch (error) {
          set({ error: String(error), loading: false })
        }
      },

      updateSetting: async (key, value) => {
        set({ loading: true, error: null })
        try {
          await invoke('set_setting', {
            key,
            value: JSON.stringify(value),
          })

          set((state) => ({
            settings: { ...state.settings, [key]: value },
            loading: false,
          }))

          // 发送跨窗口事件，通知其他窗口设置已更新
          await emit('settings-updated', { key, value })
          console.log('[SettingsStore] Emitted settings-updated event:', key, value)
        } catch (error) {
          set({ error: String(error), loading: false })
        }
      },

      resetSettings: async () => {
        set({ loading: true, error: null })
        try {
          for (const key of Object.keys(defaultSettings)) {
            await invoke('set_setting', {
              key,
              value: JSON.stringify(defaultSettings[key as keyof Settings]),
            })
          }

          set({ settings: defaultSettings, loading: false })
        } catch (error) {
          set({ error: String(error), loading: false })
        }
      },
    }),
    {
      name: 'settings-storage',
      storage: createJSONStorage(() => localStorage),
    },
  ),
)
