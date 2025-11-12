import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface Transcription {
  id?: number
  text: string
  audio_duration?: number
  model_version?: string
  language: string
  created_at: string
  app_context?: string
}

interface HistoryStore {
  transcriptions: Transcription[]
  loading: boolean
  error: string | null
  searchQuery: string

  // Actions
  loadRecent: (limit?: number) => Promise<void>
  search: (query: string) => Promise<void>
  deleteItem: (id: number) => Promise<void>
  deleteAll: () => Promise<void>
  setSearchQuery: (query: string) => void
  refresh: () => Promise<void>
}

export const useHistoryStore = create<HistoryStore>((set, get) => ({
  transcriptions: [],
  loading: false,
  error: null,
  searchQuery: '',

  loadRecent: async (limit = 50) => {
    set({ loading: true, error: null })
    try {
      const transcriptions = await invoke<Transcription[]>('get_recent_transcriptions', {
        limit,
      })
      set({ transcriptions, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  search: async (query: string) => {
    set({ loading: true, error: null, searchQuery: query })
    try {
      if (!query.trim()) {
        // If query is empty, load recent
        await get().loadRecent()
        return
      }

      const transcriptions = await invoke<Transcription[]>('search_transcriptions', {
        query,
      })
      set({ transcriptions, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  deleteItem: async (id: number) => {
    set({ loading: true, error: null })
    try {
      await invoke('delete_transcription', { id })
      set((state) => ({
        transcriptions: state.transcriptions.filter((t) => t.id !== id),
        loading: false,
      }))
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  deleteAll: async () => {
    set({ loading: true, error: null })
    try {
      await invoke('delete_all_transcriptions')
      set({ transcriptions: [], loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query })
  },

  refresh: async () => {
    const { searchQuery } = get()
    if (searchQuery) {
      await get().search(searchQuery)
    } else {
      await get().loadRecent()
    }
  },
}))
