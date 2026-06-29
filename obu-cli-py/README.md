# obu-cli (PyPI distribution)

`pip`-installable distribution of the
[`obu`](https://crates.io/crates/obu-cli) command-line binary —
the oboron family's **unauthenticated** obfuscation layer
(schemes `upcbc` and `zdcbc`), with a 64-hex secret and obtext
encoding.

> **`obu` is NOT cryptographically secure.** `upcbc` gives
> confidentiality without integrity and `zdcbc` is obfuscation
> only — neither is authenticated. Never use `obu` for sensitive
> data; use the authenticated [`ob`](https://crates.io/crates/oboron-cli)
> or [`obcrypt`](https://crates.io/crates/obcrypt-cli) for anything
> security-critical. obu shares no code with the secure core.

The wheel is **binary-only**: it ships the prebuilt Rust `obu`
binary, packaged so `pip install obu-cli` drops it on `$PATH`.
There is no Python module — `import obu_cli` will not work. Use
it from the shell, or via `subprocess`.

## Install

```bash
pip install obu-cli
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv tool install obu-cli
```

## What you get

A single binary, `obu`, supporting:

```text
obu <SUBCOMMAND>

Subcommands:
  enc (e)       Encode+obfuscate a plaintext string
  dec (d)       Decode a obtext string (scheme supplied by the caller)
  secretgen     Generate a fresh random 64-character hex secret
  init (i)      Initialize configuration with a fresh secret profile
  config (c)    Show or update configuration
  profile (p)   Manage secret profiles
  secret (s)    Print the active profile's secret
  completion    Generate shell completion script
```

The 256-bit secret is supplied via `--secret`, the
`OBORON_SECRET` environment variable, or `--keyless` (the fixed
public test secret); the built-in default scheme is `upcbc`. Full
CLI documentation lives in the Rust crate's
[README on crates.io](https://crates.io/crates/obu-cli) and its
[repository](https://gitlab.com/oboron/oboron-tools-rs/-/tree/master/obu-cli).

## Why ship a Rust binary via pip?

The Python ecosystem has the broadest reach for ad-hoc tool
installation across operating systems. Users who already manage
their tooling with `pip` or `uv` can pull in `obu` without adding
another package manager. Functionally identical to `cargo install
obu-cli` — different distribution channel, same binary.

## Conformance

The `obu` binary inside the wheel is the same one published to
crates.io, validated against the canonical obu test vectors by
[`obu-cli-conformance`](https://crates.io/crates/obu-cli-conformance)
1.0.0 — a suite kept separate from the authenticated
`oboron-cli-conformance`, mirroring obu's segregation from the
secure core.

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
