//! Key string normalization.
//!
//! During the base64 → hex migration period (until oboron 1.0), keys
//! may arrive as either 128-character hex (canonical) or 86-character
//! base64 (legacy, deprecated). [`normalize_key_classify`] reports
//! which form was used so callers can warn / migrate; the simpler
//! [`normalize_key_to_hex`] just returns the canonical hex.

use anyhow::{anyhow, bail, Result};
use data_encoding::BASE64URL_NOPAD;

/// Format the key arrived in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    /// 128-character hex (canonical).
    Hex,
    /// 86-character URL-safe base64 (legacy; will be removed before
    /// oboron 1.0).
    LegacyBase64,
}

/// Convert a key string to canonical 128-character hex, accepting
/// either form, and report which form was actually given.
///
/// - 128 hex chars (canonical) → `KeyFormat::Hex`, lowercased
/// - 86 base64 chars (legacy)  → `KeyFormat::LegacyBase64`, re-encoded to hex
///
/// Any other length / invalid encoding is an error.
pub fn normalize_key_classify(key: &str) -> Result<(String, KeyFormat)> {
    let trimmed = key.trim();
    match trimmed.len() {
        128 => {
            hex::decode(trimmed).map_err(|e| anyhow!("not a valid hex key: {e}"))?;
            Ok((trimmed.to_lowercase(), KeyFormat::Hex))
        }
        86 => {
            let bytes = BASE64URL_NOPAD
                .decode(trimmed.as_bytes())
                .map_err(|e| anyhow!("not a valid base64 key: {e}"))?;
            if bytes.len() != 64 {
                bail!("decoded base64 key is {} bytes, expected 64", bytes.len());
            }
            Ok((hex::encode(bytes), KeyFormat::LegacyBase64))
        }
        n => bail!("key has length {n}; expected 128 (hex) or 86 (legacy base64)"),
    }
}

/// Like [`normalize_key_classify`] but discards the format tag.
pub fn normalize_key_to_hex(key: &str) -> Result<String> {
    Ok(normalize_key_classify(key)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_passes_through() {
        let h = "0".repeat(128);
        let (out, fmt) = normalize_key_classify(&h).unwrap();
        assert_eq!(out, h);
        assert_eq!(fmt, KeyFormat::Hex);
    }

    #[test]
    fn base64_classifies_as_legacy() {
        let b64 = "A".repeat(86);
        let (_, fmt) = normalize_key_classify(&b64).unwrap();
        assert_eq!(fmt, KeyFormat::LegacyBase64);
    }

    #[test]
    fn hex_lowercased() {
        let mixed = "AaBbCcDd".to_string() + &"0".repeat(120);
        let n = normalize_key_to_hex(&mixed).unwrap();
        assert_eq!(n.chars().next().unwrap(), 'a');
    }

    #[test]
    fn base64_round_trips_to_hex() {
        // 86 'A's = base64 of 64 zero bytes
        let b64 = "A".repeat(86);
        let h = normalize_key_to_hex(&b64).unwrap();
        assert_eq!(h, "0".repeat(128));
    }

    #[test]
    fn wrong_length_rejected() {
        assert!(normalize_key_to_hex(&"a".repeat(50)).is_err());
        assert!(normalize_key_to_hex(&"a".repeat(127)).is_err());
        assert!(normalize_key_to_hex("").is_err());
    }

    #[test]
    fn trims_whitespace() {
        let h = "0".repeat(128);
        let padded = format!("  {h}\n");
        assert_eq!(normalize_key_to_hex(&padded).unwrap(), h);
    }
}
