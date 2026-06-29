//! Integration tests for the `ob` binary against oboron protocol
//! spec 1.0. Every scheme is authenticated; keys are 128-char hex;
//! `dec` uses the supplied scheme and never auto-detects.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn test_home_dir() -> PathBuf {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from(format!("./test_home_{}", test_id))
}

fn cleanup_test_home(dir: &PathBuf) {
    if dir.exists() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

// Two valid 128-character hex keys (64 bytes / 512 bits each).
const TEST_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const TEST_KEY_HEX_ALT: &str = "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

fn ob() -> Command {
    Command::cargo_bin("ob").unwrap()
}

// --- enc with the fixed public test key (-K / --keyless) --------------

#[test]
fn test_enc_keyless_default_format() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&home);
}

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_keyless_dsiv_b32() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--dsiv")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&home);
}

#[cfg(feature = "psiv")]
#[test]
fn test_enc_keyless_psiv() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--psiv")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    cleanup_test_home(&home);
}

// --- scheme short flags: -s/-S/-g/-G -----------------------------------

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_short_alias_dsiv() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-s")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success();
    cleanup_test_home(&home);
}

#[cfg(feature = "psiv")]
#[test]
fn test_enc_short_alias_psiv() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-S")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success();
    cleanup_test_home(&home);
}

#[cfg(feature = "dgcmsiv")]
#[test]
fn test_enc_short_alias_dgcmsiv() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-g")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success();
    cleanup_test_home(&home);
}

#[cfg(feature = "pgcmsiv")]
#[test]
fn test_enc_short_alias_pgcmsiv() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("-G")
        .arg("--b32")
        .arg("test123")
        .assert()
        .success();
    cleanup_test_home(&home);
}

// --- encoding short flags: -c/-b/-B/-x ---------------------------------

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_different_encodings() {
    let home = test_home_dir();
    for enc in ["-c", "-b", "-B", "-x"] {
        ob().env("HOME", home.as_os_str())
            .arg("enc")
            .arg("-K")
            .arg("--dsiv")
            .arg(enc)
            .arg("test")
            .assert()
            .success();
    }
    cleanup_test_home(&home);
}

// --- roundtrip with the public test key (probabilistic) ----------------

#[cfg(feature = "psiv")]
#[test]
fn test_enc_dec_roundtrip_keyless_psiv() {
    let home = test_home_dir();

    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--psiv")
        .arg("--b32")
        .arg("hello_world")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();
    assert!(!ot.is_empty());

    ob().env("HOME", home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--psiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_world"));

    cleanup_test_home(&home);
}

// --- explicit 128-hex --key -------------------------------------------

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_dec_with_explicit_hex_key() {
    let home = test_home_dir();

    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_HEX_ALT)
        .arg("--dsiv")
        .arg("--b32")
        .arg("sensitive_data")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();

    ob().env("HOME", home.as_os_str())
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_HEX_ALT)
        .arg("--dsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("sensitive_data"));

    cleanup_test_home(&home);
}

#[cfg(feature = "dgcmsiv")]
#[test]
fn test_enc_dec_with_explicit_hex_key_dgcmsiv() {
    let home = test_home_dir();

    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_HEX_ALT)
        .arg("--dgcmsiv")
        .arg("--b32")
        .arg("sensitive_data")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();

    ob().env("HOME", home.as_os_str())
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_HEX_ALT)
        .arg("--dgcmsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("sensitive_data"));

    cleanup_test_home(&home);
}

// --- a non-hex / wrong-length key is rejected --------------------------

#[test]
fn test_enc_rejects_short_key() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg("deadbeef") // too short to be a 128-hex key
        .arg("test")
        .assert()
        .failure();
    cleanup_test_home(&home);
}

// --- --format string and its mutual exclusion --------------------------

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_with_format_string() {
    let home = test_home_dir();
    for fmt in ["dsiv.b32", "dsiv.b64", "dsiv.hex"] {
        ob().env("HOME", home.as_os_str())
            .arg("enc")
            .arg("-K")
            .arg("--format")
            .arg(fmt)
            .arg("test_format")
            .assert()
            .success();
    }
    cleanup_test_home(&home);
}

#[cfg(feature = "dsiv")]
#[test]
fn test_format_conflicts_with_scheme_flag() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--format")
        .arg("dsiv.b32")
        .arg("--dsiv")
        .arg("test")
        .assert()
        .failure();
    cleanup_test_home(&home);
}

// --- dec does NOT auto-detect the scheme -------------------------------

#[cfg(all(feature = "dsiv", feature = "dgcmsiv"))]
#[test]
fn test_dec_does_not_autodetect_scheme() {
    let home = test_home_dir();

    // Encrypt under dsiv with the public test key.
    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--dsiv")
        .arg("--b32")
        .arg("scheme_locked")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();

    // Decrypting the same obtext while *declaring the wrong scheme*
    // (dgcmsiv) must fail — dec uses the supplied scheme and does not
    // trial-decrypt across schemes.
    ob().env("HOME", home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--dgcmsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .failure();

    // With the correct scheme it succeeds.
    ob().env("HOME", home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--dsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("scheme_locked"));

    cleanup_test_home(&home);
}

// --- $OBORON_KEY (128-hex) --------------------------------------------

#[cfg(feature = "dsiv")]
#[test]
fn test_enc_dec_with_env_key() {
    let home = test_home_dir();

    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_HEX)
        .arg("enc")
        .arg("--dsiv")
        .arg("--b32")
        .arg("env_key_test")
        .output()
        .unwrap();
    assert!(
        enc_out.status.success(),
        "enc failed: {}",
        String::from_utf8_lossy(&enc_out.stderr)
    );
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();

    ob().env("HOME", home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_HEX)
        .arg("dec")
        .arg("--dsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("env_key_test"));

    cleanup_test_home(&home);
}

#[cfg(feature = "dsiv")]
#[test]
fn test_env_key_overridden_by_flag() {
    let home = test_home_dir();

    // env holds one key, --key holds another; --key must win.
    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_HEX_ALT)
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_HEX)
        .arg("--dsiv")
        .arg("--b32")
        .arg("flag_wins_test")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let ot = String::from_utf8(enc_out.stdout).unwrap().trim().to_string();

    ob().env("HOME", home.as_os_str())
        .env("OBORON_KEY", TEST_KEY_HEX_ALT)
        .arg("dec")
        .arg("--key")
        .arg(TEST_KEY_HEX)
        .arg("--dsiv")
        .arg("--b32")
        .arg(&ot)
        .assert()
        .success()
        .stdout(predicate::str::contains("flag_wins_test"));

    cleanup_test_home(&home);
}

// --- --key conflicts with --keyless -----------------------------------

#[test]
fn test_key_and_keyless_conflict() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("enc")
        .arg("--key")
        .arg(TEST_KEY_HEX)
        .arg("-K")
        .arg("test")
        .assert()
        .failure();
    cleanup_test_home(&home);
}

// --- --raw framing round-trips exactly (no newline added/stripped) -----

#[cfg(feature = "dsiv")]
#[test]
fn test_raw_framing_roundtrip() {
    let home = test_home_dir();

    let enc_out = ob()
        .env("HOME", home.as_os_str())
        .arg("enc")
        .arg("-K")
        .arg("--dsiv")
        .arg("--b32")
        .arg("--raw")
        .write_stdin("payload-no-newline")
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    // In --raw mode stdout has no trailing newline.
    assert!(!enc_out.stdout.ends_with(b"\n"));
    let ot = String::from_utf8(enc_out.stdout).unwrap();

    let dec_out = ob()
        .env("HOME", home.as_os_str())
        .arg("dec")
        .arg("-K")
        .arg("--dsiv")
        .arg("--b32")
        .arg("--raw")
        .write_stdin(ot)
        .output()
        .unwrap();
    assert!(dec_out.status.success());
    assert_eq!(dec_out.stdout, b"payload-no-newline");

    cleanup_test_home(&home);
}

// --- keygen ------------------------------------------------------------

#[test]
fn test_keygen_prints_fresh_hex_key() {
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("keygen")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^[0-9a-f]{128}\n$").unwrap());
    cleanup_test_home(&home);
}

#[test]
fn test_keygen_differs_each_run() {
    let home = test_home_dir();
    let run = || {
        ob().env("HOME", home.as_os_str())
            .arg("keygen")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone()
    };
    assert_ne!(run(), run(), "two keygen runs should differ");
    cleanup_test_home(&home);
}

// --- --version single line --------------------------------------------

#[test]
fn test_version_line_format() {
    let home = test_home_dir();
    let out = ob()
        .env("HOME", home.as_os_str())
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let line = String::from_utf8(out.stdout).unwrap();
    // ob <impl> <version> protocol=1.0 cli=1.0, exactly one line.
    assert!(
        predicate::str::is_match(
            r"^ob oboron-tools-rs \d+\.\d+\.\d+ protocol=1\.0 cli=1\.0\n$"
        )
        .unwrap()
        .eval(&line),
        "unexpected --version output: {line:?}"
    );
    cleanup_test_home(&home);
}

#[test]
fn test_version_before_subcommand() {
    // --version must work with no subcommand and no key/stdin.
    let home = test_home_dir();
    ob().env("HOME", home.as_os_str())
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ob oboron-tools-rs "));
    cleanup_test_home(&home);
}
