# Transcription

## The shape of a result

`Model::transcribe()` returns a `Transcription`:

```php
$result = $model->transcribe('episode.wav');

$result->text();       // string — the full transcript
$result->segments();   // list<array{start: float, end: float, text: string}>
$result->count();      // int — number of segments
$result->duration();   // float — end of the last segment, in seconds
```

Segment offsets are **seconds** (floats). The concatenated segment
texts equal `text()` modulo whitespace, so you can store segments and
derive the transcript, or vice versa.

The segment row shape is deliberately identical to
[`Displace\AI\Contracts\Transcriber`](https://github.com/DisplaceTech/ai-contracts)'s
documented return — a contracts adapter is two lines:

```php
public function transcribe(string $audioPath, array $options = []): array
{
    $t = $this->model->transcribe($audioPath, $options);
    return ['text' => $t->text(), 'segments' => $t->segments()];
}
```

## Options

```php
$model->transcribe('interview.wav', [
    'language'  => 'de',    // ISO 639-1 hint; omit for auto-detect
    'translate' => true,    // translate to English (multilingual models)
    'threads'   => 4,       // pin the decoder thread count
]);
```

Unknown keys are ignored (forward compatibility); present-but-wrong
types throw with the key named.

## Deployment notes

- A loaded model is resident memory (75MB–500MB depending on size).
  The FPM guidance from ext-infer applies verbatim: load in CLI tools,
  queue workers, and daemons — not per-FPM-worker.
- `transcribe()` is synchronous and CPU-bound; budget roughly
  real-time × 0.1–0.5 on modern cores with `tiny`/`base` models.
- One `Model` handle is safe to share across threads: each call builds
  its own whisper state and no mutable state is shared.

## The pipeline this feeds

Transcribe → chunk ([ai-toolkit](https://github.com/DisplaceTech/ai-toolkit)'s
`SentenceChunker` fits transcripts well) → embed
([ext-infer](https://infer.displace.tech)) → index
([ext-turbovec](https://github.com/DisplaceTech/ext-turbovec)):
searchable audio archives, entirely on your hardware.
