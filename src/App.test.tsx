import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import App from './App'

describe('App', () => {
  it('renders the app title', () => {
    render(<App />)
    expect(screen.getByText(/聆码 Lingcode/i)).toBeInTheDocument()
  })

  it('renders the subtitle', () => {
    render(<App />)
    expect(screen.getByText(/跨应用语音听写工具/i)).toBeInTheDocument()
  })

  it('increments counter when button is clicked', () => {
    render(<App />)
    const button = screen.getByRole('button', { name: /计数: 0/i })

    fireEvent.click(button)
    expect(screen.getByText(/计数: 1/i)).toBeInTheDocument()

    fireEvent.click(button)
    expect(screen.getByText(/计数: 2/i)).toBeInTheDocument()
  })

  it('displays feature list', () => {
    render(<App />)
    expect(screen.getByText(/✓ React 19 \+ TypeScript/i)).toBeInTheDocument()
    expect(screen.getByText(/✓ Tauri 桌面框架/i)).toBeInTheDocument()
    expect(screen.getByText(/✓ TailwindCSS v3 样式/i)).toBeInTheDocument()
  })
})
