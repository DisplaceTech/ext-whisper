# ext-whisper

**Local speech-to-text for PHP, in-process.** `ext-whisper` is a PHP
8.3+ extension that loads a [whisper.cpp](https://github.com/ggml-org/whisper.cpp)
model and transcribes audio *inside the PHP process* — no Python
sidecar, no remote API, no audio leaving the box.

```php
use Displace\Whisper\Model;

$model  = Model::load('models/ggml-tiny.en.bin');
$result = $model->transcribe('meeting.wav');

echo $result->text();

foreach ($result->segments() as $segment) {
    printf("[%6.2fs → %6.2fs] %s\n", $segment['start'], $segment['end'], $segment['text']);
}
```

Written in Rust on top of [`ext-php-rs`](https://github.com/davidcole1340/ext-php-rs)
and the [`whisper-rs`](https://github.com/tazz4843/whisper-rs) bindings.

## Part of a stack

ext-whisper is the ingest stage of the
[Displace local-first AI stack](https://github.com/DisplaceTech):
transcribe with ext-whisper, embed with
[ext-infer](https://infer.displace.tech), index and search with
[ext-turbovec](https://github.com/DisplaceTech/ext-turbovec) — a
complete audio-archive semantic-search pipeline with zero services.

## Deliberately out of scope (v0.1)

- **Audio decoding** — input is 16kHz mono 16-bit PCM WAV, full stop.
  See [Preparing audio](./guide/audio.md) for the one-line ffmpeg
  conversion. Decoding (mp3/m4a/ogg) is a candidate for v0.2.
- **Streaming / realtime transcription** — file in, transcription out.
- **Speaker diarization, word-level timestamps, GPU-default builds** —
  later, if the scope test passes.
- **Windows** — out of scope platform-wide until someone funds it.
