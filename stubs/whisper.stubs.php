<?php

// Stubs for ext-whisper — IDE / static-analysis only, not loaded at runtime.
//
// Regenerate from the registered classes after building:
//
//     make stubs   # wraps `cargo php stubs --stubs stubs/whisper.stubs.php`

namespace Displace\Whisper;

/**
 * Base exception for all ext-whisper failures. Extends \RuntimeException
 * so existing catch (\RuntimeException $e) clauses continue to work.
 */
class WhisperException extends \RuntimeException
{
}

/**
 * A model file cannot be loaded — missing path, unreadable file, or an
 * unsupported model format.
 */
class ModelLoadException extends \Displace\Whisper\WhisperException
{
}

/**
 * The audio input is unreadable or not 16kHz mono 16-bit PCM WAV. The
 * message always includes the ffmpeg one-liner that produces a
 * conforming file.
 */
class AudioException extends \Displace\Whisper\WhisperException
{
}

/**
 * whisper.cpp failed after the model and audio both loaded — state
 * creation, decode, or use-after-close errors.
 */
class TranscriptionException extends \Displace\Whisper\WhisperException
{
}

/**
 * Result of `Model::transcribe()`.
 *
 * Read-only. Carries the full transcript plus whisper's time-aligned
 * segments; offsets are in seconds. Direct construction is refused — a
 * transcription built by PHP would lie about which model and audio
 * produced it.
 */
final class Transcription
{
    /** @throws \Displace\Whisper\WhisperException Always. */
    public function __construct() {}

    /** The full transcript — whisper's segments joined in order, trimmed. */
    public function text(): string {}

    /**
     * Time-aligned segments, in order. Offsets are seconds; the
     * concatenated segment texts equal `text()` modulo whitespace.
     * Deliberately the row shape of
     * `Displace\AI\Contracts\Transcriber::transcribe()`.
     *
     * @return list<array{start: float, end: float, text: string}>
     */
    public function segments(): array {}

    /** Number of segments. */
    public function count(): int {}

    /**
     * End timestamp of the last segment, in seconds — the transcribed
     * duration. `0.0` when there are no segments.
     */
    public function duration(): float {}
}

class Model
{
    /**
     * Load a whisper model (ggml/GGUF `.bin` from the whisper.cpp model
     * zoo) from disk.
     *
     * @param array<string, mixed> $options Recognised keys:
     *                                      - `use_gpu` (bool, default false) — CPU is the
     *                                        portable default platform-wide.
     *
     * @throws \Displace\Whisper\ModelLoadException If the file cannot be read or parsed.
     */
    public static function load(string $path, array $options = []): self {}

    /**
     * Transcribe a 16kHz mono 16-bit PCM WAV file.
     *
     * Other rates/layouts/containers throw `AudioException` with the
     * exact ffmpeg command that produces a conforming file:
     * `ffmpeg -i input.ext -ar 16000 -ac 1 -c:a pcm_s16le out.wav`.
     *
     * @param array<string, mixed> $options Recognised keys:
     *                                      - `language` (string, default auto-detect) — ISO 639-1
     *                                        hint, e.g. 'en'; English-only models ignore it
     *                                      - `translate` (bool, default false) — translate the
     *                                        result to English (multilingual models only)
     *                                      - `threads` (int, default whisper.cpp's choice)
     *
     * @throws \Displace\Whisper\AudioException         If the WAV is missing or malformed.
     * @throws \Displace\Whisper\TranscriptionException If whisper.cpp fails, or the model
     *                                                  has been closed.
     * @throws \Displace\Whisper\WhisperException       On an invalid option.
     */
    public function transcribe(string $wavPath, array $options = []): \Displace\Whisper\Transcription {}

    /** Release the underlying model weights. Idempotent. */
    public function close(): void {}
}
