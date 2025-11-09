import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';

export interface Settings {
  language: string;
  model: string;
  shortcut: string;
  theme: 'light' | 'dark' | 'auto';
  autoStart: boolean;
  notifications: boolean;
}

interface SettingsStore {
  settings: Settings;
  loading: boolean;
  error: string | null;

  // Actions
  loadSettings: () => Promise<void>;
  updateSetting: <K extends keyof Settings>(
    key: K,
    value: Settings[K],
  ) => Promise<void>;
  resetSettings: () => Promise<void>;
}

const defaultSettings: Settings = {
  language: 'zh',
  model: 'base',
  shortcut: 'Cmd+Shift+S',
  theme: 'auto',
  autoStart: false,
  notifications: true,
};

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set, get) => ({
      settings: defaultSettings,
      loading: false,
      error: null,

      loadSettings: async () => {
        set({ loading: true, error: null });
        try {
          const allSettings = await invoke<Array<{ key: string; value: string }>>(
            'get_all_settings',
          );

          const settings = { ...defaultSettings };
          allSettings.forEach(({ key, value }) => {
            if (key in settings) {
              try {
                settings[key as keyof Settings] = JSON.parse(value);
              } catch {
                settings[key as keyof Settings] = value as any;
              }
            }
          });

          set({ settings, loading: false });
        } catch (error) {
          set({ error: String(error), loading: false });
        }
      },

      updateSetting: async (key, value) => {
        set({ loading: true, error: null });
        try {
          await invoke('set_setting', {
            key,
            value: JSON.stringify(value),
          });

          set((state) => ({
            settings: { ...state.settings, [key]: value },
            loading: false,
          }));
        } catch (error) {
          set({ error: String(error), loading: false });
        }
      },

      resetSettings: async () => {
        set({ loading: true, error: null });
        try {
          for (const key of Object.keys(defaultSettings)) {
            await invoke('set_setting', {
              key,
              value: JSON.stringify(defaultSettings[key as keyof Settings]),
            });
          }

          set({ settings: defaultSettings, loading: false });
        } catch (error) {
          set({ error: String(error), loading: false });
        }
      },
    }),
    {
      name: 'settings-storage',
      storage: createJSONStorage(() => localStorage),
    },
  ),
);
