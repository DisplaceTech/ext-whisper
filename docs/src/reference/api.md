# API surface

The complete public PHP API. For an authoritative copy in PHP-stub
form (consumed by IDEs and static analyzers), see
[`stubs/whisper.stubs.php`](https://github.com/DisplaceTech/ext-whisper/blob/main/stubs/whisper.stubs.php).

## `Displace\Whisper\Model`

```php
final class Model
{
    public static function load(
        string $path,
        array  $options = [],   // use_gpu (bool, default false)
    ): self;

    public function transcribe(
        string $wavPath,
        array  $options = [],   // language (string), translate (bool), threads (int)
    ): \Displace\Whisper\Transcription;

    public function close(): void;   // idempotent
}
```

## `Displace\Whisper\Transcription`

```php
final class Transcription
{
    public function text(): string;

    /** @return list<array{start: float, end: float, text: string}> */
    public function segments(): array;

    public function count(): int;
    public function duration(): float;   // seconds
}
```

Read-only; constructed only by `Model::transcribe()`. Offsets in
seconds.

## Exception hierarchy

```php
\RuntimeException
└── Displace\Whisper\WhisperException
    ├── Displace\Whisper\ModelLoadException      // load() failures
    ├── Displace\Whisper\AudioException          // bad/unsupported WAV input
    └── Displace\Whisper\TranscriptionException  // whisper.cpp failures, use-after-close
```

## Environment variables

| Variable | Effect |
|---|---|
| `EXT_WHISPER_LOG=1` | Restore whisper.cpp's verbose stderr logging (silenced by default). |

## Conventions

- Direct construction is refused on `Model` and `Transcription` —
  each throws `WhisperException` pointing at the right factory.
- Unknown option keys are ignored; present-but-wrong-typed keys throw
  with the key named.
- Audio-format errors always embed the ffmpeg one-liner that produces
  a conforming file.
