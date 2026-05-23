# Changelog

All notable changes to `oboron-cli` are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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
  `obz` / `obcrypt` don't clobber each other's settings).
- **`oboron` dependency bumped** from `0.7.1` to `0.9.0`.
- **`obz` keyless-mode now reads `oboron::HARDCODED_KEY_BYTES`
  directly** instead of decoding the deprecated
  `HARDCODED_KEY_BASE64` constant — eliminates the deprecation
  warnings that surfaced in `0.9.0`.
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
by `oboron-cli-conformance` v0.2.0: 4197 pass, 0 fail, 0 skip
across all five suites (`ob-smoke` 35, `ob-vectors` 3320,
`obz-smoke` 13, `obz-ztier-vectors` 664, `obz-legacy-vectors`
165).

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

- `ob` binary for secure encryption schemes (a-tier, u-tier).
- `obz` binary for z-tier obfuscation schemes.
- Profile-based key management with automatic backup.
- Shell completion support (bash, zsh, fish, PowerShell).
- Stdin piping support.
- Format specification with `--format` flag.
- Feature-gated scheme selection.
