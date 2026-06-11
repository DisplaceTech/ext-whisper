# ext-whisper — plan

Living document. Status / surface description lives in
[`README.md`](README.md); how-to-cut-a-release lives in
[`RELEASE.md`](RELEASE.md).

## Status snapshot

| Surface | Status |
| --- | --- |
| `Model::load` + `use_gpu` option | built |
| `Model::transcribe` (language/translate/threads) | built |
| `Model::close` (idempotent) | built |
| `Transcription` (text/segments/count/duration) | built |
| Strict 16kHz mono WAV reader (ffmpeg-hint errors, Rust unit tests) | built |
| `WhisperException` hierarchy | built |
| PHPT suite (7 tests; transcription gated on `WHISPER_TEST_MODEL`) | built |
| CI matrix (8.3/8.4/8.5 × {macos-arm64, ubuntu, ubuntu-arm64}) | built |
| Tag-triggered PIE release workflow + `cargo about` manifest | shipped, exercised through v0.1.0 |
| mdbook docs (whisper.displace.tech) | live |
| `composer.json` (PIE-compatible) | built |

## Releases

| Version | Date | Notes |
| --- | --- | --- |
| v0.1.0 | 2026-06-11 | First public release: load/transcribe/close, Transcription with seconds-offset segments, strict WAV reader, 9-leg PIE binaries + license manifest on the GitHub release. |

## Up next

- [x] DNS, Packagist, v0.1.0 tag + published release — done 2026-06-11.
- [ ] Heads-up issue to whisper-rs upstream (bindings courtesy link),
      per the umbrella roadmap's upstream-relations policy — awaiting
      Eric's go-ahead (outreach in his name).

## Later (v0.2 candidates, in rough order)

- **Audio decoding via `symphonia`** — accept mp3/m4a/ogg/flac and
  resample internally. The big ergonomic unlock; also the big
  dependency cliff, which is why v0.1 ships without it.
- **Word-level timestamps** — whisper.cpp's token-level data exists;
  needs an API shape that doesn't bloat `segments()`.
- **`language()` on Transcription** — surface the detected language id.
- **Progress callback** — long files currently transcribe silently.
- **Metal / GPU feature flags** — mirror ext-infer's opt-in pattern.

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
