# Changelog

All notable changes to `oboron-cli` are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-06-29

First stable release of `oboron-cli`. It finalizes the 1.0
surface previewed by `1.0.0-rc1` and tracks the published
`oboron` 1.0.1 / `obcrypt` 1.0.0 libraries. The `ob` binary is
authenticated-only with hex-only keys, and `dec` now behaves as
a strict non-oracle.

### Changed

- **Tracks `oboron` 1.0.1.** Built against the stabilized
  protocol libraries (`oboron` 1.0.1, `obcrypt` 1.0.0).
- **`dec` enforces a uniform-error contract.** Every failure —
  bad encoding, wrong length, failed authentication, invalid
  UTF-8, or empty input — reports the same single stderr message
  and exits `1`, so `dec` cannot be used as a decryption oracle
  (CLI spec §8).
- **Usage errors exit `2`.** Conflicting or invalid flags and a
  malformed `--format` are usage errors and exit with status
  `2`, kept distinct from the `dec` failure status.

### Removed

- **Legacy base64 keys.** Keys are canonical hex only — 128 hex
  characters. base64 key input, base64 key output, and the
  automatic base64 → hex profile migration are all gone; there
  is no auto-migration of stored keys.

## [1.0.0-rc1] — 2026-06-16

Migrates `oboron-cli` to the oboron protocol spec 1.0 (rev3),
tracking `oboron` / `obcrypt` 1.0.0-rc1. The crate is now
authenticated-only: it produces a single `ob` binary, with the
scheme namespace cut down to the four authenticated schemes and
keys reduced to hex-only.

### Added

- **`--raw` / `-0` line-framing.** Selects raw (NUL-free /
  newline-terminated) line framing for batch I/O, alongside the
  existing modes.
- **`--version` now prints provenance.** `ob --version` emits
  `ob oboron-tools-rs <version> protocol=1.0 cli=1.0`, surfacing
  the protocol and CLI surface levels this build targets.

### Changed

- **Migrated to oboron protocol spec 1.0 (rev3).** Tracks
  `oboron` / `obcrypt` 1.0.0-rc1.
- **Schemes renamed** to `dsiv` / `psiv` / `dgcmsiv` / `pgcmsiv`
  (formerly `aasv` / `apsv` / `aags` / `apgs`).
- **Keys are hex-only** — 128 hex characters. Base64 keys are no
  longer accepted on input, and `ob key` prints hex.
- **`dec` uses the supplied scheme and never auto-detects.** The
  scheme comes from the `--scheme` flag (or `--format`), falling
  back to the built-in default `dsiv.c32` when none is given.
  oboron output carries no scheme marker, so there is no
  auto-detection.

### Removed

- **The bundled unauthenticated obfuscation binary is removed.**
  oboron-cli now produces only the authenticated `ob` binary; the
  unauthenticated layer ships separately as the `obu` crate.
- **The non-core schemes are removed,** leaving only the four
  authenticated core schemes (`dsiv` / `psiv` / `dgcmsiv` /
  `pgcmsiv`).
- **The tier-aggregate cargo features are removed.**

### Licensing

- Dual-licensed `MIT OR Apache-2.0`.

## [0.5.0] — 2026-05-25

Adds a standalone key generator and fixes `ob key`'s output
format.

### Added

- **`ob keygen`.** Prints a fresh random 128-char hex key to
  stdout and exits — a scripting convenience that creates or
  modifies no profile and needs no config dir. Mirrors the
  existing `obcrypt keygen`.
- **`ob key --base64` (`-B`).** Opt-in legacy base64 output for
  the rare caller that still needs it, emitting a deprecation
  warning on stderr. Conflicts with `--hex`. base64 support is
  still slated for removal before oboron 1.0.

### Changed

- **Key-flag help text** on `enc` / `dec` and `profile create` /
  `profile set` now reads "128 hex chars, or legacy 86-char
  base64" instead of the base64-only wording, matching the
  canonical format.

### Fixed

- **`ob key` defaulted to deprecated base64 output.** Unlike
  `init`, `config show`, and `profile show` — and unlike
  `obcrypt key` — `ob key` re-encoded the canonical hex key back
  to legacy base64 for display, so after a profile was migrated
  to hex it still printed base64 and looked unchanged. It now
  prints canonical 128-char hex by default; `-x`/`--hex` is
  retained as an accepted no-op.
- **`ob key` did not migrate a legacy profile on display.**
  Despite the 0.4.0 note that profile-key migration fires on
  `key`, the command read the key without rewriting it. It now
  rewrites a legacy base64 profile to canonical hex in place
  (stderr notice + timestamped backup under `~/.oboron/bkp/`),
  matching `config show` / `profile show`.

### Conformance

Validated end-to-end against the canonical oboron test vectors
by `oboron-cli-conformance`: the `ob` suites pass (`ob-smoke`
35, `ob-vectors` 3320), 0 fail — unchanged from 0.4.0: a
secret's bytes are identical whether supplied as hex or base64,
so the protocol output is unaffected by the format work.

## [0.4.0] — 2026-05-23

First release of `oboron-cli` published from the
[`oboron-tools-rs`](https://gitlab.com/oboron/oboron-tools-rs)
workspace. Previous releases (`0.1.0`, `0.3.0`, `0.3.1`) came
from the original `oboron-rs` repo before `oboron-cli` moved
here in 0.3.1's "Future releases publish from oboron-tools-rs"
note.

### Added

- **Automatic migration of the legacy `~/.ob/` config dir.** On
  first run, if `~/.ob/` exists as a real directory and
  `~/.oboron/` doesn't, the legacy dir is renamed to
  `~/.oboron/` and a `~/.ob` → `~/.oboron` symlink is left in
  place so any older binary still installed continues to read
  and write the same data. Refuses to migrate ambiguous state
  where both dirs exist as real directories, surfacing the
  conflict so the user resolves manually. No-op on fresh
  installs and on every subsequent invocation.
- **Eager base64 → hex profile-key migration.** A profile that
  still carries a legacy 86-char base64 key is rewritten in
  place to canonical 128-char hex on display (`config show`,
  `profile show`, `key`), not just on encrypt / decrypt. The
  stderr notice and timestamped backup under `~/.oboron/bkp/`
  fire on the first display, ensuring `config show` no longer
  prints the raw base64 indefinitely.

### Changed

- **Refactored to consume
  [`oboron-cli-core 0.1.0`](https://crates.io/crates/oboron-cli-core).**
  The shared `~/.oboron/` config / profile plumbing, key
  normalization, and `init` / `config` / `profile` / `key`
  command handlers have been lifted out of this crate into
  `oboron-cli-core`, which is also consumed by
  [`obcrypt-cli`](https://crates.io/crates/obcrypt-cli).
  Behavior is preserved; the cross-binary file format remains
  the same (writes preserve unknown JSON fields so `ob` /
  `obcrypt` don't clobber each other's settings).
- **`oboron` dependency bumped** from `0.7.1` to `0.9.0`.
- **Repository URL** stabilized at
  `gitlab.com/oboron/oboron-tools-rs` (where this crate now
  lives).

### Removed

- **Vestigial `pyproject.toml`.** The crate had a leftover
  maturin packaging file from a previous experiment. It was
  never used to ship a Python distribution and its URLs
  pointed at the long-frozen GitHub mirror; deleted.

### Conformance

Validated end-to-end against the canonical oboron test vectors
by `oboron-cli-conformance`: the `ob` suites pass (`ob-smoke`
35, `ob-vectors` 3320), 0 fail.

## [0.3.1] — 2026-05-20

Final release of `oboron-cli` from the `oboron-rs` workspace.

### Changed

- Repository URL → `gitlab.com/oboron/oboron-tools-rs`.
- `oboron` dependency bumped from `0.7.0` to `0.7.1`.
- README updates reflecting the move.

## [0.3.0] — 2026-03-03

### Changed

- `oboron` dependency bumped from `0.6.0` to `0.7.0`.
- Version jump from `0.1.0` (skipping `0.2.x`) to track the
  `oboron` library version going forward.

## [0.1.0] — 2026-03-02

Initial public release.

### Added

- `ob` binary for the secure encryption schemes.
- Profile-based key management with automatic backup.
- Shell completion support (bash, zsh, fish, PowerShell).
- Stdin piping support.
- Format specification with `--format` flag.
- Feature-gated scheme selection.
