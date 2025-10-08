use std::ptr;
use wasm_bindgen::prelude::*;

/// WebAssembly wrapper for Opus decoder
#[wasm_bindgen]
pub struct OpusDecoder {
    decoder: *mut u8,
    channels: i32,
    sample_rate: i32,
}

/// Opus error codes
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusError {
    BadArg = -1,
    BufferTooSmall = -2,
    InternalError = -3,
    InvalidPacket = -4,
    Unimplemented = -5,
    InvalidState = -6,
    AllocFail = -7,
}

impl OpusError {
    fn from_code(code: i32) -> Option<Self> {
        match code {
            -1 => Some(OpusError::BadArg),
            -2 => Some(OpusError::BufferTooSmall),
            -3 => Some(OpusError::InternalError),
            -4 => Some(OpusError::InvalidPacket),
            -5 => Some(OpusError::Unimplemented),
            -6 => Some(OpusError::InvalidState),
            -7 => Some(OpusError::AllocFail),
            _ => None,
        }
    }
}

#[wasm_bindgen]
impl OpusDecoder {
    /// Create a new Opus decoder
    ///
    /// # Parameters
    /// - `sample_rate`: Sample rate, typically 48000, 24000, 16000, 12000, or 8000
    /// - `channels`: Number of channels, 1 (mono) or 2 (stereo)
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: i32, channels: i32) -> Result<OpusDecoder, JsValue> {
        // Validate parameters
        if channels != 1 && channels != 2 {
            return Err(JsValue::from_str("channels must be 1 or 2"));
        }

        if ![8000, 12000, 16000, 24000, 48000].contains(&sample_rate) {
            return Err(JsValue::from_str("invalid sample rate"));
        }

        unsafe {
            let mut error: i32 = 0;

            // Call audiopus_sys C FFI functions
            let decoder =
                audiopus_sys::opus_decoder_create(sample_rate, channels, &mut error as *mut i32);

            if error != 0 {
                return Err(JsValue::from_str(&format!(
                    "Failed to create decoder: error code {}",
                    error
                )));
            }

            if decoder.is_null() {
                return Err(JsValue::from_str("Failed to create decoder: null pointer"));
            }

            Ok(OpusDecoder {
                decoder: decoder as *mut u8,
                channels,
                sample_rate,
            })
        }
    }

    /// Decode an Opus packet
    ///
    /// # Parameters
    /// - `data`: Opus encoded audio data
    /// - `frame_size`: Number of samples per channel (e.g., 960 for 20ms @ 48kHz)
    ///
    /// # Returns
    /// Decoded PCM data (f32, range -1.0 to 1.0)
    pub fn decode(&mut self, data: &[u8], frame_size: i32) -> Result<Vec<f32>, JsValue> {
        if frame_size <= 0 {
            return Err(JsValue::from_str("frame_size must be positive"));
        }

        let total_samples = (frame_size * self.channels) as usize;
        let mut output = vec![0f32; total_samples];

        unsafe {
            let samples = audiopus_sys::opus_decode_float(
                self.decoder as *mut audiopus_sys::OpusDecoder,
                data.as_ptr(),
                data.len() as i32,
                output.as_mut_ptr(),
                frame_size,
                0, // decode_fec: 0 means no FEC
            );

            if samples < 0 {
                let error_msg = if let Some(err) = OpusError::from_code(samples) {
                    format!("Decode error: {:?} ({})", err, samples)
                } else {
                    format!("Decode error: unknown error code {}", samples)
                };
                return Err(JsValue::from_str(&error_msg));
            }

            // Adjust output size to actual decoded samples
            let actual_samples = (samples * self.channels) as usize;
            output.truncate(actual_samples);
        }

        Ok(output)
    }

    /// Reset decoder state
    pub fn reset(&mut self) -> Result<(), JsValue> {
        unsafe {
            let result = audiopus_sys::opus_decoder_ctl(
                self.decoder as *mut audiopus_sys::OpusDecoder,
                audiopus_sys::OPUS_RESET_STATE as i32,
            );

            if result != audiopus_sys::OPUS_OK as i32 {
                return Err(JsValue::from_str(&format!(
                    "Failed to reset decoder: error code {}",
                    result
                )));
            }
        }

        Ok(())
    }

    /// Get decoder gain
    pub fn get_gain(&self) -> Result<i32, JsValue> {
        unsafe {
            let mut gain: i32 = 0;
            let result = audiopus_sys::opus_decoder_ctl(
                self.decoder as *mut audiopus_sys::OpusDecoder,
                audiopus_sys::OPUS_GET_GAIN_REQUEST as i32,
                &mut gain as *mut i32,
            );

            if result != audiopus_sys::OPUS_OK as i32 {
                return Err(JsValue::from_str(&format!(
                    "Failed to get gain: error code {}",
                    result
                )));
            }

            Ok(gain)
        }
    }

    /// Set decoder gain
    pub fn set_gain(&mut self, gain: i32) -> Result<(), JsValue> {
        unsafe {
            let result = audiopus_sys::opus_decoder_ctl(
                self.decoder as *mut audiopus_sys::OpusDecoder,
                audiopus_sys::OPUS_SET_GAIN_REQUEST as i32,
                gain,
            );

            if result != audiopus_sys::OPUS_OK as i32 {
                return Err(JsValue::from_str(&format!(
                    "Failed to set gain: error code {}",
                    result
                )));
            }
        }

        Ok(())
    }

    /// Get number of channels
    #[wasm_bindgen(getter)]
    pub fn channels(&self) -> i32 {
        self.channels
    }

    /// Get sample rate
    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }
}

impl Drop for OpusDecoder {
    fn drop(&mut self) {
        unsafe {
            if !self.decoder.is_null() {
                audiopus_sys::opus_decoder_destroy(self.decoder as *mut audiopus_sys::OpusDecoder);
                self.decoder = ptr::null_mut();
            }
        }
    }
}

/// Get Opus library version string
#[wasm_bindgen]
pub fn opus_get_version() -> String {
    unsafe {
        let version_ptr = audiopus_sys::opus_get_version_string();
        if version_ptr.is_null() {
            return "unknown".to_string();
        }

        let c_str = std::ffi::CStr::from_ptr(version_ptr);
        c_str.to_string_lossy().into_owned()
    }
}

/// Test function: verify basic functionality
#[wasm_bindgen]
pub fn test_basic_functionality() -> Result<String, JsValue> {
    // Test 1: Create decoder
    let decoder = OpusDecoder::new(48000, 2)?;

    // Test 2: Get version
    let version = opus_get_version();

    // Test 3: Check properties
    if decoder.channels() != 2 {
        return Err(JsValue::from_str("channels mismatch"));
    }

    if decoder.sample_rate() != 48000 {
        return Err(JsValue::from_str("sample_rate mismatch"));
    }

    Ok(format!("✅ All tests passed! Opus version: {}", version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let result = OpusDecoder::new(48000, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_channels() {
        let result = OpusDecoder::new(48000, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_sample_rate() {
        let result = OpusDecoder::new(44100, 2);
        assert!(result.is_err());
    }
}

/// WebAssembly wrapper for Opus encoder
#[wasm_bindgen]
pub struct OpusEncoder {
    encoder: *mut u8,
    channels: i32,
    sample_rate: i32,
}

#[wasm_bindgen]
impl OpusEncoder {
    /// Create a new Opus encoder
    ///
    /// # Parameters
    /// - `sample_rate`: Sample rate, typically 48000, 24000, 16000, 12000, or 8000
    /// - `channels`: Number of channels, 1 (mono) or 2 (stereo)
    /// - `application`: Application type (2048: VOIP, 2049: AUDIO, 2051: RESTRICTED_LOWDELAY)
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: i32, channels: i32, application: i32) -> Result<OpusEncoder, JsValue> {
        // 验证参数
        if channels != 1 && channels != 2 {
            return Err(JsValue::from_str("channels must be 1 or 2"));
        }

        let valid_rates = [8000, 12000, 16000, 24000, 48000];
        if !valid_rates.contains(&sample_rate) {
            return Err(JsValue::from_str("invalid sample_rate"));
        }

        unsafe {
            let mut error: i32 = 0;
            let encoder = audiopus_sys::opus_encoder_create(
                sample_rate,
                channels,
                application,
                &mut error as *mut i32,
            );

            if error != 0 || encoder.is_null() {
                if let Some(err) = OpusError::from_code(error) {
                    return Err(JsValue::from_str(&format!(
                        "Failed to create encoder: {:?}",
                        err
                    )));
                } else {
                    return Err(JsValue::from_str("Failed to create encoder: unknown error"));
                }
            }

            Ok(OpusEncoder {
                encoder: encoder as *mut u8,
                channels,
                sample_rate,
            })
        }
    }

    /// Encode audio data
    ///
    /// # Parameters
    /// - `pcm`: Input PCM audio data (f32 format, range -1.0 to 1.0)
    /// - `frame_size`: Number of samples per channel (must be one of 2.5ms, 5ms, 10ms, 20ms, 40ms, 60ms)
    ///
    /// # Returns
    /// Encoded Opus packet
    pub fn encode(&self, pcm: &[f32], frame_size: i32) -> Result<Vec<u8>, JsValue> {
        // Validate input size
        let expected_samples = (frame_size * self.channels) as usize;
        if pcm.len() != expected_samples {
            return Err(JsValue::from_str(&format!(
                "Invalid PCM size: expected {} samples, got {}",
                expected_samples,
                pcm.len()
            )));
        }

        unsafe {
            // Allocate output buffer (max 4000 bytes should be sufficient)
            let mut output = vec![0u8; 4000];

            let result = audiopus_sys::opus_encode_float(
                self.encoder as *mut audiopus_sys::OpusEncoder,
                pcm.as_ptr(),
                frame_size,
                output.as_mut_ptr(),
                output.len() as i32,
            );

            if result < 0 {
                if let Some(err) = OpusError::from_code(result) {
                    return Err(JsValue::from_str(&format!("Encode error: {:?}", err)));
                } else {
                    return Err(JsValue::from_str("Encode error: unknown"));
                }
            }

            // Truncate to actual encoded bytes
            output.truncate(result as usize);
            Ok(output)
        }
    }

    /// Set bitrate
    pub fn set_bitrate(&self, bitrate: i32) -> Result<(), JsValue> {
        unsafe {
            let result = audiopus_sys::opus_encoder_ctl(
                self.encoder as *mut audiopus_sys::OpusEncoder,
                4002, // OPUS_SET_BITRATE_REQUEST
                bitrate,
            );

            if result != 0 {
                return Err(JsValue::from_str(&format!(
                    "Failed to set bitrate: {}",
                    result
                )));
            }

            Ok(())
        }
    }

    /// Get number of channels
    pub fn channels(&self) -> i32 {
        self.channels
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }
}

impl Drop for OpusEncoder {
    fn drop(&mut self) {
        unsafe {
            if !self.encoder.is_null() {
                audiopus_sys::opus_encoder_destroy(self.encoder as *mut audiopus_sys::OpusEncoder);
                self.encoder = ptr::null_mut();
            }
        }
    }
}

/// Test function: verify encoder functionality
#[wasm_bindgen]
pub fn test_encoder_functionality() -> Result<String, JsValue> {
    // Create encoder (48kHz, stereo, AUDIO application)
    let encoder = OpusEncoder::new(48000, 2, 2049)?;

    // Set bitrate
    encoder.set_bitrate(128000)?;

    // Create test PCM data (20ms @ 48kHz = 960 samples per channel)
    let frame_size = 960;
    let pcm_data: Vec<f32> = (0..frame_size * 2)
        .map(|i| {
            // Generate 440Hz sine wave (A4 note)
            let t = i as f32 / 48000.0;
            (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5
        })
        .collect();

    // Encode
    let encoded = encoder.encode(&pcm_data, frame_size)?;

    Ok(format!(
        "✅ Encoder test passed! Encoded {} samples to {} bytes",
        pcm_data.len(),
        encoded.len()
    ))
}
