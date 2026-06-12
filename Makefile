# Convenience wrappers around cargo + cargo-php so PHP contributors don't
# need to memorize cargo invocations. Each target is a thin pass-through.

CARGO       ?= cargo
PHP         ?= php
PHP_CONFIG  ?= php-config
FEATURES    ?=

CARGO_FLAGS := $(if $(FEATURES),--features $(FEATURES),)

.PHONY: help build release test clippy fmt fmt-check stubs install uninstall clean check-cargo-php

help:
	@echo "ext-whisper — common tasks"
	@echo ""
	@echo "  make build       Debug build (cargo build)"
	@echo "  make release     Optimized build (cargo build --release)"
	@echo "  make test        Run PHPT tests against the just-built extension"
	@echo "  make clippy      cargo clippy -- -D warnings"
	@echo "  make fmt         cargo fmt"
	@echo "  make fmt-check   cargo fmt --check"
	@echo "  make stubs       Regenerate stubs/whisper.stubs.php"
	@echo "  make install     cargo php install (loads the extension into your PHP)"
	@echo "  make uninstall   cargo php remove"
	@echo "  make clean       cargo clean"
	@echo ""
	@echo "Build deps (Linux): cmake build-essential libclang-dev"
	@echo "Variables: FEATURES=...  -> passed through to cargo --features"

build:
	$(CARGO) build $(CARGO_FLAGS)

release:
	$(CARGO) build --release $(CARGO_FLAGS)

clippy:
	$(CARGO) clippy --all-targets $(CARGO_FLAGS) -- -D warnings

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

# Run the PHPT suite against the just-built shared object. `cargo test`
# would only exercise Rust unit tests; for end-to-end coverage we load
# the extension into a real PHP and run the upstream PHP test harness.
# `run-tests.php` isn't bundled with most binary PHP distributions, so when
# it's missing we fetch the copy matching the local PHP minor from php-src.
# Override RUN_TESTS_PHP to point at your own (e.g. from a source build).
RUN_TESTS_PHP ?= run-tests.php

$(RUN_TESTS_PHP):
	@PHP_BRANCH=PHP-$$($(PHP) -r 'echo PHP_MAJOR_VERSION, ".", PHP_MINOR_VERSION;'); \
	echo "fetching run-tests.php from php-src branch $$PHP_BRANCH"; \
	curl -fsSL "https://raw.githubusercontent.com/php/php-src/$$PHP_BRANCH/run-tests.php" -o $(RUN_TESTS_PHP)
	@test -s $(RUN_TESTS_PHP)

# Cargo names the cdylib per host convention: `.dylib` on macOS, `.so`
# everywhere else we support. (`php-config --extension-suffix` is unreliable
# — Homebrew's php-config, for example, omits it — so we detect by `uname`.)
UNAME_S       := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
EXT_SUFFIX    := dylib
else
EXT_SUFFIX    := so
endif
EXT_PATH      := $(CURDIR)/target/debug/libwhisper.$(EXT_SUFFIX)

test: build $(RUN_TESTS_PHP)
	@test -f "$(EXT_PATH)" || { echo "missing $(EXT_PATH) — run 'make build'"; exit 1; }
	$(PHP) -n -d extension=$(EXT_PATH) \
		-r 'if (!extension_loaded("whisper")) { fwrite(STDERR, "whisper not loaded\n"); exit(1); }'
	@# `run-tests.php` requires TEST_PHP_EXECUTABLE to be an *absolute* path
	@# (it `file_exists()`-checks it), and parses TEST_PHP_ARGS by splitting
	@# on single spaces — so the ini override must be `-d extension=path`,
	@# not `-dextension=path`.
	@# `-n` skips the system php.ini: a PIE-installed copy of this very
	@# extension would otherwise double-load and fail every test with a
	@# "module already loaded" warning. PHPTs may only rely on always-in
	@# extensions (core, SPL, json) as a result.
	TEST_PHP_EXECUTABLE=$$(command -v $(PHP)) \
	TEST_PHP_ARGS="-n -d extension=$(EXT_PATH)" \
		$(PHP) $(RUN_TESTS_PHP) -q --show-diff tests/phpt

stubs: check-cargo-php build
	$(CARGO) php stubs --stubs stubs/whisper.stubs.php

install: check-cargo-php
	$(CARGO) php install --release $(CARGO_FLAGS)

uninstall: check-cargo-php
	$(CARGO) php remove

clean:
	$(CARGO) clean

check-cargo-php:
	@command -v cargo-php >/dev/null 2>&1 || { \
		echo "cargo-php not found. Install with: cargo install cargo-php"; \
		exit 1; \
	}
