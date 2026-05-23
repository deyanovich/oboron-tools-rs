# oboron-cli-core

Shared CLI plumbing for the oboron-protocol tooling — consumed by
both `oboron-cli` (the `ob` binary) and
[`obcrypt-cli`](https://crates.io/crates/obcrypt-cli) (the
`obcrypt` binary). Published on crates.io because the consuming
binaries pull it in via crates.io for downstream installation;
the API is shaped around what those two binaries need and may
change between minor versions.

What lives here:

- Path resolution for the `~/.oboron/` directory tree.
- Profile name validation.
- Key string normalization (hex canonical, legacy base64 accepted).
- `config.json` and `profiles/<NAME>.json` read/write, preserving
  unknown fields so the two binaries don't clobber each other's
  metadata.
- Automatic backups on profile overwrite/delete.
- Command-handler implementations for the `init` / `config` /
  `profile` / `key` subcommands, parameterized over a `CliInfo`
  supplying per-binary defaults.

## License

MIT — see [LICENSE](LICENSE).
