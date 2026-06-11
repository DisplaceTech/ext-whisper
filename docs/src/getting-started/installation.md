# Installation

## Via PIE (recommended)

Once tagged releases are published, prebuilt binaries install with
[PIE](https://github.com/php/pie) on macOS arm64, Linux x86_64, and
Linux arm64, for PHP 8.3 / 8.4 / 8.5:

```sh
php pie.phar install displace/ext-whisper
```

## From source

Requirements: Rust (the version pinned in `rust-toolchain.toml`
installs automatically), `cmake`, and a C/C++ toolchain.

```sh
git clone https://github.com/DisplaceTech/ext-whisper
cd ext-whisper
make build                 # target/debug/libwhisper.{so,dylib}
php -d extension=$PWD/target/debug/libwhisper.so \
    -r 'var_dump(extension_loaded("whisper"));'
```

## Get a model

Models come from the whisper.cpp zoo on Hugging Face:

```sh
mkdir -p models
curl -L -o models/ggml-tiny.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin
```

| Model | Size | Notes |
|---|---|---|
| `tiny.en` | 75MB | English-only; fast; fine for clean speech |
| `base.en` | 142MB | Noticeably better punctuation/names |
| `small` | 466MB | Multilingual; the quality sweet spot on CPU |

## Verify

```sh
php -d extension=$PWD/target/debug/libwhisper.so examples/transcribe.php \
    models/ggml-tiny.en.bin tests/fixtures/jfk.wav
```

Expected output starts with *"And so my fellow Americans…"*.
