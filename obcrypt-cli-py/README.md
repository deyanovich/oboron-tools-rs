# obcrypt-cli (PyPI distribution)

`pip`-installable distribution of the
[`obcrypt`](https://crates.io/crates/obcrypt) command-line
binary — bytes-in / bytes-out symmetric encryption (oboron
protocol, a-tier + u-tier).

The wheel is **binary-only**: it ships the prebuilt Rust `obcrypt`
binary, packaged so `pip install obcrypt-cli` drops it on
`$PATH`. There is no Python module — `import obcrypt_cli` will
not work. Use it from the shell, or via `subprocess`.

## Install

```bash
pip install obcrypt-cli
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv tool install obcrypt-cli
```

## What you get

A single binary, `obcrypt`, supporting:

```text
obcrypt <SUBCOMMAND>

Subcommands:
  encrypt (e)   Encrypt plaintext bytes under a scheme
  decrypt (d)   Decrypt ciphertext bytes (auto-detects scheme by default)
  keygen  (k)   Generate a fresh random 128-character hex key
  init    (i)   Initialize configuration with a fresh profile
  config  (c)   Show or update configuration
  profile (p)   Manage key profiles
  key           Print the active profile's key
  completions   Generate shell completion script
```

Full CLI documentation lives in the Rust crate's
[README on crates.io](https://crates.io/crates/obcrypt-cli) and
its [repository](https://gitlab.com/oboron/oboron-tools-rs/-/tree/master/obcrypt-cli).

## Why ship a Rust binary via pip?

The Python ecosystem has the broadest reach for ad-hoc tool
installation across operating systems. Users who already manage
their tooling with `pip` or `uv` can pull in `obcrypt` without
adding another package manager. Functionally identical to
`cargo install obcrypt-cli` — different distribution channel,
same binary.

## Conformance

The `obcrypt` binary inside the wheel is the same one published
to crates.io, validated end-to-end against the canonical oboron
test vectors by
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
v0.2.0.

## License

MIT — see [LICENSE](LICENSE).
