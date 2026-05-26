# Changelog

All notable changes to `oboron-cli` are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.5.0] — 2026-05-25

Brings z-tier (`obz`) secrets in line with the hex-canonical key
format already used by `ob` and `obcrypt`, adds standalone
key/secret generators, unifies `obz`'s config under the shared
`~/.oboron/` root, and fixes `ob key`'s output format.

### Added

- **`ob keygen`.** Prints a fresh random 128-char hex key to
  stdout and exits — a scripting convenience that creates or
  modifies no profile and needs no config dir. Mirrors the
  existing `obcrypt keygen`.
- **`obz secretgen`.** The z-tier parallel: prints a fresh random
  64-char hex secret to stdout, touching no profile.
- **`ob key --base64` / `obz secret --base64` (`-B`).** Opt-in
  legacy base64 output for the rare caller that still needs it,
  emitting a deprecation warning on stderr. Conflicts with
  `--hex`. base64 support is still slated for removal before
  oboron 1.0.

### Changed

- **`obz` secrets are now canonical 64-char hex** (32 bytes),
  matching the move `ob` / `obcrypt` keys already made. `obz
  secretgen`, `obz init`, and `obz profile create` generate hex;
  `obz secret`, `config show`, and `profile show` display hex
  (`-x`/`--hex` is now an accepted no-op). Input accepts **both**
  64-char hex and legacy 43-char base64 — via `--secret`,
  `$OBORON_SECRET`, and stored profiles — warning on base64. A
  profile still holding a base64 secret is rewritten to hex in
  place on first read (stderr notice + timestamped backup),
  exactly as `ob` does for keys.
- **`obz` config moved from `~/.obz/` to `~/.oboron/ztier/`,**
  unifying it under the same root as `ob` / `obcrypt`. `obz`
  keeps its own `config.json` in that subdir because its z-tier
  `scheme` namespace is mutually exclusive with the secure
  schemes, so a shared file would collide. A one-time migration
  on first run renames the legacy dir and leaves a `~/.obz` →
  `~/.oboron/ztier` symlink for backward compatibility, mirroring
  the earlier `~/.ob` → `~/.oboron` migration; it refuses to act
  on an ambiguous state where both exist as real directories.
- **Key-/secret-flag help text** on `enc` / `dec` and `profile
  create` / `profile set` now reads "128 hex chars, or legacy
  86-char base64" (`ob`) and "64 hex chars, or legacy 43-char
  base64" (`obz`) instead of the base64-only wording, matching
  the canonical formats.

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
by `oboron-cli-conformance` v0.2.0: 4197 pass, 0 fail, 0 skip
across all five `ob` / `obz` suites (`ob-smoke` 35, `ob-vectors`
3320, `obz-smoke` 13, `obz-ztier-vectors` 664,
`obz-legacy-vectors` 165) — unchanged from 0.4.0: a secret's
bytes are identical whether supplied as hex or base64, so the
protocol output is unaffected by the format work.

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
