//! Per-binary runtime environment, installed once at process start.
//!
//! The shared path and key logic varies in two ways across the family
//! of CLIs: the config directory (`.oboron` for the authenticated
//! `ob` / `obcrypt`, `.obu` for the unauthenticated `obu`) and the
//! canonical secret length (128 hex chars / 512-bit key vs 64 hex
//! chars / 256-bit secret). A binary installs its environment once,
//! first thing in `main`; everything else reads it via [`env`].

use std::sync::OnceLock;

/// Per-binary configuration read by the shared path and key logic.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct CliEnv {
    /// Home-relative config directory, e.g. `.oboron` or `.obu`.
    pub config_dir: &'static str,
    /// Canonical key/secret length in hex characters — 128 for the
    /// authenticated 512-bit key, 64 for the 256-bit obu secret.
    pub secret_hex_len: usize,
}

impl CliEnv {
    /// Default environment for the authenticated `ob` / `obcrypt`
    /// CLIs: `~/.oboron`, 128-hex keys.
    pub const OBORON: CliEnv = CliEnv {
        config_dir: ".oboron",
        secret_hex_len: 128,
    };

    /// Environment for the unauthenticated `obu` CLI: `~/.obu`, 64-hex
    /// secrets.
    pub const OBU: CliEnv = CliEnv {
        config_dir: ".obu",
        secret_hex_len: 64,
    };
}

static ENV: OnceLock<CliEnv> = OnceLock::new();

/// Install the per-binary environment. Call once, as the first thing in
/// `main`, before any config/profile/key operation. A second call — or
/// a call after the environment has already been read — is ignored.
pub fn set_env(env: CliEnv) {
    let _ = ENV.set(env);
}

/// The active environment. Defaults to [`CliEnv::OBORON`] if no binary
/// installed one, so `ob` / `obcrypt` (which don't call [`set_env`])
/// keep their historical `.oboron` / 128-hex behavior unchanged.
pub fn env() -> CliEnv {
    *ENV.get_or_init(|| CliEnv::OBORON)
}
