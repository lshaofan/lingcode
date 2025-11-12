use crate::audio::{
    check_permission, list_devices, open_system_preferences, request_permission, AudioConfig,
    AudioDevice, AudioRecorder, PermissionStatus, RecordingInfo,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Global audio recorder - using Lazy static to avoid Send/Sync issues with cpal::Stream
pub static AUDIO_RECORDER: Lazy<Mutex<AudioRecorder>> =
    Lazy::new(|| Mutex::new(AudioRecorder::default()));

/// Global last recording buffer
pub static LAST_RECORDING: Lazy<Mutex<Option<Vec<i16>>>> = Lazy::new(|| Mutex::new(None));

/// Global flag to track if audio system is available
static AUDIO_AVAILABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Initialize audio system with error handling
#[tauri::command]
pub async fn initialize_audio_system() -> Result<(), String> {
    info!("Initializing audio system...");

    // Test basic audio device access
    match list_devices() {
        Ok(devices) => {
            info!(
                "Audio system initialized successfully. Found {} devices",
                devices.len()
            );
            if devices.is_empty() {
                warn!("No audio devices found - recording may not work");
                AUDIO_AVAILABLE.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to initialize audio system: {}", e);
            AUDIO_AVAILABLE.store(false, std::sync::atomic::Ordering::SeqCst);
            Err(format!("Failed to initialize audio system: {}", e))
        }
    }
}

/// Check microphone permission status
#[tauri::command]
pub async fn check_microphone_permission() -> Result<String, String> {
    info!("🎤 [Audio Command] check_microphone_permission called");
    let status = check_permission();
    let status_str = match status {
        PermissionStatus::Granted => "granted",
        PermissionStatus::Denied => "denied",
        PermissionStatus::NotDetermined => "not_determined",
        PermissionStatus::Restricted => "restricted",
    };
    info!("🎤 [Audio Command] Permission status: {}", status_str);
    Ok(status_str.to_string())
}

/// Request microphone permission
#[tauri::command]
pub async fn request_microphone_permission() -> Result<String, String> {
    info!("🎤 [Audio Command] request_microphone_permission called");
    let result = request_permission().await;
    match result {
        Ok(status) => {
            let status_str = match status {
                PermissionStatus::Granted => "granted",
                PermissionStatus::Denied => "denied",
                PermissionStatus::NotDetermined => "not_determined",
                PermissionStatus::Restricted => "restricted",
            };
            info!("🎤 [Audio Command] Permission request result: {}", status_str);
            Ok(status_str.to_string())
        }
        Err(e) => {
            error!("🎤 [Audio Command] Permission request failed: {}", e);
            Err(e)
        }
    }
}

/// Open system preferences for microphone settings
#[tauri::command]
pub async fn open_microphone_settings() -> Result<(), String> {
    open_system_preferences()
}

/// Get list of available audio input devices
#[tauri::command]
pub async fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    list_devices()
}

/// Set preferred audio input device
#[tauri::command]
pub async fn set_audio_device(device_id: String) -> Result<(), String> {
    info!("🎤 [Audio Command] set_audio_device called: {}", device_id);

    let recorder = AUDIO_RECORDER.lock();

    // 如果是 "auto"，清除偏好设置
    if device_id == "auto" {
        recorder.set_preferred_device(None);
        info!("🎤 [Audio Command] Cleared preferred device, will use system default");
    } else {
        // 验证设备是否存在
        let devices = list_devices()?;
        let device_exists = devices.iter().any(|d| d.id == device_id);

        if !device_exists {
            return Err(format!("Device not found: {}", device_id));
        }

        recorder.set_preferred_device(Some(device_id.clone()));
        info!("🎤 [Audio Command] Set preferred device to: {}", device_id);
    }

    Ok(())
}

/// Start audio recording
#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    info!("🎤 [Audio Command] start_recording called");

    // Check microphone permission first (but don't block if NotDetermined)
    let permission = check_permission();
    info!("🎤 [Audio Command] Permission status: {:?}", permission);

    if permission == PermissionStatus::Denied || permission == PermissionStatus::Restricted {
        error!("🎤 [Audio Command] Permission denied or restricted");
        return Err("Microphone permission denied. Please grant permission in System Preferences > Security & Privacy > Privacy > Microphone".to_string());
    }
    // If NotDetermined or Granted, proceed and let cpal trigger the permission prompt if needed

    // Check if audio system is available
    if !AUDIO_AVAILABLE.load(std::sync::atomic::Ordering::SeqCst) {
        error!("🎤 [Audio Command] Audio system not available");
        return Err(
            "Audio system is not available. Please check your audio devices and permissions."
                .to_string(),
        );
    }

    info!("🎤 [Audio Command] Starting audio recording via Tauri command");

    // First test if we can access audio devices
    info!("Testing audio device access...");
    match list_devices() {
        Ok(devices) => {
            info!("Found {} audio devices", devices.len());
            if devices.is_empty() {
                return Err("No audio input devices found".to_string());
            }

            // Check if default device is available
            let has_default = devices.iter().any(|d| d.is_default);
            if !has_default {
                return Err("No default audio input device available".to_string());
            }
        }
        Err(e) => {
            error!("Failed to list audio devices: {}", e);
            return Err(format!("Failed to access audio devices: {}", e));
        }
    }

    // Try the real audio recorder with comprehensive panic protection
    let result = std::panic::catch_unwind(|| {
        // Get the audio recorder and start recording
        let recorder = AUDIO_RECORDER.lock();

        // Start recording within the same thread to avoid borrowing issues
        let start_result = recorder.start().map_err(|e| e.to_string());

        // Return the result for further processing
        start_result
    });

    match result {
        Ok(Ok(_)) => {
            info!("Audio recording started successfully via Tauri command");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Audio recording failed: {}", e);
            Err(format!("Failed to start recording: {}. Please check your microphone and system permissions.", e))
        }
        Err(panic) => {
            error!("Panic occurred during audio recording: {:?}", panic);

            // Mark audio system as unavailable after panic
            AUDIO_AVAILABLE.store(false, std::sync::atomic::Ordering::SeqCst);

            // Check if this might be a known cpal issue
            if let Some(panic_msg) = panic.downcast_ref::<&str>() {
                if panic_msg.contains("cpal") || panic_msg.contains("audio") {
                    return Err("Audio system temporarily unavailable. This is a known issue with cpal on some macOS configurations. Please try again or restart the application.".to_string());
                }
            }

            Err("Audio recording encountered a system error. This may be due to audio device issues or permissions.".to_string())
        }
    }
}

/// Stop audio recording and return sample count
#[tauri::command]
pub async fn stop_recording() -> Result<usize, String> {
    info!("🛑 [Audio Command] stop_recording called");
    info!("Stopping audio recording via Tauri command");

    let result = std::panic::catch_unwind(|| {
        let recorder = AUDIO_RECORDER.lock();
        recorder.stop().map_err(|e| e.to_string())
    });

    let sample_count = match result {
        Ok(Ok(data)) => {
            // 真实的录音数据
            let count = data.len();

            // 计算音频数据的简单哈希以便调试
            let sum: i64 = data.iter().take(100).map(|&x| x as i64).sum();
            let avg = if !data.is_empty() { sum / data.len().min(100) as i64 } else { 0 };
            info!("📊 Audio data stats: {} samples, first 100 avg: {}", count, avg);

            *LAST_RECORDING.lock() = Some(data);
            info!("Real audio recording stopped, captured {} samples", count);
            count
        }
        Ok(Err(e)) => {
            error!("Failed to stop real audio recording: {}", e);
            return Err(format!("Failed to stop recording: {}", e));
        }
        Err(panic) => {
            error!("Panic occurred while stopping audio recording: {:?}", panic);
            return Err("Failed to stop audio recording due to system error".to_string());
        }
    };

    Ok(sample_count)
}

/// Pause audio recording
#[tauri::command]
pub async fn pause_recording() -> Result<(), String> {
    let recorder = AUDIO_RECORDER.lock();
    recorder.pause().map_err(|e| e.to_string())
}

/// Resume audio recording
#[tauri::command]
pub async fn resume_recording() -> Result<(), String> {
    let recorder = AUDIO_RECORDER.lock();
    recorder.resume().map_err(|e| e.to_string())
}

/// Get recording info
#[tauri::command]
pub async fn get_recording_info() -> Result<RecordingInfo, String> {
    let recorder = AUDIO_RECORDER.lock();
    Ok(recorder.get_info())
}

/// Clear audio buffer
#[tauri::command]
pub async fn clear_audio_buffer() -> Result<(), String> {
    let recorder = AUDIO_RECORDER.lock();
    recorder.clear_buffer();
    Ok(())
}

/// Save last recording to WAV file
#[tauri::command]
pub async fn save_recording(path: String) -> Result<(), String> {
    let last_recording = LAST_RECORDING.lock();
    let data = last_recording.as_ref().ok_or("No recording available")?;

    let recorder = AUDIO_RECORDER.lock();
    let path = PathBuf::from(path);
    recorder.save_wav(&path, data)?;

    Ok(())
}

/// Get last recording as base64-encoded WAV data
#[tauri::command]
pub async fn get_recording_data() -> Result<String, String> {
    let last_recording = LAST_RECORDING.lock();
    let data = last_recording.as_ref().ok_or("No recording available")?;

    // Create temporary WAV file
    let temp_path = std::env::temp_dir().join(format!("recording_{}.wav", std::process::id()));

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&temp_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;

    for &sample in data.iter() {
        writer
            .write_sample(sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    // Finalize the writer
    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

    // Read the WAV data
    let wav_data =
        std::fs::read(&temp_path).map_err(|e| format!("Failed to read WAV file: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(base64::encode(&wav_data))
}

// Helper module for base64 encoding
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut result = String::new();
        let mut i = 0;

        while i + 3 <= data.len() {
            let b1 = data[i];
            let b2 = data[i + 1];
            let b3 = data[i + 2];

            let _ = write!(
                &mut result,
                "{}{}{}{}",
                CHARSET[(b1 >> 2) as usize] as char,
                CHARSET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char,
                CHARSET[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char,
                CHARSET[(b3 & 0x3f) as usize] as char,
            );

            i += 3;
        }

        // Handle remaining bytes
        match data.len() - i {
            1 => {
                let b1 = data[i];
                let _ = write!(
                    &mut result,
                    "{}{}==",
                    CHARSET[(b1 >> 2) as usize] as char,
                    CHARSET[((b1 & 0x03) << 4) as usize] as char,
                );
            }
            2 => {
                let b1 = data[i];
                let b2 = data[i + 1];
                let _ = write!(
                    &mut result,
                    "{}{}{}=",
                    CHARSET[(b1 >> 2) as usize] as char,
                    CHARSET[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char,
                    CHARSET[((b2 & 0x0f) << 2) as usize] as char,
                );
            }
            _ => {}
        }

        result
    }
}
