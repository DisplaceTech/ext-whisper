//! Strict WAV ingestion: 16kHz, mono, 16-bit PCM — exactly what
//! whisper.cpp's encoder consumes, nothing else.
//!
//! Accepting only one shape is a deliberate v0.1 decision (see PLAN.md):
//! every accepted-but-resampled format hides a quality decision the
//! caller should own, and audio *decoding* (mp3/m4a/ogg) is a separate,
//! much larger dependency surface. The error messages carry the exact
//! ffmpeg invocation that produces a conforming file, so the failure
//! mode is a copy-paste away from the fix.

use std::path::Path;

use crate::error::WhisperError;

/// The ffmpeg one-liner embedded in every format error.
const FFMPEG_HINT: &str =
    "convert with: ffmpeg -i input.ext -ar 16000 -ac 1 -c:a pcm_s16le out.wav";

/// Read `path` as 16kHz mono 16-bit PCM WAV and return normalized f32
/// samples in `[-1.0, 1.0]` — the input layout `whisper_full` expects.
pub fn read_16khz_mono(path: &str) -> Result<Vec<f32>, WhisperError> {
    if !Path::new(path).is_file() {
        return Err(WhisperError::Audio(format!("no such file: {path}")));
    }

    let mut reader = hound::WavReader::open(path)
        .map_err(|e| WhisperError::Audio(format!("{path}: not a readable WAV file ({e})")))?;

    let spec = reader.spec();

    if spec.channels != 1 {
        return Err(WhisperError::Audio(format!(
            "{path}: expected mono audio, got {} channels — {FFMPEG_HINT}",
            spec.channels
        )));
    }

    if spec.sample_rate != 16_000 {
        return Err(WhisperError::Audio(format!(
            "{path}: expected a 16000Hz sample rate, got {}Hz — {FFMPEG_HINT}",
            spec.sample_rate
        )));
    }

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(WhisperError::Audio(format!(
            "{path}: expected 16-bit PCM samples, got {}-bit {} — {FFMPEG_HINT}",
            spec.bits_per_sample,
            match spec.sample_format {
                hound::SampleFormat::Int => "integer",
                hound::SampleFormat::Float => "float",
            },
        )));
    }

    let samples = reader
        .samples::<i16>()
        .map(|s| s.map(|v| f32::from(v) / 32_768.0))
        .collect::<Result<Vec<f32>, _>>()
        .map_err(|e| WhisperError::Audio(format!("{path}: truncated or corrupt WAV data ({e})")))?;

    if samples.is_empty() {
        return Err(WhisperError::Audio(format!(
            "{path}: WAV file contains no samples"
        )));
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(spec: hound::WavSpec, samples: &[i16]) -> tempfile::NamedTempFile {
        let file = tempfile::Builder::new()
            .suffix(".wav")
            .tempfile()
            .expect("tempfile");
        let mut writer = hound::WavWriter::create(file.path(), spec).expect("writer");
        for &s in samples {
            writer.write_sample(s).expect("sample");
        }
        writer.finalize().expect("finalize");
        file
    }

    fn spec(channels: u16, sample_rate: u32, bits: u16) -> hound::WavSpec {
        hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: bits,
            sample_format: hound::SampleFormat::Int,
        }
    }

    #[test]
    fn reads_conforming_wav_and_normalizes() {
        let file = write_wav(spec(1, 16_000, 16), &[0, 16_384, -32_768, 32_767]);

        let samples = read_16khz_mono(file.path().to_str().unwrap()).expect("should read");

        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0], 0.0);
        assert!((samples[1] - 0.5).abs() < 1e-6);
        assert_eq!(samples[2], -1.0);
        assert!((samples[3] - 0.99997).abs() < 1e-4);
    }

    #[test]
    fn rejects_stereo_with_ffmpeg_hint() {
        let file = write_wav(spec(2, 16_000, 16), &[0, 0, 0, 0]);

        let err = read_16khz_mono(file.path().to_str().unwrap()).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("2 channels"), "got: {message}");
        assert!(message.contains("ffmpeg -i"), "got: {message}");
    }

    #[test]
    fn rejects_wrong_sample_rate() {
        let file = write_wav(spec(1, 44_100, 16), &[0, 0]);

        let message = read_16khz_mono(file.path().to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(message.contains("44100Hz"), "got: {message}");
    }

    #[test]
    fn rejects_non_16_bit_samples() {
        let file = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(file.path(), spec).unwrap();
        writer.write_sample(0.0f32).unwrap();
        writer.finalize().unwrap();

        let message = read_16khz_mono(file.path().to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(message.contains("32-bit float"), "got: {message}");
    }

    #[test]
    fn rejects_missing_file() {
        let message = read_16khz_mono("/no/such/file.wav")
            .unwrap_err()
            .to_string();

        assert!(message.contains("no such file"), "got: {message}");
    }

    #[test]
    fn rejects_non_wav_bytes() {
        let file = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        std::fs::write(file.path(), b"definitely not RIFF data").unwrap();

        let message = read_16khz_mono(file.path().to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(message.contains("not a readable WAV"), "got: {message}");
    }

    #[test]
    fn rejects_empty_wav() {
        let file = write_wav(spec(1, 16_000, 16), &[]);

        let message = read_16khz_mono(file.path().to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(message.contains("no samples"), "got: {message}");
    }
}
