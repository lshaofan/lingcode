import { useState, useEffect, useRef } from 'react';
import { Mic, X, Trash2, Copy, CornerDownLeft } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useRecordingStore } from '../../stores';

export const RecordingFloat = () => {
  console.log('[RecordingFloat] 🎬 Component function called');

  const status = useRecordingStore((state) => state.state);
  const transcribedText = useRecordingStore((state) => state.transcribedText);
  const audioLevel = useRecordingStore((state) => state.audioLevel);

  console.log('[RecordingFloat] 📊 Current state:', { status, transcribedText, audioLevel });
  console.log('[RecordingFloat] 🔍 Status type:', typeof status, 'Value:', status);

  const startRecording = useRecordingStore((state) => state.startRecording);
  const stopRecording = useRecordingStore((state) => state.stopRecording);
  const clearText = useRecordingStore((state) => state.clearText);
  const copyText = useRecordingStore((state) => state.copyText);
  const insertText = useRecordingStore((state) => state.insertText);

  const [showCopiedFeedback, setShowCopiedFeedback] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  // Auto-resize window based on content size
  const resizeWindow = async () => {
    try {
      let width = 350;
      let height = 120;

      // Adjust based on state
      if (status === 'idle' && transcribedText) {
        // Result display state - generous sizing to ensure everything fits
        const textLength = transcribedText.length;

        // Width: ensure enough space for min-w-[400px] content + padding + outer margin
        // CSS has min-w-[400px], plus px-5 (40px total padding), plus p-4 outer (32px total)
        width = 1000; // Increased base width to ensure content fits

        // For longer text, increase width up to max
        if (textLength > 20) {
          width = Math.min(950, 600 + textLength * 3);
        }

        // Height: text row + button row + all padding
        // - Outer p-4: 32px
        // - Text row py-3: 24px (12px top + 12px bottom)
        // - Text content max-h-[60px]: 60px
        // - Button row py-3 + pb-3: 24px
        // - Button height: 36px
        // Total: 32 + 24 + 60 + 24 + 36 = 176px, add buffer
        const estimatedLines = Math.ceil(textLength / 50);
        const textExtraHeight = Math.min(estimatedLines * 15, 40);
        height = 180 + textExtraHeight;
      } else {
        // Recording/Processing/Idle state - small capsule
        // CSS has w-[280px] h-[50px], need to add outer space
        width = 350;  // 280 + 70 buffer
        height = 120; // 50 + 70 buffer
      }

      console.log('[RecordingFloat] Resizing window to:', width, 'x', height, 'Status:', status, 'Has text:', !!transcribedText);

      await invoke('resize_recording_float', {
        width,
        height
      });
    } catch (error) {
      console.error('[RecordingFloat] Failed to resize window:', error);
    }
  };

  // Monitor content size changes and resize window accordingly
  useEffect(() => {
    // Initial resize with a small delay
    const timeoutId = setTimeout(() => {
      resizeWindow();
    }, 50);

    return () => {
      clearTimeout(timeoutId);
    };
  }, [status, transcribedText]); // Resize when status or text changes

  useEffect(() => {
    console.log('[RecordingFloat] 🚀 Mount useEffect running');
    console.log('[RecordingFloat] 🪟 Window location:', window.location.href);
    console.log('[RecordingFloat] 📄 Document title:', document.title);

    // Alert to make sure we can see it even if console is not working
    console.error('🔴 RecordingFloat MOUNTED - THIS SHOULD APPEAR IN CONSOLE!');

    // Set transparent background for the window
    document.documentElement.style.backgroundColor = 'transparent';
    document.documentElement.style.overflow = 'hidden';
    document.body.style.backgroundColor = 'transparent';
    document.body.style.overflow = 'hidden';
    document.body.style.margin = '0';
    document.body.style.padding = '0';

    console.log('[RecordingFloat] ✅ Component mounted and styles applied');
  }, []);

  useEffect(() => {
    console.log('[RecordingFloat] Setting up event listeners');
    let unlistenStart: (() => void) | null = null;
    let unlistenStop: (() => void) | null = null;

    const setupListeners = async () => {
      console.log('[RecordingFloat] Registering event listeners...');

      unlistenStart = await listen('shortcut-start-recording', () => {
        console.log('[RecordingFloat] Received shortcut-start-recording event');
        startRecording();
      });

      unlistenStop = await listen('shortcut-stop-recording', () => {
        console.log('[RecordingFloat] Received shortcut-stop-recording event');
        stopRecording();
      });

      console.log('[RecordingFloat] Event listeners registered');
    };

    setupListeners().catch((error) => {
      console.error('[RecordingFloat] Failed to setup event listeners:', error);
    });

    return () => {
      console.log('[RecordingFloat] Cleaning up event listeners');
      if (unlistenStart) unlistenStart();
      if (unlistenStop) unlistenStop();
    };
  }, [startRecording, stopRecording]);

  // Handle Esc key to close window
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleClose = async () => {
    console.log('[RecordingFloat] Closing window');
    const window = getCurrentWindow();
    await window.hide();
    clearText();
  };

  const handleCopy = async () => {
    await copyText();
    setShowCopiedFeedback(true);
    setTimeout(() => setShowCopiedFeedback(false), 2000);
  };

  const handleInsert = async () => {
    await insertText();
    // Close window after insert
    await handleClose();
  };

  console.log('[RecordingFloat] 🎯 Rendering decision:', { status, hasText: !!transcribedText });

  // Recording/Processing state UI (280x50)
  if (status === 'recording' || status === 'processing') {
    console.log('[RecordingFloat] 🎨 Rendering recording UI');
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <div
          ref={contentRef}
          className="relative flex items-center gap-3 px-5 py-3
            w-[280px] h-[50px] rounded-full
            bg-gray-700/80 backdrop-blur-xl shadow-2xl"
        >
          {/* Left: Microphone icon */}
          <Mic className="w-4 h-4 text-white/90 flex-shrink-0" />

          {/* Center: Waveform animation */}
          <div className="flex-1 flex items-center justify-center gap-[3px] h-6">
            {status === 'recording' ? (
              // Animated waveform bars - 6 thin bars
              <>
                {[...Array(6)].map((_, i) => (
                  <div
                    key={i}
                    className="w-[3px] bg-white/70 rounded-full animate-pulse"
                    style={{
                      height: `${12 + Math.random() * 10}px`,
                      animationDelay: `${i * 0.1}s`,
                      animationDuration: '0.8s',
                    }}
                  />
                ))}
              </>
            ) : (
              // Processing state
              <div className="flex items-center gap-1.5">
                <div className="w-1.5 h-1.5 bg-white/70 rounded-full animate-bounce" style={{ animationDelay: '0s' }} />
                <div className="w-1.5 h-1.5 bg-white/70 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }} />
                <div className="w-1.5 h-1.5 bg-white/70 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
              </div>
            )}
          </div>

          {/* Right: Close button */}
          <button
            onClick={handleClose}
            className="flex-shrink-0 w-5 h-5 rounded-full bg-gray-600/60 hover:bg-gray-600/80
              flex items-center justify-center transition-colors"
          >
            <X className="w-3 h-3 text-white/90" />
          </button>
        </div>
      </div>
    );
  }

  // Result display state - horizontal capsule with text and buttons
  if (status === 'idle' && transcribedText) {
    return (
      <div className="fixed inset-0 flex items-center justify-center p-4">
        <div
          ref={contentRef}
          className="relative flex flex-col
            min-w-[400px] max-w-[950px] w-full
            rounded-2xl
            bg-gray-700/85 backdrop-blur-xl shadow-2xl
            overflow-hidden"
        >
          {/* Top row: Icon + Text + Close button */}
          <div className="flex items-center gap-4 px-6 py-3.5">
            {/* Left: Microphone icon */}
            <Mic className="w-4 h-4 text-white/90 flex-shrink-0" />

            {/* Center: Text (scrollable if long) */}
            <div className="flex-1 max-h-[60px] overflow-y-auto">
              <p className="text-white text-sm leading-relaxed whitespace-pre-wrap">
                {transcribedText}
              </p>
            </div>

            {/* Right: Close button */}
            <button
              onClick={handleClose}
              className="flex-shrink-0 w-5 h-5 rounded-full bg-gray-600/60 hover:bg-gray-600/80
                flex items-center justify-center transition-colors"
            >
              <X className="w-3 h-3 text-white/90" />
            </button>
          </div>

          {/* Bottom row: Action buttons */}
          <div className="flex items-center justify-end gap-2 px-6 pb-3 pt-1">
            {/* Clear button */}
            <button
              onClick={clearText}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-md
                bg-white/5 hover:bg-white/10 border border-white/10
                transition-all group"
              title="清空"
            >
              <Trash2 className="w-3 h-3 text-white/70 group-hover:text-white transition-colors" />
              <span className="text-white/80 text-xs group-hover:text-white">清空</span>
            </button>

            {/* Copy button */}
            <button
              onClick={handleCopy}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-md
                bg-white/5 hover:bg-white/10 border border-white/10
                transition-all group relative"
              title="复制"
            >
              <Copy className="w-3 h-3 text-white/70 group-hover:text-white transition-colors" />
              <span className="text-white/80 text-xs group-hover:text-white">复制</span>
              {showCopiedFeedback && (
                <span className="absolute -top-8 left-1/2 -translate-x-1/2
                  px-2 py-1 rounded-md bg-green-500 text-white text-xs whitespace-nowrap shadow-lg">
                  已复制
                </span>
              )}
            </button>

            {/* Insert button */}
            <button
              onClick={handleInsert}
              className="flex items-center gap-1.5 px-3 py-1 rounded-md
                bg-blue-600/80 hover:bg-blue-600 border border-blue-500/50
                transition-all group"
              title="插入"
            >
              <span className="text-white text-xs font-medium">插入</span>
              <CornerDownLeft className="w-3 h-3 text-white" />
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Default state: show idle recording UI
  // This handles the brief moment when window is shown but recording hasn't started yet
  console.log('[RecordingFloat] ⚠️  Idle state without text - rendering idle recording UI');
  return (
    <div className="fixed inset-0 flex items-center justify-center">
      <div
        ref={contentRef}
        className="relative flex items-center gap-3 px-5 py-3
          w-[280px] h-[50px] rounded-full
          bg-gray-700/80 backdrop-blur-xl shadow-2xl"
      >
        {/* Left: Microphone icon */}
        <Mic className="w-4 h-4 text-white/60 flex-shrink-0" />

        {/* Center: Ready state indicator */}
        <div className="flex-1 flex items-center justify-center">
          <span className="text-white/60 text-xs">准备就绪</span>
        </div>

        {/* Right: Close button */}
        <button
          onClick={handleClose}
          className="flex-shrink-0 w-5 h-5 rounded-full bg-gray-600/60 hover:bg-gray-600/80
            flex items-center justify-center transition-colors"
        >
          <X className="w-3 h-3 text-white/90" />
        </button>
      </div>
    </div>
  );
};
