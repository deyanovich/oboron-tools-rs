# Changelog

All notable changes to `oboron-cli-core` are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

`oboron-cli-core` is the shared-plumbing crate consumed by
[`oboron-cli`](https://crates.io/crates/oboron-cli) (the `ob`
binary),
[`obcrypt-cli`](https://crates.io/crates/obcrypt-cli) (the
`obcrypt` binary), and
[`obu-cli`](https://crates.io/crates/obu-cli) (the `obu`
binary). Published on crates.io because the consuming binaries
depend on it via the registry for downstream installation. As of
1.0 the public API is stable under SemVer; the growable types
(`Config`, `KeyProfile`, `CliEnv`) are `#[non_exhaustive]` with
`new()` constructors, so fields can be added without a breaking
change.

## [1.0.0] — 2026-06-29

First stable release. The public API is now sealed under SemVer.

### Removed

- **Legacy base64 key API** — `KeyFormat`,
  `normalize_key_classify`, `LoadedKey`, and `load_profile_key`
  are gone. Keys and secrets are canonical hex only; base64
  acceptance and the eager base64 → hex migration are no longer
  part of the surface. `normalize_key_to_hex` and
  `load_profile_key_as_hex` remain as the hex-only entry points.

### Changed

- **Sealed public types** — `Config`, `KeyProfile`, and `CliEnv`
  are `#[non_exhaustive]` and gain `new()` constructors, so future
  fields can be added without breaking downstream callers.

### Added

- **Per-binary environment** — a new `env` module exposes
  `CliEnv`, the per-binary environment (config root, defaults)
  threaded through the command handlers. This is what lets the
  unauthenticated `obu` CLI reuse the same plumbing as `ob` and
  `obcrypt`.

## [0.1.0] — 2026-05-23

Initial public release.

### Added

- **Path resolution** for the `~/.oboron/` config tree —
  `config_root`, `config_path`, `profile_dir`, `profile_path`,
  `backup_dir`.
- **Config I/O** — `Config`, `load_config`, `save_config`. Writes
  preserve unknown JSON fields so the two consuming binaries don't
  clobber each other's metadata.
- **Profile I/O** — `KeyProfile`, `LoadedKey`, `load_profile`,
  `save_profile`, `load_profile_key`, `load_profile_key_as_hex`,
  `list_profiles`, `delete_profile`, `rename_profile`,
  `validate_profile_name`. Same unknown-field-preserving write
  behavior as `Config`.
- **Key normalization** — `normalize_key_to_hex`,
  `normalize_key_classify`, `KeyFormat`. Accepts the canonical
  128-char hex form *or* the legacy 86-char base64 form (during
  the base64 deprecation period) and returns canonical hex.
- **Automatic backups** — profile overwrite and delete leave a
  timestamped copy under `~/.oboron/bkp/`.
- **Command handlers** — `commands::*` implements the user-facing
  `init` / `config` / `profile` / `key` subcommands shared by both
  binaries, parameterized over a `commands::CliInfo` supplying the
  per-binary defaults (binary name, default scheme, default
  encoding) used in error hints and `init` output.
- **Legacy-dir migration** — `migration::ensure_config_root_migrated`
  moves a leftover `~/.ob/` to `~/.oboron/` on first run of the
  current tooling, leaving a `~/.ob` → `~/.oboron` symlink so any
  older binary still installed keeps reading and writing the same
  data. No-op on fresh installs and on every subsequent
  invocation.
- **Eager base64 → hex migration** — `load_profile_key_with_notice`
  rewrites a profile carrying a legacy 86-char base64 key to
  canonical 128-char hex on display (with stderr notice and
  timestamped backup), so `config show` / `profile show` / `key`
  surface the canonical form instead of the legacy one.
