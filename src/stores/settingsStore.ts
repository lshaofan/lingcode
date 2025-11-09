import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import { invoke } from '@tauri-apps/api/core'

export interface Settings {
  language: string
  model: 'base' | 'small' | 'medium' | 'large'
  shortcut: string
  microphone: string
  theme: 'light' | 'dark' | 'auto'
  autoStart: boolean
  showInDock: boolean
  notifications: boolean
  autoDetectLanguage: boolean
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
  autoDetectLanguage: true,
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
