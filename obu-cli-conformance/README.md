# obu-cli-conformance

Cross-implementation conformance suite for the **obu** CLI — the
`obu` binary of the oboron family's unauthenticated / obfuscation
layer (schemes `upcbc` and `zdcbc`).

> **The obu layer is not authenticated and provides no
> confidentiality guarantee against an active attacker.** This suite
> validates wire-format and round-trip conformance only; it makes no
> security claim. See the obu specification for details.

## Why a separate crate

The insecure obu layer is segregated from the secure core
throughout the project: its own `obu` library, its own `obu`
binary, its own `~/.obu/` config dir — sharing no code with the
authenticated path. Its conformance is segregated the same way:
this crate is independent of the authenticated
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
suite, which certifies the secure `ob` / `obcrypt` surface. An
implementer of the secure protocol never has to touch obu, and an
implementer of obu never pulls in the secure suite.

## Usage

Validate an `obu` implementation in any language by pointing the
binary at it:

```bash
cargo install obu-cli-conformance
obu-cli-conformance --obu ./my-obu
```

If `obu` is already on `$PATH`, no arguments are needed:

```bash
obu-cli-conformance
```

The canonical obu test vectors are embedded at build time, so the
installed binary is self-contained. Each vector binds to the fixed
public test secret (obu spec §6.4), applied via `obu --keyless`.

| Scheme  | Test                                                   |
|:--------|:-------------------------------------------------------|
| `zdcbc` | Exact-match enc and dec (deterministic).               |
| `upcbc` | Dec the stored obtext, then enc → dec roundtrip.       |

## Build / test

```bash
cargo build -p obu-cli-conformance
cargo test  -p obu-cli-conformance   # needs `obu` on $PATH
```

The vectors live in a git submodule
(`tests/vectors` → oboron-test-vectors); clone with
`git submodule update --init` before building from source.

## License

Licensed under either of Apache-2.0
([LICENSE-APACHE](LICENSE-APACHE)) or MIT
([LICENSE-MIT](LICENSE-MIT)) at your option.
