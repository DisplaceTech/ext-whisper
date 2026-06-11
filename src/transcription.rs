//! `Displace\Whisper\Transcription` — the result of `Model::transcribe()`.
//!
//! Carries the full transcript plus whisper's time-aligned segments.
//! Segments surface to PHP as `['start' => float, 'end' => float,
//! 'text' => string]` rows (offsets in seconds) — deliberately the exact
//! shape `Displace\AI\Contracts\Transcriber::transcribe()` documents, so
//! a contracts adapter is `['text' => $t->text(), 'segments' =>
//! $t->segments()]` and nothing more.

use ext_php_rs::boxed::ZBox;
use ext_php_rs::prelude::*;
use ext_php_rs::types::ZendHashTable;

use crate::error::WhisperError;

/// One time-aligned segment, Rust-side. Timestamps are kept in whisper's
/// native centiseconds and converted to seconds at the PHP boundary.
pub(crate) struct Segment {
    pub start_cs: i64,
    pub end_cs: i64,
    pub text: String,
}

/// Result of a transcription run.
///
/// Read-only. Instances are produced by `Model::transcribe()`; direct
/// construction is refused — a transcription built by PHP would lie about
/// which model and audio produced it.
#[php_class]
#[php(name = "Displace\\Whisper\\Transcription")]
#[derive(Default)]
pub struct Transcription {
    text: String,
    segments: Vec<Segment>,
}

#[php_impl]
impl Transcription {
    /// Refuse direct construction.
    pub fn __construct() -> PhpResult<Self> {
        Err(WhisperError::InvalidConstruction(
            "Displace\\Whisper\\Transcription is produced by Model::transcribe(); \
             do not instantiate directly"
                .into(),
        )
        .into())
    }

    /// The full transcript — whisper's segments joined in order, trimmed.
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Time-aligned segments, in order. Each row is
    /// `['start' => float, 'end' => float, 'text' => string]` with
    /// offsets in seconds; the concatenated segment texts are equivalent
    /// to `text()` modulo whitespace.
    pub fn segments(&self) -> PhpResult<Vec<ZBox<ZendHashTable>>> {
        let mut rows = Vec::with_capacity(self.segments.len());

        for segment in &self.segments {
            let mut row = ZendHashTable::new();
            row.insert("start", segment.start_cs as f64 / 100.0)?;
            row.insert("end", segment.end_cs as f64 / 100.0)?;
            row.insert("text", segment.text.clone())?;
            rows.push(row);
        }

        Ok(rows)
    }

    /// Number of segments.
    pub fn count(&self) -> usize {
        self.segments.len()
    }

    /// End timestamp of the last segment, in seconds — the transcribed
    /// duration. `0.0` for an (impossible in practice) empty result.
    pub fn duration(&self) -> f64 {
        self.segments
            .last()
            .map_or(0.0, |s| s.end_cs as f64 / 100.0)
    }
}

impl Transcription {
    /// Rust-side constructor used by `Model::transcribe()`.
    pub(crate) fn from_segments(segments: Vec<Segment>) -> Self {
        let text = segments
            .iter()
            .map(|s| s.text.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        Self { text, segments }
    }
}
