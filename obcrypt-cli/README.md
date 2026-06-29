# obcrypt-cli

Command-line interface for [`obcrypt`](https://crates.io/crates/obcrypt) —
the bytes-in / bytes-out cryptographic core of the
[oboron](https://oboron.org/) protocol.

The binary is named `obcrypt` (parallels `oboron-cli`'s `ob`).

## Install

```bash
cargo install --path obcrypt-cli
# or, from a release:
cargo install obcrypt-cli
```

## Usage

```text
obcrypt <SUBCOMMAND>

Subcommands:
  encrypt (e)   Encrypt plaintext bytes under a scheme
  decrypt (d)   Decrypt ciphertext bytes (scheme required)
  keygen  (k)   Generate a fresh random 128-character hex key
  init    (i)   Initialize configuration with a fresh profile
  config  (c)   Show or update configuration
  profile (p)   Manage key profiles
  key           Print the active profile's key
  completions   Generate shell completion script
```

### Quick start

```bash
# 1. Generate a key (128-char hex)
$ obcrypt keygen
50947ce0edfc65f791543ad169590a877a52ca591e09…

# 2. Encrypt under dsiv (raw ciphertext bytes — pipe to a file)
$ KEY=$(obcrypt keygen)
$ obcrypt encrypt --scheme dsiv --key "$KEY" "hello world" > ct.bin

# 2'. Or get terminal-friendly hex output with -x
$ obcrypt encrypt -x -s dsiv -k "$KEY" "hello world"
ab567ee2b3fcb75d9fcd40e44d66cf0e46653e557338ca98ffa0c3aab8

# 3. Decrypt — the scheme must be supplied (no marker, no auto-detection)
$ obcrypt decrypt -X -s dsiv -k "$KEY" ab567ee2b3fc…aab8
hello world

# 3'. Or pipe the raw bytes back in
$ obcrypt decrypt -s dsiv -k "$KEY" < ct.bin
hello world
```

### Subcommand details

#### `encrypt` / `e`

```text
obcrypt encrypt [OPTIONS] [TEXT]

Args:
  [TEXT]    Plaintext (reads stdin if absent).

Options:
  -s, --scheme <SCHEME>    dsiv | dgcmsiv | psiv | pgcmsiv
                           Required if no default is set in config.
  -k, --key <KEY>          128 hex chars.
  -p, --profile <NAME>     Use ~/.oboron/profiles/<NAME>.json.
  -x, --hex                Hex-encode the ciphertext on output. Without
                           this flag the ciphertext is raw bytes.
  -X, --hex-in             Decode the plaintext input as hex first
                           (convenient for binary plaintext you have as
                           a hex string).
      --in-file <PATH>     Read plaintext from file (instead of TEXT/stdin).
      --out-file <PATH>    Write ciphertext to file (instead of stdout).
```

By default plaintext input and ciphertext output are both **raw
bytes** — `obcrypt` is encoding-agnostic. The `-x`/`-X` flags are
purely about terminal I/O: terminals can't reliably print arbitrary
binary, and pasting hex is sometimes easier than piping bytes.

#### `decrypt` / `d`

```text
obcrypt decrypt [OPTIONS] [TEXT]

Same option set as `encrypt`, but:

  -s, --scheme <SCHEME>    REQUIRED on decrypt (unless a default is set
                           in config). obcrypt ciphertext carries no
                           scheme marker, so the scheme cannot be
                           auto-detected — it must be supplied.

  -x, --hex                Hex-encode the plaintext on output (terminal-
                           safe display when the plaintext is binary).
  -X, --hex-in             Decode the ciphertext input as hex first
                           (round-trips with `encrypt -x`).
```

The `-x`/`-X` flags are parallel across both subcommands: `-x`
governs **output** byte format, `-X` governs **input** byte format.

#### `keygen` / `k`

Prints a fresh 128-character hex key sourced from the OS RNG. No
options — keys are bytes, no scheme or other metadata to attach.

```bash
$ obcrypt keygen > my.key
$ obcrypt encrypt -s dsiv -k "$(cat my.key)" hello
```

#### `init` / `i`, `config` / `c`, `profile` / `p`, `key`

Profile / config management for `~/.oboron/`, shared with
[`ob`](https://crates.io/crates/oboron-cli):

```bash
$ obcrypt init                       # create the default profile
$ obcrypt profile create work        # add a second profile
$ obcrypt profile activate work      # switch active profile
$ obcrypt config show                # print active config + key
$ obcrypt config set --scheme psiv   # change the default scheme
$ obcrypt key                        # print the active profile's key
```

#### `completions`

```bash
$ obcrypt completions bash > /etc/bash_completion.d/obcrypt
$ obcrypt completions zsh  > "${fpath[1]}/_obcrypt"
$ obcrypt completions fish > ~/.config/fish/completions/obcrypt.fish
```

## Key sources

In priority order:

1. **`--key <KEY>`** — direct (128 hex chars).
2. **`--profile <NAME>`** — read `~/.oboron/profiles/<NAME>.json`.
3. **Active profile from `~/.oboron/config.json`** — if neither
   `--key` nor `--profile` was given, the active profile name is
   looked up in the global config and its key is loaded.

If none of those resolve, `obcrypt` errors with a hint pointing at
`obcrypt init`.

## Shared config dir

`obcrypt` and [`ob`](https://crates.io/crates/oboron-cli) share
`~/.oboron/`:

```text
~/.oboron/
├── config.json            # global config — active profile, default scheme
└── profiles/
    ├── default.json       # { "key": "<128-hex-chars>" }
    └── otherproject.json
```

Both binaries can read and write this directory; writes preserve any
fields they don't recognize so the two CLIs don't clobber each
other's settings (e.g. `ob`'s `encoding` field is left intact when
`obcrypt config set` updates the scheme). Keys are 128-char hex.

## Differences from `oboron-cli`'s `ob`

| | `obcrypt` (obcrypt-cli) | `ob` (oboron-cli) |
|---|---|---|
| Output | raw bytes (`-x` for hex display) | obtext string |
| Encoding | none — protocol-level encoding is `ob`'s job | protocol-level (`c32`/`b32`/`b64`/`hex`) |
| UTF-8 validation | none | yes |
| Schemes | the authenticated core schemes | the authenticated core schemes |
| Subcommands | `encrypt`/`decrypt` (full names) | `enc`/`dec` (short names) |
| Aliases | `e` / `d` / `k` | `e` / `d` / `k` |
| Keys | hex (128 chars) | hex (128 chars) |

Use `obcrypt` for binary contexts, embedded use, low-level
integration, or when you don't want the obtext encoding layer. Use
`ob` for text contexts (identifiers, URLs, copy-paste-able strings).

## Conformance

The `obcrypt` binary's encrypt / decrypt behavior is validated
end-to-end against the canonical oboron test vectors by
[`oboron-cli-conformance`](https://crates.io/crates/oboron-cli-conformance)
1.0.0 — the same cross-implementation harness used to qualify
alternative-language implementations of the protocol.

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
