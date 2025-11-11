import { create } from 'zustand';

type Page = 'home' | 'notes' | 'settings';
type SettingsTab = 'environment' | 'general' | 'system' | 'model';

interface UIStore {
  // 当前页面
  currentPage: Page;
  setCurrentPage: (page: Page) => void;

  // 设置弹窗
  isSettingsOpen: boolean;
  openSettings: () => void;
  closeSettings: () => void;

  // 设置标签页
  settingsTab: SettingsTab;
  setSettingsTab: (tab: SettingsTab) => void;

  // 语言选择弹窗
  isLanguageSelectorOpen: boolean;
  openLanguageSelector: () => void;
  closeLanguageSelector: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  currentPage: 'home',
  setCurrentPage: (page) => set({ currentPage: page }),

  isSettingsOpen: false,
  openSettings: () => set({ isSettingsOpen: true, settingsTab: 'environment' }),
  closeSettings: () => set({ isSettingsOpen: false }),

  settingsTab: 'environment',
  setSettingsTab: (tab) => set({ settingsTab: tab }),

  isLanguageSelectorOpen: false,
  openLanguageSelector: () => set({ isLanguageSelectorOpen: true }),
  closeLanguageSelector: () => set({ isLanguageSelectorOpen: false }),
}));