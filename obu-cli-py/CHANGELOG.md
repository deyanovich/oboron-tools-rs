# Changelog

All notable changes to the `obu-cli` PyPI distribution are
documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/).

The version of this distribution exactly tracks the version of
the wrapped [`obu-cli`](https://crates.io/crates/obu-cli) Rust
crate — each PyPI release is a wheel-wrapped build of that crate
at the same version. For per-binary changes, see the
[crate's CHANGELOG](https://gitlab.com/oboron/oboron-tools-rs/-/blob/master/obu-cli/CHANGELOG.md).

## [1.0.0] — 2026-06-29

Initial PyPI release. Ships the `obu` binary from
[`obu-cli` 1.0.0](https://crates.io/crates/obu-cli/1.0.0) as a
maturin bin-only wheel, bringing the unauthenticated obfuscation
layer (`upcbc` / `zdcbc`, 64-hex secret) to `pip` / `uv`
installs alongside the authenticated `oboron-cli` and
`obcrypt-cli` wheels.

`obu` is **not** authenticated and is for obfuscation only — see
the README and the crate docs.
