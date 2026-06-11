--TEST--
Model::load() throws ModelLoadException for a missing file
--SKIPIF--
<?php
if (!extension_loaded('whisper')) {
    echo 'skip ext-whisper not loaded';
}
?>
--FILE--
<?php
use Displace\Whisper\Model;
use Displace\Whisper\ModelLoadException;

try {
    Model::load('/no/such/model.bin');
    echo "FAIL\n";
} catch (ModelLoadException $e) {
    echo "throws: yes\n";
    echo "names_path: ", str_contains($e->getMessage(), '/no/such/model.bin') ? "yes" : "no", "\n";
}
?>
--EXPECT--
throws: yes
names_path: yes
