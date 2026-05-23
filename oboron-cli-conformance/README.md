# oboron-cli-conformance

Cross-implementation conformance test suite for the
[oboron](https://oboron.org/) protocol CLI surface. Spawns
`ob`, `obz`, and `obcrypt` end-to-end against the canonical
test vectors and reports pass/fail. Implementers of the
oboron protocol in other languages point this tool at their
binaries to validate conformance.

Distributed as both a library (for in-workspace use) and a
standalone binary (`oboron-cli-conformance`).

## Install

```sh
cargo install oboron-cli-conformance
```

## Use as a binary

If your `ob`, `obz`, and `obcrypt` binaries are on `$PATH`:

```sh
oboron-cli-conformance
```

To point at specific binaries (e.g. your own
implementation):

```sh
oboron-cli-conformance \
  --ob /path/to/my-ob \
  --obz /path/to/my-obz \
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

Six suites covering the CLI surface:

| Suite                | Binary    | Surface                  |
|----------------------|-----------|--------------------------|
| `ob-smoke`           | `ob`      | flag parsing, encoding   |
|                      |           | defaults, roundtrips,    |
|                      |           | error handling           |
| `ob-vectors`         | `ob`      | vector-driven enc/dec    |
|                      |           | for the secure schemes   |
|                      |           | (`aags`, `aasv`, `apgs`, |
|                      |           | `apsv`, `upbc`)          |
| `obcrypt-vectors`    | `obcrypt` | same vectors filtered    |
|                      |           | to `.hex` formats        |
| `obz-smoke`          | `obz`     | flag parsing + error     |
|                      |           | handling for z-tier      |
| `obz-ztier-vectors`  | `obz`     | z-tier vectors           |
|                      |           | (`zrbcx`)                |
| `obz-legacy-vectors` | `obz`     | legacy-scheme vectors    |

### Strategy per scheme class

- **Deterministic** (`aags`, `aasv`, `zrbcx`, `legacy`):
  obtext is fully determined by plaintext + key. The suite
  asserts exact match for both `enc(plaintext) → obtext`
  and `dec(obtext) → plaintext`.
- **Probabilistic** (`apgs`, `apsv`, `upbc`): obtext
  varies per call. The suite asserts exact match for
  `dec(canned obtext) → plaintext`, then exercises the
  encrypt path via a fresh encrypt-then-decrypt roundtrip.

## Hardcoded test key

The vector suites that exercise `-K` / `--keyless` mode use
the protocol's canonical hardcoded test key, defined in the
[oboron CLI spec, §8](https://oboron.org/cli-spec-v1-rev1#s8).
The key is inlined in the binary at compile time — no
dependency on any specific implementation under test. The
legacy suite uses a per-file secret carried in the vector
data's meta line.

## Test vectors

The vector data lives in a separate repository,
[`oboron-test-vectors`](https://gitlab.com/oboron/oboron-test-vectors),
and is consumed here as a git submodule at
`tests/vectors/`. The three JSONL files are embedded into
the crate at compile time via `include_str!`, so the
installed binary works without external file lookups.

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
- `run_obcrypt_vectors(cfg) -> Report`
- `run_obz_smoke(cfg) -> Report`
- `run_obz_ztier_vectors(cfg) -> Report`
- `run_obz_legacy_vectors(cfg) -> Report`

## Feature flags

| Feature | Enables                                          |
|---------|--------------------------------------------------|
| `aags`  | a-tier `aags` scheme suites                      |
| `aasv`  | a-tier `aasv` scheme suites                      |
| `apgs`  | a-tier `apgs` scheme suites                      |
| `apsv`  | a-tier `apsv` scheme suites                      |
| `upbc`  | u-tier `upbc` scheme suites                      |
| `ztier` | z-tier (`zrbcx`) + legacy suites; needs `obz`    |

The default feature set enables all of the above. Vectors
for disabled schemes are reported as `Skipped`, not
counted as failures.

## Caveats

### Legacy: trailing `=` stripped on decode

The legacy scheme has a known protocol quirk: `dec` strips
trailing `=` characters from the decoded plaintext. The
`obz-legacy-vectors` suite mirrors this behavior in its
expected-value computation. Avoid round-trip tests with the
legacy scheme on inputs that end with `=`.

### Empty plaintext is rejected

Both `ob enc` and `obz enc` reject empty-string plaintext
with a non-zero exit code (`Error: enc failed: empty
plaintext`). The smoke suites assert **failure** for that
input rather than success.

## License

MIT (see the `license` field in `Cargo.toml`).
