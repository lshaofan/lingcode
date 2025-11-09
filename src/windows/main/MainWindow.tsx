import React, { useEffect } from 'react'
import { Sidebar } from './Sidebar'
import { HomePage } from './HomePage'
import { NotesPage } from './NotesPage'
import { SettingsDialog } from './SettingsDialog'
import { useUIStore } from '../../stores'
import { useSettingsStore } from '../../stores'
import { useHistoryStore } from '../../stores'

export const MainWindow: React.FC = () => {
  const { currentPage } = useUIStore()
  const { loadSettings } = useSettingsStore()
  const { loadRecent } = useHistoryStore()

  useEffect(() => {
    // 初始化加载设置和历史记录
    const initializeData = async () => {
      try {
        await loadSettings()
        await loadRecent(50)
      } catch (error) {
        console.error('Failed to initialize data:', error)
      }
    }

    void initializeData()
  }, [loadSettings, loadRecent])

  const renderContent = () => {
    switch (currentPage) {
      case 'home':
        return <HomePage />
      case 'notes':
        return <NotesPage />
      default:
        return <HomePage />
    }
  }

  return (
    <div className="h-screen flex overflow-hidden bg-white">
      <Sidebar />
      <main className="flex-1 overflow-y-auto">{renderContent()}</main>
      <SettingsDialog />
    </div>
  )
}
