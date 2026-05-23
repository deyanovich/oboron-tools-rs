#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn test_home_dir() -> PathBuf {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    PathBuf::from(format!("./test_home_{}", test_id))
}

fn cleanup_test_home(dir: &PathBuf) {
    if dir.exists() {
        let _ = fs::remove_dir_all(dir);
    }
}

// Valid 86-character base64 key (64 bytes = 512 bits)
const TEST_KEY_B64: &str =
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const TEST_KEY_B64_ALT: &str =
    "ZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[test]
fn test_enc_keyless() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--aasv")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "apsv")]
#[test]
fn test_enc_keyless_apsv() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--apsv")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_short_alias_aasv() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-s")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "apsv")]
#[test]
fn test_enc_short_alias_apsv() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-S")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "apsv")]
#[test]
fn test_enc_dec_roundtrip_keyless() {
    let test_home = test_home_dir();

    // Encode
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--apsv")
        .arg("--b32")
        .arg("hello_world")
        .output()
        .unwrap();

    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();
    assert!(!encd.is_empty());

    // Decode - use same scheme as enc for autodetection
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--apsv")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_world"));

    cleanup_test_home(&test_home);
}

#[test]
fn test_enc_with_explicit_key() {
    let test_home = test_home_dir();

    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_B64)
        .arg("--aasv")
        .arg("--b32")
        .arg("test_data")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());

    cleanup_test_home(&test_home);
}

#[cfg(feature = "apsv")]
#[test]
fn test_enc_with_explicit_key_apsv() {
    let test_home = test_home_dir();

    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_B64)
        .arg("--apsv")
        .arg("--b32")
        .arg("test_data")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());

    cleanup_test_home(&test_home);
}

#[test]
fn test_enc_dec_with_explicit_key() {
    let test_home = test_home_dir();

    // Encode with aags
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_B64_ALT)
        .arg("--aasv")
        .arg("--b32")
        .arg("sensitive_data")
        .output()
        .unwrap();

    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode - use same scheme as enc
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_B64_ALT)
        .arg("--aasv") // Use same scheme as enc
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("sensitive_data"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aags")]
#[test]
fn test_enc_dec_with_explicit_key_aags() {
    let test_home = test_home_dir();

    // Encode with aags
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_B64_ALT)
        .arg("--aags")
        .arg("--b32")
        .arg("sensitive_data")
        .output()
        .unwrap();

    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode - use same scheme as enc
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_B64_ALT)
        .arg("--aags") // Use same scheme as enc
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("sensitive_data"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aags")]
#[cfg(feature = "aasv")]
#[cfg(feature = "upbc")]
#[cfg(feature = "apgs")]
#[cfg(feature = "apsv")]
#[test]
fn test_enc_different_schemes() {
    let test_home = test_home_dir();
    let schemes = vec!["--aags", "--aasv", "--upbc", "--apgs", "--apsv"];

    for scheme in schemes {
        let mut cmd = Command::cargo_bin("ob").unwrap();
        cmd.env("HOME", test_home.as_os_str())
            .arg("enc")
            .arg("-K")
            .arg(scheme)
            .arg("--b32")
            .arg("test")
            .assert()
            .success();
    }

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_different_encodings() {
    let test_home = test_home_dir();
    let encodings = vec!["--b32", "--b64", "--hex"];

    for encoding in encodings {
        let mut cmd = Command::cargo_bin("ob").unwrap();
        cmd.env("HOME", test_home.as_os_str())
            .arg("enc")
            .arg("-K")
            .arg("--aasv")
            .arg(encoding)
            .arg("test")
            .assert()
            .success();
    }

    cleanup_test_home(&test_home);
}

#[cfg(feature = "apsv")]
#[cfg(feature = "aags")]
#[cfg(feature = "aasv")]
#[test]
fn test_enc_with_format_string() {
    let test_home = test_home_dir();
    let formats = vec!["apsv.b32", "aags.b64", "aasv.hex"];

    for format in formats {
        let mut cmd = Command::cargo_bin("ob").unwrap();
        cmd.env("HOME", test_home.as_os_str())
            .arg("enc")
            .arg("-K")
            .arg("--format")
            .arg(format)
            .arg("test_format")
            .assert()
            .success();
    }

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_short_alias_b64() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--aasv")
        .arg("-B")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_short_alias_b32() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--aasv")
        .arg("-b")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_short_alias_c32() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--aasv")
        .arg("-c")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

// Valid 43-character base64 secret (32 bytes)
const TEST_SECRET_B64: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_short_alias_zrbcx() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-s")
        .arg(TEST_SECRET_B64)
        .arg("-r")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_short_alias_b32() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-s")
        .arg(TEST_SECRET_B64)
        .arg("--zrbcx")
        .arg("-b")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_short_alias_b64() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-s")
        .arg(TEST_SECRET_B64)
        .arg("--zrbcx")
        .arg("-B")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_short_alias_c32() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-s")
        .arg(TEST_SECRET_B64)
        .arg("--zrbcx")
        .arg("-c")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_combined_short_aliases() {
    // Test the convenience combined-flag usage: obz e -rb 'abc'
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("e")
        .arg("-s")
        .arg(TEST_SECRET_B64)
        .arg("-rb")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "upbc")]
#[test]
fn test_enc_short_alias_upbc() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("ob").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-u")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&test_home);
}

#[cfg(feature = "upbc")]
#[test]
fn test_dec_short_alias_upbc() {
    let test_home = test_home_dir();

    // First encode
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-u")
        .arg("--b32")
        .arg("hello123")
        .output()
        .unwrap();
    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode with -u alias
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("-u")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello123"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "legacy")]
#[test]
fn test_obz_enc_legacy_rejects_b32_flag() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--legacy")
        .arg("--b32")
        .arg("test123")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--legacy is incompatible with encoding flags"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "legacy")]
#[test]
fn test_obz_enc_legacy_rejects_b64_flag() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--legacy")
        .arg("--b64")
        .arg("test123")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--legacy is incompatible with encoding flags"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "legacy")]
#[test]
fn test_obz_enc_legacy_rejects_hex_flag() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--legacy")
        .arg("--hex")
        .arg("test123")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--legacy is incompatible with encoding flags"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "legacy")]
#[test]
fn test_obz_enc_legacy_rejects_c32_flag() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--legacy")
        .arg("--c32")
        .arg("test123")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--legacy is incompatible with encoding flags"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "legacy")]
#[test]
fn test_obz_dec_legacy_rejects_encoding_flags() {
    let test_home = test_home_dir();
    let mut cmd = Command::cargo_bin("obz").unwrap();
    cmd.env("HOME", test_home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--legacy")
        .arg("--b64")
        .arg("sometext")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--legacy is incompatible with encoding flags"));
    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_enc_dec_with_env_key() {
    let test_home = test_home_dir();

    // Encode using $OBORON_KEY env var (no ob init)
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64)
        .arg("enc")
        .arg("--aasv")
        .arg("--b32")
        .arg("env_key_test")
        .output()
        .unwrap();
    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode using $OBORON_KEY env var
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64)
        .arg("dec")
        .arg("--aasv")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("env_key_test"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_env_key_overridden_by_flag() {
    let test_home = test_home_dir();

    // Encode with explicit --key flag; env var holds a different key
    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64_ALT) // env var has different key
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_B64) // --key flag wins
        .arg("--aasv")
        .arg("--b32")
        .arg("flag_wins_test")
        .output()
        .unwrap();
    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode must use the --key flag key, not the env var key
    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64_ALT)
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_B64)
        .arg("--aasv")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("flag_wins_test"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "aasv")]
#[test]
fn test_env_key_without_config() {
    // Use a fresh HOME with no ~/.ob/ directory at all
    let test_home = test_home_dir();

    let mut enc_cmd = Command::cargo_bin("ob").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64)
        .arg("enc")
        .arg("--format")
        .arg("aasv.b32")
        .arg("no_config_test")
        .output()
        .unwrap();
    assert!(
        enc_output.status.success(),
        "enc failed: {}",
        String::from_utf8_lossy(&enc_output.stderr)
    );
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let mut dec_cmd = Command::cargo_bin("ob").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_B64)
        .arg("dec")
        .arg("--format")
        .arg("aasv.b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("no_config_test"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_enc_dec_with_env_secret() {
    let test_home = test_home_dir();

    // Encode using $OBORON_SECRET env var (no obz init)
    let mut enc_cmd = Command::cargo_bin("obz").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_SECRET", TEST_SECRET_B64)
        .arg("enc")
        .arg("--zrbcx")
        .arg("--b32")
        .arg("env_secret_test")
        .output()
        .unwrap();
    assert!(
        enc_output.status.success(),
        "enc failed: {}",
        String::from_utf8_lossy(&enc_output.stderr)
    );
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode using $OBORON_SECRET env var
    let mut dec_cmd = Command::cargo_bin("obz").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_SECRET", TEST_SECRET_B64)
        .arg("dec")
        .arg("--zrbcx")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("env_secret_test"));

    cleanup_test_home(&test_home);
}

#[cfg(feature = "zrbcx")]
#[test]
fn test_obz_env_secret_overridden_by_flag() {
    const TEST_SECRET_B64_ALT: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
    let test_home = test_home_dir();

    // Encode with explicit --secret flag; env var holds a different secret
    let mut enc_cmd = Command::cargo_bin("obz").unwrap();
    let enc_output = enc_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_SECRET", TEST_SECRET_B64_ALT) // env var has different secret
        .arg("enc")
        .arg("--secret")
        .arg(TEST_SECRET_B64) // --secret flag wins
        .arg("--zrbcx")
        .arg("--b32")
        .arg("obz_flag_wins_test")
        .output()
        .unwrap();
    assert!(enc_output.status.success());
    let encd = String::from_utf8(enc_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Decode must use the --secret flag secret
    let mut dec_cmd = Command::cargo_bin("obz").unwrap();
    dec_cmd
        .env("HOME", test_home.as_os_str())
        .env("OBORON_SECRET", TEST_SECRET_B64_ALT)
        .arg("dec")
        .arg("--secret")
        .arg(TEST_SECRET_B64)
        .arg("--zrbcx")
        .arg("--b32")
        .arg(&encd)
        .assert()
        .success()
        .stdout(predicate::str::contains("obz_flag_wins_test"));

    cleanup_test_home(&test_home);
}
