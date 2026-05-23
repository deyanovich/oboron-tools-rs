# Oboron CLI

[![Crates.io](https://img.shields.io/crates/v/oboron-cli.svg)](https://crates.io/crates/oboron-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.77-blue.svg)](https://blog.rust-lang.org/2023/11/16/Rust-1.77.0.html)
[![oboron](https://img.shields.io/crates/v/oboron?label=oboron)](https://crates.io/crates/oboron)
[![oboron-py](https://img.shields.io/crates/v/oboron-py?label=oboron-py)](https://crates.io/crates/oboron-py)

CLI for [Oboron](https://crates.io/crates/oboron) — general-purpose symmetric encryption and
encoding.  Provides two binaries:
- **`ob`** — Secure encryption CLI (a-tier and u-tier schemes: `aasv`, `aags`, `apsv`, `apgs`,
  `upbc`)
- **`obz`** — Z-tier obfuscation CLI (non-secure; requires the `ztier` feature, included in
  the default `all-schemes` feature)

## Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Environment Variables](#environment-variables)
- [Commands Reference](#commands-reference)
  - [ob enc / ob e](#ob-enc--ob-e)
  - [ob dec / ob d](#ob-dec--ob-d)
  - [ob init / ob i](#ob-init--ob-i)
  - [ob config / ob c](#ob-config--ob-c)
  - [ob profile / ob p](#ob-profile--ob-p)
  - [ob key / ob k](#ob-key--ob-k)
  - [ob completion](#ob-completion)
- [The `obz` Binary](#the-obz-binary)
- [Profile Management](#profile-management)
- [Feature Flags](#feature-flags)
- [Shell Completions](#shell-completions)
- [Schemes Reference](#schemes-reference)
- [Encodings Reference](#encodings-reference)
- [Related Crates](#related-crates)
- [License](#license)

## Installation

Install with all schemes enabled (default):
```shell
cargo install oboron-cli
```

Install with secure schemes only (no z-tier / no `obz` binary):
```shell
cargo install oboron-cli --no-default-features --features secure-schemes
```

Install with a single scheme (minimal binary):
```shell
cargo install oboron-cli --no-default-features --features aasv
```

> **Note:** The `obz` binary requires the `ztier` feature (included in the default
> `all-schemes`).  If you install without it, only the `ob` binary will be built.

## Quick Start

Initialize with a randomly-generated key profile:
```shell
ob init
```

Encrypt a string:
```shell
ob enc "hello, world"
```

Decrypt the obtext:
```shell
ob dec <obtext>
```

Pipe from stdin:
```shell
echo "hello" | ob enc
```

Encrypt with an explicit key:
```shell
ob enc -k <KEY> "hello, world"
```

Encrypt with the hardcoded/public key (testing only — not secure):
```shell
ob enc -K "hello, world"
```

Encrypt with a specific format:
```shell
ob enc -f aasv.b64 "hello, world"
```

## Environment Variables

Both CLIs support environment variables for key/secret resolution, enabling use without
`ob init` / `obz init` (e.g., in CI/CD or containerized environments).

| Variable        | CLI   | Description                                               |
|-----------------|-------|-----------------------------------------------------------|
| `OBORON_KEY`    | `ob`  | 86-character base64url-nopad encryption key (512-bit)     |
| `OBORON_SECRET` | `obz` | 43-character base64url-nopad obfuscation secret (256-bit) |

**Precedence order (highest to lowest):**

1. `--key` / `--secret` CLI flag (explicit, one-shot)
2. `$OBORON_KEY` / `$OBORON_SECRET` env var
3. `--profile <NAME>` → profile file lookup
4. Default profile from `~/.ob/config.json` / `~/.obz/config.json`
5. Error with helpful message

**CI/CD example — no `ob init` required:**

```shell
export OBORON_KEY="$(ob key)"   # or inject from your secret store
ob enc --aasv --b32 "data"      # works without ob init
echo "data" | ob enc -sB        # piping also works
```

**Security note:** Environment variables are visible to child processes and in
`/proc/*/environ` on Linux. For ephemeral/CI contexts they are convenient; for persistent
workstation use, `ob init` with file-based profiles (written with `chmod 600`) is more secure.

## Commands Reference

### `ob enc` / `ob e`

Encrypt+encode a plaintext string.

```
USAGE:
    ob enc [OPTIONS] [TEXT]

ARGS:
    [TEXT]    Plaintext string (reads from stdin if not provided)

OPTIONS:
    -k, --key <KEY>         Encryption key (86 base64 chars)
    -p, --profile <NAME>    Use named key profile
    -K, --keyless           Use hardcoded key (INSECURE - testing only)
    -f, --format <FORMAT>   Format specification, e.g. "aasv.b64"
                            Cannot be combined with scheme or encoding flags
    -s, --aasv              Use aasv scheme (deterministic AES-SIV)
    -S, --apsv              Use apsv scheme (probabilistic AES-SIV)
    -g, --aags              Use aags scheme (deterministic AES-GCM-SIV)
    -G, --apgs              Use apgs scheme (probabilistic AES-GCM-SIV)
    -u, --upbc              Use upbc scheme (probabilistic unauthenticated AES-CBC)
    -c, --c32               Use Crockford base32 encoding
    -b, --b32               Use RFC base32 encoding
    -B, --b64               Use base64 encoding
    -x, --hex               Use hex encoding
    -h, --help              Print help
```

Flags `-k`/`--key`, `-p`/`--profile`, and `-K`/`--keyless` are mutually exclusive.
Flag `-f`/`--format` cannot be combined with individual scheme or encoding flags.

### `ob dec` / `ob d`

Decode+decrypt an obtext string.

```
USAGE:
    ob dec [OPTIONS] [TEXT]

ARGS:
    [TEXT]    Obtext string (reads from stdin if not provided)

OPTIONS:
    -k, --key <KEY>         Encryption key (86 base64 chars)
    -p, --profile <NAME>    Use named key profile
    -K, --keyless           Use hardcoded key (INSECURE - testing only)
    -f, --format <FORMAT>   Format specification, e.g. "aasv.b64"
    -s, --aasv              Use aasv scheme
    -S, --apsv              Use apsv scheme
    -g, --aags              Use aags scheme
    -G, --apgs              Use apgs scheme (probabilistic AES-GCM-SIV)
    -u, --upbc              Use upbc scheme
    -c, --c32               Use Crockford base32 encoding
    -b, --b32               Use RFC base32 encoding
    -B, --b64               Use base64 encoding
    -x, --hex               Use hex encoding
    -h, --help              Print help
```

When no scheme flag is given, `ob dec` uses auto-detection to determine the scheme from the
obtext payload.

### `ob init` / `ob i`

Initialize configuration with a randomly-generated key profile.

```
USAGE:
    ob init [NAME]

ARGS:
    [NAME]    Name for the key profile [default: default]

OPTIONS:
    -h, --help    Print help
```

Creates `~/.ob/config.json` and `~/.ob/profiles/<NAME>.json` with a fresh 512-bit key.  Safe
to re-run — existing profiles are backed up to `~/.ob/bkp/` before being overwritten.

### `ob config` / `ob c`

Manage configuration.

```
USAGE:
    ob config [OPTIONS] [COMMAND]

COMMANDS:
    show    Show current configuration (default when no subcommand given)
    set     Set configuration values

OPTIONS:
    -K, --keyless    Use hardcoded key (INSECURE - testing only)
    -h, --help       Print help
```

#### `ob config show`

Print the current configuration (profile, scheme, encoding).

#### `ob config set`

```
USAGE:
    ob config set [OPTIONS]

OPTIONS:
    -s, --aasv              Set default scheme to aasv
    -S, --apsv              Set default scheme to apsv
    -g, --aags              Set default scheme to aags
    -G, --apgs              Set default scheme to apgs
    -u, --upbc              Set default scheme to upbc
    -c, --c32               Set default encoding to c32
    -b, --b32               Set default encoding to b32
    -B, --b64               Set default encoding to b64
    -x, --hex               Set default encoding to hex
    -p, --profile <NAME>    Set default key profile
    -h, --help              Print help
```

### `ob profile` / `ob p`

Manage key profiles.

```
USAGE:
    ob profile <COMMAND>

COMMANDS:
    list     (alias: l)        List all key profiles
    show     (alias: g, get)   Show a specific key profile
    activate (alias: a, use)   Set a profile as the default
    create   (alias: c)        Create a new key profile
    delete   (alias: d)        Delete a key profile
    rename   (alias: r, mv)    Rename a key profile
    set                        Set the key for a profile
```

#### `ob profile list` / `ob p l`

List all available key profiles.

#### `ob profile show [NAME]` / `ob p g [NAME]`

Show details of a profile.  If `NAME` is omitted, the active (default) profile is shown.

#### `ob profile activate <NAME>` / `ob p a <NAME>` / `ob p use <NAME>`

Set `<NAME>` as the active (default) profile used by `ob enc`/`ob dec`.

#### `ob profile create <NAME> [-k KEY]` / `ob p c <NAME>`

Create a new profile named `<NAME>`.  If `--key`/`-k` is omitted, a fresh key is generated.

#### `ob profile delete <NAME>` / `ob p d <NAME>`

Delete a key profile.

#### `ob profile rename <OLD> <NEW>` / `ob p r <OLD> <NEW>` / `ob p mv <OLD> <NEW>`

Rename a profile.

#### `ob profile set <NAME> [-k KEY]`

Set (replace) the key stored in an existing profile.  If `--key`/`-k` is omitted, a fresh
key is generated.

### `ob key` / `ob k`

Output the encryption key for the active (or specified) profile.

```
USAGE:
    ob key [OPTIONS]

OPTIONS:
    -p, --profile <NAME>    Use named key profile
    -K, --keyless           Output the hardcoded key (INSECURE - testing only)
    -x, --hex               Output key as hex instead of base64
    -h, --help              Print help
```

### `ob completion`

Generate shell completion scripts.

```
USAGE:
    ob completion <SHELL>

SUBCOMMANDS:
    bash        Generate bash completion script
    zsh         Generate zsh completion script
    fish        Generate fish completion script
    powershell  Generate PowerShell completion script
```

See [Shell Completions](#shell-completions) for installation instructions.

## The `obz` Binary

`obz` mirrors `ob` but operates on z-tier obfuscation schemes (`zrbcx`, `zmock`, `legacy`).

> ⚠️ **WARNING: `obz` provides NO cryptographic security.**
> Use only for obfuscation (e.g., hiding sequential IDs in non-security contexts).
> Never use `obz` to protect sensitive data.

Key differences from `ob`:

| | `ob` | `obz` |
|---|---|---|
| Security | Cryptographically secure (AES-SIV, AES-GCM-SIV, AES-CBC) | Not secure |
| Terminology | "key" (86 base64 chars, 512-bit) | "secret" (43 base64 chars, 256-bit) |
| Config location | `~/.ob/` | `~/.obz/` |
| Default scheme | `aasv` | `zrbcx` |
| Feature flag | always available (a/u-tier) | requires `ztier` |

Available `obz` scheme flags:
- `-r`, `--zrbcx` — XOR-based obfuscation (deterministic)
- `-l`, `--legacy` — Base32-based legacy obfuscation (fixed encoding)

`obz` encoding short flags match `ob`: `-c`/`--c32`, `-b`/`--b32`, `-B`/`--b64`, `-x`/`--hex`.

Commands and subcommands are otherwise identical to `ob`, substituting `obz` for `ob`,
`secret` for `key`, and using `--secret`/`-s` instead of `--key`/`-k`.

**Short-alias convenience examples:**

`ob enc/dec`:
```shell
# Instead of: ob enc --aasv --b32 'abc'
ob e -sb 'abc'

# Instead of: ob enc --aasv --b64 'abc'
ob e -sB 'abc'

# Instead of: ob enc --aasv --c32 'abc'
ob e -sc 'abc'
```

`obz enc/dec`:
```shell
# Instead of: obz enc --zrbcx --b32 'abc'
obz e -rb 'abc'

# Instead of: obz enc --zrbcx --b64 'abc'
obz e -rB 'abc'

# Instead of: obz enc --zrbcx --c32 'abc'
obz e -rc 'abc'
```

Example:
```shell
obz init
obz enc "hello"
obz dec <obtext>
```

## Profile Management

Profiles store encryption keys locally, eliminating the need to pass keys on the command line.

**Directory layout (`ob`):**
```
~/.ob/
├── config.json          # active profile, default scheme and encoding
├── profiles/
│   ├── default.json     # default key profile
│   └── <name>.json      # additional profiles
└── bkp/                 # automatic backups before overwrite
```

**Typical workflow:**

```shell
# One-time setup
ob init                  # creates "default" profile with a random key

# Encrypt and decrypt using the active profile (no key flag needed)
ob enc "hello, world"
ob dec <obtext>
```

**Multi-profile workflow:**

```shell
ob profile create prod   # generates a new key for "prod"
ob profile activate prod # set "prod" as the active profile
ob enc "secret data"     # uses the "prod" key
```

**File permissions:** Profile files are written with `0o600` permissions on Unix systems
(owner-read/write only).

For deeper details on key management see the
[oboron library documentation](https://docs.rs/oboron).

## Feature Flags

Features control which encryption schemes are compiled in, reducing binary size.

**Default:** `all-schemes` (all schemes including z-tier)

### Individual schemes

| Feature  | Scheme  | Description |
|----------|---------|-------------|
| `aasv`   | `aasv`  | Deterministic AES-SIV (authenticated) |
| `aags`   | `aags`  | Deterministic AES-GCM-SIV (authenticated) |
| `apsv`   | `apsv`  | Probabilistic AES-SIV (authenticated) |
| `apgs`   | `apgs`  | Probabilistic AES-GCM-SIV (authenticated) |
| `upbc`   | `upbc`  | Probabilistic AES-CBC (unauthenticated) |
| `zrbcx`  | `zrbcx` | XOR-based obfuscation (z-tier, not secure) |
| `zmock`  | `zmock` | Mock z-tier scheme (testing) |
| `legacy` | `legacy`| Legacy base32 obfuscation (z-tier, not secure) |
| `mock`   | —       | Mock schemes for testing |

### Category features

| Feature                  | Includes |
|--------------------------|----------|
| `atier`                  | `aasv`, `aags`, `apsv`, `apgs` |
| `utier`                  | `upbc` |
| `ztier`                  | `zrbcx`, `zmock`, `legacy` (enables `obz` binary) |
| `secure-schemes`         | `atier` + `utier` |
| `authenticated-schemes`  | `atier` |
| `deterministic-schemes`  | `aasv`, `aags` |
| `probabilistic-schemes`  | `apsv`, `apgs`, `upbc` |
| `all-schemes`            | `atier` + `utier` + `ztier` *(default)* |

### Examples

```toml
# Cargo.toml — minimal single-scheme install
oboron-cli = { version = "0.1", default-features = false, features = ["aasv"] }

# Secure schemes only (no obz binary)
oboron-cli = { version = "0.1", default-features = false, features = ["secure-schemes"] }
```

Or via cargo install:
```shell
# Secure schemes only
cargo install oboron-cli --no-default-features --features secure-schemes

# Single scheme
cargo install oboron-cli --no-default-features --features aasv
```

## Shell Completions

Generate and install completion scripts for your shell.

### Bash

```shell
ob completion bash > ~/.local/share/bash-completion/completions/ob
```

### Zsh

```shell
ob completion zsh > "${fpath[1]}/_ob"
```

### Fish

```shell
ob completion fish > ~/.config/fish/completions/ob.fish
```

### PowerShell

```shell
ob completion powershell | Out-String | Invoke-Expression
```

To persist PowerShell completions, add the above line to your `$PROFILE`.

## Schemes Reference

For full details see the [`oboron` library on crates.io](https://crates.io/crates/oboron) or its [repository](https://gitlab.com/oboron/oboron-rs).

| Scheme    | Algorithm   | Deterministic? | Authenticated? | Notes                              |
|:----------|:------------|:---------------|:---------------|:-----------------------------------|
| `aasv`    | AES-SIV     | Yes            | Yes            | General purpose, deterministic     |
| `aags`    | AES-GCM-SIV | Yes            | Yes            | Deterministic alternative          |
| `apsv`    | AES-SIV     | No             | Yes            | Maximum privacy protection         |
| `apgs`    | AES-GCM-SIV | No             | Yes            | Probabilistic alternative          |
| `upbc`    | AES-CBC     | No             | No             | Unauthenticated — use with caution |
| `zrbcx`   | XOR         | Yes            | No             | Obfuscation only — not secure      |
| `legacy`  | Base32      | Yes            | No             | Legacy obfuscation — not secure    |

All `a`-tier and `u`-tier schemes use 256-bit AES encryption.  Z-tier schemes are not
cryptographically secure.

## Encodings Reference

| Encoding | Flag    | Description |
|----------|---------|-------------|
| `c32`    | `--c32` | Crockford base32 — lowercase, avoids accidental obscenity words |
| `b32`    | `--b32` | RFC 4648 base32 — uppercase alphanumeric |
| `b64`    | `--b64` | URL-safe base64 (RFC 4648 §5) — most compact, includes `-` and `_` |
| `hex`    | `--hex` / `-x` | Hexadecimal — longest output, slightly faster |

## Related Crates

- [`oboron`](https://crates.io/crates/oboron) — Core Rust library ([docs.rs](https://docs.rs/oboron))
- [`oboron-cli-core`](https://crates.io/crates/oboron-cli-core) — Shared CLI plumbing (consumed by this crate and `obcrypt-cli`)
- [`obcrypt-cli`](https://crates.io/crates/obcrypt-cli) — Sibling CLI for the obcrypt bytes-in/bytes-out subset (`obcrypt` binary)
- [`oboron-py`](https://crates.io/crates/oboron-py) — Python bindings ([PyPI](https://pypi.org/project/oboron/))

## Conformance

The `ob` and `obz` binaries' encrypt / decrypt behavior is
validated end-to-end against the canonical oboron test vectors
by
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
v0.2.0 — the same cross-implementation harness used to qualify
alternative-language implementations of the protocol.

## License

Licensed under the MIT license ([LICENSE](LICENSE)).
