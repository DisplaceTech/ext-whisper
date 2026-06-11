--TEST--
Exception hierarchy: WhisperException extends RuntimeException; subclasses extend it
--SKIPIF--
<?php
if (!extension_loaded('whisper')) {
    echo 'skip ext-whisper not loaded';
}
?>
--FILE--
<?php
use Displace\Whisper\AudioException;
use Displace\Whisper\ModelLoadException;
use Displace\Whisper\TranscriptionException;
use Displace\Whisper\WhisperException;

echo "base_is_runtime: ", is_subclass_of(WhisperException::class, \RuntimeException::class) ? "yes" : "no", "\n";

foreach ([ModelLoadException::class, AudioException::class, TranscriptionException::class] as $class) {
    echo basename(str_replace('\\', '/', $class)), ": ",
        is_subclass_of($class, WhisperException::class) ? "yes" : "no",
        "\n";
}
?>
--EXPECT--
base_is_runtime: yes
ModelLoadException: yes
AudioException: yes
TranscriptionException: yes
