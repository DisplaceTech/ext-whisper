//! Build script for `ext-whisper`.
//!
//! Single job today: emit the `cdylib` link flags needed to defer Zend
//! symbols (`zend_throw_exception`, `zval_ptr_dtor`, `spl_ce_RuntimeException`,
//! ...) to runtime resolution against the host PHP binary.
//!
//! `.cargo/config.toml` carries the same flags, but its `rustflags` array
//! is silently replaced when a `RUSTFLAGS` environment variable is set
//! (cargo precedence: env var > config). Our CI sets `RUSTFLAGS=-D warnings`
//! globally, which would clobber the link flag and break the macOS build
//! at link time with "Undefined symbols: _spl_ce_RuntimeException, ...".
//!
//! `cargo:rustc-cdylib-link-arg=` is a *separate* mechanism that always
//! applies — it doesn't go through `rustflags` — so emitting from build.rs
//! is robust regardless of how the consumer's environment is configured.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Apple ld64 / new ld-prime: defer undefined symbols. The "deprecated"
    // warning on newer SDKs is cosmetic; the flag is still honored and
    // there is no documented replacement for cdylibs that load into a
    // host process providing the symbols.
    //
    // GNU ld (Linux glibc): undefined symbols in a cdylib are deferred
    // already, so the flag is harmless. We emit it unconditionally on
    // non-Windows targets to keep the build behavior identical across
    // platforms; the loader resolves against the host PHP either way.
    if target_os != "windows" {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-undefined,dynamic_lookup");
    }

    // musl cdylibs cannot statically link the C runtime — they need to
    // resolve libc against the host (PHP, the FPM worker, ...) at load.
    if target_env == "musl" {
        println!("cargo:rustc-link-arg=-Wl,-Bdynamic");
    }
}
