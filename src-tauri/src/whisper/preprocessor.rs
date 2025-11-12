/// 音频预处理模块
/// 用于将音频数据转换为 Whisper 模型所需的格式

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PreprocessError {
    InvalidSampleRate(u32),
    InvalidChannels(u16),
    EmptyAudioData,
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreprocessError::InvalidSampleRate(rate) => {
                write!(f, "Invalid sample rate: {}. Expected 16000Hz", rate)
            }
            PreprocessError::InvalidChannels(channels) => {
                write!(f, "Invalid channels: {}. Expected mono (1 channel)", channels)
            }
            PreprocessError::EmptyAudioData => {
                write!(f, "Audio data is empty")
            }
        }
    }
}

impl Error for PreprocessError {}

/// 将 i16 音频样本转换为 f32 格式（范围 [-1.0, 1.0]）
///
/// Whisper 模型需要 f32 格式的音频数据
/// 转换公式：f32 = i16 / 32768.0
pub fn convert_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|&s| s as f32 / 32768.0).collect()
}

/// 重采样音频数据从 48kHz 到 16kHz
///
/// Whisper 模型需要 16kHz 的音频数据
/// 使用简单的降采样（每3个样本取1个）
pub fn resample_48khz_to_16khz(samples: &[f32]) -> Vec<f32> {
    // 48kHz -> 16kHz = 3:1 ratio
    // 使用简单的降采样方法：每3个样本取1个
    samples.iter().step_by(3).copied().collect()
}

/// 验证采样率是否为 16kHz
pub fn validate_sample_rate(rate: u32) -> Result<(), PreprocessError> {
    if rate != 16000 {
        return Err(PreprocessError::InvalidSampleRate(rate));
    }
    Ok(())
}

/// 验证声道数是否为单声道
pub fn validate_channels(channels: u16) -> Result<(), PreprocessError> {
    if channels != 1 {
        return Err(PreprocessError::InvalidChannels(channels));
    }
    Ok(())
}

/// 音频归一化处理
/// 将音频音量归一化到合适的范围
pub fn normalize_audio(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }

    // 找到最大绝对值
    let max_abs = samples
        .iter()
        .map(|&s| s.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    // 如果最大值太小，不进行归一化（避免放大噪音）
    if max_abs < 0.01 {
        return;
    }

    // 归一化到 [-0.95, 0.95] 范围（留一点余量避免削波）
    let scale = 0.95 / max_abs;
    for sample in samples.iter_mut() {
        *sample *= scale;
    }
}

/// 验证音频数据是否有效
pub fn validate_audio_data(samples: &[f32]) -> Result<(), PreprocessError> {
    if samples.is_empty() {
        return Err(PreprocessError::EmptyAudioData);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_i16_to_f32() {
        let i16_samples = vec![0i16, 16384, -16384, 32767, -32768];
        let f32_samples = convert_i16_to_f32(&i16_samples);

        assert!((f32_samples[0] - 0.0).abs() < 0.0001);
        assert!((f32_samples[1] - 0.5).abs() < 0.001);
        assert!((f32_samples[2] + 0.5).abs() < 0.001);
        assert!((f32_samples[3] - 0.9999).abs() < 0.001);
        assert!((f32_samples[4] + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_validate_sample_rate() {
        assert!(validate_sample_rate(16000).is_ok());
        assert!(validate_sample_rate(44100).is_err());
        assert!(validate_sample_rate(48000).is_err());
    }

    #[test]
    fn test_validate_channels() {
        assert!(validate_channels(1).is_ok());
        assert!(validate_channels(2).is_err());
    }

    #[test]
    fn test_normalize_audio() {
        let mut samples = vec![0.5, -0.3, 0.8, -0.6];
        normalize_audio(&mut samples);

        // 最大值应该接近 0.95
        let max_abs = samples.iter().map(|&s| s.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((max_abs - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_validate_empty_audio() {
        let empty: Vec<f32> = vec![];
        assert!(validate_audio_data(&empty).is_err());

        let non_empty = vec![0.1, 0.2];
        assert!(validate_audio_data(&non_empty).is_ok());
    }
}
