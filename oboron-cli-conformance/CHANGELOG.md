# Changelog

All notable changes to `oboron-cli-conformance` are documented
here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-06-29

First stable release, covering the oboron CLI specification 1.0
surface for the secure `ob` / `obcrypt` binaries. Tracks the
published `oboron` 1.0.1 and `obcrypt` 1.0.0 libraries.

This crate covers the authenticated core only. Conformance for the
unauthenticated obu CLI lives in a separate crate,
[`obu-cli-conformance`](https://crates.io/crates/obu-cli-conformance),
mirroring the obu layer's segregation from the secure core: obu
has its own library, its own `obu` binary, and its own `~/.obu/`
config dir, sharing no code with the authenticated path. An
implementer of the secure protocol never pulls in the obu suite,
and an implementer of obu never pulls in this one.

### Added

- **Negative-vector suite (`ob-negative`).** Drives the embedded
  `negative-test-vectors.jsonl` through `ob`. Each vector names an
  `op` (`enc` / `dec`), a `format`, an offending `input`, and a
  `reason`, and the run asserts that the operation fails as an
  operation failure with exit status `1` (oboron CLI spec §10).
  Together with the uniform `dec` failure message (§8), this
  certifies that `dec` does not act as a decryption oracle.
- **Loud skips for an absent binary.** The `tests/*.rs` wrappers
  probe the binary under test and, when it is not installed, print
  a `SKIPPED:` notice naming the missing binary and the override
  flag instead of failing. A conformance suite cannot validate a
  binary that is absent, and a hard spawn failure would otherwise
  read as a real conformance failure.

### Changed

- **Scoped to the secure `ob` / `obcrypt` surface.** Four suites:
  `ob-smoke` (flag parsing, encoding defaults, roundtrips, error
  handling), `ob-vectors` (vector-driven enc / dec for the four
  authenticated schemes `dgcmsiv`, `dsiv`, `pgcmsiv`, `psiv`),
  `ob-negative` (the new negative suite), and `obcrypt-vectors`
  (the same positive vectors filtered to the `.hex` formats, run
  through `obcrypt`).
- **Two embedded vector files** — `test-vectors.jsonl` (positive
  core) and `negative-test-vectors.jsonl` — inlined at compile
  time via `include_str!`, so the installed binary needs no
  external file lookups.

### Conformance

Run against the 1.0 `ob` / `obcrypt` binaries, the full suite
passes — `ob-smoke`, `ob-vectors`, `ob-negative`, and
`obcrypt-vectors` all green, 0 fail.

### Licensing

- Dual-licensed `MIT OR Apache-2.0`.

## [0.2.0] — 2026-05-23

Renamed the public API and suite surface from the old `obc`
spelling to `obcrypt`, matching the binary's canonical name.

## [0.1.0] — 2026-05-20

Initial release (first published as `oboron-cli-tests`, then
renamed to `oboron-cli-conformance`): a cross-implementation
conformance suite and binary for the oboron CLI surface, driving
the binaries under test against the canonical JSONL test vectors.
