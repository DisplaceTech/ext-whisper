# Releasing `ext-whisper`

End-to-end guide for cutting a release. Each step is a single command
plus a sentence on why it exists.

## TL;DR

```sh
# 1. Bump versions
edit Cargo.toml       # [package].version = "0.1.0"
edit composer.json    # (no version key; PIE reads the git tag)
cargo update --workspace

# 2. Verify locally
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
WHISPER_TEST_MODEL=$PWD/models/ggml-tiny.en.bin make test
composer validate composer.json

# 3. Land the bump
git commit -am "chore(release): v0.1.0"
git push

# 4. Tag and push the tag — CI builds + uploads artifacts
git tag v0.1.0
git push --tags

# 5. Visit the draft release on GitHub, edit notes, hit Publish.
```

The rest of this document expands on each step.

## Versioning

We follow [SemVer](https://semver.org/) with one nuance: pre-1.0,
breaking changes happen between minors (0.1.x → 0.2.x), not patches.
Patches are bug-fixes only.

Two files carry the version explicitly:

- `Cargo.toml` (`[package].version`). The cargo build script consumes
  this for `CARGO_PKG_VERSION`; we don't surface it to PHP today, but
  may in a future `Displace\Infer\VERSION` constant.
- The `git` tag (`v{semver}`). PIE reads its version from the tag, not
  from `composer.json`. Composer's docs are explicit:
  > "PIE follows the usual PHP extension build and install process";
  > tags are how Packagist (and therefore PIE) learn about releases.

`composer.json` does **not** carry a version key — that would conflict
with the tag-derived version Composer whispers. The `branch-alias` under
`extra` exists only so `dev-main` resolves to `0.x.x-dev` for users
pinning a dev branch.

## Pre-flight checklist

Before tagging, run:

```sh
# Rust formatting + lint
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings

# Build the release-mode artifact at least once locally
make release

# Full PHPT suite against a real model
WHISPER_TEST_MODEL=$PWD/models/ggml-tiny.en.bin make test

# composer.json shape
composer validate composer.json

# Optional: regenerate IDE stubs and diff against the committed copy
make stubs && git diff stubs/whisper.stubs.php
```

If any of those fail, fix before tagging. The release workflow runs the
same checks, but catching them locally avoids a failed draft Release
sitting around in the project's listing.

## Cutting the tag

```sh
git tag v0.1.0
git push --tags
```

That's the *only* user-facing action that triggers a release. The tag
must:

- be a regular tag (no signature requirement today; we'll add
  `--sign` once we have a maintainer GPG key story)
- match the glob `v*` (the release workflow's trigger)
- correspond to a clean tree (the version bump + lint should already
  be on `main`)

## What the release workflow does

`.github/workflows/release.yml` fires on the `v*` tag push and runs
nine parallel jobs — three PHP minors (8.3, 8.4, 8.5) × three
platforms (macos-arm64, linux-x86_64, linux-arm64):

| Job                                       | Runner             |
| ----------------------------------------- | ------------------ |
| build php8.3-arm64-darwin                 | macos-14           |
| build php8.4-arm64-darwin                 | macos-14           |
| build php8.5-arm64-darwin                 | macos-14           |
| build php8.3-x86_64-linux-glibc           | ubuntu-latest      |
| build php8.4-x86_64-linux-glibc           | ubuntu-latest      |
| build php8.5-x86_64-linux-glibc           | ubuntu-latest      |
| build php8.3-arm64-linux-glibc            | ubuntu-24.04-arm   |
| build php8.4-arm64-linux-glibc            | ubuntu-24.04-arm   |
| build php8.5-arm64-linux-glibc            | ubuntu-24.04-arm   |

Each job:

1. Installs system deps (`cmake`, build-essential, clang).
2. Installs the matrix PHP via `shivammathur/setup-php@v2`.
3. Runs `cargo build --release`.
4. Stages `whisper.so` / `whisper.dylib` in the right shape.
5. Tarballs it as
   `php_whisper-{version}_php{minor}-{arch}-{os}[-{libc}].tar.gz`
   per [PIE's filename convention](https://github.com/php/pie/blob/1.5.x/docs/extension-maintainers.md).
6. Computes a `.sha256` sidecar.
7. Uploads both to the GitHub Release (created as **draft**).

The first matrix leg creates the draft Release; later legs add files
to the same one.

## Publishing the draft

After CI is green:

1. Visit https://github.com/DisplaceTech/ext-whisper/releases.
2. Find the draft for the tag, click *Edit*.
3. Write the release notes. Suggested skeleton:
   ```
   ## Highlights
   - <one-line summary of headline feature / breaking change>

   ## Added
   - …

   ## Changed
   - …

   ## Fixed
   - …

   ## Known caveats
   - <e.g. "ZTS support compiles but is not stress-tested">
   ```
4. Verify all 9 tarballs + 9 sidecars (18 files total) are attached.
5. Hit *Publish release*.

Publishing is the action that exposes the release to GitHub's public
Releases API. Until you publish, drafts are visible only to repo
maintainers — PIE, Packagist, and `gh release view` from a non-owner
account all see nothing.

## One-time Packagist registration

PIE installs via Composer, which resolves packages through Packagist
by default. The first time you ship `ext-whisper`, register the package:

1. Go to <https://packagist.org/login/> and sign in with GitHub.
2. Click **Submit** in the top nav.
3. Paste `https://github.com/DisplaceTech/ext-whisper` into the repo
   URL field and submit.
4. Packagist reads `composer.json`, validates the `type: php-ext`
   block, and registers the package as `displace/ext-whisper`.

That step is one-time. After it, every tag pushed to the repo needs
to make it back to Packagist. Two ways:

- **Recommended — connect your GitHub account once.** On Packagist's
  profile page, link your GitHub account. Packagist auto-installs a
  webhook on every repo you own, so every future tag triggers a
  metadata refresh within seconds. Set-and-forget.
- **Per-repo webhook.** If you don't want the account-wide hook:
  GitHub → ext-whisper settings → Webhooks → Add webhook. URL is
  `https://packagist.org/api/github?username=<your-handle>&apiToken=<token>`
  (token from Packagist's profile page). Push events only.

Without one of those, you have to manually click *Update* on the
Packagist package page after every release, which someone will
forget to do.

### Tags are immutable on Packagist

Once Packagist has indexed a stable version (`vX.Y.Z` with no
`-rc.N` / `-beta.N` suffix), the source/dist reference for that
version is **locked**. Re-tagging the same name at a different
commit gets rejected:

> The displace/ext-whisper package of which you are a maintainer had
> an attempted update to version vX.Y.Z blocked, because a published
> stable version's source/dist reference changed in your git
> repository.

If you need to ship a fix for a broken release, **always bump to the
next patch version** (e.g. `v0.1.0` → `v0.1.1`). Even if no one has
installed the broken tag yet, re-tagging breaks Packagist's
immutability guarantee — for everyone, not just you.

Prerelease tags (`v0.1.0-rc.1`, `v0.1.0-beta.2`) are *not* immutable
on Packagist, so the RC dance in RELEASE.md's
[verify section](#what-the-release-workflow-does) is safe to redo.
Stable tags are not.

### Verifying the Packagist hook is working

After a release publishes, the Packagist package page should show
the new version within a minute or two. If it doesn't:

```sh
# Manual nudge from the maintainer's machine:
curl -XPOST -H 'content-type:application/json' \
  "https://packagist.org/api/update-package?username=<you>&apiToken=<token>" \
  -d '{"repository":{"url":"https://github.com/DisplaceTech/ext-whisper"}}'
```

If that updates Packagist but the auto-hook didn't fire, check the
webhook delivery log under GitHub repo settings → Webhooks.

## PIE-side install (smoke test post-release)

```sh
# Install PIE if you don't have it
curl -L --output pie.phar https://github.com/php/pie/releases/latest/download/pie.phar
chmod +x pie.phar && sudo mv pie.phar /usr/local/bin/pie

# Install your freshly released extension
pie install displace/ext-whisper

# Confirm
php -m | grep whisper
```

PIE will fetch the correct
`php_whisper-{version}_php{minor}-{arch}-{os}-{libc}.tar.gz` for the
caller's environment, extract `whisper.{so,dylib}`, and drop it into the
PHP extension directory.

## Hotfix / patch releases

For a bug-fix release (e.g. `0.1.0` → `0.1.1`):

1. Branch from the tag: `git checkout -b hotfix/0.1.1 v0.1.0`
2. Apply the fix (single focused commit).
3. Bump `Cargo.toml` to `0.1.1`.
4. PR into `main`, merge, then tag from `main`.

Don't tag directly from the hotfix branch — `main` should always be
the source of truth for tags so `git log main` reflects shipped
history.

## Yanking a release

If a release is broken:

1. Mark the GitHub Release as a "pre-release" (lowest-effort signal)
   or delete it.
2. Open an issue documenting the problem.
3. Cut a fixed release with the next patch version. PIE always
   resolves to the latest non-yanked version.

We don't have a Packagist "abandon" workflow yet because we haven't
published to Packagist — the project lives entirely on GitHub Releases
for now.

## Caveats / known gaps

- **ZTS PHP** is enabled in `composer.json` (`support-zts: true`) and
  the code is thread-safe by design (the model context is shared
  read-only; every `transcribe()` builds and drops its own whisper
  state). It is *not* exercised in CI yet — neither
  the regular CI matrix nor the release workflow builds against a
  ZTS-PHP runner. Treat ZTS as "should work, please report bugs".
- **Windows** is intentionally excluded. The `os-families-exclude`
  block in `composer.json` makes PIE skip Windows hosts cleanly.
- **musl Linux** is not in the release matrix. The build script's
  `.cargo/config.toml` carries the right `crt-static` opt-out so
  someone building from source on Alpine should succeed, but we don't
  ship binaries.
- **Apple Metal** is opt-in via the `metal` cargo feature
  (`make release FEATURES=metal`). The default release tarball is
  CPU-only, even on macos-14, because the Metal-enabled build embeds
  Apple Silicon GPU code that we haven't validated against the macos
  GitHub runner's hardware mix yet.

## When something goes wrong

| Symptom                                              | First thing to check |
| ---------------------------------------------------- | -------------------- |
| Release workflow doesn't fire                        | Did you push the tag? `git push --tags`. |
| One matrix leg fails to compile whisper.cpp            | Check the runner's cmake version — bump system-deps install if needed. |
| PIE can't find a matching binary                     | Verify the tarball filename — PIE matches verbatim on arch/os/libc. |
| `php -m` doesn't show `whisper` after `pie install`    | Re-run PIE with `-v` to see where it dropped the artifact and which `php.ini` it added to. |
