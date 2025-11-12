/**
 * 音频反馈工具类
 * 用于播放用户操作的音效反馈
 */

type FeedbackSound = 'start' | 'ok'

class AudioFeedback {
  private audioElements: Map<FeedbackSound, HTMLAudioElement> = new Map()
  private preloadPromises: Map<FeedbackSound, Promise<void>> = new Map()

  constructor() {
    this.preload()
  }

  /**
   * 预加载音频文件
   */
  private preload() {
    const sounds: Array<{ key: FeedbackSound; path: string }> = [
      { key: 'start', path: '/assets/audio/start.MP3' },
      { key: 'ok', path: '/assets/audio/OK.MP3' },
    ]

    sounds.forEach(({ key, path }) => {
      const audio = new Audio(path)
      audio.preload = 'auto'
      this.audioElements.set(key, audio)

      // 创建预加载 Promise
      const preloadPromise = new Promise<void>((resolve) => {
        audio.addEventListener('canplaythrough', () => resolve(), {
          once: true,
        })
        audio.addEventListener('error', () => {
          console.warn(`Failed to preload audio: ${path}`)
          resolve() // 即使失败也 resolve，避免阻塞
        })
      })

      this.preloadPromises.set(key, preloadPromise)
    })
  }

  /**
   * 播放指定的音效
   * @param sound 音效类型
   * @returns Promise，播放完成后 resolve
   */
  async play(sound: FeedbackSound): Promise<void> {
    try {
      // 等待预加载完成
      const preloadPromise = this.preloadPromises.get(sound)
      if (preloadPromise) {
        await preloadPromise
      }

      const audio = this.audioElements.get(sound)
      if (!audio) {
        console.warn(`Audio element not found: ${sound}`)
        return
      }

      // 重置播放位置（如果正在播放）
      audio.currentTime = 0

      // 播放音频
      await audio.play()
    } catch (error) {
      console.warn(`Failed to play audio: ${sound}`, error)
      // 不抛出错误，避免影响主流程
    }
  }

  /**
   * 播放开始录制音效
   */
  async playStart(): Promise<void> {
    return this.play('start')
  }

  /**
   * 播放完成音效
   */
  async playOk(): Promise<void> {
    return this.play('ok')
  }

  /**
   * 停止所有正在播放的音效
   */
  stopAll() {
    this.audioElements.forEach((audio) => {
      audio.pause()
      audio.currentTime = 0
    })
  }

  /**
   * 清理资源
   */
  destroy() {
    this.stopAll()
    this.audioElements.forEach((audio) => {
      audio.src = ''
    })
    this.audioElements.clear()
    this.preloadPromises.clear()
  }
}

// 导出单例
export const audioFeedback = new AudioFeedback()
