# oboron-cli (PyPI distribution)

`pip`-installable distribution of the
[`oboron-cli`](https://crates.io/crates/oboron-cli) command-line
binaries — `ob` (string-in / string-out symmetric encryption with
obtext encoding) and `obz` (z-tier obfuscation).

The wheel is **binary-only**: it ships both prebuilt Rust
binaries (`ob` and `obz`), packaged so `pip install oboron-cli`
drops them on `$PATH`. There is no Python module — `import
oboron_cli` will not work. Use the binaries from the shell, or
via `subprocess`.

## Install

```bash
pip install oboron-cli
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv tool install oboron-cli
```

## What you get

Two binaries:

- **`ob`** — secure encryption (a-tier and u-tier schemes:
  `aasv`, `aags`, `apsv`, `apgs`, `upbc`).
- **`obz`** — z-tier obfuscation (non-secure; included with the
  default `all-schemes` feature).

```text
ob <SUBCOMMAND>

Subcommands:
  enc     (e)  Encrypt plaintext (output: obtext)
  dec     (d)  Decrypt obtext (auto-detects scheme by default)
  keygen  (k)  Generate a fresh random 128-character hex key
  init    (i)  Initialize configuration with a fresh profile
  config  (c)  Show or update configuration
  profile (p)  Manage key profiles
  key          Print the active profile's key
  completion   Generate shell completion script
```

Full CLI documentation lives in the Rust crate's
[README on crates.io](https://crates.io/crates/oboron-cli) and
its [repository](https://gitlab.com/oboron/oboron-tools-rs/-/tree/master/oboron-cli).

## Relation to the previous `oboron-cli` PyPI releases

This is the first release of `oboron-cli` on PyPI from the
[`oboron-tools-rs`](https://gitlab.com/oboron/oboron-tools-rs)
workspace; previous PyPI releases (0.1.0, 0.2.0, 0.3.0) came
from the predecessor `oboron-rs` workspace before `oboron-cli`
moved here. The 0.4.0 jump mirrors the version of the underlying
[Rust crate](https://crates.io/crates/oboron-cli/0.4.0).

## Why ship a Rust binary via pip?

The Python ecosystem has the broadest reach for ad-hoc tool
installation across operating systems. Users who already manage
their tooling with `pip` or `uv` can pull in `ob` / `obz`
without adding another package manager. Functionally identical
to `cargo install oboron-cli` — different distribution channel,
same binaries.

## Conformance

The `ob` and `obz` binaries inside the wheel are the same ones
published to crates.io, validated end-to-end against the
canonical oboron test vectors by
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
v0.2.0 (4197 pass, 0 fail across all five `ob` / `obz` suites).

## License

MIT — see [LICENSE](LICENSE).
