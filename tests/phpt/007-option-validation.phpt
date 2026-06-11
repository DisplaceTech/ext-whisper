--TEST--
Malformed options are rejected with the key named; unknown keys are ignored
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
use Displace\Whisper\Model;
use Displace\Whisper\WhisperException;

$wav = __DIR__ . '/../fixtures/jfk.wav';
$model = Model::load(getenv('WHISPER_TEST_MODEL'));

// Wrong type names the key.
try {
    $model->transcribe($wav, ['threads' => 'many']);
    echo "type: FAIL\n";
} catch (WhisperException $e) {
    echo "type_named: ", str_contains($e->getMessage(), 'threads') ? "yes" : "no", "\n";
}

// threads: 0 is rejected with guidance, not silently passed to whisper.
try {
    $model->transcribe($wav, ['threads' => 0]);
    echo "zero: FAIL\n";
} catch (WhisperException $e) {
    echo "zero_named: ", str_contains($e->getMessage(), 'at least 1') ? "yes" : "no", "\n";
}

// Unknown keys are ignored (forward compatibility), valid known keys work.
$result = $model->transcribe($wav, ['threads' => 2, 'made_up_option' => true]);
echo "unknown_ignored: ", str_contains(strtolower($result->text()), 'fellow americans') ? "yes" : "no", "\n";

$model->close();
?>
--EXPECT--
type_named: yes
zero_named: yes
unknown_ignored: yes
