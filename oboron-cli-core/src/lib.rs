//! Shared CLI plumbing for the oboron-protocol CLIs (`ob` and `obc`).
//!
//! Both binaries share a config directory at `~/.oboron/`:
//!
//! ```text
//! ~/.oboron/
//! ├── config.json            # active profile + per-binary defaults
//! ├── profiles/<name>.json   # per-profile key + metadata
//! └── bkp/<name>-<ts>.json   # automatic backups on overwrite/delete
//! ```
//!
//! This crate provides:
//!
//! - **Path resolution** — `config_path`, `profile_dir`, `profile_path`, `backup_dir`.
//! - **Name validation** — `validate_profile_name`.
//! - **Key normalization** — `normalize_key_to_hex` accepts the canonical
//!   128-char hex form *or* the legacy 86-char base64 form (during the
//!   base64 deprecation period) and returns canonical hex.
//! - **Config / profile I/O** — `load_config`, `save_config`, `load_profile`,
//!   `save_profile`, `list_profiles`, `delete_profile`, `rename_profile`.
//!   File writes preserve unknown JSON fields so the two binaries don't
//!   clobber each other's metadata.
//! - **Backups** — `backup_profile` saves a timestamped copy before
//!   overwrite/delete.
//! - **Command handlers** — `commands::*` implements the user-facing
//!   `init` / `config` / `profile` / `key` subcommands shared by both
//!   binaries, parameterized over a [`commands::CliInfo`] supplying the
//!   per-binary defaults and the binary name used in error hints.
//! - **Legacy-dir migration** — `migration::ensure_config_root_migrated`
//!   moves a leftover `~/.ob/` to `~/.oboron/` on first run of the
//!   current tooling, leaving a symlink so any older binary still
//!   on the system keeps working against the same data.

pub mod commands;
pub mod config;
pub mod key;
pub mod migration;
pub mod paths;
pub mod profile;

pub use config::{load_config, save_config, Config};
pub use key::{normalize_key_classify, normalize_key_to_hex, KeyFormat};
pub use paths::{backup_dir, config_path, config_root, profile_dir, profile_path};
pub use profile::{
    delete_profile, list_profiles, load_profile, load_profile_key, load_profile_key_as_hex,
    rename_profile, save_profile, validate_profile_name, KeyProfile, LoadedKey,
};
