use anyhow::{Context, Result};
use data_encoding::BASE64URL_NOPAD;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".obz";
const PROFILES_SUBDIR: &str = "profiles";
const BACKUP_SUBDIR: &str = "bkp";
const CONFIG_FILENAME: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "profile")]
    pub profile: String,
    pub scheme: String,
    pub encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(CONFIG_DIR)
        .join(CONFIG_FILENAME)
}

pub fn profile_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(CONFIG_DIR)
        .join(PROFILES_SUBDIR)
}

pub fn profile_path(name: &str) -> PathBuf {
    profile_dir().join(format!("{}.json", name))
}

/// Validate that a profile name is safe and contains no path traversal characters.
/// Only allows alphanumeric characters, hyphens, and underscores.
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Profile name cannot be empty");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        anyhow::bail!("Profile name '{}' contains invalid path characters", name);
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "Profile name '{}' contains invalid characters. Only alphanumeric characters, hyphens, and underscores are allowed",
            name
        );
    }
    Ok(())
}

pub fn backup_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(CONFIG_DIR)
        .join(BACKUP_SUBDIR)
}

fn backup_profile(name: &str) -> Result<PathBuf> {
    let path = profile_path(name);

    if !path.exists() {
        anyhow::bail!("Profile '{}' does not exist", name);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let backup_path = backup_dir().join(format!("{}-{}.json", name, timestamp));

    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent).context("Failed to create backup directory")?;
    }

    fs::copy(&path, &backup_path).context(format!("Failed to backup profile '{}'", name))?;

    Ok(backup_path)
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    let content = fs::read_to_string(&path).context(format!(
        "Failed to read config file {}\nHint: Run 'obz init' to create a config file",
        path.display()
    ))?;

    let mut config: Config =
        serde_json::from_str(&content).context("Failed to parse config file")?;

    if config.scheme.is_empty() {
        config.scheme = "zrbcx".to_string();
    }

    if config.encoding.is_empty() {
        config.encoding = "c32".to_string();
    }

    if config.profile.is_empty() {
        config.profile = "default".to_string();
    }

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&path, content).context("Failed to write config file")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

pub fn load_profile(name: &str) -> Result<SecretProfile> {
    validate_profile_name(name)?;
    let path = profile_path(name);
    let content = fs::read_to_string(&path).context(format!(
        "Failed to read secret profile '{}'\nHint: Run 'obz init' or 'obz profile create {}' to create this profile",
        name, name
    ))?;

    let profile: SecretProfile = serde_json::from_str(&content)
        .context(format!("Failed to parse secret profile '{}'", name))?;

    Ok(profile)
}

pub fn save_profile(name: &str, profile: &SecretProfile) -> Result<()> {
    validate_profile_name(name)?;
    let path = profile_path(name);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create profile directory")?;
    }

    if path.exists() {
        let backup_path = backup_profile(name)?;
        println!("Backed up existing profile to:  {}", backup_path.display());
    }

    let content = serde_json::to_string_pretty(profile).context("Failed to serialize profile")?;

    fs::write(&path, content).context("Failed to write profile file")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

/// Generate a z-tier secret (32 bytes = 43 base64 chars)
pub fn generate_secret() -> String {
    loop {
        let mut secret_bytes = [0u8; 32];
        RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret_bytes);
        let secret_base64 = BASE64URL_NOPAD.encode(&secret_bytes);
        if !secret_base64.contains('-') && !secret_base64.contains('_') {
            return secret_base64;
        }
    }
}

pub fn validate_base64_secret(secret_str: &str) -> Result<()> {
    if secret_str.len() != 43 {
        anyhow::bail!(
            "Secret must be 43 base64 chars, got {} chars",
            secret_str.len()
        );
    }

    let secret_bytes = BASE64URL_NOPAD
        .decode(secret_str.as_bytes())
        .context("Invalid secret base64 encoding")?;

    if secret_bytes.len() != 32 {
        anyhow::bail!(
            "Secret must decode to 32 bytes, got {} bytes",
            secret_bytes.len()
        );
    }

    Ok(())
}

pub fn init_command(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let path = profile_path(name);
    if path.exists() {
        eprintln!("❌ Error: Profile '{}' already exists", name);
        eprintln!();
        eprintln!(
            "To avoid accidental data loss, 'obz init' cannot overwrite an existing profile."
        );
        eprintln!();
        eprintln!("Options:");
        eprintln!("  1. Create a new profile with a different name:");
        eprintln!("     obz init <new-profile-name>");
        eprintln!();
        eprintln!("  2. Delete the existing profile first:");
        eprintln!("     obz profile delete {}", name);
        eprintln!();
        eprintln!("  3. Manually delete the profile file:");
        eprintln!("     rm {}", path.display());

        anyhow::bail!("Profile '{}' already exists", name);
    }

    let secret = generate_secret();

    let profile = SecretProfile {
        secret: Some(secret.clone()),
    };

    save_profile(name, &profile)?;

    let config = Config {
        profile: name.to_string(),
        scheme: "zrbcx".to_string(),
        encoding: "c32".to_string(),
    };

    save_config(&config)?;

    println!("✓ Configuration saved to {}", config_path().display());
    println!("\nYour profile '{}':", name);
    println!("  Default scheme:   zrbcx");
    println!("  Default encoding: c32");
    println!("  Secret:  {}", secret);
    println!("\n⚠️  Z-tier schemes provide NO cryptographic security!");
    println!("    Use only for obfuscation, never for actual encryption.");

    Ok(())
}

pub fn config_show_command(keyless: bool) -> Result<()> {
    if keyless {
        println!("Using public profile (INSECURE - testing only):");
        let secret_bytes = oboron::HARDCODED_KEY_BYTES;
        let b64_secret = BASE64URL_NOPAD.encode(&secret_bytes[0..32]);
        println!("Secret: {}", b64_secret);
        return Ok(());
    }

    let config = load_config()?;
    let profile = load_profile(&config.profile)?;

    println!("Current configuration:");
    println!("  Profile:  {}", config.profile);
    println!("  Scheme:   {}", config.scheme);
    println!("  Encoding: {}", config.encoding);
    if let Some(secret) = &profile.secret {
        println!("  Secret:   {}", secret);
    }

    Ok(())
}

pub fn profile_list_command() -> Result<()> {
    let profile_dir = profile_dir();

    if !profile_dir.exists() {
        println!("No profiles found.  Run 'obz init' to create one.");
        return Ok(());
    }

    let entries = fs::read_dir(&profile_dir)?;
    let mut profiles = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                profiles.push(name.to_string());
            }
        }
    }

    if profiles.is_empty() {
        println!("No profiles found.");
        return Ok(());
    }

    profiles.sort();

    let config = load_config().ok();
    let active_profile = config.as_ref().map(|c| c.profile.as_str());

    println!("Available profiles:");
    for profile in profiles {
        let marker = if Some(profile.as_str()) == active_profile {
            " (active)"
        } else {
            ""
        };
        println!("  {}{}", profile, marker);
    }

    Ok(())
}

pub fn profile_show_command(name: Option<&str>) -> Result<()> {
    let profile_name = if let Some(n) = name {
        n.to_string()
    } else {
        let config = load_config()?;
        config.profile
    };

    let profile = load_profile(&profile_name)?;

    println!("Profile '{}':", profile_name);
    if let Some(secret) = &profile.secret {
        println!("  Secret: {}", secret);
    }

    Ok(())
}

pub fn profile_activate_command(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    load_profile(name)?;

    let mut config = load_config().unwrap_or(Config {
        profile: "default".to_string(),
        scheme: "zrbcx".to_string(),
        encoding: "c32".to_string(),
    });

    config.profile = name.to_string();
    save_config(&config)?;

    println!("✓ Activated profile '{}'", name);

    Ok(())
}

pub fn profile_create_command(name: &str, secret: Option<&str>) -> Result<()> {
    validate_profile_name(name)?;
    let secret_str = if let Some(s) = secret {
        validate_base64_secret(s)?;
        s.to_string()
    } else {
        generate_secret()
    };
    let profile = SecretProfile { secret: Some(secret_str.clone()) };

    save_profile(name, &profile)?;

    println!("✓ Created profile '{}'", name);
    println!("  Secret: {}", secret_str);
    println!("\n⚠️  Keep this profile secure!");

    Ok(())
}

pub fn profile_delete_command(name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let path = profile_path(name);

    if !path.exists() {
        anyhow::bail!("Profile '{}' does not exist", name);
    }

    if let Ok(config) = load_config() {
        if config.profile == name {
            eprintln!("❌ Error: Cannot delete active profile '{}'", name);
            eprintln!();
            eprintln!(
                "The profile '{}' is currently set as the active profile.",
                name
            );
            eprintln!();
            eprintln!("To delete this profile:");
            eprintln!("  1. First activate a different profile:");
            eprintln!("     obz profile activate <other-profile-name>");
            eprintln!();
            eprintln!("  2. Or create a new profile:");
            eprintln!("     obz profile create <new-profile-name>");
            eprintln!("     obz profile activate <new-profile-name>");
            eprintln!();
            eprintln!("  3. Then delete this profile:");
            eprintln!("     obz profile delete {}", name);

            anyhow::bail!("Cannot delete active profile '{}'", name);
        }
    }

    let backup_path = backup_profile(name)?;
    fs::remove_file(&path)?;

    println!("✓ Deleted profile '{}'", name);
    println!("  Backup saved to: {}", backup_path.display());

    Ok(())
}

pub fn profile_rename_command(old_name: &str, new_name: &str) -> Result<()> {
    validate_profile_name(old_name)?;
    validate_profile_name(new_name)?;
    let old_path = profile_path(old_name);
    let new_path = profile_path(new_name);

    if !old_path.exists() {
        anyhow::bail!("Profile '{}' does not exist", old_name);
    }

    if new_path.exists() {
        anyhow::bail!(
            "Profile '{}' already exists.  Cannot rename to an existing profile name.",
            new_name
        );
    }

    let backup_path = backup_profile(old_name)?;
    fs::rename(&old_path, &new_path).context(format!(
        "Failed to rename profile '{}' to '{}'",
        old_name, new_name
    ))?;

    if let Ok(mut config) = load_config() {
        if config.profile == old_name {
            config.profile = new_name.to_string();
            save_config(&config)?;
            println!(
                "✓ Renamed profile '{}' to '{}' (active profile updated)",
                old_name, new_name
            );
        } else {
            println!("✓ Renamed profile '{}' to '{}'", old_name, new_name);
        }
    } else {
        println!("✓ Renamed profile '{}' to '{}'", old_name, new_name);
    }

    println!("  Backup saved to:  {}", backup_path.display());

    Ok(())
}

pub fn profile_set_command(name: &str, secret: Option<&str>) -> Result<()> {
    validate_profile_name(name)?;
    let mut profile = load_profile(name)?;

    if let Some(s) = secret {
        validate_base64_secret(s)?;
        profile.secret = Some(s.to_string());
    } else {
        anyhow::bail!("--secret must be provided");
    }

    save_profile(name, &profile)?;

    println!("✓ Updated profile '{}'", name);

    Ok(())
}
