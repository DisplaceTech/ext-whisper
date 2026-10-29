<h1 align="center">ext-whisper</h1>

<p align="center">
  <strong>Local speech-to-text for PHP, in-process.</strong><br>
  16kHz WAV in, text + timestamped segments out — no Python sidecar, no remote API, no audio leaving the box.
</p>

<p align="center">
  <a href="https://github.com/DisplaceTech/ext-whisper/actions/workflows/ci.yml"><img src="https://github.com/DisplaceTech/ext-whisper/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <img src="https://img.shields.io/badge/PHP-8.3%20%7C%208.4%20%7C%208.5-777BB4?logo=php&logoColor=white" alt="PHP 8.3 / 8.4 / 8.5" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License" /></a>
  <a href="https://whisper.displace.tech"><img src="https://img.shields.io/badge/docs-whisper.displace.tech-blue" alt="Documentation" /></a>
</p>

---

## Status

Feature-complete for its scope. Maintained: dependency updates and
new-model support ship as patch/minor releases — see the
[CHANGELOG](CHANGELOG.md). Last verified: PHP 8.3/8.4/8.5 (NTS on macOS arm64 + Linux x86_64; ZTS 8.4/8.5 on Linux), `whisper-rs` 0.16.0 (whisper.cpp 1.8.3), `ggml-tiny.en`, 2026-10-29.

Remaining ideas are tracked as [`future` issues](https://github.com/DisplaceTech/ext-whisper/issues?q=is%3Aissue+is%3Aopen+label%3Afuture).
**Co-maintainers welcome** — if you run this in production and want a
say in where it goes, open an issue or a PR.

## What is ext-whisper?

`ext-whisper` is a PHP 8.3+ extension that loads a
[whisper.cpp](https://github.com/ggml-org/whisper.cpp) model and runs
speech-to-text *in the PHP process*, on CPU. Written in Rust on top of
[`ext-php-rs`](https://github.com/davidcole1340/ext-php-rs) and
[`whisper-rs`](https://github.com/tazz4843/whisper-rs).

- 🎙️ **Transcription with timestamps** — full text plus time-aligned segments, offsets in seconds.
- 🧾 **Contracts-shaped output** — segment rows match [`Displace\AI\Contracts\Transcriber`](https://github.com/DisplaceTech/ai-contracts) exactly; the adapter is two lines.
- 🧰 **Actionable errors** — a non-conforming WAV throws with the precise ffmpeg one-liner that fixes it.
- 🌍 **Multilingual + translate** — `['language' => 'de']` hints, `['translate' => true]` to English (multilingual models).
- 🧵 **Thread-safe by construction** — one model handle, a fresh whisper state per call, no shared mutable state.
- 🤫 **Quiet by default** — whisper.cpp's stderr firehose is silenced; `EXT_WHISPER_LOG=1` restores it.

## Quick start

```sh
mkdir -p models
curl -L -o models/ggml-tiny.en.bin \
    https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin

make build
php -d extension=$PWD/target/debug/libwhisper.so examples/transcribe.php \
    models/ggml-tiny.en.bin tests/fixtures/jfk.wav
```

```php
<?php
use Displace\Whisper\Model;

$model  = Model::load('models/ggml-tiny.en.bin');
$result = $model->transcribe('audio/meeting.wav');

echo $result->text(), PHP_EOL;

foreach ($result->segments() as $s) {
    printf("[%6.2fs → %6.2fs] %s\n", $s['start'], $s['end'], $s['text']);
}

$model->close();
```

Input must be **16kHz mono 16-bit PCM WAV**; everything else converts
in one line (`ffmpeg -i in.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav`)
and the error messages carry that exact command.

## Documentation

[**whisper.displace.tech**](https://whisper.displace.tech) — install,
audio preparation, the full API surface. Built from
[`docs/`](docs/) with mdbook, deployed on every push to `main`.

## Part of a stack

Transcribe → chunk → embed → search → answer, entirely on your hardware:

| Stage | Package | What it does |
| ----- | ------- | ------------ |
| Transcribe | **ext-whisper** (this repo) | Speech-to-text over whisper.cpp |
| Chunk | [ai-toolkit](https://github.com/DisplaceTech/ai-toolkit) | Text chunkers and packed-vector math, pure PHP |
| Embed · rerank · answer | [ext-infer](https://github.com/DisplaceTech/ext-infer) | Local LLM chat, embeddings and reranking over llama.cpp |
| Search | [ext-turbovec](https://github.com/DisplaceTech/ext-turbovec) | Compressed in-memory vector search (TurboQuant) |
| Integrate | [ai-contracts](https://github.com/DisplaceTech/ai-contracts) | The interfaces frameworks code against |
| Reference app | [rag-over-markdown](https://github.com/DisplaceTech/rag-over-markdown) | The whole pipeline over a folder of markdown |

Every package links the others; the extensions share one scaffold,
[ext-template](https://github.com/DisplaceTech/ext-template).

For ext-whisper specifically: transcribe → chunk → embed → search is a
searchable audio archive, entirely on your hardware, and the
[ai-contracts](https://github.com/DisplaceTech/ai-contracts)
`Transcriber` interface is the integration surface.

## Compatibility

|                | macOS arm64 | Linux x86_64 | Linux arm64 | Windows |
| -------------- | :---------: | :----------: | :---------: | :-----: |
| **PHP 8.3**    |      ✅     |      ✅      |      ✅     |    —    |
| **PHP 8.4**    |      ✅     |      ✅      |      ✅     |    —    |
| **PHP 8.5**    |      ✅     |      ✅      |      ✅     |    —    |

ZTS is enabled in `composer.json` and CI runs the PHPT suite on ZTS PHP
8.4/8.5 (Linux) weekly; release binaries are NTS-only.

## Roadmap

**Shipped** &nbsp; `Model::load()` / `transcribe()` / `close()` ·
`Transcription` with seconds-offset segments in the ai-contracts shape ·
strict WAV reader with ffmpeg-hint errors · typed exceptions · Rust unit
tests + PHPT suite · CI matrix · PIE binaries + license manifest.

**Remaining ideas** (audio decoding via symphonia, word-level
timestamps, detected language, progress callback, …) live as
[`future` issues](https://github.com/DisplaceTech/ext-whisper/issues?q=is%3Aissue+is%3Aopen+label%3Afuture), each with its reasoning.
[`PLAN.md`](PLAN.md) holds the design notes.

## Deliberately out of scope (v0.1)

**Audio decoding** (mp3/m4a/ogg — the ffmpeg one-liner is the API;
`symphonia`-based decoding is a v0.2 candidate) · **streaming /
realtime transcription** · **speaker diarization** · **word-level
timestamps** · **GPU-default builds** (CPU-first platform-wide;
`use_gpu` exists for custom builds) · **Windows**.

## Writing about this

- [ext-whisper: PHP-Native Audio Transcription](https://eric.mann.blog/ext-whisper-php-native-audio-transcription/)
- [From Voice to Vectors: Building a Searchable Audio Archive in PHP](https://eric.mann.blog/voice-to-vectors-audio-search-php/)

The stack's launch series ran Sept 22 – Oct 1, 2026; everything else
is at [eric.mann.blog/writing](https://eric.mann.blog/writing/).

## License

[MIT](LICENSE) &copy; 2026 Eric Mann / Displace Technologies.
Statically links [whisper.cpp](https://github.com/ggml-org/whisper.cpp)
(MIT, © The ggml authors) — see
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
