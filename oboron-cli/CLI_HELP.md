# Oboron CLI — Quick Reference

Reversible hash-like references (secure schemes)

## Usage

```
ob <COMMAND>
```

---

## `enc` (alias: `e`)

Encrypt+encode a plaintext string.

```
ob enc [OPTIONS] [TEXT]
```

| Flag / Option | Short | Description |
|---|---|---|
| `--key <KEY>` | `-k` | Encryption key (86 base64 chars); conflicts with `--profile`/`--keyless` |
| `--profile <NAME>` | `-p` | Use named key profile; conflicts with `--key`/`--keyless` |
| `--keyless` | `-K` | Use hardcoded key (INSECURE — testing only); conflicts with `--key`/`--profile` |
| `--format <FORMAT>` | `-f` | Format string, e.g. `aasv.b64`; cannot combine with scheme/encoding flags |
| `--aasv` | `-s` | Use aasv scheme (deterministic AES-SIV) |
| `--apsv` | `-S` | Use apsv scheme (probabilistic AES-SIV) |
| `--aags` | `-g` | Use aags scheme (deterministic AES-GCM-SIV) |
| `--apgs` | `-G` | Use apgs scheme (probabilistic AES-GCM-SIV) |
| `--upbc` | `-u` | Use upbc scheme (probabilistic AES-CBC, unauthenticated) |
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
| `--key <KEY>` | `-k` | Encryption key (86 base64 chars); conflicts with `--profile`/`--keyless` |
| `--profile <NAME>` | `-p` | Use named key profile; conflicts with `--key`/`--keyless` |
| `--keyless` | `-K` | Use hardcoded key (INSECURE — testing only); conflicts with `--key`/`--profile` |
| `--format <FORMAT>` | `-f` | Format string, e.g. `aasv.b64`; cannot combine with scheme/encoding flags |
| `--aasv` | `-s` | Use aasv scheme |
| `--apsv` | `-S` | Use apsv scheme |
| `--aags` | `-g` | Use aags scheme |
| `--apgs` | `-G` | Use apgs scheme |
| `--upbc` | `-u` | Use upbc scheme |
| `--c32` | `-c` | Use Crockford base32 encoding |
| `--b32` | `-b` | Use RFC base32 encoding |
| `--b64` | `-B` | Use base64 encoding |
| `--hex` | `-x` | Use hex encoding |
| `--help` | `-h` | Print help |

If `[TEXT]` is omitted, input is read from stdin.  When no scheme flag is given, the scheme is
auto-detected from the obtext payload.

---

## `init` (alias: `i`)

Initialize configuration with a randomly-generated key profile.

```
ob init [NAME]
```

| Argument | Description |
|---|---|
| `[NAME]` | Profile name (default: `default`) |

Creates `~/.ob/config.json` and `~/.ob/profiles/<NAME>.json`.  Backs up any existing profile
to `~/.ob/bkp/` before overwriting.

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
| `--aasv` | `-s` | Set default scheme to aasv |
| `--apsv` | `-S` | Set default scheme to apsv |
| `--aags` | `-g` | Set default scheme to aags |
| `--apgs` | `-G` | Set default scheme to apgs |
| `--upbc` | `-u` | Set default scheme to upbc |
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
| `--key <KEY>` | `-k` | Encryption key (86 base64 chars); generated if omitted |
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
| `--key <KEY>` | `-k` | Encryption key (86 base64 chars); generated if omitted |
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
| `--hex` | `-x` | Output key as hex instead of base64 |
| `--help` | `-h` | Print help |

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

## `obz` — Z-tier obfuscation tool

> ⚠️ **NOT SECURE** — for obfuscation only, never for sensitive data.

`obz` mirrors `ob` with these differences:

- Uses "secret" (`--secret`/`-s`, 43 base64 chars) instead of "key" (`--key`/`-k`)
- Config stored in `~/.obz/` instead of `~/.ob/`
- Default scheme is `zrbcx` instead of `aasv`
- Scheme flags: `--zrbcx`/`-r`, `--legacy`/`-l`
- Encoding flags: `--c32`/`-c`, `--b32`/`-b`, `--b64`/`-B`, `--hex`/`-x`
- `secret` command (alias: `s`) instead of `key`/`k`

All subcommands (`enc`/`e`, `dec`/`d`, `init`/`i`, `config`/`c`, `profile`/`p`,
`completion`) accept the same flags as their `ob` counterparts.

**Short-alias convenience examples:**

`ob enc/dec`:
```
# Instead of: ob enc --aasv --b32 'abc'
ob e -sb 'abc'

# Instead of: ob enc --aasv --b64 'abc'
ob e -sB 'abc'

# Instead of: ob enc --aasv --c32 'abc'
ob e -sc 'abc'
```

`obz enc/dec`:
```
# Instead of: obz enc --zrbcx --b32 'abc'
obz e -rb 'abc'

# Instead of: obz enc --zrbcx --b64 'abc'
obz e -rB 'abc'

# Instead of: obz enc --zrbcx --c32 'abc'
obz e -rc 'abc'
```

---

## Environment Variables

Both CLIs support environment variables for key/secret resolution.

| Variable | CLI | Description |
|---|---|---|
| `$OBORON_KEY` | `ob` | 86-character base64url-nopad encryption key (512-bit) |
| `$OBORON_SECRET` | `obz` | 43-character base64url-nopad obfuscation secret (256-bit) |

**Precedence order (highest to lowest):**

| Priority | Source |
|---|---|
| 1 | `--key` / `--secret` CLI flag |
| 2 | `$OBORON_KEY` / `$OBORON_SECRET` env var |
| 3 | `--profile <NAME>` → profile file |
| 4 | Default profile from config file |

When `$OBORON_KEY` (or `$OBORON_SECRET`) is set and the format is explicitly given,
the CLI works without any config or profile on disk.

