import { create } from 'zustand'

interface LoadingTask {
  id: string
  message: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  error?: string
}

interface LoadingStore {
  // 全局初始化状态
  isInitializing: boolean
  initializationTasks: LoadingTask[]
  initializationProgress: number

  // 设置修改的加载状态
  isSettingLoading: boolean
  settingLoadingMessage: string

  // Actions
  setInitializing: (isInitializing: boolean) => void
  addInitializationTask: (task: LoadingTask) => void
  updateInitializationTask: (id: string, updates: Partial<LoadingTask>) => void
  completeInitialization: () => void

  setSettingLoading: (isLoading: boolean, message?: string) => void
}

export const useLoadingStore = create<LoadingStore>((set) => ({
  // Initial state
  isInitializing: true,
  initializationTasks: [],
  initializationProgress: 0,
  isSettingLoading: false,
  settingLoadingMessage: '',

  // Actions
  setInitializing: (isInitializing) => set({ isInitializing }),

  addInitializationTask: (task) => {
    set((state) => ({
      initializationTasks: [...state.initializationTasks, task],
    }))
  },

  updateInitializationTask: (id, updates) => {
    set((state) => {
      const tasks = state.initializationTasks.map((task) =>
        task.id === id ? { ...task, ...updates } : task,
      )

      // 计算进度
      const completedTasks = tasks.filter(
        (t) => t.status === 'completed' || t.status === 'failed',
      ).length
      const progress = tasks.length > 0 ? (completedTasks / tasks.length) * 100 : 0

      return {
        initializationTasks: tasks,
        initializationProgress: Math.round(progress),
      }
    })
  },

  completeInitialization: () => {
    set({
      isInitializing: false,
      initializationProgress: 100,
    })
  },

  setSettingLoading: (isLoading, message = '') => {
    set({
      isSettingLoading: isLoading,
      settingLoadingMessage: message,
    })
  },
}))
