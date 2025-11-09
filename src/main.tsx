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
const renderApp = async () => {
  console.log('[main.tsx] 🚀 Starting renderApp function')

  const window = getCurrentWindow()
  const label = window.label

  console.log('[main.tsx] 🏷️ Window label:', label)
  console.log('[main.tsx] 📦 Will render:', label === 'recording-float' ? 'RecordingFloat' : 'App')

  const component = label === 'recording-float' ? <RecordingFloat /> : <App />

  console.log('[main.tsx] 🎨 Creating React root...')
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      {component}
    </React.StrictMode>,
  )

  console.log('[main.tsx] ✅ Component rendered successfully')
}

console.log('[main.tsx] 🌟 main.tsx script loaded, calling renderApp()')
renderApp()
