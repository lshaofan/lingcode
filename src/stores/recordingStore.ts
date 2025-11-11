import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingsStore } from './settingsStore'

export type RecordingState = 'idle' | 'recording' | 'processing' | 'error'
export type OperationMode = 'direct' | 'preview'

interface RecordingStore {
  state: RecordingState
  duration: number
  transcription: string | null
  transcribedText: string // New: Current transcribed text for display
  error: string | null
  audioLevel: number
  operationMode: OperationMode // 当前操作模式

  // Actions
  startRecording: (skipBackendCall?: boolean) => Promise<void>
  stopRecording: () => Promise<void>
  cancelRecording: () => void
  setAudioLevel: (level: number) => void
  resetState: () => void
  setOperationMode: (mode: OperationMode) => void

  // New text actions
  clearText: () => void
  copyText: () => Promise<void>
  insertText: () => Promise<void>
  setTranscribedText: (text: string) => void
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  state: 'idle',
  duration: 0,
  transcription: null,
  transcribedText: '',
  error: null,
  audioLevel: 0,
  operationMode: 'preview', // 默认预览模式

  startRecording: async (skipBackendCall = false) => {
    // 🚀 CRITICAL FIX: 立即检查并设置状态，防止并发调用
    const currentState = get().state
    if (currentState === 'recording') {
      console.log('[RecordingStore] ⚠️  Already recording, ignoring duplicate call')
      return
    }

    console.log('🎤🎤🎤 [RecordingStore] ========== START RECORDING CALLED ==========')
    console.log('[RecordingStore] skipBackendCall:', skipBackendCall)

    try {
      console.log('[RecordingStore] Starting recording...')

      // 如果 skipBackendCall=true，说明后端已经启动录音了
      // 这种情况下只需要更新前端状态和启动计时器
      if (skipBackendCall) {
        console.log('[RecordingStore] ⚡ Skipping backend call - recording already started by shortcut handler')

        // 直接设置为 recording 状态
        set({ state: 'recording', error: null, transcription: null, duration: 0 })

        // 启动计时器
        const timer = setInterval(() => {
          set((state) => ({
            duration: state.duration + 0.1,
          }))
        }, 100)

        // Store timer ID for cleanup
        ;(window as any).__recordingTimer = timer
        return
      }

      // 正常流程：前端主动调用（用户点击按钮）
      // 1. 首先检查麦克风权限（在设置录音状态之前）
      console.log('[RecordingStore] Step 1: Checking microphone permission...')
      const permissionStatus = await invoke<string>('check_microphone_permission')
      console.log('[RecordingStore] ✅ Permission status:', permissionStatus)

      if (permissionStatus !== 'granted') {
        console.log('[RecordingStore] ⚠️  Permission not granted, requesting...')
        const newStatus = await invoke<string>('request_microphone_permission')
        console.log('[RecordingStore] New permission status:', newStatus)

        if (newStatus !== 'granted') {
          console.error('[RecordingStore] ❌ Permission denied')
          set({ state: 'error', error: '❌ 麦克风权限未授权\n\n请在系统设置中允许访问麦克风：\n系统设置 > 隐私与安全性 > 麦克风' })
          // 抛出错误，阻止悬浮窗显示
          throw new Error('Microphone permission denied')
        }
      }

      // 2. 权限检查通过后，设置为 recording 状态
      console.log('[RecordingStore] Setting state to recording...')
      set({ state: 'recording', error: null, transcription: null, duration: 0 })

      // 3. 开始音频录制
      console.log('[RecordingStore] Step 2: Calling backend start_recording command...')
      await invoke('start_recording')
      console.log('[RecordingStore] ✅✅✅ Recording started successfully!')

      // 3. 启动计时器
      const timer = setInterval(() => {
        set((state) => ({
          duration: state.duration + 0.1,
        }))
      }, 100)

      // Store timer ID for cleanup
      ;(window as any).__recordingTimer = timer
    } catch (error) {
      console.error('[RecordingStore] Failed to start recording:', error)
      set({ state: 'error', error: String(error) })
    }
  },

  stopRecording: async () => {
    try {
      console.log('[RecordingStore] ⭐ stopRecording called, current state:', get().state)

      // 不管当前状态是什么，都尝试停止录音（因为后台可能在录音）
      // 这可以处理由于重复调用导致的状态不一致问题

      // Clear timer
      if ((window as any).__recordingTimer) {
        clearInterval((window as any).__recordingTimer)
        delete (window as any).__recordingTimer
      }

      const recordingDuration = get().duration
      set({ state: 'processing' })

      // 1. 停止录音
      console.log('[RecordingStore] Step 1: Calling stop_recording command...')
      const sampleCount = await invoke<number>('stop_recording')
      console.log('[RecordingStore] ✅ Recording stopped, captured', sampleCount, 'samples')

      // 2. 重新加载设置，确保使用最新的模型配置（避免跨窗口同步问题）
      console.log('[RecordingStore] Step 2: Reloading settings from backend...')
      await useSettingsStore.getState().loadSettings()

      const settings = useSettingsStore.getState().settings
      console.log('[RecordingStore] ✅ Settings reloaded. Current model:', settings.model)

      // 默认使用中文，除非明确设置为其他语言
      // 由于 Whisper 自动检测对中文支持不够好，我们默认强制使用中文
      let language: string | null = settings.language || 'zh'

      // 只有在明确设置自动检测且语言不是中文时才使用自动检测
      if (settings.autoDetectLanguage && settings.language !== 'zh') {
        language = null
      } else if (!settings.language || settings.language === 'zh') {
        // 如果没有设置语言或者设置为中文，强制使用中文
        language = 'zh'
      }

      const modelVersion = settings.model || 'base'

      console.log('[RecordingStore] Step 3: Settings - Model:', modelVersion, 'Language:', language || 'auto')
      console.log('[RecordingStore] Auto-detect:', settings.autoDetectLanguage, 'Configured language:', settings.language)

      // 判断模型类型
      const funasrModels = ['paraformer-zh', 'paraformer-large', 'sensevoice-small']
      const isFunASR = funasrModels.includes(modelVersion)

      let transcriptionText: string

      if (isFunASR) {
        // 3.5 确保 FunASR 引擎已初始化
        console.log('[RecordingStore] Step 3.5: Initializing FunASR engine with model:', modelVersion)
        try {
          await invoke('initialize_funasr', { modelName: modelVersion })
          console.log('[RecordingStore] ✅ FunASR engine initialized')
        } catch (error) {
          console.error('[RecordingStore] Failed to initialize FunASR engine:', error)
          // 如果初始化失败，尝试继续（可能已经初始化了）
        }

        // 4. 调用 FunASR 转录
        console.log('[RecordingStore] Step 4: Calling transcribe_last_recording_funasr...')
        transcriptionText = await invoke<string>('transcribe_last_recording_funasr', {
          language: language,
        })
      } else {
        // 3.5 确保 Whisper 引擎已初始化（使用当前选中的模型）
        console.log('[RecordingStore] Step 3.5: Initializing Whisper engine with model:', modelVersion)
        try {
          await invoke('initialize_whisper', { modelName: modelVersion })
          console.log('[RecordingStore] ✅ Whisper engine initialized')
        } catch (error) {
          console.error('[RecordingStore] Failed to initialize Whisper engine:', error)
          // 如果初始化失败，尝试继续（可能已经初始化了）
        }

        // 4. 调用 Whisper 转录
        console.log('[RecordingStore] Step 4: Calling transcribe_last_recording...')
        transcriptionText = await invoke<string>('transcribe_last_recording', {
          language: language,
        })
      }

      console.log('[RecordingStore] ✅ Transcription result:', transcriptionText)
      console.log('[RecordingStore] Transcription result type:', typeof transcriptionText)
      console.log('[RecordingStore] Transcription result length:', transcriptionText?.length)

      // 5. 保存转录到数据库
      console.log('[RecordingStore] Step 5: Saving transcription to database...')
      await invoke('create_transcription', {
        transcription: {
          text: transcriptionText,
          audio_duration: recordingDuration,
          model_version: modelVersion,
          language: language || 'auto',
          created_at: new Date().toISOString(),
          app_context: null,
        },
      })

      // 根据操作模式决定后续行为（直接从 settingsStore 读取，确保同步）
      const mode = settings.operationMode || 'preview'
      console.log('[RecordingStore] Operation mode from settings:', mode)

      if (mode === 'direct') {
        // 直接插入模式：转录完成后保持 processing 状态，显示"正在插入..."
        console.log('[RecordingStore] Direct mode: keeping processing state for text insertion')
        set({
          state: 'processing', // 保持 processing 状态
          transcription: transcriptionText,
          transcribedText: '正在插入文本...', // 显示插入中的提示
          duration: 0,
          audioLevel: 0,
        })

        // 插入文本（后端会自动激活原应用）
        console.log('[RecordingStore] Inserting text...')
        await get().insertText()
        console.log('[RecordingStore] ✅ Text inserted successfully')

        // 插入完成后隐藏窗口
        const window = getCurrentWindow()
        await window.hide()

        // 重置状态
        set({
          state: 'idle',
          transcribedText: '',
        })
      } else {
        // 预览模式：设置为 idle 状态，保持窗口显示，等待用户操作
        set({
          state: 'idle',
          transcription: transcriptionText,
          transcribedText: transcriptionText,
          duration: 0,
          audioLevel: 0,
        })
      }
    } catch (error) {
      console.error('[RecordingStore] Transcription error:', error)

      // 根据操作模式处理错误（直接从 settingsStore 读取，确保同步）
      const settings = useSettingsStore.getState().settings
      const mode = settings.operationMode || 'preview'
      if (mode === 'direct') {
        // 直接插入模式：转录失败时隐藏窗口，不打扰用户
        console.log('[RecordingStore] Direct mode: hiding window on transcription error')
        const window = getCurrentWindow()
        await window.hide()
        set({ state: 'idle', error: null, transcribedText: '' })
      } else {
        // 预览模式：显示错误信息
        set({ state: 'error', error: String(error) })
      }
    }
  },

  cancelRecording: async () => {
    // Clear timer
    if ((window as any).__recordingTimer) {
      clearInterval((window as any).__recordingTimer)
      delete (window as any).__recordingTimer
    }

    // Call Tauri command to stop recording and clear buffer
    try {
      await invoke('stop_recording')
      await invoke('clear_audio_buffer')
    } catch (error) {
      console.error('[RecordingStore] Failed to cancel recording:', error)
    }

    set({
      state: 'idle',
      duration: 0,
      transcription: null,
      transcribedText: '',
      error: null,
      audioLevel: 0,
    })
  },

  setAudioLevel: (level: number) => {
    set({ audioLevel: level })
  },

  resetState: () => {
    set({
      state: 'idle',
      duration: 0,
      transcription: null,
      transcribedText: '',
      error: null,
      audioLevel: 0,
    })
  },

  setOperationMode: (mode: OperationMode) => {
    set({ operationMode: mode })
  },

  // New text actions
  setTranscribedText: (text: string) => {
    set({ transcribedText: text })
  },

  clearText: () => {
    set({ transcribedText: '', state: 'idle' })
  },

  copyText: async () => {
    const text = get().transcribedText
    if (text) {
      try {
        await writeText(text)
        console.log('[RecordingStore] Text copied to clipboard')
      } catch (error) {
        console.error('[RecordingStore] Failed to copy text:', error)
        set({ error: 'Failed to copy text to clipboard' })
      }
    }
  },

  insertText: async () => {
    const text = get().transcribedText
    if (text) {
      try {
        console.log('[RecordingStore] Checking accessibility permission...')

        // 检查辅助功能权限
        const hasPermission = await invoke<boolean>('check_accessibility_permission_cmd')
        console.log('[RecordingStore] Accessibility permission:', hasPermission)

        if (!hasPermission) {
          console.log('[RecordingStore] Requesting accessibility permission...')
          await invoke('request_accessibility_permission_cmd')
          set({ error: '需要辅助功能权限才能插入文本。请在系统设置中授权。' })
          return
        }

        // 插入文本
        console.log('[RecordingStore] Inserting text:', text)
        await invoke('insert_text_at_cursor_cmd', { text })
        console.log('[RecordingStore] Text inserted successfully')
      } catch (error) {
        console.error('[RecordingStore] Failed to insert text:', error)
        set({ error: `插入文本失败: ${String(error)}` })
      }
    }
  },
}))
