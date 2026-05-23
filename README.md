# oboron-tools-rs

Tooling workspace for the [oboron](https://oboron.org/)
protocol. Currently ships a single crate:

- [`./oboron-cli-conformance`](./oboron-cli-conformance) —
  cross-implementation conformance test suite for the oboron
  protocol CLI surface (`ob`, `obz`, `obcrypt`). Distributed
  as both a library and an `oboron-cli-conformance` binary
  for validating alternative-language implementations.

Additional tooling crates (`oboron-cli`, `obcrypt-cli`,
shared CLI plumbing) will land in future releases.

## Build

```bash
cargo build --workspace
cargo test --workspace
```

The conformance suite spawns `ob`, `obz`, and `obcrypt`
end-to-end; have those binaries available on `$PATH` before
`cargo test`, or pass explicit paths to the
`oboron-cli-conformance` binary via `--ob`, `--obz`,
`--obcrypt`.

## License

MIT — see [LICENSE](LICENSE).
