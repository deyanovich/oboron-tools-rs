# Changelog

All notable changes to the `oboron-cli` PyPI distribution are
documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to [Semantic Versioning](https://semver.org/).

The version of this distribution exactly tracks the version of
the wrapped [`oboron-cli`](https://crates.io/crates/oboron-cli)
Rust crate — each PyPI release is a wheel-wrapped build of that
crate at the same version. For per-binary changes, see the
[crate's CHANGELOG](https://gitlab.com/oboron/oboron-tools-rs/-/blob/master/oboron-cli/CHANGELOG.md).

## [0.5.0] — 2026-05-25

Ships the `ob` and `obz` binaries from
[`oboron-cli` 0.5.0](https://crates.io/crates/oboron-cli/0.5.0)
as a single maturin bin-only wheel.

See the underlying crate's CHANGELOG for the substantive changes
in 0.5.0: `obz` secrets become canonical 64-char hex (matching
the `ob` / `obcrypt` key format; legacy base64 still accepted),
`obz` config moves under the shared `~/.oboron/ztier/`, new `ob
keygen` / `obz secretgen` generator commands, and the `ob key`
hex-default fix.

## [0.4.0] — 2026-05-23

First PyPI release from the
[`oboron-tools-rs`](https://gitlab.com/oboron/oboron-tools-rs)
workspace. Previous PyPI releases (0.1.0, 0.2.0, 0.3.0) came
from the predecessor `oboron-rs` workspace before `oboron-cli`
moved here; the 0.4.0 jump mirrors the underlying
[Rust crate](https://crates.io/crates/oboron-cli/0.4.0)'s
version.

Ships the `ob` and `obz` binaries from
[`oboron-cli` 0.4.0](https://crates.io/crates/oboron-cli/0.4.0)
as a single maturin bin-only wheel.

See the underlying crate's CHANGELOG for the substantive
changes in 0.4.0 (config-dir migration, eager base64 → hex
profile-key migration, refactor onto `oboron-cli-core`, etc.).
