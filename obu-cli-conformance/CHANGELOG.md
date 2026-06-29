# Changelog

All notable changes to `obu-cli-conformance` are documented here.
The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-06-29

Initial release — a cross-implementation conformance suite and
binary (`obu-cli-conformance`) for the obu CLI, the `obu` binary
of the oboron family's unauthenticated / obfuscation layer
(schemes `upcbc` and `zdcbc`).

The suite is deliberately kept separate from the authenticated
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
suite, mirroring the obu layer's segregation from the secure core:
obu has its own library, its own `obu` binary, and its own
`~/.obu/` config dir, sharing no code with the authenticated path,
and its conformance is segregated the same way. An implementer of
the secure protocol never has to touch obu, and an implementer of
obu never pulls in the secure suite.

### Added

- **obu positive-vector coverage.** Validates the canonical obu
  positive vectors — `upcbc` and `zdcbc` across every encoding —
  against the fixed public test secret (obu spec §6.4), applied
  via `obu --keyless`. `zdcbc` is checked by exact-match enc and
  dec (deterministic); `upcbc` by decrypting the stored obtext
  and then confirming an enc → dec roundtrip.
- **The `obu-cli-conformance` binary.** Points at any `obu`
  implementation via `--obu <path>`, falling back to `obu` on
  `$PATH`. The canonical vectors are embedded at build time, so
  the installed binary is self-contained.

### Conformance

Run against the 1.0 `obu` binary, the suite passes all 1328 obu
positive vectors (`upcbc` / `zdcbc` over every encoding), 0 fail.
obu negative-vector coverage will follow once the test-vectors
repo publishes `obu-negative-test-vectors.jsonl`; until then the
suite certifies positive (wire-format and round-trip) conformance
only and makes no security claim.

### Licensing

- Dual-licensed `MIT OR Apache-2.0`.
