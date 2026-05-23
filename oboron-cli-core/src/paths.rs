//! Path resolution for the `~/.oboron/` directory tree.

use anyhow::{anyhow, Result};
use std::path::PathBuf;

const CONFIG_DIR: &str = ".oboron";
const PROFILES_SUBDIR: &str = "profiles";
const BACKUP_SUBDIR: &str = "bkp";
const CONFIG_FILENAME: &str = "config.json";

pub fn config_root() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("could not locate home directory"))?
        .join(CONFIG_DIR))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_root()?.join(CONFIG_FILENAME))
}

pub fn profile_dir() -> Result<PathBuf> {
    Ok(config_root()?.join(PROFILES_SUBDIR))
}

pub fn profile_path(name: &str) -> Result<PathBuf> {
    crate::profile::validate_profile_name(name)?;
    Ok(profile_dir()?.join(format!("{name}.json")))
}

pub fn backup_dir() -> Result<PathBuf> {
    Ok(config_root()?.join(BACKUP_SUBDIR))
}
