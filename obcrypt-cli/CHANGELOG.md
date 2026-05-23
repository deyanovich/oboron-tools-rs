# Changelog

All notable changes to `obcrypt-cli` are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] — 2026-05-23

Initial public release of the `obcrypt` binary — a command-line
interface for [`obcrypt`](https://crates.io/crates/obcrypt), the
bytes-in / bytes-out cryptographic core of the
[oboron](https://oboron.org/) protocol.

### Added

- `obcrypt encrypt` / `decrypt` / `keygen` (with aliases `e` / `d`
  / `k`) for raw-byte symmetric encryption under the obcrypt schemes
  (`aags`, `apgs`, `aasv`, `apsv`, `upbc`).
- Terminal I/O byte format flags, parallel on `encrypt` and
  `decrypt`: `-x`/`--hex` hex-encodes the **output** (ciphertext for
  `encrypt`, plaintext for `decrypt`), and `-X`/`--hex-in` hex-decodes
  the **input** before processing. Defaults to raw bytes on both
  sides.
- Hex-only keys (canonical 128 hex chars). Legacy 86-char base64
  keys are auto-detected and accepted during the deprecation period,
  with a stderr notice nudging users toward hex. Profiles stored
  with legacy base64 keys are rewritten in place to canonical hex
  on first use (encrypt / decrypt / key / config show / profile
  show), with a timestamped backup of the pre-migration file under
  `~/.oboron/bkp/`.
- Automatic migration of the legacy `~/.ob/` config dir from older
  oboron-cli releases: on first run, the dir is renamed to
  `~/.oboron/` and a `~/.ob` → `~/.oboron` symlink is left in
  place so any older binary still installed continues to read /
  write the same data. No-op on fresh installs and on every
  subsequent invocation.
- Key sourcing precedence: `--key` → `--profile` → active profile in
  `~/.oboron/config.json`.
- Profile / config management subcommands — `init`, `config`,
  `profile {list,show,activate,create,delete,rename,set}`, `key` —
  sharing the `~/.oboron/` directory with the `ob` CLI from
  [oboron-cli](https://gitlab.com/oboron/oboron-rs). Writes preserve
  unknown JSON fields so the two binaries don't clobber each other's
  settings.
- `obcrypt completions {bash,zsh,fish,…}` for shell completion
  scripts.

### Feature surface

`obcrypt-cli` is all-secure (a-tier authenticated, u-tier
unauthenticated but still real cryptography); there is no unsecure
subset, so no aggregate feature is exposed. The `default` enables
the five individual schemes directly (`aags`, `apgs`, `aasv`,
`apsv`, `upbc`), mirroring `obcrypt` 0.2.0's feature surface.
