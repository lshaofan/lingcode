import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { RecordingFloat } from './windows/recording'
import './index.css'
import { getCurrentWindow } from '@tauri-apps/api/window'

const rootElement = document.getElementById('root')
if (!rootElement) {
  throw new Error('Failed to find the root element')
}

// 根据窗口 label 决定渲染哪个组件
const renderApp = () => {
  console.log('[main.tsx] 🚀 Starting renderApp function')

  const window = getCurrentWindow()
  const label = window.label

  console.log('[main.tsx] 🏷️ Window label:', label)
  console.log('[main.tsx] 📦 Will render:', label === 'recording-float' ? 'RecordingFloat' : 'App')

  const component = label === 'recording-float' ? <RecordingFloat /> : <App />

  console.log('[main.tsx] 🎨 Creating React root...')

  // 🔑 关键修复：对于 recording-float 窗口禁用 StrictMode
  // StrictMode 会导致组件双重挂载，创建两个 AudioCapture 实例
  // 这会导致孤儿实例继续发送事件
  const shouldUseStrictMode = label !== 'recording-float'

  ReactDOM.createRoot(rootElement).render(
    shouldUseStrictMode ? <React.StrictMode>{component}</React.StrictMode> : component,
  )

  console.log('[main.tsx] ✅ Component rendered successfully')
}

console.log('[main.tsx] 🌟 main.tsx script loaded, calling renderApp()')
void renderApp()
