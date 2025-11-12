/**
 * 前端音频采集模块
 * 使用 getUserMedia API 进行音频采集，确保 macOS 麦克风图标正确显示
 *
 * 特性：
 * - 使用 Web Audio API + getUserMedia (确保系统麦克风指示器显示)
 * - 支持实时音频流采集
 * - 支持音频格式转换 (Float32 → PCM16)
 * - 为未来的实时转录功能预留扩展点
 */

export interface AudioCaptureConfig {
  sampleRate?: number // 采样率，默认 16000 Hz
  channelCount?: number // 声道数，默认 1 (单声道)
  echoCancellation?: boolean // 回声消除，默认 true
  noiseSuppression?: boolean // 降噪，默认 true
  autoGainControl?: boolean // 自动增益控制，默认 true
}

export interface AudioCaptureDevice {
  deviceId: string
  label: string
  groupId: string
}

// 🔑 全局实例跟踪，用于防止泄漏
const activeInstances = new Set<AudioCapture>()

// 🔑 全局清理：页面卸载时强制清理所有实例
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    console.log('[AudioCapture] 🚨 Page unloading, force cleaning all instances')
    const instances = Array.from(activeInstances)
    instances.forEach((instance) => {
      instance.cancel()
    })
  })
}

export class AudioCapture {
  private stream: MediaStream | null = null
  private audioContext: AudioContext | null = null
  private mediaRecorder: MediaRecorder | null = null
  private audioChunks: Blob[] = []
  private config: Required<AudioCaptureConfig>
  private _isDestroyed = false // 标记实例是否已销毁
  private _isPrewarmed = false // 标记是否已预热(getUserMedia完成但MediaRecorder未start)

  constructor(config: AudioCaptureConfig = {}) {
    this.config = {
      sampleRate: config.sampleRate ?? 16000,
      channelCount: config.channelCount ?? 1,
      echoCancellation: config.echoCancellation ?? true,
      noiseSuppression: config.noiseSuppression ?? true,
      autoGainControl: config.autoGainControl ?? true,
    }

    // 🔑 注册实例
    activeInstances.add(this)
    console.log('[AudioCapture] 📝 Instance created, total active instances:', activeInstances.size)
  }

  /**
   * 获取音频流
   * 用于AudioCacheManager监听音频轨道状态
   */
  get stream_(): MediaStream | null {
    return this.stream
  }

  /**
   * 检查浏览器是否支持 getUserMedia
   */
  static isSupported(): boolean {
    const supported = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia)
    console.log('[AudioCapture] 🔍 Checking getUserMedia support:')
    console.log('  - navigator.mediaDevices:', !!navigator.mediaDevices)
    console.log('  - getUserMedia method:', !!navigator.mediaDevices?.getUserMedia)
    console.log('  - Current URL:', window.location.href)
    console.log('  - Is secure context:', window.isSecureContext)
    return supported
  }

  /**
   * 清理所有活动的 AudioCapture 实例
   * 用于防止热重载时的实例泄漏
   */
  static cleanupAllInstances(): void {
    console.log('[AudioCapture] 🧹 Cleaning up all active instances, count:', activeInstances.size)
    const instances = Array.from(activeInstances)
    instances.forEach((instance) => {
      instance.cancel()
    })
    console.log('[AudioCapture] ✅ All instances cleaned up')
  }

  /**
   * 预热：提前初始化 getUserMedia 和 MediaRecorder，但不开始录音
   * 这样可以减少用户按下快捷键到真正开始录音之间的延迟
   * @param deviceId 可选的设备 ID
   */
  async prewarm(deviceId?: string): Promise<void> {
    console.log('[AudioCapture] 🔥 Prewarming - initializing getUserMedia and MediaRecorder...')

    // 防止在已销毁的实例上调用
    if (this._isDestroyed) {
      console.error('[AudioCapture] ❌ Cannot prewarm on destroyed instance')
      throw new Error('Cannot prewarm on destroyed AudioCapture instance')
    }

    if (this.stream) {
      console.warn('[AudioCapture] Already initialized, skipping prewarm')
      return
    }

    try {
      console.log('[AudioCapture] Step 1: Requesting getUserMedia...')

      // 构建音频约束
      const constraints: MediaStreamConstraints = {
        audio: {
          deviceId: deviceId ? { exact: deviceId } : undefined,
          sampleRate: { ideal: this.config.sampleRate },
          channelCount: { ideal: this.config.channelCount },
          echoCancellation: this.config.echoCancellation,
          noiseSuppression: this.config.noiseSuppression,
          autoGainControl: this.config.autoGainControl,
        },
      }

      // 🎯 关键：获取麦克风权限并创建音频流
      // 这会触发浏览器的麦克风权限请求(如果需要)
      // 并显示系统麦克风指示器
      this.stream = await navigator.mediaDevices.getUserMedia(constraints)
      console.log('[AudioCapture] ✅ getUserMedia completed, stream created')

      // 创建 AudioContext (用于后续的音频转换)
      this.audioContext = new AudioContext({
        sampleRate: this.config.sampleRate,
      })
      console.log('[AudioCapture] ✅ AudioContext created')

      // 确定支持的 MIME 类型
      const mimeType = this.getSupportedMimeType()
      console.log('[AudioCapture] Using MIME type:', mimeType)

      // 创建 MediaRecorder (但不启动)
      this.mediaRecorder = new MediaRecorder(this.stream, {
        mimeType,
        audioBitsPerSecond: 128000,
      })
      console.log('[AudioCapture] ✅ MediaRecorder created (ready to start)')

      // 设置事件处理器
      this.audioChunks = []
      this.mediaRecorder.ondataavailable = (event) => {
        if (this._isDestroyed) {
          console.log('[AudioCapture] ⚠️  Ignoring chunk from destroyed instance')
          return
        }
        if (event.data.size > 0) {
          this.audioChunks.push(event.data)
          console.log('[AudioCapture] Audio chunk received:', event.data.size, 'bytes')
        }
      }

      // 标记为已预热
      this._isPrewarmed = true
      console.log('[AudioCapture] 🔥✅ Prewarm completed! Ready to start recording instantly.')
    } catch (error) {
      console.error('[AudioCapture] Prewarm failed:', error)
      this.cleanup()
      throw error
    }
  }

  /**
   * 获取可用的音频输入设备列表
   */
  static async getDevices(): Promise<AudioCaptureDevice[]> {
    if (!AudioCapture.isSupported()) {
      throw new Error('getUserMedia is not supported in this browser')
    }

    const devices = await navigator.mediaDevices.enumerateDevices()
    return devices
      .filter((device) => device.kind === 'audioinput')
      .map((device) => ({
        deviceId: device.deviceId,
        label: device.label || `Microphone ${device.deviceId.slice(0, 8)}`,
        groupId: device.groupId,
      }))
  }

  /**
   * 开始音频采集
   * @param deviceId 可选的设备 ID，不指定则使用默认设备
   */
  async start(deviceId?: string): Promise<void> {
    // 防止在已销毁的实例上调用
    if (this._isDestroyed) {
      console.error('[AudioCapture] ❌ Cannot start on destroyed instance')
      throw new Error('Cannot start on destroyed AudioCapture instance')
    }

    console.log('[AudioCapture] 🎤 Starting audio capture...')

    // 🔥 检查是否已经预热
    if (this._isPrewarmed && this.mediaRecorder) {
      console.log('[AudioCapture] ⚡ Using prewarmed MediaRecorder - INSTANT START!')
      // MediaRecorder 已经就绪,直接开始录音
      this.mediaRecorder.start()
      console.log('[AudioCapture] ✅ MediaRecorder started instantly (prewarmed)')
      return
    }

    // 未预热,走完整的初始化流程
    if (this.stream) {
      console.warn('[AudioCapture] Already recording')
      return
    }

    console.log('[AudioCapture] 🎤 Starting audio capture with config:', this.config)

    try {
      // 构建约束条件
      const constraints: MediaStreamConstraints = {
        audio: {
          sampleRate: this.config.sampleRate,
          channelCount: this.config.channelCount,
          echoCancellation: this.config.echoCancellation,
          noiseSuppression: this.config.noiseSuppression,
          autoGainControl: this.config.autoGainControl,
          ...(deviceId && { deviceId: { exact: deviceId } }),
        },
        video: false,
      }

      // 请求麦克风权限并获取音频流
      // 🎯 关键：这一步会触发 macOS 的麦克风图标显示
      console.log('[AudioCapture] 🎤 Requesting getUserMedia with constraints:', constraints)
      this.stream = await navigator.mediaDevices.getUserMedia(constraints)
      console.log('[AudioCapture] ✅ Got media stream, microphone indicator should be active')

      // 🔍 诊断：检查音频轨道状态
      const audioTracks = this.stream.getAudioTracks()
      console.log('[AudioCapture] 📊 Audio tracks:', audioTracks.length)
      audioTracks.forEach((track, index) => {
        console.log(`[AudioCapture] Track ${index}:`, {
          label: track.label,
          enabled: track.enabled,
          muted: track.muted,
          readyState: track.readyState,
          id: track.id,
        })
      })

      // 创建 AudioContext（用于音频处理）
      this.audioContext = new AudioContext({
        sampleRate: this.config.sampleRate,
      })

      // 创建 MediaRecorder 用于录制
      // 使用 webm 格式，因为浏览器原生支持
      const mimeType = this.getSupportedMimeType()
      this.mediaRecorder = new MediaRecorder(this.stream, {
        mimeType,
        audioBitsPerSecond: 128000, // 128 kbps
      })
      console.log('[AudioCapture] 📹 MediaRecorder created with mimeType:', mimeType)

      // 监听数据事件
      this.audioChunks = []
      this.mediaRecorder.ondataavailable = (event) => {
        // 🔑 关键修复：如果实例已销毁，直接忽略事件
        if (this._isDestroyed) {
          console.log('[AudioCapture] ⚠️  Ignoring chunk from destroyed instance')
          return
        }

        if (event.data.size > 0) {
          this.audioChunks.push(event.data)
          console.log('[AudioCapture] Audio chunk received:', event.data.size, 'bytes')
        }
      }

      // 开始录制
      // 🔑 关键修复：不设置 timeslice，避免音频丢失
      // 不传参数时，MediaRecorder 会在 stop() 时一次性返回所有音频数据
      // 这确保了音频的完整性，不会丢失开头或结尾
      this.mediaRecorder.start()
      console.log(
        '[AudioCapture] ✅ MediaRecorder started (no timeslice - will collect all data on stop)',
      )
    } catch (error) {
      console.error('[AudioCapture] Failed to start audio capture:', error)
      this.cleanup()
      throw error
    }
  }

  /**
   * 停止音频采集并返回录音数据
   * @returns 返回音频 Blob 数据
   */
  async stop(): Promise<Blob> {
    return new Promise((resolve, reject) => {
      if (!this.mediaRecorder || !this.stream) {
        reject(new Error('Not recording'))
        return
      }

      console.log('[AudioCapture] 🛑 Stopping audio capture...')

      // 🔑 关键修复：先保存 chunks 的引用
      const chunks = this.audioChunks

      // 设置停止事件监听器
      this.mediaRecorder.onstop = () => {
        console.log('[AudioCapture] MediaRecorder stopped, chunks:', chunks.length)

        // 合并所有音频块
        const mimeType = this.getSupportedMimeType()
        const audioBlob = new Blob(chunks, { type: mimeType })
        console.log('[AudioCapture] ✅ Audio blob created:', audioBlob.size, 'bytes')

        // 清理资源
        this.cleanup()

        resolve(audioBlob)
      }

      // 🔑 重要：不要在 stop() 之前清除 ondataavailable！
      // 当不使用 timeslice 时，stop() 会触发最后一次 ondataavailable 事件
      // 这个事件包含了所有录制的音频数据
      // 我们需要等这个事件完成后，在 cleanup() 中清除处理器

      // 停止录制
      console.log('[AudioCapture] 📤 Calling stop() - will trigger final ondataavailable')
      this.mediaRecorder.stop()
    })
  }

  /**
   * 取消录制
   */
  cancel(): void {
    console.log('[AudioCapture] Cancelling audio capture')
    this.cleanup()
  }

  /**
   * 获取当前是否正在录制
   */
  isRecording(): boolean {
    return this.mediaRecorder?.state === 'recording'
  }

  /**
   * 清理资源
   */
  private cleanup(): void {
    // 防止重复清理
    if (this._isDestroyed) {
      console.log('[AudioCapture] ⚠️  Already destroyed, skipping cleanup')
      return
    }

    console.log('[AudioCapture] 🧹 Starting cleanup...')
    this._isDestroyed = true

    // 🔑 关键修复：先停止 MediaRecorder 并清除事件处理器
    if (this.mediaRecorder) {
      // 如果还在录制状态，先停止
      if (this.mediaRecorder.state === 'recording' || this.mediaRecorder.state === 'paused') {
        try {
          this.mediaRecorder.stop()
          console.log('[AudioCapture] 🛑 MediaRecorder stopped in cleanup')
        } catch (error) {
          console.error('[AudioCapture] Error stopping MediaRecorder:', error)
        }
      }

      // 清除所有事件处理器（防止继续触发）
      this.mediaRecorder.ondataavailable = null
      this.mediaRecorder.onstop = null
      this.mediaRecorder.onerror = null
      this.mediaRecorder = null
      console.log('[AudioCapture] 🧹 MediaRecorder event handlers cleared')
    }

    // 停止所有音频轨道（这会关闭麦克风并隐藏指示器）
    if (this.stream) {
      this.stream.getTracks().forEach((track) => {
        track.stop()
        console.log('[AudioCapture] 🎤 Audio track stopped')
      })
      this.stream = null
    }

    // 关闭 AudioContext
    if (this.audioContext) {
      void this.audioContext.close()
      this.audioContext = null
    }

    // 清理音频块数组
    this.audioChunks = []

    // 🔑 从全局跟踪中移除
    activeInstances.delete(this)
    console.log(
      '[AudioCapture] ✅ Cleanup completed, remaining active instances:',
      activeInstances.size,
    )
  }

  /**
   * 获取支持的 MIME 类型
   */
  private getSupportedMimeType(): string {
    const types = ['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus', 'audio/mp4']

    for (const type of types) {
      if (MediaRecorder.isTypeSupported(type)) {
        console.log('[AudioCapture] Using MIME type:', type)
        return type
      }
    }

    console.warn('[AudioCapture] No supported MIME type found, using default')
    return 'audio/webm'
  }
}

/**
 * 音频格式转换工具
 */
export class AudioConverter {
  /**
   * 将 Blob 转换为 ArrayBuffer
   */
  static async blobToArrayBuffer(blob: Blob): Promise<ArrayBuffer> {
    return blob.arrayBuffer()
  }

  /**
   * 将 Blob 转换为 Base64 字符串
   */
  static async blobToBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onloadend = () => {
        const result = reader.result as string
        // 移除 data URL 前缀
        const base64 = result.split(',')[1] || ''
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(blob)
    })
  }

  /**
   * 将音频 Blob 解码为 PCM 数据
   * @param blob 音频 Blob
   * @param targetSampleRate 目标采样率（默认 16000）
   * @returns PCM16 数据的 Uint8Array
   */
  static async blobToPCM16(blob: Blob, targetSampleRate: number = 16000): Promise<Uint8Array> {
    // 将 Blob 转换为 ArrayBuffer
    const arrayBuffer = await blob.arrayBuffer()

    // 创建 AudioContext 解码音频
    const audioContext = new AudioContext({ sampleRate: targetSampleRate })
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer)

    // 获取音频数据（单声道）
    let audioData: Float32Array
    if (audioBuffer.numberOfChannels === 1) {
      audioData = audioBuffer.getChannelData(0)
    } else {
      // 如果是多声道，混合为单声道
      const left = audioBuffer.getChannelData(0)
      const right = audioBuffer.getChannelData(1)
      audioData = new Float32Array(left.length)
      for (let i = 0; i < left.length; i++) {
        audioData[i] = ((left[i] ?? 0) + (right[i] ?? 0)) / 2
      }
    }

    // 转换为 16-bit PCM
    const pcm16 = AudioConverter.float32ToPCM16(audioData)

    // 关闭 AudioContext
    await audioContext.close()

    return pcm16
  }

  /**
   * 将 Float32 音频数据转换为 16-bit PCM
   */
  static float32ToPCM16(float32Array: Float32Array): Uint8Array {
    const pcm16 = new Int16Array(float32Array.length)

    for (let i = 0; i < float32Array.length; i++) {
      // 限制在 [-1.0, 1.0] 范围内
      const s = Math.max(-1, Math.min(1, float32Array[i] ?? 0))
      // 转换为 16-bit 整数
      pcm16[i] = s < 0 ? s * 0x8000 : s * 0x7fff
    }

    // 返回字节数组（小端序）
    return new Uint8Array(pcm16.buffer)
  }

  /**
   * 将音频 Blob 转换为 WAV 格式
   * 注意：这个方法返回的 WAV 数据可以直接发送到后端
   */
  static async blobToWAV(
    blob: Blob,
    sampleRate: number = 16000,
    numChannels: number = 1,
    bitDepth: number = 16,
  ): Promise<Uint8Array> {
    // 先转换为 PCM16
    const pcm16 = await AudioConverter.blobToPCM16(blob, sampleRate)

    // 创建 WAV 文件头
    const wavHeader = AudioConverter.createWAVHeader(
      pcm16.length,
      sampleRate,
      numChannels,
      bitDepth,
    )

    // 合并头部和数据
    const wavData = new Uint8Array(wavHeader.length + pcm16.length)
    wavData.set(wavHeader, 0)
    wavData.set(pcm16, wavHeader.length)

    return wavData
  }

  /**
   * 创建 WAV 文件头
   */
  private static createWAVHeader(
    dataSize: number,
    sampleRate: number,
    numChannels: number,
    bitDepth: number,
  ): Uint8Array {
    const header = new ArrayBuffer(44)
    const view = new DataView(header)

    // RIFF chunk descriptor
    this.writeString(view, 0, 'RIFF')
    view.setUint32(4, 36 + dataSize, true) // File size - 8
    this.writeString(view, 8, 'WAVE')

    // fmt sub-chunk
    this.writeString(view, 12, 'fmt ')
    view.setUint32(16, 16, true) // Subchunk1Size (16 for PCM)
    view.setUint16(20, 1, true) // AudioFormat (1 for PCM)
    view.setUint16(22, numChannels, true)
    view.setUint32(24, sampleRate, true)
    view.setUint32(28, (sampleRate * numChannels * bitDepth) / 8, true) // ByteRate
    view.setUint16(32, (numChannels * bitDepth) / 8, true) // BlockAlign
    view.setUint16(34, bitDepth, true)

    // data sub-chunk
    this.writeString(view, 36, 'data')
    view.setUint32(40, dataSize, true)

    return new Uint8Array(header)
  }

  /**
   * 写入字符串到 DataView
   */
  private static writeString(view: DataView, offset: number, string: string): void {
    for (let i = 0; i < string.length; i++) {
      view.setUint8(offset + i, string.charCodeAt(i))
    }
  }
}
