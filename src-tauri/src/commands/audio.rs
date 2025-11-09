use crate::audio::{
    check_permission, list_devices, open_system_preferences, request_permission, AudioConfig,
    AudioDevice, AudioRecorder, PermissionStatus, RecordingInfo,
};
use parking_lot::Mutex;
use std::path::PathBuf;
use tauri::State;

/// Global audio recorder state
pub struct AudioState {
    pub recorder: Mutex<AudioRecorder>,
    pub last_recording: Mutex<Option<Vec<i16>>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            recorder: Mutex::new(AudioRecorder::default()),
            last_recording: Mutex::new(None),
        }
    }
}

/// Check microphone permission status
#[tauri::command]
pub async fn check_microphone_permission() -> Result<PermissionStatus, String> {
    Ok(check_permission())
}

/// Request microphone permission
#[tauri::command]
pub async fn request_microphone_permission() -> Result<PermissionStatus, String> {
    request_permission().await
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

/// Start audio recording
#[tauri::command]
pub async fn start_recording(state: State<'_, AudioState>) -> Result<(), String> {
    let recorder = state.recorder.lock();
    recorder.start().map_err(|e| e.to_string())
}

/// Stop audio recording and return sample count
#[tauri::command]
pub async fn stop_recording(state: State<'_, AudioState>) -> Result<usize, String> {
    let recorder = state.recorder.lock();
    let data = recorder.stop().map_err(|e| e.to_string())?;
    let sample_count = data.len();

    // Store recording for later use
    *state.last_recording.lock() = Some(data);

    Ok(sample_count)
}

/// Pause audio recording
#[tauri::command]
pub async fn pause_recording(state: State<'_, AudioState>) -> Result<(), String> {
    let recorder = state.recorder.lock();
    recorder.pause().map_err(|e| e.to_string())
}

/// Resume audio recording
#[tauri::command]
pub async fn resume_recording(state: State<'_, AudioState>) -> Result<(), String> {
    let recorder = state.recorder.lock();
    recorder.resume().map_err(|e| e.to_string())
}

/// Get recording info
#[tauri::command]
pub async fn get_recording_info(state: State<'_, AudioState>) -> Result<RecordingInfo, String> {
    let recorder = state.recorder.lock();
    Ok(recorder.get_info())
}

/// Clear audio buffer
#[tauri::command]
pub async fn clear_audio_buffer(state: State<'_, AudioState>) -> Result<(), String> {
    let recorder = state.recorder.lock();
    recorder.clear_buffer();
    Ok(())
}

/// Save last recording to WAV file
#[tauri::command]
pub async fn save_recording(
    state: State<'_, AudioState>,
    path: String,
) -> Result<(), String> {
    let last_recording = state.last_recording.lock();
    let data = last_recording
        .as_ref()
        .ok_or("No recording available")?;

    let recorder = state.recorder.lock();
    let path = PathBuf::from(path);
    recorder.save_wav(&path, data)?;

    Ok(())
}

/// Get last recording as base64-encoded WAV data
#[tauri::command]
pub async fn get_recording_data(state: State<'_, AudioState>) -> Result<String, String> {
    let last_recording = state.last_recording.lock();
    let data = last_recording
        .as_ref()
        .ok_or("No recording available")?;

    // Create temporary WAV in memory
    let cursor = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::new(cursor, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    for &sample in data.iter() {
        writer
            .write_sample(sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    let cursor = writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

    // Encode as base64
    let wav_data = cursor.into_inner();
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
