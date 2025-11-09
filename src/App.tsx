import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ToastProvider } from './components'
import { ComponentsDemo } from './ComponentsDemo'

function App() {
  const [showDemo, setShowDemo] = useState(false)
  const [count, setCount] = useState(0)

  const toggleRecordingFloat = async () => {
    console.log('toggleRecordingFloat 被点击')
    try {
      console.log('开始调用 toggle_recording_float 命令')
      const result = await invoke('toggle_recording_float')
      console.log('toggle_recording_float 调用成功:', result)
    } catch (error) {
      console.error('切换悬浮球失败:', error)
    }
  }

  const listWindows = async () => {
    console.log('listWindows 被点击')
    try {
      const windows = await invoke<string[]>('list_windows')
      console.log('所有窗口:', windows)
      alert(`窗口列表:\n${windows.join('\n')}`)
    } catch (error) {
      console.error('获取窗口列表失败:', error)
    }
  }

  return (
    <ToastProvider>
      {showDemo ? (
        <ComponentsDemo />
      ) : (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4">
          <div className="max-w-2xl w-full bg-white rounded-2xl shadow-xl p-8">
            <h1 className="text-4xl font-bold text-gray-900 mb-4 text-center">聆码 Lingcode</h1>
            <p className="text-lg text-gray-600 mb-8 text-center">跨应用语音听写工具</p>

            <div className="flex flex-col items-center gap-4">
              <button
                onClick={() => setCount((count) => count + 1)}
                className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors duration-200"
              >
                计数: {count}
              </button>

              <button
                onClick={() => setShowDemo(true)}
                className="px-6 py-3 bg-green-600 hover:bg-green-700 text-white font-medium rounded-lg transition-colors duration-200"
              >
                查看组件演示
              </button>

              <button
                onClick={toggleRecordingFloat}
                className="px-6 py-3 bg-purple-600 hover:bg-purple-700 text-white font-medium rounded-lg transition-colors duration-200"
              >
                切换录音悬浮球
              </button>

              <button
                onClick={listWindows}
                className="px-6 py-3 bg-orange-600 hover:bg-orange-700 text-white font-medium rounded-lg transition-colors duration-200"
              >
                查看窗口列表 (调试)
              </button>

              <p className="text-sm text-gray-500">项目初始化成功！开始构建您的语音转文字应用。</p>
            </div>

            <div className="mt-8 p-4 bg-gray-50 rounded-lg">
              <h2 className="text-lg font-semibold text-gray-800 mb-2">特性</h2>
              <ul className="space-y-2 text-sm text-gray-600">
                <li>✓ React 19 + TypeScript</li>
                <li>✓ Tauri 桌面框架</li>
                <li>✓ TailwindCSS v3 样式</li>
                <li>✓ 本地语音处理</li>
                <li>✓ 基础 UI 组件库</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </ToastProvider>
  )
}

export default App
