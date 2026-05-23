//! Per-profile files at `~/.oboron/profiles/<NAME>.json`.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::config::{read_json_or_empty, write_json};
use crate::key::{normalize_key_classify, normalize_key_to_hex, KeyFormat};
use crate::paths::{backup_dir, profile_dir, profile_path};

#[derive(Debug, Default, Clone)]
pub struct KeyProfile {
    /// The key as stored. May be hex (128 chars, canonical) or base64
    /// (86 chars, legacy/deprecated).
    pub key: Option<String>,
}

/// Validate a profile name (no path traversal; alphanumeric + `-` + `_` only).
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("profile name is empty");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("profile name '{name}' contains invalid path characters");
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!(
            "profile name '{name}' contains invalid characters; \
             only alphanumeric, '-' and '_' allowed"
        );
    }
    Ok(())
}

pub fn load_profile(name: &str) -> Result<KeyProfile> {
    let path = profile_path(name)?;
    if !path.exists() {
        bail!("profile '{name}' not found (looked at {})", path.display());
    }
    let v = crate::config::read_json(&path)?;
    let key = v.get("key").and_then(Value::as_str).map(str::to_string);
    Ok(KeyProfile { key })
}

/// Load `<name>`'s key and return it as a 128-char hex string.
pub fn load_profile_key_as_hex(name: &str) -> Result<String> {
    let p = load_profile(name)?;
    let key = p
        .key
        .ok_or_else(|| anyhow!("profile '{name}' has no `key` field"))?;
    normalize_key_to_hex(&key).with_context(|| format!("invalid key in profile '{name}'"))
}

/// Result of [`load_profile_key`]: the canonical hex key plus, if a
/// migration happened, the path to the backup of the pre-migration
/// profile file.
#[derive(Debug, Clone)]
pub struct LoadedKey {
    pub hex: String,
    /// `Some(path)` if the stored profile had a legacy base64 key
    /// that we just rewrote in place to canonical hex; `None`
    /// otherwise. Callers should display this to the user as a
    /// migration notice.
    pub migrated_backup: Option<PathBuf>,
}

/// Load `<name>`'s key as canonical hex, **auto-migrating** any
/// legacy base64 profile in place.
///
/// If the stored key was 86-char base64, this rewrites the profile
/// file with the equivalent 128-char hex (creating a backup of the
/// pre-migration file). The returned [`LoadedKey::migrated_backup`]
/// is `Some(path)` in that case, so callers can print a notice.
///
/// Used by the `ob` and `obc` CLIs during the base64 → hex transition;
/// once base64 support is removed before oboron 1.0, this function
/// becomes equivalent to [`load_profile_key_as_hex`].
pub fn load_profile_key(name: &str) -> Result<LoadedKey> {
    let p = load_profile(name)?;
    let key = p
        .key
        .ok_or_else(|| anyhow!("profile '{name}' has no `key` field"))?;
    let (hex, fmt) = normalize_key_classify(&key)
        .with_context(|| format!("invalid key in profile '{name}'"))?;
    let migrated_backup = if fmt == KeyFormat::LegacyBase64 {
        // Rewrite the profile with the canonical hex form.
        save_profile(
            name,
            &KeyProfile {
                key: Some(hex.clone()),
            },
        )?
    } else {
        None
    };
    Ok(LoadedKey {
        hex,
        migrated_backup,
    })
}

/// Save a profile. Preserves unknown fields; backs up the existing
/// file (if any) before overwriting. Returns the backup path, if one
/// was created.
pub fn save_profile(name: &str, profile: &KeyProfile) -> Result<Option<PathBuf>> {
    validate_profile_name(name)?;
    let backup = backup_profile(name)?;
    let path = profile_path(name)?;
    let mut v = read_json_or_empty(&path)?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} is not a JSON object", path.display()))?;
    if let Some(k) = &profile.key {
        obj.insert("key".into(), Value::String(k.clone()));
    }
    write_json(&path, &v)?;
    Ok(backup)
}

/// List all profile names, sorted.
pub fn list_profiles() -> Result<Vec<String>> {
    let dir = profile_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).context("read profile directory")? {
        let entry = entry.context("read profile entry")?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn delete_profile(name: &str) -> Result<Option<PathBuf>> {
    validate_profile_name(name)?;
    let path = profile_path(name)?;
    if !path.exists() {
        bail!("profile '{name}' does not exist");
    }
    let backup = backup_profile(name)?;
    fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    Ok(backup)
}

pub fn rename_profile(old_name: &str, new_name: &str) -> Result<Option<PathBuf>> {
    validate_profile_name(old_name)?;
    validate_profile_name(new_name)?;
    let old_path = profile_path(old_name)?;
    let new_path = profile_path(new_name)?;
    if !old_path.exists() {
        bail!("profile '{old_name}' does not exist");
    }
    if new_path.exists() {
        bail!("profile '{new_name}' already exists — cannot rename onto it");
    }
    let backup = backup_profile(old_name)?;
    fs::rename(&old_path, &new_path).with_context(|| {
        format!("rename {} → {}", old_path.display(), new_path.display())
    })?;
    Ok(backup)
}

/// Back up a profile file (if it exists). Returns the backup path, or
/// `None` if the source didn't exist.
fn backup_profile(name: &str) -> Result<Option<PathBuf>> {
    let path = profile_path(name)?;
    if !path.exists() {
        return Ok(None);
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| anyhow!("system time error: {e}"))?
        .as_secs();
    let backup_path = backup_dir()?.join(format!("{name}-{ts}.json"));
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent).context("create backup directory")?;
    }
    fs::copy(&path, &backup_path)
        .with_context(|| format!("backup profile '{name}'"))?;
    Ok(Some(backup_path))
}
