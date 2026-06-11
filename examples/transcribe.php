<?php

declare(strict_types=1);

/**
 * Transcribe a WAV file with timestamped segments.
 *
 * Usage:
 *     php -d extension=target/debug/libwhisper.so examples/transcribe.php \
 *         models/ggml-tiny.en.bin tests/fixtures/jfk.wav
 *
 * Models: https://huggingface.co/ggerganov/whisper.cpp — `tiny.en`
 * (~75MB) is plenty for clean English speech; `base`/`small` trade
 * speed for accuracy and multilingual support.
 *
 * Audio must be 16kHz mono 16-bit PCM WAV. Anything else, convert first:
 *     ffmpeg -i input.mp3 -ar 16000 -ac 1 -c:a pcm_s16le out.wav
 */

use Displace\Whisper\Model;

[$_, $modelPath, $wavPath] = $argv + [null, 'models/ggml-tiny.en.bin', 'tests/fixtures/jfk.wav'];

$model = Model::load($modelPath);

$started = microtime(true);
$result = $model->transcribe($wavPath);
$elapsed = microtime(true) - $started;

echo $result->text(), "\n\n";

foreach ($result->segments() as $segment) {
    printf("[%7.2fs → %7.2fs]  %s\n", $segment['start'], $segment['end'], trim($segment['text']));
}

printf(
    "\n%d segment(s), %.1fs of audio transcribed in %.2fs\n",
    $result->count(),
    $result->duration(),
    $elapsed,
);

$model->close();
