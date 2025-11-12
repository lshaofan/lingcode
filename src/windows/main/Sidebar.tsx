import React from 'react'
import { useUIStore } from '../../stores'

type Page = 'home' | 'notes' | 'settings'

interface NavItem {
  id: Page
  icon: string
  label: string
}

const navItems: NavItem[] = [
  { id: 'home', icon: '🏠', label: '首页' },
  { id: 'notes', icon: '📝', label: '笔记' },
  { id: 'settings', icon: '⚙️', label: '设置' },
]

export const Sidebar: React.FC = () => {
  const { currentPage, setCurrentPage, openSettings, isSettingsOpen } = useUIStore()

  const handleNavClick = (page: Page) => {
    if (page === 'settings') {
      openSettings()
    } else {
      setCurrentPage(page)
    }
  }

  return (
    <aside className="w-52 bg-gray-50 border-r border-gray-200 flex flex-col">
      <div className="p-4">
        <h1 className="text-xl font-bold text-gray-900">聆码 Lingcode</h1>
      </div>

      <nav className="flex-1 px-2 py-4 space-y-1">
        {navItems.map((item) => {
          // 设置项的高亮状态：当前页面是设置 或 设置弹窗打开时
          const isActive =
            item.id === 'settings'
              ? item.id === currentPage || isSettingsOpen
              : item.id === currentPage
          return (
            <button
              key={item.id}
              onClick={() => handleNavClick(item.id)}
              className={`
                w-full flex items-center gap-3 px-4 py-3 rounded-lg
                text-left transition-colors duration-200
                ${
                  isActive
                    ? 'bg-white text-green-600 shadow-sm'
                    : 'text-gray-700 hover:bg-white hover:text-gray-900'
                }
              `}
            >
              <span className="text-xl">{item.icon}</span>
              <span className="font-medium">{item.label}</span>
            </button>
          )
        })}
      </nav>
    </aside>
  )
}
