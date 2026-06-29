# Oboron CLI — Quick Reference

Reversible hash-like references (authenticated schemes: `dsiv`, `psiv`, `dgcmsiv`, `pgcmsiv`)

## Usage

```
ob [GLOBAL OPTIONS] <COMMAND>
```

Global options (valid before or after the command name):

| Flag / Option | Short | Description |
|---|---|---|
| `--version` | `-V` | Print version provenance and exit |
| `--help` | `-h` | Print help |

`ob --version` prints a single provenance line, e.g.
`ob oboron-tools-rs 1.0.0 protocol=1.0 cli=1.0`.

---

## `enc` (alias: `e`)

Encrypt+encode a plaintext string.

```
ob enc [OPTIONS] [TEXT]
```

| Flag / Option | Short | Description |
|---|---|---|
| `--key <KEY>` | `-k` | Encryption key (128 hex chars); conflicts with `--profile`/`--keyless` |
| `--profile <NAME>` | `-p` | Use named key profile; conflicts with `--key`/`--keyless` |
| `--keyless` | `-K` | Use hardcoded key (INSECURE — testing only); conflicts with `--key`/`--profile` |
| `--format <FORMAT>` | `-f` | Format string, e.g. `dsiv.b64`; cannot combine with scheme/encoding flags |
| `--raw` | `-0` | Disable line framing (no stdin newline strip, no stdout newline) |
| `--dsiv` | `-s` | Use dsiv scheme (deterministic AES-SIV) |
| `--psiv` | `-S` | Use psiv scheme (probabilistic AES-SIV) |
| `--dgcmsiv` | `-g` | Use dgcmsiv scheme (deterministic AES-GCM-SIV) |
| `--pgcmsiv` | `-G` | Use pgcmsiv scheme (probabilistic AES-GCM-SIV) |
| `--c32` | `-c` | Use Crockford base32 encoding |
| `--b32` | `-b` | Use RFC base32 encoding |
| `--b64` | `-B` | Use base64 encoding |
| `--hex` | `-x` | Use hex encoding |
| `--help` | `-h` | Print help |

If `[TEXT]` is omitted, input is read from stdin.

---

## `dec` (alias: `d`)

Decode+decrypt an obtext string.

```
ob dec [OPTIONS] [TEXT]
```

| Flag / Option | Short | Description |
|---|---|---|
| `--key <KEY>` | `-k` | Encryption key (128 hex chars); conflicts with `--profile`/`--keyless` |
| `--profile <NAME>` | `-p` | Use named key profile; conflicts with `--key`/`--keyless` |
| `--keyless` | `-K` | Use hardcoded key (INSECURE — testing only); conflicts with `--key`/`--profile` |
| `--format <FORMAT>` | `-f` | Format string, e.g. `dsiv.b64`; cannot combine with scheme/encoding flags |
| `--raw` | `-0` | Disable line framing (no stdin newline strip, no stdout newline) |
| `--dsiv` | `-s` | Use dsiv scheme |
| `--psiv` | `-S` | Use psiv scheme |
| `--dgcmsiv` | `-g` | Use dgcmsiv scheme |
| `--pgcmsiv` | `-G` | Use pgcmsiv scheme |
| `--c32` | `-c` | Use Crockford base32 encoding |
| `--b32` | `-b` | Use RFC base32 encoding |
| `--b64` | `-B` | Use base64 encoding |
| `--hex` | `-x` | Use hex encoding |
| `--help` | `-h` | Print help |

If `[TEXT]` is omitted, input is read from stdin.  When no scheme flag is given, the default
scheme (`dsiv`) is used; the scheme is not auto-detected from the obtext.

---

## `init` (alias: `i`)

Initialize configuration with a randomly-generated key profile.

```
ob init [NAME]
```

| Argument | Description |
|---|---|
| `[NAME]` | Profile name (default: `default`) |

Creates `~/.oboron/config.json` and
`~/.oboron/profiles/<NAME>.json`.  Backs up any existing profile
to `~/.oboron/bkp/` before overwriting.

---

## `config` (alias: `c`)

Manage configuration.

```
ob config [OPTIONS] [SUBCOMMAND]
```

| Option | Short | Description |
|---|---|---|
| `--keyless` | `-K` | Use hardcoded key (INSECURE — testing only) |
| `--help` | `-h` | Print help |

Subcommands:

| Subcommand | Description |
|---|---|
| `show` | Print current configuration (default when no subcommand given) |
| `set` | Set configuration values |

### `config set`

```
ob config set [OPTIONS]
```

| Flag / Option | Short | Description |
|---|---|---|
| `--dsiv` | `-s` | Set default scheme to dsiv |
| `--psiv` | `-S` | Set default scheme to psiv |
| `--dgcmsiv` | `-g` | Set default scheme to dgcmsiv |
| `--pgcmsiv` | `-G` | Set default scheme to pgcmsiv |
| `--c32` | `-c` | Set default encoding to c32 |
| `--b32` | `-b` | Set default encoding to b32 |
| `--b64` | `-B` | Set default encoding to b64 |
| `--hex` | `-x` | Set default encoding to hex |
| `--profile <NAME>` | `-p` | Set default key profile |
| `--help` | `-h` | Print help |

---

## `profile` (alias: `p`)

Manage key profiles.

```
ob profile <SUBCOMMAND>
```

### `profile list` (alias: `l`)

List all key profiles.

```
ob profile list
```

### `profile show [NAME]` (aliases: `g`, `get`)

Show a specific key profile.  Defaults to the active profile if `[NAME]` is omitted.

```
ob profile show [NAME]
ob profile g [NAME]
ob profile get [NAME]
```

### `profile activate <NAME>` (aliases: `a`, `use`)

Set a profile as the active (default) profile.

```
ob profile activate <NAME>
ob profile a <NAME>
ob profile use <NAME>
```

### `profile create <NAME>` (alias: `c`)

Create a new key profile.

```
ob profile create [OPTIONS] <NAME>
ob profile c [OPTIONS] <NAME>
```

| Option | Short | Description |
|---|---|---|
| `--key <KEY>` | `-k` | Encryption key (128 hex chars); generated if omitted |
| `--help` | `-h` | Print help |

### `profile delete <NAME>` (alias: `d`)

Delete a key profile.

```
ob profile delete <NAME>
ob profile d <NAME>
```

### `profile rename <OLD> <NEW>` (aliases: `r`, `mv`)

Rename a key profile.

```
ob profile rename <OLD_NAME> <NEW_NAME>
ob profile r <OLD_NAME> <NEW_NAME>
ob profile mv <OLD_NAME> <NEW_NAME>
```

### `profile set <NAME>`

Set (replace) the key for an existing profile.

```
ob profile set [OPTIONS] <NAME>
```

| Option | Short | Description |
|---|---|---|
| `--key <KEY>` | `-k` | Encryption key (128 hex chars); generated if omitted |
| `--help` | `-h` | Print help |

---

## `key` (alias: `k`)

Output the encryption key for the active or specified profile.

```
ob key [OPTIONS]
```

| Option | Short | Description |
|---|---|---|
| `--profile <NAME>` | `-p` | Use named key profile |
| `--keyless` | `-K` | Output the hardcoded key (INSECURE — testing only) |
| `--help` | `-h` | Print help |

---

## `keygen`

Generate a fresh random key and print it to stdout.  Touches no
profile or config and needs no key source.  Has **no** alias
(unlike `key`, whose alias is `k`).

```
ob keygen
```

Prints a fresh canonical 128-character hex key.

---

## `completion`

Generate shell completion script.

```
ob completion <SHELL>
```

| Subcommand | Description |
|---|---|
| `bash` | Generate bash completion script |
| `zsh` | Generate zsh completion script |
| `fish` | Generate fish completion script |
| `powershell` | Generate PowerShell completion script |

---

## Environment Variables

The CLI supports an environment variable for key resolution.

| Variable | CLI | Description |
|---|---|---|
| `$OBORON_KEY` | `ob` | 128-character lowercase-hex encryption key (512-bit) |

**Precedence order (highest to lowest):**

| Priority | Source |
|---|---|
| 1 | `--keyless` flag (fixed public test key — INSECURE) |
| 2 | `--key` CLI flag |
| 3 | `$OBORON_KEY` env var |
| 4 | `--profile <NAME>` → profile file |
| 5 | Default profile from config file |

When `$OBORON_KEY` is set and the format is explicitly given,
the CLI works without any config or profile on disk.

