//! Error types for `ext-whisper`.
//!
//! On the Rust side, every fallible operation returns a [`WhisperError`].
//! On the PHP side, those errors surface as a small hierarchy of exception
//! classes rooted at `Displace\Whisper\WhisperException`, which itself
//! extends the built-in `\RuntimeException`.
//!
//! ```text
//! \RuntimeException
//!   └── Displace\Whisper\WhisperException
//!         ├── Displace\Whisper\ModelLoadException
//!         ├── Displace\Whisper\AudioException
//!         └── Displace\Whisper\TranscriptionException
//! ```

use ext_php_rs::exception::PhpException;
use ext_php_rs::ffi::zend_class_entry;
use ext_php_rs::prelude::*;
use ext_php_rs::zend::ClassEntry;
use thiserror::Error;

/// Internal error type for fallible operations inside the extension.
///
/// Variants map 1:1 to the PHP-visible exception subclasses defined below.
#[derive(Debug, Error)]
pub enum WhisperError {
    /// Failed to load a whisper model from disk.
    #[error("failed to load model: {0}")]
    ModelLoad(String),

    /// The audio input is unreadable or has the wrong format. The message
    /// always says exactly what was wrong *and* the ffmpeg one-liner that
    /// produces a conforming file — format errors are the most common
    /// first-run failure and the fix should be in the error itself.
    #[error("invalid audio input: {0}")]
    Audio(String),

    /// whisper.cpp failed mid-transcription.
    #[error("transcription failed: {0}")]
    Transcription(String),

    /// The model handle has already been closed.
    #[error("model has been closed")]
    Closed,

    /// A caller-supplied option was malformed (wrong type, out of range).
    #[error("invalid option {name}: {reason}")]
    InvalidOption {
        /// The option key as supplied by the PHP caller.
        name: String,
        /// Human-readable explanation of what was wrong.
        reason: String,
    },

    /// Direct `new ClassName()` is not supported.
    #[error("{0}")]
    InvalidConstruction(String),
}

impl From<WhisperError> for PhpException {
    fn from(err: WhisperError) -> Self {
        let message = err.to_string();
        match err {
            WhisperError::ModelLoad(_) => PhpException::from_class::<ModelLoadException>(message),
            WhisperError::Audio(_) => PhpException::from_class::<AudioException>(message),
            WhisperError::Transcription(_) | WhisperError::Closed => {
                PhpException::from_class::<TranscriptionException>(message)
            }
            WhisperError::InvalidOption { .. } | WhisperError::InvalidConstruction(_) => {
                PhpException::from_class::<WhisperException>(message)
            }
        }
    }
}

/// Base exception for all `ext-whisper` failures. Extends `\RuntimeException`
/// so existing `catch (\RuntimeException $e)` clauses continue to work.
#[php_class]
#[php(name = "Displace\\Whisper\\WhisperException")]
#[php(extends(ce = runtime_exception_ce, stub = "\\RuntimeException"))]
#[derive(Default)]
pub struct WhisperException;

/// Thrown when a model file cannot be loaded — missing path, unreadable
/// file, or an unsupported model format.
#[php_class]
#[php(name = "Displace\\Whisper\\ModelLoadException")]
#[php(extends(WhisperException))]
#[derive(Default)]
pub struct ModelLoadException;

/// Thrown when the audio input is unreadable or not 16kHz mono 16-bit PCM
/// WAV. The message includes the ffmpeg command that fixes it.
#[php_class]
#[php(name = "Displace\\Whisper\\AudioException")]
#[php(extends(WhisperException))]
#[derive(Default)]
pub struct AudioException;

/// Thrown when whisper.cpp fails after the model and audio both loaded —
/// state creation, decode, or use-after-close errors.
#[php_class]
#[php(name = "Displace\\Whisper\\TranscriptionException")]
#[php(extends(WhisperException))]
#[derive(Default)]
pub struct TranscriptionException;

// `\RuntimeException` is defined by SPL, which exposes its `zend_class_entry *`
// as a `PHPAPI` global — same convention as the engine's `zend_ce_*` globals.
// SPL is a built-in module loaded before user extensions, so by the time our
// MINIT runs this pointer is non-null. (`ClassEntry::try_find` would go
// through `EG(class_table)`, which is not yet initialized during MINIT.)
#[allow(non_upper_case_globals)]
unsafe extern "C" {
    static spl_ce_RuntimeException: *mut zend_class_entry;
}

/// Class-entry accessor for PHP's SPL `\RuntimeException`, used by the
/// `extends(ce = ...)` linkage on [`WhisperException`].
fn runtime_exception_ce() -> &'static ClassEntry {
    // SAFETY: `spl_ce_RuntimeException` is a stable PHPAPI symbol exported by
    // any SAPI we support. It is written once during SPL's MINIT (well before
    // ours) and never reassigned, so reading it as a shared `&'static` is
    // sound. A null pointer here would mean the host PHP is not SPL-enabled,
    // which is unsupported.
    unsafe { spl_ce_RuntimeException.as_ref() }
        .expect("SPL \\RuntimeException is required (host PHP missing the SPL extension?)")
}
