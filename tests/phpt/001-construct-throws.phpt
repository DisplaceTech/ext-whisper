--TEST--
new Model() and new Transcription() refuse direct construction
--SKIPIF--
<?php
if (!extension_loaded('whisper')) {
    echo 'skip ext-whisper not loaded';
}
?>
--FILE--
<?php
use Displace\Whisper\Model;
use Displace\Whisper\Transcription;
use Displace\Whisper\WhisperException;

try {
    new Model();
    echo "model_ctor: FAIL\n";
} catch (WhisperException $e) {
    echo "model_ctor_throws: ", str_contains($e->getMessage(), 'Model::load()') ? "yes" : "no", "\n";
}

try {
    new Transcription();
    echo "transcription_ctor: FAIL\n";
} catch (WhisperException $e) {
    echo "transcription_ctor_throws: yes\n";
}
?>
--EXPECT--
model_ctor_throws: yes
transcription_ctor_throws: yes
