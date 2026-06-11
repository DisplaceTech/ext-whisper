--TEST--
transcribe() rejects missing and non-conforming WAV input with the ffmpeg hint
--SKIPIF--
<?php
if (!extension_loaded('whisper')) {
    echo 'skip ext-whisper not loaded';
    exit;
}
$path = getenv('WHISPER_TEST_MODEL');
if (!$path || !is_file($path)) {
    echo 'skip WHISPER_TEST_MODEL not set to an existing whisper model';
}
?>
--FILE--
<?php
use Displace\Whisper\AudioException;
use Displace\Whisper\Model;

// Minimal valid WAV writer: 44-byte RIFF header + PCM data.
function write_wav(string $path, int $rate, int $channels, array $samples): void
{
    $data = pack('v*', ...$samples);
    $byteRate = $rate * $channels * 2;
    $header = 'RIFF' . pack('V', 36 + strlen($data)) . 'WAVEfmt '
        . pack('VvvVVvv', 16, 1, $channels, $rate, $byteRate, $channels * 2, 16)
        . 'data' . pack('V', strlen($data));
    file_put_contents($path, $header . $data);
}

$model = Model::load(getenv('WHISPER_TEST_MODEL'));
$tmp = tempnam(sys_get_temp_dir(), 'whisper-test-') . '.wav';

// Missing file.
try {
    $model->transcribe('/no/such/audio.wav');
    echo "missing: FAIL\n";
} catch (AudioException $e) {
    echo "missing_throws: yes\n";
}

// Wrong sample rate — error names the rate and carries the ffmpeg hint.
write_wav($tmp, 44100, 1, array_fill(0, 1000, 0));
try {
    $model->transcribe($tmp);
    echo "rate: FAIL\n";
} catch (AudioException $e) {
    echo "rate_named: ", str_contains($e->getMessage(), '44100Hz') ? "yes" : "no", "\n";
    echo "ffmpeg_hint: ", str_contains($e->getMessage(), 'ffmpeg -i') ? "yes" : "no", "\n";
}

// Stereo.
write_wav($tmp, 16000, 2, array_fill(0, 1000, 0));
try {
    $model->transcribe($tmp);
    echo "stereo: FAIL\n";
} catch (AudioException $e) {
    echo "stereo_named: ", str_contains($e->getMessage(), '2 channels') ? "yes" : "no", "\n";
}

// Not a WAV at all.
file_put_contents($tmp, 'these are not the bytes you are looking for');
try {
    $model->transcribe($tmp);
    echo "garbage: FAIL\n";
} catch (AudioException $e) {
    echo "garbage_throws: yes\n";
}

unlink($tmp);
$model->close();
?>
--EXPECT--
missing_throws: yes
rate_named: yes
ffmpeg_hint: yes
stereo_named: yes
garbage_throws: yes
