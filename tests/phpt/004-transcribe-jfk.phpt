--TEST--
Model::transcribe() returns text + timestamped segments for the JFK sample
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
use Displace\Whisper\Transcription;

$model = Model::load(getenv('WHISPER_TEST_MODEL'));
$result = $model->transcribe(__DIR__ . '/../fixtures/jfk.wav');

echo "is_transcription: ", $result instanceof Transcription ? "yes" : "no", "\n";

// The fixture is the canonical whisper.cpp JFK sample; any usable model
// produces this phrase.
echo "text_matches: ",
    str_contains(strtolower($result->text()), 'ask not what your country can do for you') ? "yes" : "no",
    "\n";

$segments = $result->segments();
echo "has_segments: ", count($segments) >= 1 ? "yes" : "no", "\n";
echo "count_matches: ", count($segments) === $result->count() ? "yes" : "no", "\n";

$previousEnd = 0.0;
$ordered = true;
$joined = '';
foreach ($segments as $segment) {
    if ($segment['start'] < $previousEnd || $segment['end'] < $segment['start']) {
        $ordered = false;
    }
    $previousEnd = $segment['end'];
    $joined .= ' ' . trim($segment['text']);
}
echo "segments_time_ordered: ", $ordered ? "yes" : "no", "\n";

// The audio clip is ~11s; duration() reports the last segment's end.
echo "duration_plausible: ", ($result->duration() > 5.0 && $result->duration() < 15.0) ? "yes" : "no", "\n";

// Concatenated segments equal the transcript modulo whitespace.
$normalize = fn (string $s): string => preg_replace('/\s+/', ' ', trim($s));
echo "segments_join_to_text: ", $normalize($joined) === $normalize($result->text()) ? "yes" : "no", "\n";

$model->close();
?>
--EXPECT--
is_transcription: yes
text_matches: yes
has_segments: yes
count_matches: yes
segments_time_ordered: yes
duration_plausible: yes
segments_join_to_text: yes
