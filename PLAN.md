# ext-whisper — design notes

Design notes and decision history: why the surface looks the way it
does and the constraints that bind future work. It is not a to-do list.

- Status and surface: [`README.md`](README.md)
- What shipped, per release: [`CHANGELOG.md`](CHANGELOG.md)
- Remaining work: [`future` issues](https://github.com/DisplaceTech/ext-whisper/issues?q=is%3Aissue+is%3Aopen+label%3Afuture), one per idea
- How to cut a release: [`RELEASE.md`](RELEASE.md)

## Surface (as of 0.1.1)

| Surface | Status |
| --- | --- |
| `Model::load` + `use_gpu` option | shipped |
| `Model::transcribe` (language/translate/threads) | shipped |
| `Model::close` (idempotent) | shipped |
| `Transcription` (text/segments/count/duration) | shipped |
| Strict 16kHz mono WAV reader (ffmpeg-hint errors, Rust unit tests) | shipped |
| `WhisperException` hierarchy | shipped |
| PHPT suite (7 tests; transcription gated on `WHISPER_TEST_MODEL`) | shipped |
| CI matrix (8.3/8.4/8.5 × {macos-arm64, ubuntu} + ZTS 8.4/8.5; weekly schedule) | shipped |
| Tag-triggered PIE release workflow + `cargo about` manifest | shipped, exercised through v0.1.0 |
| mdbook docs (whisper.displace.tech) | live |
| `composer.json` (PIE-compatible) | shipped |

## Releases

Details per release live in [`CHANGELOG.md`](CHANGELOG.md).

| Version | Date | Notes |
| --- | --- | --- |
| v0.1.1 | unreleased | ext-php-rs 0.16.3, scheduled + ZTS CI. whisper-rs stays 0.16.0 (whisper.cpp 1.8.3; no newer release). |
| v0.1.0 | 2026-06-11 | First public release: load/transcribe/close, Transcription with seconds-offset segments, strict WAV reader, 9-leg PIE binaries + license manifest on the GitHub release. |

## Remaining work

Tracked as [`future` issues](https://github.com/DisplaceTech/ext-whisper/issues?q=is%3Aissue+is%3Aopen+label%3Afuture), each with its reasoning:

- [#2](https://github.com/DisplaceTech/ext-whisper/issues/2) Audio decoding via `symphonia` (the headline v0.2 candidate)
- [#3](https://github.com/DisplaceTech/ext-whisper/issues/3) Word-level timestamps
- [#4](https://github.com/DisplaceTech/ext-whisper/issues/4) `Transcription::language()`
- [#5](https://github.com/DisplaceTech/ext-whisper/issues/5) Progress callback for long files
- [#6](https://github.com/DisplaceTech/ext-whisper/issues/6) Opt-in Metal / GPU feature flags
- [#7](https://github.com/DisplaceTech/ext-whisper/issues/7) whisper.cpp 1.9.x, once a whisper-rs release bundles it

The heads-up to the whisper-rs maintainers (a bindings courtesy link,
per the umbrella roadmap's upstream-relations policy) is Eric's call
and happens in his name.

## Design notes

- **Strict WAV input is a feature.** Internal resampling hides quality
  decisions; the ffmpeg hint in every error makes the conversion the
  caller's explicit, controllable step. See `src/wav.rs` module docs.
- **Per-call whisper state** — `Model` owns the context (weights);
  every `transcribe()` builds and drops its own state. Same
  no-shared-mutable-state shape as ext-infer's per-call contexts.
- **Segment rows match the ai-contracts `Transcriber` shape** so the
  framework adapter is a two-liner. The strategic consumer is the
  audio-archive pipeline: whisper → infer(embed) → turbovec.
