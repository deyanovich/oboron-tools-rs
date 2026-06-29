# Changelog

All notable changes to `obu-cli` are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-06-29

Initial release of `obu-cli`, the oboron family's
unauthenticated / obfuscation layer. It produces a single `obu`
binary, tracking the `obu` library 1.0.0.

**obu is NOT authenticated.** The `upcbc` scheme gives
confidentiality without integrity and `zdcbc` is obfuscation
only — neither is cryptographically secure. Never use `obu` for
sensitive data; use `ob`, the authenticated oboron core CLI, for
anything security-critical. obu shares no code with the secure
core.

### Added

- **`obu` binary** mirroring the core `ob` CLI surface — the
  same `enc` / `dec` commands, profile and config management,
  `--format` strings, and line framing — over the
  unauthenticated `upcbc` and `zdcbc` schemes. The built-in
  default scheme is `upcbc`.
- **64-hex secrets.** A 256-bit secret (`--secret` /
  `$OBORON_SECRET`, 64 hex characters) replaces the 512-bit key
  used by `ob` / `obcrypt`. Secrets are canonical lowercase hex
  only — no base64 input.
- **`~/.obu/` config directory,** separate from the `~/.oboron/`
  root used by the secure `ob` / `obcrypt` CLIs, keeping the
  unauthenticated layer segregated from the secure one.
- **`obu secretgen`.** Prints a fresh random 64-hex secret to
  stdout, touching no profile or config.
- **Commands:** `enc`, `dec`, `secretgen`, `init`, `config`,
  `profile`, `secret`, and `completion`. Profiles and config
  are an implementation convenience, not required by the spec.

### Behavior

- **`dec` reports a single uniform error.** Every decode failure
  — bad encoding, wrong length, invalid UTF-8, or empty input —
  is reported through one stderr message and exit `1`, so `dec`
  is not a distinguishing oracle (OBU spec §2.1 / §6.4).
- **Usage errors exit `2`.** Conflicting or invalid flags and
  malformed `--format` strings exit `2`, inheriting the
  exit-code contract of the core oboron CLI spec (§8).

### Conformance

Validated end-to-end against the canonical obu test vectors by
`obu-cli-conformance`: all 1328 obu vectors pass. The
unauthenticated obu suite is segregated from the secure
`oboron-cli-conformance` suite exactly as the obu layer is
segregated from the oboron core.

### Licensing

- Dual-licensed `MIT OR Apache-2.0`.
