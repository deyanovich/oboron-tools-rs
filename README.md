# oboron-tools-rs

Tooling workspace for the [oboron](https://oboron.org/)
protocol. Six member crates:

- [`./oboron-cli`](./oboron-cli) — the `ob` binary: string-in /
  string-out symmetric encryption over the secure oboron core.
- [`./obcrypt-cli`](./obcrypt-cli) — the `obcrypt` binary:
  bytes-in / bytes-out access to the same secure core, without
  oboron's text encoding layer.
- [`./obu-cli`](./obu-cli) — the `obu` binary: the
  unauthenticated obfuscation tier. NOT cryptographically
  secure; segregated from the secure core and intended only for
  obfuscation, never confidentiality.
- [`./oboron-cli-core`](./oboron-cli-core) — shared CLI plumbing
  (config, key handling, env) used by the binaries above.
- [`./oboron-cli-conformance`](./oboron-cli-conformance) —
  cross-implementation conformance suite for the SECURE CLI
  surface (`ob` and `obcrypt`). Distributed as a library and an
  `oboron-cli-conformance` binary.
- [`./obu-cli-conformance`](./obu-cli-conformance) — separate
  conformance suite for the unauthenticated `obu` surface, kept
  apart from the secure suite. Distributed as a library and an
  `obu-cli-conformance` binary.

## Build

```bash
cargo build --workspace
cargo test --workspace
```

The build produces the `ob`, `obcrypt`, and `obu` binaries
alongside the two conformance runners. The conformance suites
spawn the CLIs end-to-end, so have `ob`, `obcrypt`, and `obu`
on `$PATH` before `cargo test` — or pass explicit paths via the
`--ob`, `--obcrypt`, and `--obu` flags on the conformance
binaries.

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
