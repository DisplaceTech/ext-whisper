# Preparing audio

ext-whisper accepts exactly one input shape: **16kHz, mono, 16-bit PCM
WAV** — the format whisper.cpp's encoder consumes natively.

Everything else converts in one line:

```sh
ffmpeg -i input.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav
```

That line handles mp3, m4a, ogg, flac, mp4/mkv audio tracks, other WAV
flavors — anything ffmpeg reads. From PHP:

```php
$command = sprintf(
    'ffmpeg -y -i %s -ar 16000 -ac 1 -c:a pcm_s16le %s 2>&1',
    escapeshellarg($input),
    escapeshellarg($wav),
);
exec($command, $output, $exit);
```

## Why so strict?

Two deliberate reasons (this is a v0.1 scope decision, not a
limitation we forgot to fix):

1. **Resampling is a quality decision the caller should own.** A
   silent internal resample hides the filter choice, dithering, and
   channel-mix decisions that affect transcription quality. ffmpeg
   does this better than we would, with flags you control.
2. **Decoding is a dependency cliff.** mp3/m4a/ogg support drags in a
   codec stack several times larger than whisper itself. It's a
   candidate for v0.2 (via `symphonia`), gated on the same scope test
   as everything else.

## Failure messages carry the fix

A non-conforming input never fails with a generic error — the message
names what's wrong *and* the command that fixes it:

```text
AudioException: invalid audio input: podcast.wav: expected a 16000Hz
sample rate, got 44100Hz — convert with: ffmpeg -i input.ext -ar 16000
-ac 1 -c:a pcm_s16le out.wav
```
