use anyhow::{anyhow, bail, Context, Result};
use data_encoding::BASE64URL_NOPAD;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// obz config lives under the shared `~/.oboron/` root in a `ztier`
// subdir, so its z-tier `scheme` field never collides with the
// secure-scheme `config.json` that `ob` / `obcrypt` keep at the
// root. Migrated from the legacy standalone `~/.obz/` on first run
// (see `ensure_ztier_dir_migrated`).
const CONFIG_ROOT_DIR: &str = ".oboron";
const ZTIER_SUBDIR: &str = "ztier";
const LEGACY_CONFIG_DIR: &str = ".obz";
const PROFILES_SUBDIR: &str = "profiles";
const BACKUP_SUBDIR: &str = "bkp";
const CONFIG_FILENAME: &str = "config.json";

/// `~/.oboron/ztier` — obz's config root (sibling of the `ob` /
/// `obcrypt` config at `~/.oboron/`).
pub fn config_root() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(CONFIG_ROOT_DIR)
        .join(ZTIER_SUBDIR)
}

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
    config_root().join(CONFIG_FILENAME)
}

pub fn profile_dir() -> PathBuf {
    config_root().join(PROFILES_SUBDIR)
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
    config_root().join(BACKUP_SUBDIR)
}

/// Returned by [`ensure_ztier_dir_migrated`] when a migration ran.
#[derive(Debug, Clone)]
pub struct ZtierMigrationNotice {
    pub from: PathBuf,
    pub to: PathBuf,
    /// `true` if the backward-compat symlink at the old path was
    /// created (false on platforms without symlink support).
    pub symlink_created: bool,
}

/// One-time migration of the legacy standalone `~/.obz/` config dir
/// to `~/.oboron/ztier/`. Mirrors the `~/.ob/` → `~/.oboron/`
/// migration in `oboron-cli-core`: renames the real dir, leaves a
/// `~/.obz` → `~/.oboron/ztier` symlink so an older `obz` binary
/// keeps working, and refuses an ambiguous state where both exist as
/// real dirs. No-op on fresh installs and every subsequent run.
pub fn ensure_ztier_dir_migrated() -> Result<Option<ZtierMigrationNotice>> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow!("could not locate home directory"))?;
    migrate_ztier_at(&home.join(LEGACY_CONFIG_DIR), &config_root())
}

/// Same logic as [`ensure_ztier_dir_migrated`] but with explicit
/// paths, for unit testing.
fn migrate_ztier_at(old: &Path, new: &Path) -> Result<Option<ZtierMigrationNotice>> {
    let old_meta = match fs::symlink_metadata(old) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("stat {}", old.display())),
    };
    // Already our compat symlink, or not a directory — leave it alone.
    if old_meta.file_type().is_symlink() || !old_meta.is_dir() {
        return Ok(None);
    }
    match fs::symlink_metadata(new) {
        Ok(_) => bail!(
            "found both {} and {} — refusing to auto-migrate ambiguous \
             state; move {} contents into {} manually and remove {}",
            old.display(),
            new.display(),
            old.display(),
            new.display(),
            old.display(),
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("stat {}", new.display())),
    }
    // The target is nested (`~/.oboron/ztier`); ensure the
    // `~/.oboron` parent exists before renaming into it (a user who
    // only ever ran obz won't have it yet).
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    fs::rename(old, new)
        .with_context(|| format!("rename {} → {}", old.display(), new.display()))?;
    let symlink_created = create_compat_symlink(old, new);
    Ok(Some(ZtierMigrationNotice {
        from: old.to_path_buf(),
        to: new.to_path_buf(),
        symlink_created,
    }))
}

#[cfg(unix)]
fn create_compat_symlink(link: &Path, target: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(not(unix))]
fn create_compat_symlink(_link: &Path, _target: &Path) -> bool {
    false
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

/// Generate a fresh z-tier secret as canonical 64-char hex
/// (32 bytes). Delegates to the library's canonical generator.
pub fn generate_secret() -> String {
    oboron::generate_secret()
}

/// Format an obz secret arrived in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretFormat {
    /// 64-character hex (canonical).
    Hex,
    /// 43-character URL-safe base64 (legacy; removed before oboron 1.0).
    LegacyBase64,
}

/// Convert a z-tier secret to canonical 64-char hex, accepting either
/// form, and report which form was given. Mirrors
/// `oboron_cli_core::normalize_key_classify` for the 32-byte secret.
///
/// - 64 hex chars (canonical) → `SecretFormat::Hex`, lowercased
/// - 43 base64 chars (legacy) → `SecretFormat::LegacyBase64`, re-encoded to hex
pub fn normalize_secret_classify(secret: &str) -> Result<(String, SecretFormat)> {
    let trimmed = secret.trim();
    match trimmed.len() {
        64 => {
            hex::decode(trimmed).map_err(|e| anyhow!("not a valid hex secret: {e}"))?;
            Ok((trimmed.to_lowercase(), SecretFormat::Hex))
        }
        43 => {
            let bytes = BASE64URL_NOPAD
                .decode(trimmed.as_bytes())
                .map_err(|e| anyhow!("not a valid base64 secret: {e}"))?;
            if bytes.len() != 32 {
                bail!("decoded base64 secret is {} bytes, expected 32", bytes.len());
            }
            Ok((hex::encode(bytes), SecretFormat::LegacyBase64))
        }
        n => bail!("secret has length {n}; expected 64 (hex) or 43 (legacy base64)"),
    }
}

/// Like [`normalize_secret_classify`] but discards the format tag.
pub fn normalize_secret_to_hex(secret: &str) -> Result<String> {
    Ok(normalize_secret_classify(secret)?.0)
}

/// Load `<name>`'s secret as canonical hex, **auto-migrating** a
/// legacy base64 profile in place (backup + stderr notice). Mirrors
/// `oboron_cli_core::commands::load_profile_key_with_notice`.
pub fn load_profile_secret_with_notice(name: &str) -> Result<String> {
    let profile = load_profile(name)?;
    let secret = profile
        .secret
        .ok_or_else(|| anyhow!("Profile '{name}' has no secret"))?;
    let (hex, fmt) = normalize_secret_classify(&secret)
        .with_context(|| format!("invalid secret in profile '{name}'"))?;
    if fmt == SecretFormat::LegacyBase64 {
        let backup = migrate_secret_in_place(name, &hex)?;
        eprintln!(
            "notice: profile '{name}' had a legacy base64 secret; \
             rewrote it as canonical hex (backup: {})",
            backup.display(),
        );
        eprintln!(
            "        base64 secrets are deprecated and will be removed before \
             oboron 1.0."
        );
    }
    Ok(hex)
}

/// Rewrite `<name>`'s profile with the canonical hex `secret`, backing
/// up the pre-migration file first. Quiet (prints nothing — callers
/// emit the stderr notice). Returns the backup path.
fn migrate_secret_in_place(name: &str, hex_secret: &str) -> Result<PathBuf> {
    let backup_path = backup_profile(name)?;
    let path = profile_path(name);
    let profile = SecretProfile {
        secret: Some(hex_secret.to_string()),
    };
    let content =
        serde_json::to_string_pretty(&profile).context("Failed to serialize profile")?;
    fs::write(&path, content).context("Failed to write profile file")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }
    Ok(backup_path)
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
        println!("Secret: {}", hex::encode(&secret_bytes[0..32]));
        return Ok(());
    }

    let config = load_config()?;
    // Eager-migrate any legacy base64 secret — display always shows
    // canonical hex.
    let secret_hex = load_profile_secret_with_notice(&config.profile)?;

    println!("Current configuration:");
    println!("  Profile:  {}", config.profile);
    println!("  Scheme:   {}", config.scheme);
    println!("  Encoding: {}", config.encoding);
    println!("  Secret:   {}", secret_hex);

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
        load_config()?.profile
    };

    let secret_hex = load_profile_secret_with_notice(&profile_name)?;
    println!("Profile '{}':", profile_name);
    println!("  Secret: {}", secret_hex);

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
        normalize_secret_to_hex(s).context("invalid --secret")?
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
        profile.secret = Some(normalize_secret_to_hex(s).context("invalid --secret")?);
    } else {
        anyhow::bail!("--secret must be provided");
    }

    save_profile(name, &profile)?;

    println!("✓ Updated profile '{}'", name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_secret_classify -------------------------------------

    #[test]
    fn hex_secret_passes_through() {
        let h = "0".repeat(64);
        let (out, fmt) = normalize_secret_classify(&h).unwrap();
        assert_eq!(out, h);
        assert_eq!(fmt, SecretFormat::Hex);
    }

    #[test]
    fn hex_secret_lowercased() {
        let mixed = "ABCDEF".to_string() + &"0".repeat(58);
        let (out, _) = normalize_secret_classify(&mixed).unwrap();
        assert_eq!(&out[..6], "abcdef");
    }

    #[test]
    fn base64_secret_classifies_as_legacy_and_converts() {
        // 43 'A' = base64url of 32 zero bytes.
        let b64 = "A".repeat(43);
        let (out, fmt) = normalize_secret_classify(&b64).unwrap();
        assert_eq!(fmt, SecretFormat::LegacyBase64);
        assert_eq!(out, "0".repeat(64));
    }

    #[test]
    fn wrong_length_secret_rejected() {
        assert!(normalize_secret_classify(&"a".repeat(32)).is_err());
        assert!(normalize_secret_classify(&"a".repeat(63)).is_err());
        assert!(normalize_secret_classify("").is_err());
    }

    #[test]
    fn secret_trims_whitespace() {
        let h = "0".repeat(64);
        assert_eq!(normalize_secret_to_hex(&format!("  {h}\n")).unwrap(), h);
    }

    // --- migrate_ztier_at ----------------------------------------------

    struct TmpDir(PathBuf);
    impl TmpDir {
        fn new(label: &str) -> Self {
            let id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir()
                .join(format!("obz-mig-{label}-{id}-{}", std::process::id()));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn neither_dir_is_noop() {
        let t = TmpDir::new("neither");
        let old = t.path().join(".obz");
        let new = t.path().join(".oboron").join("ztier");
        assert!(migrate_ztier_at(&old, &new).unwrap().is_none());
        assert!(!new.exists());
    }

    #[test]
    fn legacy_dir_migrates_and_creates_nested_parent() {
        let t = TmpDir::new("migrate");
        let old = t.path().join(".obz");
        let new = t.path().join(".oboron").join("ztier"); // parent absent
        fs::create_dir_all(old.join("profiles")).unwrap();
        fs::write(old.join("config.json"), r#"{"profile":"x"}"#).unwrap();

        let notice = migrate_ztier_at(&old, &new).unwrap().expect("migration");
        assert_eq!(notice.to, new);
        assert!(new.join("config.json").is_file());
        assert!(new.join("profiles").is_dir());
        #[cfg(unix)]
        {
            assert!(notice.symlink_created);
            let meta = fs::symlink_metadata(&old).unwrap();
            assert!(meta.file_type().is_symlink());
            assert!(old.join("config.json").is_file()); // reads through symlink
        }
    }

    #[test]
    fn both_dirs_present_errors() {
        let t = TmpDir::new("both");
        let old = t.path().join(".obz");
        let new = t.path().join(".oboron").join("ztier");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&new).unwrap();
        let err = migrate_ztier_at(&old, &new).unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
        assert!(old.is_dir());
        assert!(new.is_dir());
    }
}
