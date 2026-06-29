# oboron-cli (PyPI distribution)

`pip`-installable distribution of the
[`oboron-cli`](https://crates.io/crates/oboron-cli) command-line
binary — `ob`, a string-in / string-out authenticated symmetric
encryption tool with obtext encoding.

The wheel is **binary-only**: it ships the prebuilt `ob` Rust
binary, packaged so `pip install oboron-cli` drops it on `$PATH`.
There is no Python module — `import oboron_cli` will not work. Use
the binary from the shell, or via `subprocess`.

## Install

```bash
pip install oboron-cli
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv tool install oboron-cli
```

## What you get

The `ob` binary — authenticated encryption over the four oboron
core schemes (`dsiv`, `psiv`, `dgcmsiv`, `pgcmsiv`):

```text
ob <SUBCOMMAND>

Subcommands:
  enc     (e)  Encrypt plaintext (output: obtext)
  dec     (d)  Decrypt obtext (scheme supplied by the caller)
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

`1.0.0` tracks oboron protocol spec 1.0 (rev3): authenticated
only, the four property-prefixed core schemes, hex-only keys, and
no scheme auto-detection. The obfuscation binary that earlier
releases shipped is no longer part of `oboron-cli`. The earliest
PyPI releases (0.1.0–0.3.0) came from the predecessor `oboron-rs`
workspace; from 0.4.0 the distribution has been built from the
[`oboron-tools-rs`](https://gitlab.com/oboron/oboron-tools-rs)
workspace.

## Why ship a Rust binary via pip?

The Python ecosystem has the broadest reach for ad-hoc tool
installation across operating systems. Users who already manage
their tooling with `pip` or `uv` can pull in `ob` without adding
another package manager. Functionally identical to
`cargo install oboron-cli` — different distribution channel, same
binary.

## Conformance

The `ob` binary inside the wheel is the same one published to
crates.io, validated end-to-end against the canonical oboron test
vectors by
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance).

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as
above, without any additional terms or conditions.
