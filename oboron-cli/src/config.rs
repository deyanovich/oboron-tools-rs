//! oboron-cli-specific config helpers, layered on top of `oboron-cli-core`.
//!
//! The command-handler implementations (`init`, `profile *`, etc.) live in
//! [`oboron_cli_core::commands`] and are shared with `obcrypt`. This module
//! pins the oboron-specific defaults via [`CLI_INFO`] and keeps the bits
//! that don't generalize across binaries: the `public_profile` mode of
//! `config show` (oboron uses a hardcoded test key, obcrypt has no such
//! concept) and the thin `Result<Config>` wrapper around the core's
//! `Result<Option<Config>>` shape.

use anyhow::{anyhow, Result};
pub use oboron_cli_core::{Config, KeyProfile};
use oboron_cli_core::commands::CliInfo;

const CLI_INFO: CliInfo<'static> = CliInfo {
    binary_name: "ob",
    default_scheme: "aasv",
    default_encoding: Some("c32"),
};

// ---------------------------------------------------------------------------
// Thin wrappers used by main.rs
// ---------------------------------------------------------------------------

/// Load config.json. Errors if it doesn't exist (matches the
/// pre-refactor behavior; callers use `.ok()` to opt into a missing
/// config).
pub fn load_config() -> Result<Config> {
    oboron_cli_core::load_config()?.ok_or_else(|| {
        anyhow!(
            "config not found at {}\nHint: run 'ob init' to create one",
            oboron_cli_core::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "~/.oboron/config.json".into())
        )
    })
}

pub fn save_config(cfg: &Config) -> Result<()> {
    oboron_cli_core::save_config(cfg)
}

pub fn load_profile(name: &str) -> Result<KeyProfile> {
    oboron_cli_core::load_profile(name)
}

// ---------------------------------------------------------------------------
// Command handlers — delegate to oboron_cli_core::commands.
// ---------------------------------------------------------------------------

pub fn init_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::init_command(&CLI_INFO, name, || oboron::generate_key())
}

/// `ob config show [--keyless]`. The `--keyless` mode prints oboron's
/// hardcoded test key and bypasses the normal config display, so it
/// stays here rather than in core.
pub fn config_show_command(public_profile: bool) -> Result<()> {
    if public_profile {
        println!("Using public profile (INSECURE - testing only):");
        println!("Key: {}", oboron::HARDCODED_KEY_HEX);
        return Ok(());
    }
    oboron_cli_core::commands::config_show_command(&CLI_INFO)
}

pub fn profile_list_command() -> Result<()> {
    oboron_cli_core::commands::profile_list_command(&CLI_INFO)
}

pub fn profile_show_command(name: Option<&str>) -> Result<()> {
    oboron_cli_core::commands::profile_show_command(&CLI_INFO, name)
}

pub fn profile_activate_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_activate_command(&CLI_INFO, name)
}

pub fn profile_create_command(name: &str, key: Option<&str>) -> Result<()> {
    oboron_cli_core::commands::profile_create_command(name, key, || oboron::generate_key())
}

pub fn profile_delete_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_delete_command(&CLI_INFO, name)
}

pub fn profile_rename_command(old_name: &str, new_name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_rename_command(old_name, new_name)
}

pub fn profile_set_command(name: &str, key: Option<&str>) -> Result<()> {
    let key = key.ok_or_else(|| anyhow!("--key must be provided"))?;
    oboron_cli_core::commands::profile_set_command(name, key)
}
