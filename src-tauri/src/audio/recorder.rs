use super::types::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Audio recorder for capturing microphone input
pub struct AudioRecorder {
    config: AudioConfig,
    state: Arc<Mutex<RecordingState>>,
    buffer: Arc<Mutex<Vec<i16>>>,
    stream: Arc<Mutex<Option<cpal::Stream>>>,
    device_name: Arc<Mutex<String>>,
    start_time: Arc<Mutex<Option<std::time::Instant>>>,
    actual_sample_rate: Arc<Mutex<u32>>,  // 实际使用的采样率
    preferred_device_name: Arc<Mutex<Option<String>>>,  // 用户选择的设备名称
}

impl AudioRecorder {
    /// Create a new audio recorder
    pub fn new(config: AudioConfig) -> Self {
        let sample_rate = config.sample_rate;
        Self {
            config,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: Arc::new(Mutex::new(None)),
            device_name: Arc::new(Mutex::new(String::new())),
            start_time: Arc::new(Mutex::new(None)),
            actual_sample_rate: Arc::new(Mutex::new(sample_rate)),
            preferred_device_name: Arc::new(Mutex::new(None)),
        }
    }

    /// Set preferred device name
    pub fn set_preferred_device(&self, device_name: Option<String>) {
        *self.preferred_device_name.lock() = device_name;
        info!("Set preferred device: {:?}", self.preferred_device_name.lock());
    }

    /// Get preferred device name
    pub fn get_preferred_device(&self) -> Option<String> {
        self.preferred_device_name.lock().clone()
    }

    /// Start recording
    pub fn start(&self) -> Result<(), AudioError> {
        let mut state = self.state.lock();

        if *state == RecordingState::Recording {
            return Err(AudioError::AlreadyRecording);
        }

        info!("Starting audio recording");

        // Clear buffer before starting new recording
        self.buffer.lock().clear();
        info!("Cleared audio buffer before starting new recording");

        // Get host and device
        info!("Getting CPAL host...");
        let host = cpal::default_host();

        // 优先使用用户选择的设备，否则使用系统默认设备
        let device = if let Some(ref preferred_name) = *self.preferred_device_name.lock() {
            info!("Trying to use preferred device: {}", preferred_name);
            match get_device_by_name(preferred_name) {
                Ok(dev) => {
                    info!("✅ Using preferred device: {}", preferred_name);
                    dev
                }
                Err(e) => {
                    warn!("⚠️  Preferred device not found: {}, falling back to default", e);
                    host.default_input_device()
                        .ok_or(AudioError::DeviceNotFound)?
                }
            }
        } else {
            info!("Getting default input device...");
            host.default_input_device()
                .ok_or(AudioError::DeviceNotFound)?
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        *self.device_name.lock() = device_name.clone();
        info!("Using audio device: {}", device_name);

        // Get supported configs
        info!("Getting supported input configs...");
        let supported_configs = device
            .supported_input_configs()
            .map_err(|e| AudioError::DeviceError(e.to_string()))?;

        // Collect configs into a Vec to avoid iterator consumption issues
        let configs: Vec<_> = supported_configs.collect();
        info!("Found {} supported configurations", configs.len());

        if configs.is_empty() {
            return Err(AudioError::InvalidConfig);
        }

        // Log all available configs for debugging
        for (i, c) in configs.iter().enumerate() {
            info!("Config {}: channels={}, sample_rate=[{}-{}]",
                i, c.channels(), c.min_sample_rate().0, c.max_sample_rate().0);
        }

        // Try to find a mono config first (preferred for speech recognition)
        let selected_config = configs
            .iter()
            .find(|c| c.channels() == 1)
            .or_else(|| {
                warn!("No mono config found, using first available config");
                configs.first()
            })
            .ok_or(AudioError::InvalidConfig)?;

        info!("Selected config: channels={}, sample_rate=[{}-{}]",
            selected_config.channels(),
            selected_config.min_sample_rate().0,
            selected_config.max_sample_rate().0);

        // Use a safe sample rate within the device's supported range
        // Try 16kHz if supported, otherwise use the device's default
        let target_rate = if selected_config.min_sample_rate().0 <= 16000
            && selected_config.max_sample_rate().0 >= 16000 {
            16000
        } else {
            // Use a common rate that's likely supported, or fall back to min rate
            let common_rates = [48000, 44100, 32000, 24000, 22050];
            common_rates
                .iter()
                .find(|&&rate| {
                    rate >= selected_config.min_sample_rate().0
                        && rate <= selected_config.max_sample_rate().0
                })
                .copied()
                .unwrap_or(selected_config.min_sample_rate().0)
        };

        info!("Using sample rate: {} Hz", target_rate);

        // 保存实际使用的采样率
        *self.actual_sample_rate.lock() = target_rate;

        let config = selected_config
            .clone()
            .with_sample_rate(cpal::SampleRate(target_rate));

        debug!("Using audio config: {:?}", config);

        // Create and start audio stream
        info!("Creating and starting audio stream...");
        let buffer = self.buffer.clone();

        // Build stream based on the sample format
        let stream_config: cpal::StreamConfig = config.clone().into();
        let sample_format = config.sample_format();

        info!("Sample format: {:?}", sample_format);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                info!("Building F32 stream...");
                let buf = buffer.clone();
                let callback_count = Arc::new(Mutex::new(0u32));
                let cc = callback_count.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let mut count = cc.lock();
                            *count += 1;
                            if *count == 1 {
                                info!("🎤 Audio callback triggered! Receiving {} samples", data.len());
                            }
                            if *count % 100 == 0 {
                                info!("🎤 Audio callback #{}, buffer has {} samples", *count, data.len());
                            }

                            if !data.is_empty() {
                                // Convert f32 to i16 for storage
                                let i16_data: Vec<i16> = data
                                    .iter()
                                    .map(|&sample| {
                                        let clamped = sample.max(-1.0).min(1.0);
                                        (clamped * 32767.0) as i16
                                    })
                                    .collect();

                                let mut buffer_lock = buf.lock();
                                buffer_lock.extend(i16_data);

                                // Log buffer size periodically
                                if buffer_lock.len() % 16000 == 0 {
                                    debug!("Audio buffer size: {} samples", buffer_lock.len());
                                }
                            }
                        },
                        move |err| {
                            error!("Audio stream error: {}", err);
                        },
                        None,
                    )
                    .map_err(|e| AudioError::StreamError(e.to_string()))?
            }
            cpal::SampleFormat::I16 => {
                info!("Building I16 stream...");
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            if !data.is_empty() {
                                let mut buf = buffer.lock();
                                buf.extend_from_slice(data);
                            }
                        },
                        move |err| {
                            error!("Audio stream error: {}", err);
                        },
                        None,
                    )
                    .map_err(|e| AudioError::StreamError(e.to_string()))?
            }
            sample_format => {
                error!("Unsupported sample format: {:?}", sample_format);
                return Err(AudioError::InvalidConfig);
            }
        };

        // Start the stream
        info!("Starting audio stream playback...");
        stream
            .play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        info!("✅ Audio stream is now playing");

        // Update state
        *self.stream.lock() = Some(stream);
        *state = RecordingState::Recording;
        *self.start_time.lock() = Some(std::time::Instant::now());

        info!("Audio recording started successfully");
        Ok(())
    }

    /// Stop recording and return the recorded audio buffer
    pub fn stop(&self) -> Result<Vec<i16>, AudioError> {
        let mut state = self.state.lock();

        if *state != RecordingState::Recording {
            return Err(AudioError::NotRecording);
        }

        info!("Stopping audio recording");

        // Stop stream
        let mut stream_lock = self.stream.lock();
        if let Some(stream) = stream_lock.take() {
            info!("Dropping audio stream to stop recording");
            drop(stream); // Stream is automatically stopped when dropped
        }

        // Get buffer
        let mut buffer = self.buffer.lock();
        let audio_data = buffer.clone();
        let samples_captured = audio_data.len();
        buffer.clear();

        // Update state
        *state = RecordingState::Idle;
        *self.start_time.lock() = None;

        let actual_rate = *self.actual_sample_rate.lock();
        info!(
            "Audio recording stopped, captured {} samples ({:.2} seconds at {}Hz)",
            samples_captured,
            samples_captured as f32 / actual_rate as f32,
            actual_rate
        );

        if samples_captured == 0 {
            warn!("⚠️ No audio samples were captured! The microphone may not be working.");
        }

        Ok(audio_data)
    }

    /// Pause recording
    pub fn pause(&self) -> Result<(), AudioError> {
        let mut state = self.state.lock();

        if *state != RecordingState::Recording {
            return Err(AudioError::NotRecording);
        }

        // Pause stream
        if let Some(stream) = self.stream.lock().as_ref() {
            stream
                .pause()
                .map_err(|e| AudioError::StreamError(e.to_string()))?;
        }

        *state = RecordingState::Paused;
        info!("Audio recording paused");
        Ok(())
    }

    /// Resume recording
    pub fn resume(&self) -> Result<(), AudioError> {
        let mut state = self.state.lock();

        if *state != RecordingState::Paused {
            return Err(AudioError::NotRecording);
        }

        // Resume stream
        if let Some(stream) = self.stream.lock().as_ref() {
            stream
                .play()
                .map_err(|e| AudioError::StreamError(e.to_string()))?;
        }

        *state = RecordingState::Recording;
        info!("Audio recording resumed");
        Ok(())
    }

    /// Get current recording info
    pub fn get_info(&self) -> RecordingInfo {
        let state = *self.state.lock();
        let buffer = self.buffer.lock();
        let device_name = self.device_name.lock().clone();

        let duration_ms = self
            .start_time
            .lock()
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0);

        RecordingInfo {
            state,
            duration_ms,
            sample_count: buffer.len(),
            device_name,
        }
    }

    /// Get current state
    pub fn state(&self) -> RecordingState {
        *self.state.lock()
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Clear buffer without stopping
    pub fn clear_buffer(&self) {
        self.buffer.lock().clear();
    }

    /// Get actual sample rate being used
    pub fn actual_sample_rate(&self) -> u32 {
        *self.actual_sample_rate.lock()
    }

    /// Save recorded audio to WAV file
    pub fn save_wav(&self, path: &std::path::Path, data: &[i16]) -> Result<(), String> {
        let spec = hound::WavSpec {
            channels: self.config.channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)
            .map_err(|e| format!("Failed to create WAV file: {}", e))?;

        for &sample in data {
            writer
                .write_sample(sample)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

        info!("Saved {} samples to {:?}", data.len(), path);
        Ok(())
    }
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new(AudioConfig::default())
    }
}

/// Get list of available audio input devices
pub fn list_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let default_device_name = host.default_input_device().and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = Some(&name) == default_device_name.as_ref();
            // 使用设备名称作为ID（macOS上设备名称是唯一的）
            let id = name.clone();
            result.push(AudioDevice { id, name, is_default });
        }
    }

    Ok(result)
}

/// Get device by name
fn get_device_by_name(name: &str) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    for device in devices {
        if let Ok(device_name) = device.name() {
            if device_name == name {
                return Ok(device);
            }
        }
    }

    Err(format!("Device not found: {}", name))
}

// SAFETY: AudioRecorder is safe to Send because:
// 1. The cpal::Stream is wrapped in Arc<Mutex<Option<_>>> and only accessed through the mutex
// 2. The stream is created and used on a single thread (main thread) via Tauri commands
// 3. All other fields (Arc<Mutex<T>>) are already Send + Sync
// 4. We never move the Stream object itself across threads
#[cfg(target_os = "macos")]
unsafe impl Send for AudioRecorder {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_creation() {
        let recorder = AudioRecorder::default();
        assert_eq!(recorder.state(), RecordingState::Idle);
        assert_eq!(recorder.buffer_size(), 0);
    }

    #[test]
    fn test_list_devices() {
        let devices = list_devices();
        assert!(devices.is_ok());
    }
}
