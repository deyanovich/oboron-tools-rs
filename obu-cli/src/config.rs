//! obu-cli config helpers, layered on `oboron-cli-core`.
//!
//! The shared command handlers (`init`, `profile *`, …) live in
//! [`oboron_cli_core::commands`]; this module pins the obu defaults via
//! [`CLI_INFO`] and supplies obu's secret generator. obu installs its
//! own runtime environment ([`oboron_cli_core::CliEnv::OBU`] — the
//! `~/.obu/` directory and 64-hex secrets) from `main`, before any of
//! these handlers run.

use anyhow::{anyhow, Result};
use oboron_cli_core::commands::CliInfo;
pub use oboron_cli_core::Config;

/// The fixed public test secret (OBU-CLI spec §4). Public and INSECURE;
/// matches obu's `--keyless` constructors and the obu test vectors.
pub const KEYLESS_SECRET_HEX: &str =
    "381284633d02ea5f35df8596b5cc4218310060468e8b465455a415174ea6e966";

const CLI_INFO: CliInfo<'static> = CliInfo {
    binary_name: "obu",
    default_scheme: "upcbc",
    default_encoding: Some("c32"),
};

// ---------------------------------------------------------------------------
// Thin wrappers used by main.rs
// ---------------------------------------------------------------------------

pub fn load_config() -> Result<Config> {
    oboron_cli_core::load_config()?.ok_or_else(|| {
        anyhow!(
            "config not found at {}\nHint: run 'obu init' to create one",
            oboron_cli_core::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "~/.obu/config.json".into())
        )
    })
}

pub fn save_config(cfg: &Config) -> Result<()> {
    oboron_cli_core::save_config(cfg)
}

// ---------------------------------------------------------------------------
// Command handlers — delegate to oboron_cli_core::commands.
// ---------------------------------------------------------------------------

pub fn init_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::init_command(&CLI_INFO, name, obu::generate_secret)
}

/// `obu config show [--keyless]`. The `--keyless` mode prints the fixed
/// public test secret and bypasses the normal config display.
pub fn config_show_command(public_profile: bool) -> Result<()> {
    if public_profile {
        println!("Using public test secret (INSECURE - testing only):");
        println!("Secret: {KEYLESS_SECRET_HEX}");
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

pub fn profile_create_command(name: &str, secret: Option<&str>) -> Result<()> {
    oboron_cli_core::commands::profile_create_command(name, secret, obu::generate_secret)
}

pub fn profile_delete_command(name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_delete_command(&CLI_INFO, name)
}

pub fn profile_rename_command(old_name: &str, new_name: &str) -> Result<()> {
    oboron_cli_core::commands::profile_rename_command(old_name, new_name)
}

pub fn profile_set_command(name: &str, secret: Option<&str>) -> Result<()> {
    let secret = secret.ok_or_else(|| anyhow!("--secret must be provided"))?;
    oboron_cli_core::commands::profile_set_command(name, secret)
}
