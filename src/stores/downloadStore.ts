import { create } from 'zustand'

export type DownloadStatus = 'idle' | 'downloading' | 'completed' | 'error'

interface DownloadProgress {
  progress: number // 0-100
  component?: string // 当前下载的组件名称（FunASR 使用）
  message?: string // 进度消息
  model_name?: string // 模型名称（Whisper 使用）
  downloaded?: number // 已下载字节数（Whisper 使用）
  total?: number // 总字节数（Whisper 使用）
}

interface DownloadStore {
  // 下载状态
  status: DownloadStatus
  modelName: string | null
  progress: number
  message: string
  error: string | null

  // Actions
  startDownload: (modelName: string) => void
  updateProgress: (progress: DownloadProgress) => void
  completeDownload: () => void
  failDownload: (error: string) => void
  resetDownload: () => void
}

export const useDownloadStore = create<DownloadStore>((set) => ({
  status: 'idle',
  modelName: null,
  progress: 0,
  message: '',
  error: null,

  startDownload: (modelName: string) => {
    set({
      status: 'downloading',
      modelName,
      progress: 0,
      message: '准备下载...',
      error: null,
    })
  },

  updateProgress: (progressData: DownloadProgress) => {
    set((state) => {
      // 构造消息
      let message = ''
      if (progressData.component && progressData.message) {
        // FunASR 格式
        message = `${progressData.component}: ${progressData.message}`
      } else if (
        progressData.downloaded !== undefined &&
        progressData.total !== undefined &&
        progressData.total > 0
      ) {
        // Whisper 格式
        const downloadedMB = (progressData.downloaded / 1024 / 1024).toFixed(1)
        const totalMB = (progressData.total / 1024 / 1024).toFixed(1)
        message = `已下载 ${downloadedMB}MB / ${totalMB}MB`
      } else if (progressData.progress === 0) {
        message = '开始下载...'
      } else if (progressData.progress === 100) {
        message = '下载完成'
      } else {
        message = '下载中...'
      }

      return {
        ...state,
        progress: progressData.progress,
        message,
      }
    })
  },

  completeDownload: () => {
    set({
      status: 'completed',
      progress: 100,
      message: '下载完成',
    })
  },

  failDownload: (error: string) => {
    set({
      status: 'error',
      error,
      message: '下载失败',
    })
  },

  resetDownload: () => {
    set({
      status: 'idle',
      modelName: null,
      progress: 0,
      message: '',
      error: null,
    })
  },
}))
