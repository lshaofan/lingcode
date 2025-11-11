import { useState, useEffect, useRef } from 'react';
import { Mic, X, Trash2, Copy, CornerDownLeft } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useRecordingStore } from '../../stores';
import { useSettingsStore } from '../../stores';

export const RecordingFloat = () => {
  console.log('[RecordingFloat] 🎬 Component function called');

  const status = useRecordingStore((state) => state.state);
  const transcribedText = useRecordingStore((state) => state.transcribedText);
  const audioLevel = useRecordingStore((state) => state.audioLevel);

  // 直接从设置中读取操作模式，而不是从 recordingStore
  const settings = useSettingsStore((state) => state.settings);
  const operationMode = settings.operationMode || 'preview';

  console.log('[RecordingFloat] 📊 Current state:', { status, transcribedText: transcribedText?.substring(0, 50), audioLevel, operationMode });
  console.log('[RecordingFloat] 🎯 Operation mode:', operationMode);

  const startRecording = useRecordingStore((state) => state.startRecording);
  const stopRecording = useRecordingStore((state) => state.stopRecording);
  const clearText = useRecordingStore((state) => state.clearText);
  const copyText = useRecordingStore((state) => state.copyText);
  const insertText = useRecordingStore((state) => state.insertText);
  const setOperationMode = useRecordingStore((state) => state.setOperationMode);

  const [showCopiedFeedback, setShowCopiedFeedback] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  // Auto-resize window based on content size
  const resizeWindow = async () => {
    try {
      let width;
      let height;

      // 预览模式：使用大窗口
      if (operationMode === 'preview') {
        width = 900;
        height = 200;
      } else {
        // 直接插入模式：使用较窄的窗口
        width = 400;
        height = 120;
      }

      console.log('[RecordingFloat] Resizing window to:', width, 'x', height, 'Status:', status, 'Mode:', operationMode, 'Has text:', !!transcribedText);

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
  }, [status, transcribedText, operationMode]); // Resize when status, text or mode changes

  // 同步操作模式到 recordingStore（用于 stopRecording 逻辑）
  useEffect(() => {
    if (settings.operationMode) {
      setOperationMode(settings.operationMode);
      console.log('[RecordingFloat] ✅ Operation mode synced to recordingStore:', settings.operationMode);
    }
  }, [settings.operationMode, setOperationMode]);

  // 监听设置更新事件，实现跨窗口同步
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      console.log('[RecordingFloat] Setting up settings-updated listener...');
      unlisten = await listen('settings-updated', (event: any) => {
        console.log('[RecordingFloat] 🔄 Received settings-updated event:', event.payload);

        // 当操作模式或模型变化时，重新加载设置
        if (event.payload.key === 'operationMode') {
          console.log('[RecordingFloat] Operation mode changed, reloading window...');
          window.location.reload();
        } else if (event.payload.key === 'model') {
          console.log('[RecordingFloat] Model changed to:', event.payload.value);
          // 更新本地 settingsStore
          const { loadSettings } = useSettingsStore.getState();
          loadSettings().catch((error) => {
            console.error('[RecordingFloat] Failed to reload settings:', error);
          });
        }
      });
    };

    setupListener().catch((error) => {
      console.error('[RecordingFloat] Failed to setup settings-updated listener:', error);
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // 🚀 CRITICAL FIX: Use global flag to prevent duplicate notifications across component remounts
  // React StrictMode will cause component to mount/unmount/mount, so we need a flag outside component scope
  const initializedRef = useRef(false);

  useEffect(() => {
    // Prevent duplicate initialization in React StrictMode
    // NOTE: In React 18 StrictMode, components mount twice in development
    // This only prevents double-mounting, NOT window re-use issues
    if (initializedRef.current) {
      console.log('[RecordingFloat] ⚠️ Already initialized, skipping duplicate setup');
      return;
    }
    initializedRef.current = true;

    console.log('[RecordingFloat] 🚀 Mount useEffect running');

    // Set transparent background for the window
    document.documentElement.style.backgroundColor = 'transparent';
    document.documentElement.style.overflow = 'hidden';
    document.body.style.backgroundColor = 'transparent';
    document.body.style.overflow = 'hidden';
    document.body.style.margin = '0';
    document.body.style.padding = '0';

    console.log('[RecordingFloat] ✅ Component mounted and styles applied');

    // Register event listeners
    let unlistenStart: (() => void) | null = null;
    let unlistenStop: (() => void) | null = null;

    const notifyBackendReady = async () => {
      console.log('[RecordingFloat] 📤 Notifying backend window is ready...');
      try {
        await invoke('recording_window_ready');
        console.log('[RecordingFloat] ✅ Backend notified successfully');
      } catch (error) {
        console.error('[RecordingFloat] ❌ Failed to notify backend:', error);
      }
    };

    const startHandler = async () => {
      console.log('🔥 [RecordingFloat] START event received from shortcut');
      console.log('🔥 [RecordingFloat] Backend has already started recording, just updating UI...');
      try {
        // 🚀 CRITICAL FIX: 传递 skipBackendCall=true
        // 因为后端已经在快捷键处理函数中启动了录音
        // 这里只需要更新前端UI状态和启动计时器
        await startRecording(true);
      } catch (error) {
        console.error('[RecordingFloat] ❌ startRecording failed:', error);
        // 如果录音启动失败（比如权限被拒绝），隐藏悬浮窗
        const window = getCurrentWindow();
        await window.hide();
      }
    };

    const stopHandler = async () => {
      console.log('⏹️  [RecordingFloat] STOP event received');
      const settings = useSettingsStore.getState().settings;
      const mode = settings.operationMode || 'preview';
      const currentState = useRecordingStore.getState().state;

      if (mode === 'preview') {
        clearText();
      } else {
        if (currentState === 'recording' || currentState === 'error') {
          await stopRecording();
        } else {
          const window = getCurrentWindow();
          await window.hide();
          clearText();
        }
      }
    };

    // Setup listeners and notify backend
    const setup = async () => {
      unlistenStart = await listen('shortcut-start-recording', startHandler);
      unlistenStop = await listen('shortcut-stop-recording', stopHandler);
      console.log('[RecordingFloat] ✅ Listeners registered');
      await notifyBackendReady();
    };

    setup().catch((error) => {
      console.error('[RecordingFloat] ❌ Setup failed:', error);
    });

    return () => {
      console.log('[RecordingFloat] Cleaning up');
      if (unlistenStart) unlistenStart();
      if (unlistenStop) unlistenStop();
      // Reset initialized flag on real unmount
      initializedRef.current = false;
    };
  }, []);

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

    // 预览模式：直接隐藏窗口并清空文本
    if (operationMode === 'preview') {
      await window.hide();
      clearText();
    } else {
      // 直接插入模式：隐藏窗口并清空文本
      await window.hide();
      clearText();
    }
  };

  const handleCopy = async () => {
    await copyText();
    setShowCopiedFeedback(true);
    setTimeout(() => setShowCopiedFeedback(false), 2000);
  };

  const handleInsert = async () => {
    // 关闭窗口并插入文本（应用激活由后端处理）
    const window = getCurrentWindow();
    await window.hide();
    clearText();

    // 插入文本（后端会自动激活原应用）
    await insertText();
  };

  console.log('[RecordingFloat] 🎯 Rendering decision:', { status, hasText: !!transcribedText, operationMode });

  // 预览模式：始终显示录制状态，实时转录
  if (operationMode === 'preview') {
    console.log('[RecordingFloat] 🎨 Rendering preview mode UI');
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <div
          ref={contentRef}
          className="relative flex flex-col
            w-[880px]
            rounded-2xl
            bg-gray-700/85 backdrop-blur-xl shadow-2xl
            overflow-hidden"
        >
          {/* Top row: Icon + Text + Close button */}
          <div className="flex items-center gap-4 px-6 py-3.5">
            {/* Left: Microphone icon - status-aware */}
            <div className="relative flex-shrink-0">
              <Mic
                className={`w-4 h-4 transition-colors ${
                  status === 'recording'
                    ? 'text-red-500 animate-pulse'
                    : status === 'processing'
                    ? 'text-blue-500 animate-pulse'
                    : 'text-red-500 animate-pulse'
                }`}
              />
              {/* Recording indicator - pulsing ring */}
              {status === 'recording' && (
                <span className="absolute inset-0 flex items-center justify-center">
                  <span className="absolute w-6 h-6 bg-red-500/30 rounded-full animate-ping"></span>
                </span>
              )}
              {/* Processing indicator - spinning ring */}
              {status === 'processing' && (
                <span className="absolute inset-0 flex items-center justify-center">
                  <span className="absolute w-6 h-6 border-2 border-blue-500/50 border-t-blue-500 rounded-full animate-spin"></span>
                </span>
              )}
            </div>

            {/* Center: Text content area - shows real-time transcription or processing status */}
            <div className="flex-1 min-h-[24px] max-h-[60px] overflow-y-auto">
              {status === 'processing' ? (
                <p className="text-blue-400 text-sm italic leading-relaxed animate-pulse">
                  正在转录...
                </p>
              ) : transcribedText ? (
                <p className="text-white text-sm leading-relaxed whitespace-pre-wrap">
                  {transcribedText}
                </p>
              ) : (
                <p className="text-white/40 text-sm italic leading-relaxed">
                  正在录制...
                </p>
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

          {/* Bottom row: Action buttons - disabled during processing */}
          <div className="flex items-center justify-end gap-2 px-6 pb-3 pt-1">
            {/* Clear button */}
            <button
              onClick={clearText}
              disabled={!transcribedText || status === 'processing'}
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md
                border border-white/10 transition-all group
                ${transcribedText && status !== 'processing'
                  ? 'bg-white/5 hover:bg-white/10'
                  : 'bg-white/5 opacity-50 cursor-not-allowed'}`}
              title="清空"
            >
              <Trash2 className={`w-3 h-3 transition-colors ${transcribedText && status !== 'processing' ? 'text-white/70 group-hover:text-white' : 'text-white/40'}`} />
              <span className={`text-xs ${transcribedText && status !== 'processing' ? 'text-white/80 group-hover:text-white' : 'text-white/40'}`}>清空</span>
            </button>

            {/* Copy button */}
            <button
              onClick={handleCopy}
              disabled={!transcribedText || status === 'processing'}
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md
                border border-white/10 transition-all group relative
                ${transcribedText && status !== 'processing'
                  ? 'bg-white/5 hover:bg-white/10'
                  : 'bg-white/5 opacity-50 cursor-not-allowed'}`}
              title="复制"
            >
              <Copy className={`w-3 h-3 transition-colors ${transcribedText && status !== 'processing' ? 'text-white/70 group-hover:text-white' : 'text-white/40'}`} />
              <span className={`text-xs ${transcribedText && status !== 'processing' ? 'text-white/80 group-hover:text-white' : 'text-white/40'}`}>复制</span>
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
              disabled={!transcribedText || status === 'processing'}
              className={`flex items-center gap-1.5 px-3 py-1 rounded-md
                border transition-all group
                ${transcribedText && status !== 'processing'
                  ? 'bg-blue-600/80 hover:bg-blue-600 border-blue-500/50'
                  : 'bg-blue-600/30 border-blue-500/20 opacity-50 cursor-not-allowed'}`}
              title="插入"
            >
              <span className={`text-xs font-medium ${transcribedText && status !== 'processing' ? 'text-white' : 'text-white/40'}`}>插入</span>
              <CornerDownLeft className={`w-3 h-3 ${transcribedText && status !== 'processing' ? 'text-white' : 'text-white/40'}`} />
            </button>
          </div>
        </div>
      </div>
    );
  }

  // 直接插入模式：始终显示录制/转录UI（没有"准备就绪"状态）
  if (operationMode === 'direct') {
    console.log('[RecordingFloat] 🎨 Rendering direct mode UI, status:', status);
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <div
          ref={contentRef}
          className="relative flex flex-col
            w-[380px]
            rounded-2xl
            bg-gray-700/85 backdrop-blur-xl shadow-2xl
            overflow-hidden"
        >
          {/* Top row: Icon + Status + Close button */}
          <div className="flex items-center gap-4 px-6 py-3.5">
            {/* Left: Microphone icon with status animation */}
            <div className="relative flex-shrink-0">
              <Mic
                className={`w-4 h-4 transition-colors ${
                  status === 'recording'
                    ? 'text-red-500 animate-pulse'
                    : status === 'processing'
                    ? 'text-blue-500 animate-pulse'
                    : 'text-red-500 animate-pulse'
                }`}
              />
              {/* Recording indicator - pulsing ring */}
              {(status === 'recording' || status === 'idle') && (
                <span className="absolute inset-0 flex items-center justify-center">
                  <span className="absolute w-6 h-6 bg-red-500/30 rounded-full animate-ping"></span>
                </span>
              )}
              {/* Processing indicator - spinning ring */}
              {status === 'processing' && (
                <span className="absolute inset-0 flex items-center justify-center">
                  <span className="absolute w-6 h-6 border-2 border-blue-500/50 border-t-blue-500 rounded-full animate-spin"></span>
                </span>
              )}
            </div>

            {/* Center: Status text */}
            <div className="flex-1">
              <p className="text-white/70 text-sm">
                {status === 'processing' ? '正在转录...' : '正在录制...'}
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
        </div>
      </div>
    );
  }

  // 预览模式的默认状态不应该出现
  console.log('[RecordingFloat] ⚠️  Unexpected state - should not reach here');
  return null;
};
