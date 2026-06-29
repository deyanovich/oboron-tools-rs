//! Key / secret string normalization.
//!
//! The canonical length comes from [`crate::env`]: the authenticated
//! `ob` / `obcrypt` accept a 128-character hex key; the unauthenticated
//! `obu` accepts its 64-character hex secret. Hex is the only accepted
//! form — the legacy base64 key form was removed in 1.0.

use anyhow::{anyhow, bail, Result};

/// Normalize a key/secret string to canonical hex: validate that it is
/// exactly the active environment's [`secret_hex_len`](crate::env::CliEnv::secret_hex_len)
/// hex characters (after trimming surrounding whitespace) and return it
/// lowercased. Any other length or non-hex input is an error.
pub fn normalize_key_to_hex(key: &str) -> Result<String> {
    let env = crate::env::env();
    let trimmed = key.trim();

    if trimmed.len() != env.secret_hex_len {
        bail!(
            "key has length {}; expected {} hex characters",
            trimmed.len(),
            env.secret_hex_len
        );
    }
    hex::decode(trimmed).map_err(|e| anyhow!("not a valid hex key: {e}"))?;
    Ok(trimmed.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_passes_through() {
        let h = "0".repeat(128);
        assert_eq!(normalize_key_to_hex(&h).unwrap(), h);
    }

    #[test]
    fn hex_lowercased() {
        let mixed = "AaBbCcDd".to_string() + &"0".repeat(120);
        let n = normalize_key_to_hex(&mixed).unwrap();
        assert_eq!(n.chars().next().unwrap(), 'a');
    }

    #[test]
    fn wrong_length_rejected() {
        assert!(normalize_key_to_hex(&"a".repeat(50)).is_err());
        assert!(normalize_key_to_hex(&"a".repeat(127)).is_err());
        assert!(normalize_key_to_hex("").is_err());
    }

    #[test]
    fn non_hex_rejected() {
        // Right length, wrong alphabet (86-char base64 is no longer a
        // special case — it is simply the wrong length / charset).
        assert!(normalize_key_to_hex(&"z".repeat(128)).is_err());
    }

    #[test]
    fn trims_whitespace() {
        let h = "0".repeat(128);
        let padded = format!("  {h}\n");
        assert_eq!(normalize_key_to_hex(&padded).unwrap(), h);
    }
}
