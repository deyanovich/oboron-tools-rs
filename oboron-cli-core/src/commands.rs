//! Command-handler implementations shared by the oboron-protocol CLIs.
//!
//! Each handler is parameterized over a [`CliInfo`] supplying the
//! per-binary bits (name shown in user-facing hints, default scheme,
//! default encoding). Key generation, where needed, is passed in as
//! a closure so this crate stays free of any dependency on `oboron`
//! or `obcrypt`.

use anyhow::{anyhow, Context, Result};

use crate::config::{load_config, save_config, Config};
use crate::key::normalize_key_to_hex;
use crate::paths::{config_path, profile_path};
use crate::profile::{
    delete_profile, list_profiles, load_profile, load_profile_key,
    rename_profile, save_profile, validate_profile_name, KeyProfile,
};

/// Load `<name>`'s key as canonical hex, auto-migrating any legacy
/// base64 in place and printing a stderr notice if a migration ran.
///
/// Wraps [`crate::profile::load_profile_key`]: that function does the
/// mechanical migration; this one is the user-facing CLI helper that
/// also reports the migration. Used both by the command handlers in
/// this module and directly by each binary's key-resolution paths.
pub fn load_profile_key_with_notice(name: &str) -> Result<String> {
    let loaded = load_profile_key(name)?;
    if let Some(backup) = &loaded.migrated_backup {
        eprintln!(
            "notice: profile '{name}' had a legacy base64 key; \
             rewrote it as canonical hex (backup: {})",
            backup.display(),
        );
        eprintln!(
            "        base64 keys are deprecated and will be removed before \
             oboron 1.0."
        );
    }
    Ok(loaded.hex)
}

/// Binary-specific bits a command handler needs to know.
pub struct CliInfo<'a> {
    /// Binary name as users type it on the shell — `"ob"` or `"obcrypt"`.
    pub binary_name: &'a str,
    /// Default scheme written into `config.json` on `init` / `activate`.
    pub default_scheme: &'a str,
    /// Default encoding written into `config.json`. `None` for binaries
    /// that have no encoding layer (obcrypt).
    pub default_encoding: Option<&'a str>,
}

/// Load `config.json`, erroring with a binary-specific hint if missing.
fn require_config(info: &CliInfo<'_>) -> Result<Config> {
    load_config()?.ok_or_else(|| {
        let p = config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~/.oboron/config.json".into());
        anyhow!(
            "config not found at {p}\nHint: run '{} init' to create one",
            info.binary_name
        )
    })
}

pub fn init_command(
    info: &CliInfo<'_>,
    name: &str,
    generate_key: impl FnOnce() -> String,
) -> Result<()> {
    validate_profile_name(name)?;
    let path = profile_path(name)?;
    if path.exists() {
        eprintln!("❌ Error: Profile '{name}' already exists");
        eprintln!();
        eprintln!("'{} init' will not overwrite an existing profile.", info.binary_name);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  {} init <new-profile-name>", info.binary_name);
        eprintln!("  {} profile delete {name}", info.binary_name);
        eprintln!("  {} profile create <profile-name>", info.binary_name);
        anyhow::bail!("profile '{name}' already exists");
    }

    let key = generate_key();
    save_profile(name, &KeyProfile { key: Some(key.clone()) })?;
    save_config(&Config {
        profile: Some(name.to_string()),
        scheme: Some(info.default_scheme.to_string()),
        encoding: info.default_encoding.map(str::to_string),
    })?;

    let cfg_path = config_path()?;
    println!("✓ Configuration saved to {}", cfg_path.display());
    println!("\nYour profile '{name}':");
    println!("  Default scheme:   {}", info.default_scheme);
    if let Some(enc) = info.default_encoding {
        println!("  Default encoding: {enc}");
    }
    println!("  Key:              {key}");
    println!("\n⚠️  Keep this key secure! Anyone with it can decode your data.");

    Ok(())
}

pub fn config_show_command(info: &CliInfo<'_>) -> Result<()> {
    let config = require_config(info)?;
    let profile_name = config
        .profile
        .as_deref()
        .ok_or_else(|| anyhow!("config has no active profile"))?;
    // Eager-migrate any legacy base64 key in the active profile —
    // display always shows canonical hex.
    let key_hex = load_profile_key_with_notice(profile_name)?;

    println!("Current configuration:");
    println!("  Profile:  {profile_name}");
    if let Some(s) = &config.scheme {
        println!("  Scheme:   {s}");
    }
    if let Some(e) = &config.encoding {
        println!("  Encoding: {e}");
    }
    println!("  Key:      {key_hex}");

    Ok(())
}

pub fn config_set_command(
    info: &CliInfo<'_>,
    scheme: Option<String>,
    encoding: Option<String>,
    profile: Option<String>,
) -> Result<()> {
    let mut config = load_config()?.unwrap_or_else(|| Config {
        profile: Some("default".to_string()),
        scheme: Some(info.default_scheme.to_string()),
        encoding: info.default_encoding.map(str::to_string),
    });

    if let Some(s) = scheme {
        config.scheme = Some(s);
    }
    if let Some(e) = encoding {
        config.encoding = Some(e);
    }
    if let Some(p) = profile {
        config.profile = Some(p);
    }

    save_config(&config)?;
    println!("✓ Configuration updated");
    if let Some(p) = &config.profile {
        println!("  Profile:  {p}");
    }
    if let Some(s) = &config.scheme {
        println!("  Scheme:   {s}");
    }
    if let Some(e) = &config.encoding {
        println!("  Encoding: {e}");
    }
    Ok(())
}

pub fn profile_list_command(info: &CliInfo<'_>) -> Result<()> {
    let profiles = list_profiles()?;
    if profiles.is_empty() {
        println!("No profiles found. Run '{} init' to create one.", info.binary_name);
        return Ok(());
    }

    let active = load_config().ok().flatten().and_then(|c| c.profile);
    println!("Available profiles:");
    for p in profiles {
        let marker = if Some(p.as_str()) == active.as_deref() {
            " (active)"
        } else {
            ""
        };
        println!("  {p}{marker}");
    }
    Ok(())
}

pub fn profile_show_command(info: &CliInfo<'_>, name: Option<&str>) -> Result<()> {
    let profile_name = match name {
        Some(n) => n.to_string(),
        None => require_config(info)?
            .profile
            .ok_or_else(|| anyhow!("config has no active profile"))?,
    };
    let key_hex = load_profile_key_with_notice(&profile_name)?;
    println!("Profile '{profile_name}':");
    println!("  Key: {key_hex}");
    Ok(())
}

pub fn profile_activate_command(info: &CliInfo<'_>, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    load_profile(name)?; // ensure exists

    let mut cfg = load_config()?.unwrap_or_default();
    cfg.profile = Some(name.to_string());
    if cfg.scheme.is_none() {
        cfg.scheme = Some(info.default_scheme.to_string());
    }
    if cfg.encoding.is_none() {
        cfg.encoding = info.default_encoding.map(str::to_string);
    }
    save_config(&cfg)?;
    println!("✓ Activated profile '{name}'");
    Ok(())
}

pub fn profile_create_command(
    name: &str,
    key: Option<&str>,
    generate_key: impl FnOnce() -> String,
) -> Result<()> {
    validate_profile_name(name)?;
    let key_str = if let Some(k) = key {
        normalize_key_to_hex(k).context("invalid --key")?
    } else {
        generate_key()
    };
    save_profile(name, &KeyProfile { key: Some(key_str.clone()) })?;
    println!("✓ Created profile '{name}'");
    println!("  Key: {key_str}");
    println!("\n⚠️  Keep this profile secure!");
    Ok(())
}

pub fn profile_delete_command(info: &CliInfo<'_>, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    if let Ok(Some(cfg)) = load_config() {
        if cfg.profile.as_deref() == Some(name) {
            eprintln!("❌ Error: Cannot delete active profile '{name}'");
            eprintln!();
            eprintln!("Activate a different profile first:");
            eprintln!("  {} profile activate <other-profile-name>", info.binary_name);
            anyhow::bail!("cannot delete active profile '{name}'");
        }
    }

    let backup = delete_profile(name)?;
    println!("✓ Deleted profile '{name}'");
    if let Some(p) = backup {
        println!("  Backup saved to: {}", p.display());
    }
    Ok(())
}

pub fn profile_rename_command(old_name: &str, new_name: &str) -> Result<()> {
    let backup = rename_profile(old_name, new_name)?;

    if let Ok(Some(mut cfg)) = load_config() {
        if cfg.profile.as_deref() == Some(old_name) {
            cfg.profile = Some(new_name.to_string());
            save_config(&cfg)?;
            println!(
                "✓ Renamed profile '{old_name}' to '{new_name}' (active profile updated)"
            );
        } else {
            println!("✓ Renamed profile '{old_name}' to '{new_name}'");
        }
    } else {
        println!("✓ Renamed profile '{old_name}' to '{new_name}'");
    }

    if let Some(p) = backup {
        println!("  Backup saved to: {}", p.display());
    }
    Ok(())
}

pub fn profile_set_command(name: &str, key: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut profile = load_profile(name)?;
    profile.key = Some(normalize_key_to_hex(key).context("invalid --key")?);
    save_profile(name, &profile)?;
    println!("✓ Updated profile '{name}'");
    Ok(())
}

pub fn key_command(info: &CliInfo<'_>, profile_name: Option<&str>) -> Result<()> {
    let cfg = load_config().ok().flatten();
    let active = cfg.as_ref().and_then(|c| c.profile.as_deref());

    let prof = profile_name.or(active).ok_or_else(|| {
        anyhow!(
            "no profile given and no active profile in config; \
             run '{} init' or pass --profile",
            info.binary_name
        )
    })?;
    let key_hex = load_profile_key_with_notice(prof)?;
    println!("{key_hex}");
    Ok(())
}
