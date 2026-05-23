//! obcrypt-cli-specific config helpers, layered on top of `oboron-cli-core`.
//!
//! All non-trivial logic lives in [`oboron_cli_core::commands`] and is
//! shared with the `ob` (oboron) binary. This module pins the obcrypt
//! defaults via [`CLI_INFO`], re-exports the storage primitives main.rs
//! uses, and adapts a couple of signatures to what `obcrypt`'s clap
//! surface expects.

use anyhow::Result;
pub use oboron_cli_core::load_config;
use oboron_cli_core::commands::CliInfo;

const CLI_INFO: CliInfo<'static> = CliInfo {
    binary_name: "obcrypt",
    default_scheme: "aasv",
    // obcrypt has no encoding layer — payloads are raw bytes.
    default_encoding: None,
};

// ---------------------------------------------------------------------------
// Command handlers — thin wrappers around oboron_cli_core::commands.
// ---------------------------------------------------------------------------

pub fn init_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::init_command(&CLI_INFO, name, || {
        obcrypt::generate_key().to_hex()
    })
}

pub fn config_show_command() -> Result<()> {
    oboron_cli_core::commands::config_show_command(&CLI_INFO)
}

pub fn config_set_command(scheme: Option<String>, profile: Option<String>) -> Result<()> {
    oboron_cli_core::commands::config_set_command(&CLI_INFO, scheme, None, profile)
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
    oboron_cli_core::commands::profile_create_command(name, key, || {
        obcrypt::generate_key().to_hex()
    })
}

pub fn profile_delete_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_delete_command(&CLI_INFO, name)
}

pub fn profile_rename_command(old_name: &str, new_name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_rename_command(old_name, new_name)
}

pub fn profile_set_command(name: &str, key: &str) -> Result<()> {
    oboron_cli_core::commands::profile_set_command(name, key)
}

pub fn key_command(profile: Option<&str>) -> Result<()> {
    oboron_cli_core::commands::key_command(&CLI_INFO, profile)
}
