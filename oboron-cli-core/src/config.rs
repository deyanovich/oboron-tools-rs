//! `config.json` — global config: active profile + per-binary defaults.
//!
//! Different binaries care about different fields:
//!
//! - `obc` (obcrypt-cli): `profile`, `scheme`.
//! - `ob`  (oboron-cli):  `profile`, `scheme`, `encoding`.
//!
//! [`Config`] surfaces all known fields as `Option`. When writing, we
//! read the existing file as a `serde_json::Value`, overwrite the
//! fields the caller is updating, and write the rest back unchanged —
//! so one binary's update doesn't clobber the other's settings.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::paths::config_path;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    /// Active profile name (used by both `ob` and `obc`).
    pub profile: Option<String>,
    /// Default scheme (used by both).
    pub scheme: Option<String>,
    /// Default encoding (oboron-only; obcrypt has no encoding layer).
    pub encoding: Option<String>,
}

/// Load `config.json`. Returns `Ok(None)` if the file doesn't exist.
pub fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let v = read_json(&path)?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("{} is not a JSON object", path.display()))?;
    Ok(Some(Config {
        profile: obj.get("profile").and_then(Value::as_str).map(str::to_string),
        scheme: obj.get("scheme").and_then(Value::as_str).map(str::to_string),
        encoding: obj.get("encoding").and_then(Value::as_str).map(str::to_string),
    }))
}

/// Update `config.json`, preserving any unknown JSON fields.
pub fn save_config(cfg: &Config) -> Result<()> {
    let path = config_path()?;
    let mut v = read_json_or_empty(&path)?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} is not a JSON object", path.display()))?;
    if let Some(p) = &cfg.profile {
        obj.insert("profile".into(), Value::String(p.clone()));
    }
    if let Some(s) = &cfg.scheme {
        obj.insert("scheme".into(), Value::String(s.clone()));
    }
    if let Some(e) = &cfg.encoding {
        obj.insert("encoding".into(), Value::String(e.clone()));
    }
    write_json(&path, &v)
}

// ---------------------------------------------------------------------------
// JSON I/O helpers (also re-used by `profile.rs`)
// ---------------------------------------------------------------------------

pub(crate) fn read_json(path: &PathBuf) -> Result<Value> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let v: Value = serde_json::from_str(&body)
        .with_context(|| format!("parse {}", path.display()))?;
    if !v.is_object() {
        bail!("{} is not a JSON object", path.display());
    }
    Ok(v)
}

pub(crate) fn read_json_or_empty(path: &PathBuf) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    read_json(path)
}

pub(crate) fn write_json(path: &PathBuf, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create parent directory")?;
    }
    let pretty = serde_json::to_string_pretty(value).context("serialize JSON")?;
    fs::write(path, pretty).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}
