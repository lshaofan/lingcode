import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

export type RecordingState = 'idle' | 'recording' | 'processing' | 'error';

interface RecordingStore {
  state: RecordingState;
  duration: number;
  transcription: string | null;
  transcribedText: string; // New: Current transcribed text for display
  error: string | null;
  audioLevel: number;

  // Actions
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  cancelRecording: () => void;
  setAudioLevel: (level: number) => void;
  resetState: () => void;

  // New text actions
  clearText: () => void;
  copyText: () => Promise<void>;
  insertText: () => Promise<void>;
  setTranscribedText: (text: string) => void;
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  state: 'idle',
  duration: 0,
  transcription: null,
  transcribedText: '',
  error: null,
  audioLevel: 0,

  startRecording: async () => {
    try {
      set({ state: 'recording', error: null, transcription: null, duration: 0 });

      // TODO: Call Tauri command to start audio recording
      // await invoke('start_recording');

      // Start duration timer
      const timer = setInterval(() => {
        set((state) => ({
          duration: state.duration + 0.1,
        }));
      }, 100);

      // Store timer ID for cleanup
      (window as any).__recordingTimer = timer;
    } catch (error) {
      set({ state: 'error', error: String(error) });
    }
  },

  stopRecording: async () => {
    try {
      // Clear timer
      if ((window as any).__recordingTimer) {
        clearInterval((window as any).__recordingTimer);
        delete (window as any).__recordingTimer;
      }

      set({ state: 'processing' });

      // TODO: Call Tauri command to stop recording and get transcription
      // const result = await invoke<string>('stop_recording');

      // For now, simulate processing
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const mockTranscription = '这是一段测试文本';

      // Save transcription to database
      await invoke('create_transcription', {
        transcription: {
          text: mockTranscription,
          audio_duration: get().duration,
          model_version: 'base',
          language: 'zh',
          created_at: new Date().toISOString(),
          app_context: null,
        },
      });

      set({
        state: 'idle',
        transcription: mockTranscription,
        transcribedText: mockTranscription,
        duration: 0,
        audioLevel: 0,
      });
    } catch (error) {
      set({ state: 'error', error: String(error) });
    }
  },

  cancelRecording: () => {
    // Clear timer
    if ((window as any).__recordingTimer) {
      clearInterval((window as any).__recordingTimer);
      delete (window as any).__recordingTimer;
    }

    // TODO: Call Tauri command to cancel recording
    // invoke('cancel_recording');

    set({
      state: 'idle',
      duration: 0,
      transcription: null,
      error: null,
      audioLevel: 0,
    });
  },

  setAudioLevel: (level: number) => {
    set({ audioLevel: level });
  },

  resetState: () => {
    set({
      state: 'idle',
      duration: 0,
      transcription: null,
      transcribedText: '',
      error: null,
      audioLevel: 0,
    });
  },

  // New text actions
  setTranscribedText: (text: string) => {
    set({ transcribedText: text });
  },

  clearText: () => {
    set({ transcribedText: '', state: 'idle' });
  },

  copyText: async () => {
    const text = get().transcribedText;
    if (text) {
      try {
        await writeText(text);
        console.log('[RecordingStore] Text copied to clipboard');
      } catch (error) {
        console.error('[RecordingStore] Failed to copy text:', error);
        set({ error: 'Failed to copy text to clipboard' });
      }
    }
  },

  insertText: async () => {
    const text = get().transcribedText;
    if (text) {
      try {
        // TODO: Implement text insertion via Tauri command
        // await invoke('insert_text', { text });
        console.log('[RecordingStore] Text insertion not yet implemented');
        set({ error: 'Text insertion not yet implemented' });
      } catch (error) {
        console.error('[RecordingStore] Failed to insert text:', error);
        set({ error: 'Failed to insert text' });
      }
    }
  },
}));
