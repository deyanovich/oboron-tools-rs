# obu-cli

Command-line interface for the oboron **obu** layer — the
unauthenticated schemes `upcbc` and `zdcbc`. Produces the `obu`
binary.

> ⚠️ **obu is NOT authenticated** — it is not cryptographically
> secure. `upcbc` gives confidentiality without integrity; `zdcbc`
> is obfuscation only. **Never use `obu` for sensitive data.** For
> anything security-critical, use `ob`, the authenticated oboron
> core CLI. obu shares no code with the secure core.

`obu` mirrors the core `ob` CLI, with a 256-bit **secret**
(`--secret` / `OBORON_SECRET`, 64 hex characters) in place of the
512-bit key, its own `~/.obu/` config directory, and the obu
schemes.

## Commands

- `obu enc [TEXT]` / `obu dec [TEXT]` — encode / decode. Secret via
  `--secret`, `--profile`, or `--keyless`; scheme via `-u`/`-z` or
  `--format`; the built-in default scheme is `upcbc`.
- `obu secretgen` — print a fresh random 64-hex secret.
- `obu init` / `obu config` / `obu profile` / `obu secret` — profile
  and configuration management (an implementation convenience, not
  required by the spec).
- `obu completion <shell>` — shell completion scripts.

## License

Licensed under either of Apache License, Version 2.0 or the MIT
license, at your option.
