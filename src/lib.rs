//! `ext-whisper` — PHP 8.3+ native, in-process speech-to-text via
//! whisper.cpp.
//!
//! Public surface:
//!
//! - `Displace\Whisper\Model`                  — load + transcribe
//! - `Displace\Whisper\Transcription`          — text + timestamped segments
//! - `Displace\Whisper\WhisperException`       — base exception (extends `\RuntimeException`)
//! - `Displace\Whisper\ModelLoadException`     — load-time failure
//! - `Displace\Whisper\AudioException`         — bad/unsupported audio input
//! - `Displace\Whisper\TranscriptionException` — runtime failure
//!
//! See the per-module docs for design notes.
//
// `transcribe()` uses a camelCase parameter ident (`wavPath`) on purpose:
// PHP named-arguments echo the Rust ident verbatim and the public API is
// camelCase per PSR-12. The proc-macro expansion shifts the ident into
// generated code where a per-method `#[allow]` doesn't reach.
#![allow(non_snake_case)]
#![deny(clippy::all)]

mod error;
mod model;
mod transcription;
mod wav;

use ext_php_rs::prelude::*;

// Re-export so `cargo php stubs` and module registration can see them by
// their crate-root paths.
pub use error::{AudioException, ModelLoadException, TranscriptionException, WhisperException};
pub use model::Model;
pub use transcription::Transcription;

/// PHP module entry point.
///
/// The default module name is `CARGO_PKG_NAME` (`ext-whisper`); we override
/// it to plain `whisper` so userland calls `extension_loaded('whisper')` —
/// matching PHP's convention of dropping the `ext-` prefix.
///
/// The order of `class::<T>()` calls is significant: child exceptions
/// reference their parent's `ClassEntry`, so the parent must be registered
/// first.
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .name("whisper")
        .class::<WhisperException>()
        .class::<ModelLoadException>()
        .class::<AudioException>()
        .class::<TranscriptionException>()
        .class::<Transcription>()
        .class::<Model>()
}
