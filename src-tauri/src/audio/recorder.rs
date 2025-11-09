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
}

impl AudioRecorder {
    /// Create a new audio recorder
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: Arc::new(Mutex::new(None)),
            device_name: Arc::new(Mutex::new(String::new())),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Start recording
    pub fn start(&self) -> Result<(), AudioError> {
        let mut state = self.state.lock();

        if *state == RecordingState::Recording {
            return Err(AudioError::AlreadyRecording);
        }

        info!("Starting audio recording");

        // Get host and device
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        *self.device_name.lock() = device_name.clone();
        info!("Using audio device: {}", device_name);

        // Get supported config
        let supported_configs = device
            .supported_input_configs()
            .map_err(|e| AudioError::DeviceError(e.to_string()))?;

        // Find a compatible config
        let config = supported_configs
            .filter(|c| {
                c.channels() == self.config.channels
                    && c.min_sample_rate().0 <= self.config.sample_rate
                    && c.max_sample_rate().0 >= self.config.sample_rate
            })
            .next()
            .ok_or(AudioError::InvalidConfig)?
            .with_sample_rate(cpal::SampleRate(self.config.sample_rate));

        debug!("Using audio config: {:?}", config);

        // Build stream
        let buffer = self.buffer.clone();
        let err_buffer = buffer.clone();

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut buf = buffer.lock();
                    buf.extend_from_slice(data);
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                    // Clear buffer on error to prevent corrupted data
                    err_buffer.lock().clear();
                },
                None,
            )
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        // Start stream
        stream
            .play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

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
            drop(stream); // Stream is automatically stopped when dropped
        }

        // Get buffer
        let mut buffer = self.buffer.lock();
        let audio_data = buffer.clone();
        buffer.clear();

        // Update state
        *state = RecordingState::Idle;
        *self.start_time.lock() = None;

        info!("Audio recording stopped, captured {} samples", audio_data.len());
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

    /// Save recorded audio to WAV file
    pub fn save_wav(&self, path: &std::path::Path, data: &[i16]) -> Result<(), String> {
        let spec = hound::WavSpec {
            channels: self.config.channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: self.config.bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer =
            hound::WavWriter::create(path, spec).map_err(|e| format!("Failed to create WAV file: {}", e))?;

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
    let default_device_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = Some(&name) == default_device_name.as_ref();
            result.push(AudioDevice { name, is_default });
        }
    }

    Ok(result)
}

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
