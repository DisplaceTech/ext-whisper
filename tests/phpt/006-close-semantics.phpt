--TEST--
close() is idempotent and transcribe() throws after close
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
use Displace\Whisper\TranscriptionException;

$model = Model::load(getenv('WHISPER_TEST_MODEL'));
$model->close();
$model->close();   // idempotent — must not error
echo "double_close: ok\n";

try {
    $model->transcribe(__DIR__ . '/../fixtures/jfk.wav');
    echo "after_close: FAIL\n";
} catch (TranscriptionException $e) {
    echo "after_close_throws: ", str_contains($e->getMessage(), 'closed') ? "yes" : "no", "\n";
}
?>
--EXPECT--
double_close: ok
after_close_throws: yes
