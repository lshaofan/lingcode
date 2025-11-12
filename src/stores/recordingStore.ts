import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingsStore } from './settingsStore'
import { AudioCapture, AudioConverter } from '../lib/audioCapture'
import { audioFeedback } from '../lib/audioFeedback'
import type { InlineToastType } from '../components/InlineToast'

export type RecordingState = 'idle' | 'recording' | 'processing' | 'error'
export type OperationMode = 'direct' | 'preview'

// 气泡提示状态
export interface ToastState {
  type: InlineToastType
  message: string
  dismissible: boolean // 是否可手动关闭
  duration: number // 自动消失时长（毫秒），0 表示不自动消失
}

// 扩展 Window 接口以支持自定义属性
declare global {
  interface Window {
    __recordingTimer?: number
  }
}

// 🔥 引擎初始化缓存：记录已初始化的引擎和模型，避免重复初始化
// 格式: { engineType: modelName } 例如: { whisper: 'base', funasr: 'paraformer-zh' }
const engineInitCache: { [key: string]: string } = {}

interface RecordingStore {
  state: RecordingState
  duration: number
  transcription: string | null
  transcribedText: string // New: Current transcribed text for display
  error: string | null
  audioLevel: number
  operationMode: OperationMode // 当前操作模式
  audioCapture: AudioCapture | null // 前端音频采集实例

  // 气泡提示状态
  toast: ToastState | null
  isFirstRecording: boolean // 是否首次录音（用于首次初始化提示）
  hasShownLongAudioTip: boolean // 是否已显示长音频提示

  // Actions
  prewarmRecording: () => Promise<void> // 预热：提前初始化 getUserMedia
  startRecording: (skipBackendCall?: boolean, cachedInstance?: AudioCapture | null) => Promise<void>
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

  // Toast actions
  showToast: (
    type: InlineToastType,
    message: string,
    dismissible?: boolean,
    duration?: number,
  ) => void
  clearToast: () => void
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  state: 'idle',
  duration: 0,
  transcription: null,
  transcribedText: '',
  error: null,
  audioLevel: 0,
  operationMode: 'preview', // 默认预览模式
  audioCapture: null, // 音频采集实例
  toast: null,
  isFirstRecording: true,
  hasShownLongAudioTip: false,

  prewarmRecording: async () => {
    console.log('[RecordingStore] 🔥🔥🔥 ========== PREWARM RECORDING CALLED ==========')

    try {
      // 🔑 关键：检查是否已经在录音或已有实例
      const currentState = get().state
      const existingCapture = get().audioCapture

      if (currentState === 'recording' || existingCapture) {
        console.log('[RecordingStore] ⚠️  Already recording or instance exists, skipping prewarm')
        console.log('[RecordingStore] State:', currentState, 'Has instance:', !!existingCapture)
        return
      }

      // 🔑 关键修复：先清理所有可能泄漏的旧实例（仅在 idle 状态）
      AudioCapture.cleanupAllInstances()

      // 检查浏览器支持
      if (!AudioCapture.isSupported()) {
        throw new Error('您的浏览器不支持音频录制功能。请使用最新版本的浏览器。')
      }

      // 创建音频采集实例
      console.log('[RecordingStore] Step 1: Creating AudioCapture instance...')
      const audioCapture = new AudioCapture({
        sampleRate: 16000,
        channelCount: 1,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      })

      // 🔥 预热：初始化 getUserMedia 和 MediaRecorder，但不开始录音
      console.log(
        '[RecordingStore] Step 2: Prewarming AudioCapture (getUserMedia + MediaRecorder)...',
      )
      try {
        await audioCapture.prewarm()
        console.log('[RecordingStore] 🔥✅✅✅ Prewarm completed! Ready to record instantly!')
      } catch (error) {
        // 处理权限被拒绝的情况
        if (error instanceof Error && error.name === 'NotAllowedError') {
          console.error('[RecordingStore] ❌ Microphone permission denied during prewarm')
          set({
            state: 'error',
            error: '❌ 麦克风权限未授权\\n\\n请在浏览器中允许访问麦克风，然后重试。',
          })
          throw new Error('Microphone permission denied')
        }
        throw error
      }

      // 保存预热的实例
      console.log('[RecordingStore] Step 3: Saving prewarmed instance...')
      set({ audioCapture: audioCapture })
      console.log('[RecordingStore] 🔥 Prewarm完成，实例已保存，等待 startRecording 调用')
    } catch (error) {
      console.error('[RecordingStore] Prewarm failed:', error)
      set({ state: 'error', error: String(error) })
    }
  },

  startRecording: async (skipBackendCall = false, cachedInstance = null) => {
    console.log('🎤🎤🎤 [RecordingStore] ========== START RECORDING CALLED ==========')
    console.log('[RecordingStore] Has cached instance from component:', !!cachedInstance)

    // 🚀 CRITICAL FIX: 立即检查并设置状态，防止并发调用
    const currentState = get().state
    if (currentState === 'recording') {
      console.log('[RecordingStore] ⚠️  Already recording, ignoring duplicate call')
      return
    }

    console.log('[RecordingStore] skipBackendCall:', skipBackendCall)

    try {
      console.log('[RecordingStore] Starting recording...')

      // 🔥 优先级策略：
      // 1. 优先使用组件传入的缓存实例（AudioCacheManager）
      // 2. 其次使用 store 中保存的预热实例（旧的 prewarmRecording）
      // 3. 最后进行冷启动创建新实例
      let audioCapture = cachedInstance || get().audioCapture
      if (audioCapture) {
        const source = cachedInstance
          ? 'AudioCacheManager (component-level)'
          : 'store prewarmRecording'
        console.log(
          `[RecordingStore] ⚡⚡⚡ Using cached AudioCapture from ${source} - INSTANT START!`,
        )
        // 直接开始录音（MediaRecorder 已经就绪）
        await audioCapture.start()
        console.log('[RecordingStore] ✅✅✅ Recording started instantly (cached)!')
      } else {
        console.log('[RecordingStore] ⚠️  No cached instance, using cold start...')

        // 🔑 关键修复：清理所有可能泄漏的旧实例（热重载场景）
        AudioCapture.cleanupAllInstances()

        // 检查浏览器是否支持 getUserMedia
        if (!AudioCapture.isSupported()) {
          throw new Error('您的浏览器不支持音频录制功能。请使用最新版本的浏览器。')
        }

        // 创建音频采集实例
        console.log('[RecordingStore] Step 1: Creating AudioCapture instance...')
        audioCapture = new AudioCapture({
          sampleRate: 16000,
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        })

        // 开始音频采集（这会触发浏览器请求麦克风权限）
        console.log(
          '[RecordingStore] Step 2: Starting audio capture (this will trigger mic permission)...',
        )
        try {
          await audioCapture.start()
          console.log(
            '[RecordingStore] ✅✅✅ Audio capture started! Microphone indicator should be active!',
          )
        } catch (error) {
          // 处理权限被拒绝的情况
          if (error instanceof Error && error.name === 'NotAllowedError') {
            console.error('[RecordingStore] ❌ Microphone permission denied by user')
            const errorMessage = '麦克风权限未授权\n\n请在浏览器中允许访问麦克风，然后重试。'

            // 显示权限错误提示（严重错误，需要手动关闭）
            get().showToast('error', errorMessage, true, 0)

            set({
              state: 'error',
              error: `❌ ${errorMessage}`,
            })
            throw new Error('Microphone permission denied')
          }
          throw error
        }
      }

      // 4. 检查是否首次录音，显示初始化提示
      const isFirst = get().isFirstRecording
      if (isFirst) {
        console.log('[RecordingStore] First recording detected, showing initialization tip')
        get().showToast('info', '首次启动需要初始化，请稍候...', false, 3000)
      }

      // 5. 保存音频采集实例并设置状态
      console.log('[RecordingStore] Step 3: Setting recording state...')
      set({
        state: 'recording',
        error: null,
        transcription: null,
        duration: 0,
        audioCapture: audioCapture,
        isFirstRecording: false, // 标记已完成首次录音
        hasShownLongAudioTip: false, // 重置长音频提示标记
      })

      // 6. 启动计时器
      const timer = setInterval(() => {
        set((state) => ({
          duration: state.duration + 0.1,
        }))
      }, 100)

      // Store timer ID for cleanup
      window.__recordingTimer = timer

      console.log(
        '[RecordingStore] 🎉 Recording started successfully using frontend audio capture!',
      )

      // 🔊 播放开始录制音效（异步，不阻塞主流程）
      audioFeedback.playStart().catch((err) => {
        console.warn('[RecordingStore] Failed to play start sound:', err)
      })
    } catch (error) {
      console.error('[RecordingStore] Failed to start recording:', error)
      set({ state: 'error', error: String(error) })
    }
  },

  stopRecording: async () => {
    try {
      console.log('[RecordingStore] ⭐⭐⭐ stopRecording called, current state:', get().state)

      // Clear timer
      if (window.__recordingTimer) {
        clearInterval(window.__recordingTimer)
        delete window.__recordingTimer
      }

      const recordingDuration = get().duration
      console.log('[RecordingStore] 🔵 Setting state to PROCESSING (before transcription)')
      set({ state: 'processing' })
      console.log('[RecordingStore] 🔵 State set to PROCESSING, new state:', get().state)

      // 检查录音时长，如果超过20秒且未显示过提示，则显示长音频提示
      const settings = useSettingsStore.getState().settings
      const mode = settings.operationMode || 'preview'
      if (mode === 'direct' && recordingDuration > 20 && !get().hasShownLongAudioTip) {
        console.log('[RecordingStore] Long audio detected (>20s), showing tip')
        // 长音频提示不自动消失（duration=0），直到转录完成后窗口隐藏时一起清除
        get().showToast('info', '音频较长，转录可能需要更多时间...', false, 0)
        set({ hasShownLongAudioTip: true })
      }

      // 🎯 新实现：从前端音频采集获取数据
      // 1. 停止录音并获取音频数据
      const audioCapture = get().audioCapture
      if (!audioCapture) {
        throw new Error('No audio capture instance found')
      }

      console.log('[RecordingStore] Step 1: Stopping audio capture and getting audio data...')
      const audioBlob = await audioCapture.stop()
      console.log('[RecordingStore] ✅ Audio blob received:', audioBlob.size, 'bytes')

      // 🔑 关键修复：stop() 会在内部调用 cleanup()，所以这里可以安全地清除引用
      // 但我们需要确保 stop() 完成后再清除引用
      console.log('[RecordingStore] ✅ AudioCapture cleanup completed by stop() method')
      set({ audioCapture: null })

      // 2. 将音频转换为 PCM16 格式（后端需要的格式）
      console.log('[RecordingStore] Step 2: Converting audio to PCM16 format...')
      const pcm16Bytes = await AudioConverter.blobToPCM16(audioBlob, 16000)
      console.log('[RecordingStore] ✅ Converted to PCM16:', pcm16Bytes.length, 'bytes')

      // 🎯 关键：将 Uint8Array (字节) 转换为 i16[] (样本)
      // PCM16 是小端序，每 2 个字节组成一个 i16 样本
      const pcm16Samples = new Int16Array(
        pcm16Bytes.buffer,
        pcm16Bytes.byteOffset,
        pcm16Bytes.length / 2,
      )
      console.log(
        '[RecordingStore] ✅ Converted to samples:',
        pcm16Samples.length,
        'samples (',
        (pcm16Samples.length / 16000).toFixed(2),
        'seconds )',
      )

      // 3. 使用之前已经读取的 settings（不重新加载，避免不必要的网络请求）
      console.log('[RecordingStore] Step 3: Using cached settings...')
      console.log('[RecordingStore] ✅ Using cached settings. Current model:', settings.model)

      // 默认使用中文，除非明确设置为其他语言
      let language: string | null = settings.language || 'zh'

      // 只有在明确设置自动检测且语言不是中文时才使用自动检测
      if (settings.autoDetectLanguage && settings.language !== 'zh') {
        language = null
      } else if (!settings.language || settings.language === 'zh') {
        language = 'zh'
      }

      const modelVersion = settings.model || 'base'

      console.log(
        '[RecordingStore] Step 4: Settings - Model:',
        modelVersion,
        'Language:',
        language || 'auto',
      )
      console.log(
        '[RecordingStore] Auto-detect:',
        settings.autoDetectLanguage,
        'Configured language:',
        settings.language,
      )

      // 判断模型类型
      const funasrModels = ['paraformer-zh', 'paraformer-large', 'sensevoice-small']
      const isFunASR = funasrModels.includes(modelVersion)

      let transcriptionText: string

      // 4. 调用转录
      if (isFunASR) {
        // 🔥 缓存检查：仅在模型变化时才重新初始化
        const cachedModel = engineInitCache['funasr']
        if (cachedModel !== modelVersion) {
          console.log(
            '[RecordingStore] Step 5: Initializing FunASR engine with model:',
            modelVersion,
          )
          console.log('[RecordingStore] Previous cached model:', cachedModel || 'none')
          try {
            await invoke('initialize_funasr', { modelName: modelVersion })
            engineInitCache['funasr'] = modelVersion // 更新缓存
            console.log('[RecordingStore] ✅ FunASR engine initialized and cached')
          } catch (error) {
            console.error('[RecordingStore] Failed to initialize FunASR engine:', error)
          }
        } else {
          console.log('[RecordingStore] ⚡ Using cached FunASR engine (model:', modelVersion, ')')
        }

        // 调用新的转录命令（接收前端音频数据）
        console.log(
          '[RecordingStore] Step 6: Calling transcribe_audio_funasr with frontend audio data...',
        )
        transcriptionText = await invoke<string>('transcribe_audio_funasr', {
          audioData: Array.from(pcm16Samples),
          language: language,
        })
      } else {
        // 🔥 缓存检查：仅在模型变化时才重新初始化
        const cachedModel = engineInitCache['whisper']
        if (cachedModel !== modelVersion) {
          console.log(
            '[RecordingStore] Step 5: Initializing Whisper engine with model:',
            modelVersion,
          )
          console.log('[RecordingStore] Previous cached model:', cachedModel || 'none')
          try {
            await invoke('initialize_whisper', { modelName: modelVersion })
            engineInitCache['whisper'] = modelVersion // 更新缓存
            console.log('[RecordingStore] ✅ Whisper engine initialized and cached')
          } catch (error) {
            console.error('[RecordingStore] Failed to initialize Whisper engine:', error)
          }
        } else {
          console.log('[RecordingStore] ⚡ Using cached Whisper engine (model:', modelVersion, ')')
        }

        // 调用新的转录命令（接收前端音频数据）
        console.log('[RecordingStore] Step 6: Calling transcribe_audio with frontend audio data...')
        transcriptionText = await invoke<string>('transcribe_audio', {
          audioData: Array.from(pcm16Samples),
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

      // 根据操作模式决定后续行为（使用之前已读取的 mode）
      console.log('[RecordingStore] Operation mode from settings:', mode)

      if (mode === 'direct') {
        // 直接插入模式：转录完成后保持 processing 状态，显示"正在插入..."
        console.log(
          '[RecordingStore] 🔵🔵🔵 Direct mode: keeping processing state for text insertion',
        )
        console.log('[RecordingStore] Transcription text:', transcriptionText)
        set({
          state: 'processing', // 保持 processing 状态
          transcription: transcriptionText,
          transcribedText: '正在插入文本...', // 显示插入中的提示
          duration: 0,
          audioLevel: 0,
        })
        console.log(
          '[RecordingStore] 🔵 State updated, current state:',
          get().state,
          'transcribedText:',
          get().transcribedText,
        )

        // 插入文本（后端会自动激活原应用）
        console.log('[RecordingStore] Inserting text...')
        await get().insertText()
        console.log('[RecordingStore] ✅ Text inserted successfully')

        // 🔊 播放完成音效（在隐藏窗口之前播放）
        audioFeedback.playOk().catch((err) => {
          console.warn('[RecordingStore] Failed to play ok sound:', err)
        })

        // 清除所有提示（包括长音频提示）
        get().clearToast()

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
      const errorSettings = useSettingsStore.getState().settings
      const errorMode = errorSettings.operationMode || 'preview'
      const errorMessage = `转录失败: ${String(error)}`

      if (errorMode === 'direct') {
        // 直接插入模式：显示错误提示（一般错误，5秒自动消失），然后隐藏窗口
        console.log('[RecordingStore] Direct mode: showing error toast then hiding window')
        get().showToast('error', errorMessage, false, 5000)

        // 延迟隐藏窗口，让用户看到错误提示
        setTimeout(async () => {
          const window = getCurrentWindow()
          await window.hide()
        }, 5000)

        set({ state: 'idle', error: errorMessage, transcribedText: '' })
      } else {
        // 预览模式：显示错误信息（保持现有行为）
        set({ state: 'error', error: errorMessage })
      }
    }
  },

  cancelRecording: () => {
    // Clear timer
    if (window.__recordingTimer) {
      clearInterval(window.__recordingTimer)
      delete window.__recordingTimer
    }

    // 停止前端音频采集
    const audioCapture = get().audioCapture
    if (audioCapture) {
      audioCapture.cancel()
    }

    set({
      state: 'idle',
      duration: 0,
      transcription: null,
      transcribedText: '',
      error: null,
      audioLevel: 0,
      audioCapture: null,
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
    // 使用 transcription 字段(实际的转录文本),而不是 transcribedText(UI 显示文本)
    const text = get().transcription || get().transcribedText
    if (text) {
      try {
        console.log('[RecordingStore] Checking accessibility permission...')

        // 检查辅助功能权限
        const hasPermission = await invoke<boolean>('check_accessibility_permission_cmd')
        console.log('[RecordingStore] Accessibility permission:', hasPermission)

        if (!hasPermission) {
          console.log('[RecordingStore] Requesting accessibility permission...')
          await invoke('request_accessibility_permission_cmd')

          // 显示权限错误提示（严重错误，需要手动关闭）
          get().showToast('error', '需要辅助功能权限才能插入文本。请在系统设置中授权。', true, 0)
          set({ error: '需要辅助功能权限才能插入文本。请在系统设置中授权。' })
          return
        }

        // 插入文本
        console.log('[RecordingStore] Inserting text:', text)
        await invoke('insert_text_at_cursor_cmd', { text })
        console.log('[RecordingStore] Text inserted successfully')
      } catch (error) {
        console.error('[RecordingStore] Failed to insert text:', error)
        const errorMessage = `插入文本失败: ${String(error)}`

        // 显示插入失败错误（一般错误，5秒自动消失）
        get().showToast('error', errorMessage, false, 5000)
        set({ error: errorMessage })
      }
    }
  },

  // Toast 管理方法
  showToast: (type: InlineToastType, message: string, dismissible = false, duration = 0) => {
    set({
      toast: {
        type,
        message,
        dismissible,
        duration,
      },
    })
  },

  clearToast: () => {
    set({ toast: null })
  },
}))
