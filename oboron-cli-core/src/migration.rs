//! One-time migration of the legacy `~/.ob/` config directory to
//! `~/.oboron/`.
//!
//! Older releases of `oboron-cli` stored their config at `~/.ob/`.
//! The current tooling uses `~/.oboron/`. [`ensure_config_root_migrated`]
//! detects a leftover `~/.ob/` real-directory, renames it to
//! `~/.oboron/`, and leaves a `~/.ob` → `~/.oboron` symlink so any
//! older binary still installed on the system reads/writes the same
//! data via the legacy path.

use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};

const OLD_DIR: &str = ".ob";
const NEW_DIR: &str = ".oboron";

/// Returned by [`ensure_config_root_migrated`] when an actual migration
/// took place. Callers print this to stderr so users know their config
/// dir moved.
#[derive(Debug, Clone)]
pub struct MigrationNotice {
    pub from: PathBuf,
    pub to: PathBuf,
    /// `true` if the backward-compat symlink at the old path was
    /// successfully created. `false` on platforms where symlink
    /// creation isn't supported or failed (e.g. Windows without
    /// privilege); the rename itself still succeeded.
    pub symlink_created: bool,
}

/// If `~/.ob/` exists as a real directory and `~/.oboron/` doesn't,
/// rename it to `~/.oboron/` and create a `~/.ob` → `~/.oboron`
/// symlink for backward compatibility with older binaries. No-op
/// otherwise (already migrated, fresh install, or `~/.ob` is a
/// symlink we put there ourselves).
pub fn ensure_config_root_migrated() -> Result<Option<MigrationNotice>> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("could not locate home directory"))?;
    migrate_at(&home.join(OLD_DIR), &home.join(NEW_DIR))
}

/// Same logic as [`ensure_config_root_migrated`] but with explicit
/// paths, for unit testing.
fn migrate_at(old: &Path, new: &Path) -> Result<Option<MigrationNotice>> {
    let old_meta = match std::fs::symlink_metadata(old) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e).with_context(|| format!("stat {}", old.display()));
        }
    };
    if old_meta.file_type().is_symlink() {
        // Already a symlink — assume it's the compat link we (or the
        // user) put there. Nothing to do.
        return Ok(None);
    }
    if !old_meta.is_dir() {
        // `~/.ob` exists but isn't a directory. Don't touch it.
        return Ok(None);
    }
    // `~/.ob` is a real directory. Check `~/.oboron`.
    match std::fs::symlink_metadata(new) {
        Ok(_) => bail!(
            "found both {} and {} — refusing to auto-migrate \
             ambiguous state; move {} contents into {} manually \
             and remove {}",
            old.display(),
            new.display(),
            old.display(),
            new.display(),
            old.display(),
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e).with_context(|| format!("stat {}", new.display()));
        }
    }
    // Rename `~/.ob` → `~/.oboron`. Atomic on the same filesystem.
    std::fs::rename(old, new)
        .with_context(|| format!("rename {} → {}", old.display(), new.display()))?;
    let symlink_created = create_compat_symlink(old, new);
    Ok(Some(MigrationNotice {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a fresh temp directory for one test. Cleaned up on
    /// drop via `Drop` on the returned guard.
    struct TmpDir(PathBuf);
    impl TmpDir {
        fn new(label: &str) -> Self {
            let id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let p = std::env::temp_dir()
                .join(format!("oboron-mig-{label}-{id}-{}", std::process::id()));
            fs::create_dir_all(&p).expect("create tmp dir");
            Self(p)
        }
        fn path(&self) -> &Path { &self.0 }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn neither_dir_exists_is_noop() {
        let t = TmpDir::new("neither");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        assert!(migrate_at(&old, &new).unwrap().is_none());
        assert!(!old.exists());
        assert!(!new.exists());
    }

    #[test]
    fn only_new_dir_is_noop() {
        let t = TmpDir::new("only-new");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        fs::create_dir(&new).unwrap();
        fs::write(new.join("config.json"), "{}").unwrap();
        assert!(migrate_at(&old, &new).unwrap().is_none());
        assert!(!old.exists());
        assert!(new.is_dir());
        assert!(new.join("config.json").is_file());
    }

    #[test]
    fn only_old_dir_migrates() {
        let t = TmpDir::new("only-old");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        fs::create_dir(&old).unwrap();
        fs::write(old.join("config.json"), r#"{"profile":"x"}"#).unwrap();

        let notice = migrate_at(&old, &new).unwrap().expect("expected migration");
        assert_eq!(notice.from, old);
        assert_eq!(notice.to, new);
        #[cfg(unix)]
        assert!(notice.symlink_created);

        assert!(new.is_dir());
        assert!(new.join("config.json").is_file());
        #[cfg(unix)]
        {
            // Old path is now a symlink to new.
            let meta = fs::symlink_metadata(&old).unwrap();
            assert!(meta.file_type().is_symlink());
            assert_eq!(fs::read_link(&old).unwrap(), new);
            // And reading through the symlink finds the file.
            assert!(old.join("config.json").is_file());
        }
    }

    #[test]
    fn both_dirs_present_errors() {
        let t = TmpDir::new("both");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        fs::create_dir(&old).unwrap();
        fs::create_dir(&new).unwrap();
        let err = migrate_at(&old, &new).unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
        // Both still present, untouched.
        assert!(old.is_dir());
        assert!(new.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn old_path_already_symlink_is_noop() {
        let t = TmpDir::new("symlink");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        fs::create_dir(&new).unwrap();
        std::os::unix::fs::symlink(&new, &old).unwrap();
        assert!(migrate_at(&old, &new).unwrap().is_none());
        // Symlink still there.
        let meta = fs::symlink_metadata(&old).unwrap();
        assert!(meta.file_type().is_symlink());
    }

    #[test]
    fn idempotent_under_repeated_calls() {
        let t = TmpDir::new("idempotent");
        let old = t.path().join(".ob");
        let new = t.path().join(".oboron");
        fs::create_dir(&old).unwrap();
        fs::write(old.join("a"), "x").unwrap();
        // First call migrates.
        assert!(migrate_at(&old, &new).unwrap().is_some());
        // Second call is a no-op (old is now a symlink on Unix; on
        // non-Unix the rename succeeded but no symlink was made, so
        // the second call sees a missing old path — also a no-op).
        assert!(migrate_at(&old, &new).unwrap().is_none());
    }
}
