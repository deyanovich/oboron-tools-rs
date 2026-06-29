# oboron-cli-core

Shared CLI plumbing for the oboron-protocol tooling — consumed by
`oboron-cli` (the `ob` binary),
[`obcrypt-cli`](https://crates.io/crates/obcrypt-cli) (the
`obcrypt` binary), and
[`obu-cli`](https://crates.io/crates/obu-cli) (the `obu`
binary). Each binary supplies its own per-binary environment
(`CliEnv`), so the same plumbing backs the unauthenticated `obu`
CLI as well as the secure `ob` / `obcrypt` pair. Published on
crates.io because the consuming binaries pull it in via crates.io
for downstream installation. As of 1.0 the public API is stable
under SemVer.

What lives here:

- Path resolution for the `~/.oboron/` directory tree.
- Profile name validation.
- Key string normalization (canonical hex only).
- `config.json` and `profiles/<NAME>.json` read/write, preserving
  unknown fields so the binaries don't clobber each other's
  metadata.
- Automatic backups on profile overwrite/delete.
- Command-handler implementations for the `init` / `config` /
  `profile` / `key` subcommands, parameterized over a `CliInfo`
  supplying per-binary defaults.

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
