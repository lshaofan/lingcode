use serde::{Deserialize, Serialize};

/// Audio recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Error,
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz (default: 16000)
    pub sample_rate: u32,
    /// Channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Buffer size in frames
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bits_per_sample: 16,
            buffer_size: 1024,
        }
    }
}

/// Audio recording info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingInfo {
    pub state: RecordingState,
    pub duration_ms: u64,
    pub sample_count: usize,
    pub device_name: String,
}

/// Audio device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

/// Microphone permission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

/// Audio error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioError {
    PermissionDenied,
    DeviceNotFound,
    DeviceError(String),
    StreamError(String),
    InvalidConfig,
    NotRecording,
    AlreadyRecording,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::PermissionDenied => write!(f, "Microphone permission denied"),
            AudioError::DeviceNotFound => write!(f, "Audio device not found"),
            AudioError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            AudioError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            AudioError::InvalidConfig => write!(f, "Invalid audio configuration"),
            AudioError::NotRecording => write!(f, "Not currently recording"),
            AudioError::AlreadyRecording => write!(f, "Already recording"),
        }
    }
}

impl std::error::Error for AudioError {}
