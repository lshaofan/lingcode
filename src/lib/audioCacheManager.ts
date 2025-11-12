import { AudioCapture, AudioCaptureConfig } from './audioCapture'

/**
 * 音频缓存管理器
 *
 * 核心功能:
 * 1. 首次录音时完整初始化并缓存 AudioCapture 实例
 * 2. 后续录音直接使用缓存实例 (< 10ms 启动)
 * 3. 监听设备变化事件,自动重新预热
 * 4. 提供缓存失效和清理机制
 */
export class AudioCacheManager {
  private cachedInstance: AudioCapture | null = null
  private isValid = false
  private deviceChangeListener: (() => void) | null = null
  private config: Required<AudioCaptureConfig>
  private isPrewarming = false // 防止并发预热

  constructor(config: AudioCaptureConfig = {}) {
    this.config = {
      sampleRate: config.sampleRate ?? 16000,
      channelCount: config.channelCount ?? 1,
      echoCancellation: config.echoCancellation ?? true,
      noiseSuppression: config.noiseSuppression ?? true,
      autoGainControl: config.autoGainControl ?? true,
    }

    console.log('[AudioCacheManager] 🎯 Created with config:', this.config)
    this.setupDeviceChangeListener()
  }

  /**
   * 设置设备变化监听器
   * 当音频设备变化时(插拔耳机/麦克风),自动失效缓存并重新预热
   */
  private setupDeviceChangeListener() {
    if (!navigator.mediaDevices?.addEventListener) {
      console.warn('[AudioCacheManager] ⚠️ devicechange API not supported')
      return
    }

    this.deviceChangeListener = () => {
      console.log('[AudioCacheManager] 🔄 Device changed detected!')

      // 使用 void 标记异步操作
      void (async () => {
        // 获取当前设备列表
        try {
          const devices = await navigator.mediaDevices.enumerateDevices()
          const audioInputs = devices.filter((d) => d.kind === 'audioinput')
          console.log('[AudioCacheManager] Current audio input devices:', audioInputs.length)
          audioInputs.forEach((device, i) => {
            console.log(
              `  [${i}] ${device.label || 'Unknown Device'} (${device.deviceId.substring(0, 8)}...)`,
            )
          })
        } catch (error) {
          console.error('[AudioCacheManager] Failed to enumerate devices:', error)
        }

        // 失效缓存
        this.invalidate()

        // 后台自动重新预热
        console.log('[AudioCacheManager] 🔥 Auto re-prewarming after device change...')
        try {
          await this.prewarm()
          console.log('[AudioCacheManager] ✅ Auto re-prewarm completed')
        } catch (error) {
          console.error('[AudioCacheManager] ❌ Auto re-prewarm failed:', error)
        }
      })()
    }

    navigator.mediaDevices.addEventListener('devicechange', this.deviceChangeListener)
    console.log('[AudioCacheManager] ✅ Device change listener registered')
  }

  /**
   * 预热:初始化 getUserMedia 和 MediaRecorder,但不开始录音
   * 如果已有有效缓存,直接返回;否则创建新实例
   */
  async prewarm(): Promise<AudioCapture> {
    // 如果已有有效缓存,直接返回
    if (this.isValid && this.cachedInstance) {
      console.log('[AudioCacheManager] ⚡ Using cached instance (valid)')
      return this.cachedInstance
    }

    // 防止并发预热
    if (this.isPrewarming) {
      console.log('[AudioCacheManager] ⚠️ Already prewarming, waiting...')
      // 等待当前预热完成
      while (this.isPrewarming) {
        await new Promise((resolve) => setTimeout(resolve, 50))
      }
      if (this.isValid && this.cachedInstance) {
        return this.cachedInstance
      }
    }

    this.isPrewarming = true

    try {
      console.log('[AudioCacheManager] 🔥 Creating new instance and prewarming...')

      // 清理旧实例
      this.cleanup()

      // 创建新实例
      const instance = new AudioCapture(this.config)
      await instance.prewarm()

      // 监听音频轨道状态
      this.setupTrackListeners(instance)

      // 缓存实例
      this.cachedInstance = instance
      this.isValid = true

      console.log('[AudioCacheManager] ✅ Prewarm completed and cached')
      return instance
    } catch (error) {
      console.error('[AudioCacheManager] ❌ Prewarm failed:', error)
      this.isValid = false
      this.cachedInstance = null
      throw error
    } finally {
      this.isPrewarming = false
    }
  }

  /**
   * 设置音频轨道监听器
   * 当音频轨道结束或出错时,自动失效缓存
   */
  private setupTrackListeners(instance: AudioCapture) {
    if (!instance.stream_) return

    instance.stream_.getTracks().forEach((track) => {
      track.addEventListener('ended', () => {
        console.log('[AudioCacheManager] ⚠️ Audio track ended, invalidating cache')
        this.invalidate()
      })

      track.addEventListener('mute', () => {
        console.log('[AudioCacheManager] 🔇 Audio track muted')
      })

      track.addEventListener('unmute', () => {
        console.log('[AudioCacheManager] 🔊 Audio track unmuted')
      })
    })
  }

  /**
   * 获取缓存的实例
   * @returns 如果缓存有效则返回实例,否则返回 null
   */
  getCached(): AudioCapture | null {
    if (this.isValid && this.cachedInstance) {
      console.log('[AudioCacheManager] ✅ Returning cached instance')
      return this.cachedInstance
    }
    console.log('[AudioCacheManager] ❌ No valid cached instance')
    return null
  }

  /**
   * 使缓存失效
   * 不会立即清理实例,只是标记为失效
   * 下次调用 prewarm() 时会创建新实例
   */
  invalidate() {
    console.log('[AudioCacheManager] ❌ Cache invalidated')
    this.isValid = false
  }

  /**
   * 清理缓存的实例
   * 会立即停止并释放所有资源
   */
  cleanup() {
    if (this.cachedInstance) {
      console.log('[AudioCacheManager] 🧹 Cleaning up cached instance')
      this.cachedInstance.cancel()
      this.cachedInstance = null
    }
    this.isValid = false
  }

  /**
   * 销毁缓存管理器
   * 移除所有监听器并清理资源
   */
  destroy() {
    console.log('[AudioCacheManager] 💥 Destroying cache manager')

    // 清理实例
    this.cleanup()

    // 移除设备变化监听器
    if (this.deviceChangeListener && navigator.mediaDevices?.removeEventListener) {
      navigator.mediaDevices.removeEventListener('devicechange', this.deviceChangeListener)
      this.deviceChangeListener = null
      console.log('[AudioCacheManager] ✅ Device change listener removed')
    }
  }

  /**
   * 获取缓存状态信息
   */
  getStatus(): {
    isValid: boolean
    hasCachedInstance: boolean
    isPrewarming: boolean
    hasDeviceListener: boolean
  } {
    return {
      isValid: this.isValid,
      hasCachedInstance: this.cachedInstance !== null,
      isPrewarming: this.isPrewarming,
      hasDeviceListener: this.deviceChangeListener !== null,
    }
  }
}
