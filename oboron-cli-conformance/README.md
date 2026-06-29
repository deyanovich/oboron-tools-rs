# oboron-cli-conformance

Cross-implementation conformance test suite for the
[oboron](https://oboron.org/) protocol CLI surface — the secure
`ob` and `obcrypt` binaries. Spawns them end-to-end against the
canonical test vectors and reports pass/fail. Implementers of the
oboron protocol in other languages point this tool at their
binaries to validate conformance.

The unauthenticated **obu** CLI is certified separately by
[`obu-cli-conformance`](https://crates.io/crates/obu-cli-conformance),
mirroring the obu layer's segregation from the secure core: this
crate never touches the `obu` binary or the obu vectors.

Distributed as both a library (for in-workspace use) and a
standalone binary (`oboron-cli-conformance`).

## Install

```sh
cargo install oboron-cli-conformance
```

## Use as a binary

If your `ob` and `obcrypt` binaries are on `$PATH`:

```sh
oboron-cli-conformance
```

To point at specific binaries (e.g. your own
implementation):

```sh
oboron-cli-conformance \
  --ob /path/to/my-ob \
  --obcrypt /path/to/my-obcrypt
```

Exit code is `0` iff every test passed.

### Restrict to a subset

```sh
# Single suite
oboron-cli-conformance --suite ob-vectors

# Multiple suites
oboron-cli-conformance \
  --suite ob-vectors --suite obcrypt-vectors
```

### Verbose output

```sh
oboron-cli-conformance --verbose
```

By default only failures are printed (and per-suite
counts). With `--verbose`, every test result is shown.

## What it tests

Four suites covering the secure CLI surface:

| Suite                | Binary    | Surface                      |
|----------------------|-----------|------------------------------|
| `ob-smoke`           | `ob`      | flag parsing, encoding       |
|                      |           | defaults, roundtrips,        |
|                      |           | error handling               |
| `ob-vectors`         | `ob`      | vector-driven enc/dec        |
|                      |           | for the authenticated        |
|                      |           | schemes (`dgcmsiv`, `dsiv`,  |
|                      |           | `pgcmsiv`, `psiv`)           |
| `ob-negative`        | `ob`      | negative vectors: each       |
|                      |           | input MUST fail with exit    |
|                      |           | `1` (spec §10)               |
| `obcrypt-vectors`    | `obcrypt` | the positive vectors         |
|                      |           | filtered to `.hex` formats   |

The `ob-negative` suite asserts that every input in
`negative-test-vectors.jsonl` fails its operation (`enc` / `dec`)
as an operation failure with exit status `1`, per the oboron CLI
spec §10. Together with the single uniform `dec` failure message
required by §8, this is what keeps `dec` from acting as a
decryption oracle.

### Strategy per scheme class

- **Deterministic** (`dgcmsiv`, `dsiv`): obtext is fully
  determined by plaintext + key. The suite asserts exact
  match for both `enc(plaintext) → obtext` and
  `dec(obtext) → plaintext`.
- **Probabilistic** (`pgcmsiv`, `psiv`): obtext varies per
  call. The suite asserts exact match for
  `dec(canned obtext) → plaintext`, then exercises the
  encrypt path via a fresh encrypt-then-decrypt roundtrip.

## Hardcoded test key

The vector suites that exercise `-K` / `--keyless` mode use
the protocol's canonical hardcoded test key, defined in the
[oboron CLI spec, §9](https://oboron.org/cli-spec-v1-rev1#s9).
The key is inlined in the binary at compile time — no
dependency on any specific implementation under test.

## Test vectors

The vector data lives in a separate repository,
[`oboron-test-vectors`](https://gitlab.com/oboron/oboron-test-vectors),
and is consumed here as a git submodule at `tests/vectors/`.
Two JSONL files are embedded into the crate at compile time via
`include_str!`, so the installed binary works without external
file lookups:

- `test-vectors.jsonl` — the positive core vectors, driving the
  `ob-vectors` and `obcrypt-vectors` suites.
- `negative-test-vectors.jsonl` — the negative vectors, driving
  the `ob-negative` suite.

The obu vectors (`obu-test-vectors.jsonl`) are **not** embedded
here; they belong to the unauthenticated layer and are carried by
[`obu-cli-conformance`](https://crates.io/crates/obu-cli-conformance)
instead.

Other-language implementations can also consume the vectors
directly from that repository — the README there documents
the JSONL schema.

## Use as a library

For in-workspace integration tests, depend on the crate as
a normal library and call the `run_*` functions:

```rust
use oboron_cli_conformance::{Config, run_ob_vectors};

#[test]
fn ob_vectors_conform() {
    let cfg = Config::from_path();
    run_ob_vectors(&cfg).assert_success();
}
```

Public surface:

- `Config` — binary paths + scheme filter; default
  constructor `Config::from_path()` resolves binaries via
  `$PATH`.
- `Report` — accumulating per-test result set with
  `passed()`, `failed()`, `skipped()`, `is_success()`,
  `assert_success()`.
- `run_ob_smoke(cfg) -> Report`
- `run_ob_vectors(cfg) -> Report`
- `run_ob_negative(cfg) -> Report`
- `run_obcrypt_vectors(cfg) -> Report`

## Feature flags

| Feature    | Enables                          |
|------------|----------------------------------|
| `dgcmsiv`  | `dgcmsiv` scheme suites          |
| `dsiv`     | `dsiv` scheme suites             |
| `pgcmsiv`  | `pgcmsiv` scheme suites          |
| `psiv`     | `psiv` scheme suites             |

The default feature set enables all of the above. Vectors
for disabled schemes are reported as `Skipped`, not
counted as failures.

## Caveats

### Empty plaintext is rejected

`ob enc` rejects empty-string plaintext with a non-zero
exit code (`Error: enc failed: empty plaintext`). The smoke
suite asserts **failure** for that input rather than
success.

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
