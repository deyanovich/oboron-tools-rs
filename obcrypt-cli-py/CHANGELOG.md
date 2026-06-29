# Changelog

All notable changes to the `obcrypt-cli` PyPI distribution are
documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/).

The version of this distribution exactly tracks the version of
the wrapped [`obcrypt-cli`](https://crates.io/crates/obcrypt-cli)
Rust crate — each PyPI release is a wheel-wrapped build of that
crate at the same version. For per-binary changes, see the
[crate's CHANGELOG](https://gitlab.com/oboron/oboron-tools-rs/-/blob/master/obcrypt-cli/CHANGELOG.md).

## [1.0.0] — 2026-06-29

Tracks [`obcrypt-cli` 1.0.0](https://crates.io/crates/obcrypt-cli/1.0.0).
The wrapped crate reaches its first stable release: keys and
secrets are canonical hex only (legacy base64 keys are removed,
with no auto-migration), and `decrypt` requires the caller to
supply the scheme. The wheel version tracks the crate version.

## [0.1.0] — 2026-05-23

Initial PyPI release. Ships the `obcrypt` binary from
[`obcrypt-cli` 0.1.0](https://crates.io/crates/obcrypt-cli/0.1.0)
as a maturin bin-only wheel.
