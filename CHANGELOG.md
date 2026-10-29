# Changelog

All notable changes to ext-whisper are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/)
(pre-1.0: breaking changes bump the minor). Dates are tag dates.

## [0.1.1] - 2026-10-29

Maintenance release. Transcription output is unchanged.

### Changed

- `ext-php-rs` `=0.15.13` → `=0.16.3`. A wrong-typed argument now throws
  `\TypeError` (was a plain `\Exception`); the trailing
  `?array $options` parameters report their `null` default through
  Reflection, matching the stubs. Signatures are otherwise identical.
- `whisper-rs` stays at `=0.16.0` (the latest release; it bundles
  whisper.cpp 1.8.3). No newer whisper.cpp is reachable through a
  released whisper-rs.
- CI runs weekly as well as on push, tests PHP 8.5 on macOS arm64 again,
  adds ZTS PHP 8.4/8.5 legs on Linux, and pins bindgen's libclang on
  the ubuntu-26.04 runners. Dependabot keeps the non-upstream crates
  current.

## [0.1.0] - 2026-06-11

First release.

### Added

- `Model::load()` / `transcribe()` / `close()` over whisper.cpp models;
  `language`, `translate` and `threads` options; `use_gpu` at load for
  custom GPU builds.
- `Transcription`: text plus time-aligned segments (offsets in seconds)
  in the exact ai-contracts `Transcriber` row shape.
- Strict 16kHz mono 16-bit PCM WAV input; `AudioException` messages
  carry the ffmpeg one-liner that fixes a non-conforming file.
- Per-call whisper state over a shared read-only model context.
- whisper.cpp logging silenced by default (`EXT_WHISPER_LOG=1` restores
  it); typed exceptions rooted at `WhisperException`.
- 7 Rust unit tests, 7 PHPTs with the jfk.wav fixture, 9-leg PIE
  binaries and a `cargo about` license manifest; docs at
  whisper.displace.tech.

[0.1.1]: https://github.com/DisplaceTech/ext-whisper/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DisplaceTech/ext-whisper/releases/tag/v0.1.0
