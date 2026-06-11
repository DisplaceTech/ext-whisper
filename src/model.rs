//! `Displace\Whisper\Model` — the loaded whisper model and its
//! transcription entry point.
//!
//! ```php
//! $model = \Displace\Whisper\Model::load('models/ggml-tiny.en.bin');
//!
//! $result = $model->transcribe('audio/meeting.wav');
//! echo $result->text();
//! foreach ($result->segments() as $s) {
//!     printf("[%6.2f → %6.2f] %s\n", $s['start'], $s['end'], $s['text']);
//! }
//!
//! $model->close();
//! ```
//!
//! A `Model` owns a [`WhisperContext`] (the in-memory weights). Each
//! `transcribe()` call creates a fresh [`whisper_rs::WhisperState`], runs
//! the full encoder/decoder pass synchronously, and drops the state —
//! no shared mutable state between calls, so concurrent `transcribe()`
//! calls on one handle are safe by construction (same per-call-context
//! design as ext-infer).

use std::path::Path;
use std::sync::Once;

use ext_php_rs::convert::FromZval;
use ext_php_rs::prelude::*;
use ext_php_rs::types::ZendHashTable;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::WhisperError;
use crate::transcription::{Segment, Transcription};
use crate::wav;

/// whisper.cpp logs generously to stderr (model layout, mel banks, timing).
/// Inside a PHP request that flood is noise — route it into the Rust `log`
/// facade (a no-op without a subscriber) unless the caller asks for the
/// raw firehose via `EXT_WHISPER_LOG=1`. `install_logging_hooks` mutates
/// process-global function pointers, so it runs exactly once.
fn quiet_logs_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("EXT_WHISPER_LOG").is_none() {
            whisper_rs::install_logging_hooks();
        }
    });
}

/// PHP-visible handle to a loaded whisper model.
///
/// Wraps an `Option<WhisperContext>` so `close()` releases the weights
/// deterministically rather than waiting on PHP's GC. After `close()`,
/// `transcribe()` throws `TranscriptionException`.
#[php_class]
#[php(name = "Displace\\Whisper\\Model")]
#[derive(Default)]
pub struct Model {
    inner: Option<WhisperContext>,
}

#[php_impl]
impl Model {
    /// Direct construction is not supported — use `Model::load()`.
    pub fn __construct() -> PhpResult<Self> {
        Err(WhisperError::InvalidConstruction(
            "use Displace\\Whisper\\Model::load() to construct a Model".into(),
        )
        .into())
    }

    /// Load a whisper model (ggml/GGUF `.bin` from the whisper.cpp model
    /// zoo) from disk.
    ///
    /// Recognised `$options` keys:
    /// - `use_gpu` (bool, default `false`) — CPU is the portable default
    ///   platform-wide; flip this on builds linked with GPU support.
    pub fn load(path: String, options: Option<&ZendHashTable>) -> PhpResult<Self> {
        quiet_logs_once();

        let use_gpu = get_bool(options, "use_gpu")?.unwrap_or(false);

        if !Path::new(&path).is_file() {
            return Err(WhisperError::ModelLoad(format!("no such file: {path}")).into());
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu(use_gpu);

        let context = WhisperContext::new_with_params(&path, params)
            .map_err(|e| WhisperError::ModelLoad(format!("{path}: {e}")))?;

        Ok(Self {
            inner: Some(context),
        })
    }

    /// Transcribe a 16kHz mono 16-bit PCM WAV file.
    ///
    /// Other rates/layouts/containers are rejected with an
    /// `AudioException` whose message includes the ffmpeg one-liner that
    /// produces a conforming file — decoding arbitrary audio is
    /// deliberately out of scope for v0.1 (see PLAN.md).
    ///
    /// Recognised `$options` keys:
    /// - `language` (string, default auto-detect) — ISO 639-1 hint, e.g.
    ///   `'en'`. English-only models (`*.en`) ignore it.
    /// - `translate` (bool, default `false`) — translate the result to
    ///   English (multilingual models only).
    /// - `threads` (int, default whisper.cpp's choice) — decoder thread
    ///   count; useful to pin below the host's core count in shared
    ///   environments.
    pub fn transcribe(
        &self,
        wavPath: String,
        options: Option<&ZendHashTable>,
    ) -> PhpResult<Transcription> {
        let context = self.inner.as_ref().ok_or(WhisperError::Closed)?;

        let language = get_string(options, "language")?;
        let translate = get_bool(options, "translate")?.unwrap_or(false);
        let threads = get_uint(options, "threads")?;

        let samples = wav::read_16khz_mono(&wavPath)?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_translate(translate);
        params.set_language(language.as_deref());

        if let Some(threads) = threads {
            if threads == 0 {
                return Err(WhisperError::InvalidOption {
                    name: "threads".into(),
                    reason: "must be at least 1 (omit the key for whisper.cpp's default)".into(),
                }
                .into());
            }
            params.set_n_threads(i32::try_from(threads).map_err(|_| {
                WhisperError::InvalidOption {
                    name: "threads".into(),
                    reason: "exceeds i32 range".into(),
                }
            })?);
        }

        let mut state = context
            .create_state()
            .map_err(|e| WhisperError::Transcription(format!("state creation failed: {e}")))?;

        state
            .full(params, &samples)
            .map_err(|e| WhisperError::Transcription(format!("whisper_full failed: {e}")))?;

        let n_segments = state.full_n_segments();
        let mut segments = Vec::with_capacity(usize::try_from(n_segments).unwrap_or(0));

        for i in 0..n_segments {
            let segment = state.get_segment(i).ok_or_else(|| {
                WhisperError::Transcription(format!("segment {i} vanished mid-read"))
            })?;

            segments.push(Segment {
                start_cs: segment.start_timestamp(),
                end_cs: segment.end_timestamp(),
                text: segment
                    .to_str_lossy()
                    .map(|s| s.into_owned())
                    .map_err(|e| {
                        WhisperError::Transcription(format!("segment {i} text read failed: {e}"))
                    })?,
            });
        }

        Ok(Transcription::from_segments(segments))
    }

    /// Release the underlying model weights. Idempotent.
    pub fn close(&mut self) {
        self.inner = None;
    }
}

// --- Option parsing helpers (same contract as ext-infer's) ------------------
//
// Missing array / missing key → Ok(None); present-but-wrong-type → hard
// error with the key named, never a silent default.

fn get_bool(opts: Option<&ZendHashTable>, key: &str) -> Result<Option<bool>, WhisperError> {
    let Some(zv) = opts.and_then(|o| o.get(key)) else {
        return Ok(None);
    };
    bool::from_zval(zv)
        .map(Some)
        .ok_or_else(|| WhisperError::InvalidOption {
            name: key.into(),
            reason: "expected bool".into(),
        })
}

fn get_string(opts: Option<&ZendHashTable>, key: &str) -> Result<Option<String>, WhisperError> {
    let Some(zv) = opts.and_then(|o| o.get(key)) else {
        return Ok(None);
    };
    String::from_zval(zv)
        .map(Some)
        .ok_or_else(|| WhisperError::InvalidOption {
            name: key.into(),
            reason: "expected string".into(),
        })
}

fn get_uint(opts: Option<&ZendHashTable>, key: &str) -> Result<Option<u32>, WhisperError> {
    let Some(zv) = opts.and_then(|o| o.get(key)) else {
        return Ok(None);
    };
    let n = i64::from_zval(zv).ok_or_else(|| WhisperError::InvalidOption {
        name: key.into(),
        reason: "expected integer".into(),
    })?;
    if n < 0 {
        return Err(WhisperError::InvalidOption {
            name: key.into(),
            reason: "must be non-negative".into(),
        });
    }
    u32::try_from(n)
        .map(Some)
        .map_err(|_| WhisperError::InvalidOption {
            name: key.into(),
            reason: "exceeds u32 range".into(),
        })
}
