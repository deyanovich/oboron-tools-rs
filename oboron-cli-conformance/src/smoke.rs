//! Smoke / interface tests for `ob` and `obz`. Each `run_*_smoke`
//! function exercises the CLI's flag parsing, scheme selection,
//! encoding defaults, keyless/explicit-key handling, roundtrip
//! correctness, and error paths.
//!
//! Mirrors the per-test structure of the original assert_cmd
//! suite; the per-test functions are private, the per-binary
//! runners (`run_ob_smoke`, `run_obz_smoke`) are the public
//! medium-grained entry points used by both the `tests/*.rs`
//! wrappers and the `oboron-cli-conformance` binary.

use crate::*;
use std::path::Path;
use std::process::Command;

// 86-char base64url, 64 bytes — valid placeholder key shape.
const TEST_KEY_B64: &str =
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const TEST_KEY_B64_ALT: &str =
    "ZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
// 43-char base64url, 32 bytes — valid z-tier secret shape.
const TEST_SECRET: &str =
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const TEST_SECRET_ALT: &str =
    "ZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

// ------------------- shared command helpers -------------------

fn spawn_in_sandbox(
    bin: &Path,
    home: &Path,
    args: &[&str],
) -> Result<std::process::Output, String> {
    Command::new(bin)
        .env("HOME", home)
        .args(args)
        .output()
        .map_err(|e| format!("spawn {bin:?}: {e}"))
}

fn assert_success_nonempty(
    bin: &Path,
    args: &[&str],
) -> Result<(), String> {
    let home = TempHome::new();
    let out = spawn_in_sandbox(bin, home.path(), args)?;
    if !out.status.success() {
        return Err(format!(
            "{bin:?} {args:?} exit {}; stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    if out.stdout.iter().all(|b| b.is_ascii_whitespace()) {
        return Err("empty stdout".into());
    }
    Ok(())
}

fn assert_failure(bin: &Path, args: &[&str]) -> Result<(), String> {
    let home = TempHome::new();
    let out = spawn_in_sandbox(bin, home.path(), args)?;
    if out.status.success() {
        return Err(format!(
            "expected {bin:?} {args:?} to fail, but it succeeded; stdout: {}",
            String::from_utf8_lossy(&out.stdout),
        ));
    }
    Ok(())
}

fn enc_then_dec_contains(
    bin: &Path,
    enc_args: &[&str],
    dec_args_prefix: &[&str],
    expected_substr: &str,
) -> Result<(), String> {
    let home = TempHome::new();
    let enc_out = spawn_in_sandbox(bin, home.path(), enc_args)?;
    if !enc_out.status.success() {
        return Err(format!(
            "enc {enc_args:?} failed: {}",
            String::from_utf8_lossy(&enc_out.stderr),
        ));
    }
    let encd = strip_trailing_newline(
        String::from_utf8(enc_out.stdout)
            .map_err(|e| format!("enc stdout not utf-8: {e}"))?,
    );
    if encd.is_empty() {
        return Err("enc produced empty obtext".into());
    }
    let mut dec_args: Vec<&str> = dec_args_prefix.to_vec();
    dec_args.push(&encd);
    let dec_out = spawn_in_sandbox(bin, home.path(), &dec_args)?;
    if !dec_out.status.success() {
        return Err(format!(
            "dec {dec_args:?} failed: {}",
            String::from_utf8_lossy(&dec_out.stderr),
        ));
    }
    let dec_str = String::from_utf8_lossy(&dec_out.stdout).to_string();
    if !dec_str.contains(expected_substr) {
        return Err(format!(
            "dec output {dec_str:?} missing substring {expected_substr:?}",
        ));
    }
    Ok(())
}

fn enc_with_two_keys_differ(
    bin: &Path,
    args_a: &[&str],
    args_b: &[&str],
) -> Result<(), String> {
    let home = TempHome::new();
    let out_a = spawn_in_sandbox(bin, home.path(), args_a)?;
    if !out_a.status.success() {
        return Err(format!(
            "enc(A) failed: {}",
            String::from_utf8_lossy(&out_a.stderr),
        ));
    }
    let out_b = spawn_in_sandbox(bin, home.path(), args_b)?;
    if !out_b.status.success() {
        return Err(format!(
            "enc(B) failed: {}",
            String::from_utf8_lossy(&out_b.stderr),
        ));
    }
    let a = strip_trailing_newline(
        String::from_utf8_lossy(&out_a.stdout).to_string(),
    );
    let b = strip_trailing_newline(
        String::from_utf8_lossy(&out_b.stdout).to_string(),
    );
    if a == b {
        return Err(format!(
            "expected different obtexts under different keys; got identical: {a:?}"
        ));
    }
    Ok(())
}

// ------------- record helper to keep the runner concise -------------

fn run_if(
    report: &mut Report,
    enabled: bool,
    name: &str,
    f: impl FnOnce() -> Result<(), String>,
) {
    if enabled {
        report.record(name, f());
    } else {
        report.skip(name, "scheme disabled");
    }
}

// ====================== ob smoke ======================

pub fn run_ob_smoke(cfg: &Config) -> Report {
    let mut r = Report::default();
    let ob = cfg.ob.as_path();
    let s = cfg.schemes;

    // ---------- enc keyless (per scheme) ----------
    run_if(&mut r, s.aasv, "ob_enc_keyless_aasv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--aasv", "--b32", "test123"])
    });
    run_if(&mut r, s.apsv, "ob_enc_keyless_apsv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--apsv", "--b32", "test123"])
    });
    run_if(&mut r, s.aags, "ob_enc_keyless_aags", || {
        assert_success_nonempty(ob, &["enc", "-K", "--aags", "--b32", "test123"])
    });
    run_if(&mut r, s.apgs, "ob_enc_keyless_apgs", || {
        assert_success_nonempty(ob, &["enc", "-K", "--apgs", "--b32", "test123"])
    });
    run_if(&mut r, s.upbc, "ob_enc_keyless_upbc", || {
        assert_success_nonempty(ob, &["enc", "-K", "--upbc", "--b32", "test123"])
    });

    // ---------- enc with explicit --key ----------
    run_if(&mut r, s.aasv, "ob_enc_explicit_key_aasv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--aasv", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.aags, "ob_enc_explicit_key_aags", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--aags", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.apgs, "ob_enc_explicit_key_apgs", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--apgs", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.apsv, "ob_enc_explicit_key_apsv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--apsv", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.upbc, "ob_enc_explicit_key_upbc", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--upbc", "--b32", "test_data"],
        )
    });

    // ---------- enc-dec roundtrip per scheme (keyless, b32) ----------
    run_if(&mut r, s.aasv, "ob_enc_dec_roundtrip_aasv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--aasv", "--b32", "hello_world"],
            &["dec", "-K", "--aasv", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.aags, "ob_enc_dec_roundtrip_aags", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc",
                "--key",
                TEST_KEY_B64_ALT,
                "--aags",
                "--b32",
                "hello_world",
            ],
            &["dec", "--key", TEST_KEY_B64_ALT, "--aags", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.apgs, "ob_enc_dec_roundtrip_apgs", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--apgs", "--b32", "hello_world"],
            &["dec", "-K", "--apgs", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.apsv, "ob_enc_dec_roundtrip_apsv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--apsv", "--b32", "hello_world"],
            &["dec", "-K", "--apsv", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.upbc, "ob_enc_dec_roundtrip_upbc", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--upbc", "--b32", "hello_world"],
            &["dec", "-K", "--upbc", "--b32"],
            "hello_world",
        )
    });

    // ---------- enc with all schemes / all encodings ----------
    let all_secure =
        s.aags && s.aasv && s.upbc && s.apgs && s.apsv;
    run_if(&mut r, all_secure, "ob_enc_all_schemes", || {
        for scheme in ["--aags", "--aasv", "--upbc", "--apgs", "--apsv"] {
            assert_success_nonempty(
                ob,
                &["enc", "-K", scheme, "--b32", "test"],
            )?;
        }
        Ok(())
    });
    run_if(&mut r, s.aasv, "ob_enc_all_encodings", || {
        for enc in ["--b32", "--b64", "--hex"] {
            assert_success_nonempty(
                ob,
                &["enc", "-K", "--aasv", enc, "test"],
            )?;
        }
        Ok(())
    });

    // ---------- short-alias scheme flags ----------
    run_if(&mut r, s.aasv, "ob_enc_short_alias_aasv", || {
        assert_success_nonempty(
            ob,
            &["enc", "-K", "-s", "--b32", "test123"],
        )
    });
    run_if(&mut r, s.apsv, "ob_enc_short_alias_apsv", || {
        assert_success_nonempty(
            ob,
            &["enc", "-K", "-S", "--b32", "test123"],
        )
    });
    run_if(&mut r, s.upbc, "ob_enc_short_alias_upbc", || {
        assert_success_nonempty(
            ob,
            &["enc", "-K", "-u", "--b32", "test123"],
        )
    });
    run_if(&mut r, s.upbc, "ob_dec_short_alias_upbc", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "-u", "--b32", "hello123"],
            &["dec", "-K", "-u", "--b32"],
            "hello123",
        )
    });

    // ---------- invalid keys / missing args ----------
    run_if(&mut r, s.aasv, "ob_enc_invalid_key_too_short", || {
        assert_failure(
            ob,
            &["enc", "--key", "TOOSHORT", "--aasv", "--b32", "hello"],
        )
    });
    run_if(&mut r, s.aasv, "ob_enc_invalid_key_empty", || {
        assert_failure(
            ob,
            &["enc", "--key", "", "--aasv", "--b32", "hello"],
        )
    });
    run_if(&mut r, s.aasv, "ob_dec_garbage_input", || {
        assert_failure(
            ob,
            &["dec", "-K", "--aasv", "--b32", "notvalidobtext"],
        )
    });
    run_if(&mut r, s.aasv, "ob_enc_missing_plaintext", || {
        assert_failure(ob, &["enc", "-K", "--aasv", "--b32"])
    });

    // ---------- roundtrip with explicit key ----------
    run_if(&mut r, s.aasv, "ob_enc_dec_roundtrip_explicit_key_aasv", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc", "--key", TEST_KEY_B64, "--aasv", "--b32",
                "hello_key_world",
            ],
            &["dec", "--key", TEST_KEY_B64, "--aasv", "--b32"],
            "hello_key_world",
        )
    });
    run_if(&mut r, s.apsv, "ob_enc_dec_roundtrip_explicit_key_apsv", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc", "--key", TEST_KEY_B64, "--apsv", "--b32",
                "hello_key_world",
            ],
            &["dec", "--key", TEST_KEY_B64, "--apsv", "--b32"],
            "hello_key_world",
        )
    });
    run_if(&mut r, s.upbc, "ob_enc_dec_roundtrip_explicit_key_upbc", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc", "--key", TEST_KEY_B64, "--upbc", "--b32",
                "hello_key_world",
            ],
            &["dec", "--key", TEST_KEY_B64, "--upbc", "--b32"],
            "hello_key_world",
        )
    });

    // ---------- roundtrip with different encodings ----------
    run_if(&mut r, s.aasv, "ob_enc_dec_roundtrip_b64_aasv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--aasv", "--b64", "hello_b64"],
            &["dec", "-K", "--aasv", "--b64"],
            "hello_b64",
        )
    });
    run_if(&mut r, s.aasv, "ob_enc_dec_roundtrip_hex_aasv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--aasv", "--hex", "hello_hex"],
            &["dec", "-K", "--aasv", "--hex"],
            "hello_hex",
        )
    });

    // ---------- dec short alias ----------
    run_if(&mut r, s.aasv, "ob_dec_short_alias_aasv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "-s", "--b32", "hello_alias_s"],
            &["dec", "-K", "-s", "--b32"],
            "hello_alias_s",
        )
    });
    run_if(&mut r, s.apsv, "ob_dec_short_alias_apsv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "-S", "--b32", "hello_alias_S"],
            &["dec", "-K", "-S", "--b32"],
            "hello_alias_S",
        )
    });

    // ---------- different keys produce different output ----------
    run_if(&mut r, s.aasv, "ob_enc_different_keys_differ", || {
        enc_with_two_keys_differ(
            ob,
            &["enc", "--key", TEST_KEY_B64, "--aasv", "--b32", "same_input"],
            &["enc", "--key", TEST_KEY_B64_ALT, "--aasv", "--b32", "same_input"],
        )
    });

    // ---------- empty plaintext (rejected) ----------
    run_if(&mut r, s.aasv, "ob_enc_empty_plaintext_aasv", || {
        assert_failure(ob, &["enc", "-K", "--aasv", "--b32", ""])
    });

    // ---------- --help ----------
    r.record("ob_help", help_check(ob));

    r
}

fn help_check(bin: &Path) -> Result<(), String> {
    let out = Command::new(bin)
        .arg("--help")
        .output()
        .map_err(|e| format!("spawn --help: {e}"))?;
    if !out.status.success() {
        return Err(format!("--help exit {}", out.status));
    }
    if out.stdout.is_empty() {
        return Err("--help produced empty stdout".into());
    }
    Ok(())
}

// ====================== obz smoke ======================

pub fn run_obz_smoke(cfg: &Config) -> Report {
    let mut r = Report::default();
    let obz = cfg.obz.as_path();
    let ztier = cfg.schemes.zrbcx;

    run_if(&mut r, ztier, "obz_enc_keyless", || {
        assert_success_nonempty(
            obz,
            &["enc", "-K", "--zrbcx", "--b32", "test123"],
        )
    });
    run_if(&mut r, ztier, "obz_enc_with_explicit_key", || {
        assert_success_nonempty(
            obz,
            &[
                "enc",
                "--secret",
                TEST_SECRET,
                "--zrbcx",
                "--b32",
                "test_data",
            ],
        )
    });
    run_if(&mut r, ztier, "obz_enc_dec_roundtrip", || {
        enc_then_dec_contains(
            obz,
            &["enc", "-K", "--zrbcx", "--b32", "hello_obz"],
            &["dec", "-K", "--zrbcx", "--b32"],
            "hello_obz",
        )
    });
    run_if(&mut r, ztier, "obz_enc_dec_roundtrip_b64", || {
        enc_then_dec_contains(
            obz,
            &["enc", "-K", "--zrbcx", "--b64", "hello_b64"],
            &["dec", "-K", "--zrbcx", "--b64"],
            "hello_b64",
        )
    });
    run_if(&mut r, ztier, "obz_enc_dec_roundtrip_hex", || {
        enc_then_dec_contains(
            obz,
            &["enc", "-K", "--zrbcx", "--hex", "hello_hex"],
            &["dec", "-K", "--zrbcx", "--hex"],
            "hello_hex",
        )
    });
    run_if(&mut r, ztier, "obz_enc_dec_roundtrip_explicit_key", || {
        enc_then_dec_contains(
            obz,
            &[
                "enc", "--secret", TEST_SECRET, "--zrbcx", "--b32",
                "hello_key",
            ],
            &["dec", "--secret", TEST_SECRET, "--zrbcx", "--b32"],
            "hello_key",
        )
    });
    run_if(&mut r, ztier, "obz_enc_invalid_key_too_short", || {
        assert_failure(
            obz,
            &[
                "enc", "--secret", "TOOSHORT", "--zrbcx", "--b32", "hello",
            ],
        )
    });
    run_if(&mut r, ztier, "obz_enc_invalid_key_empty", || {
        assert_failure(
            obz,
            &["enc", "--secret", "", "--zrbcx", "--b32", "hello"],
        )
    });
    run_if(&mut r, ztier, "obz_dec_garbage_input", || {
        assert_failure(
            obz,
            &["dec", "-K", "--zrbcx", "--b32", "notvalidobtext"],
        )
    });
    run_if(&mut r, ztier, "obz_enc_missing_plaintext", || {
        assert_failure(obz, &["enc", "-K", "--zrbcx", "--b32"])
    });
    run_if(&mut r, ztier, "obz_enc_different_keys_differ", || {
        enc_with_two_keys_differ(
            obz,
            &[
                "enc", "--secret", TEST_SECRET, "--zrbcx", "--b32",
                "same_input",
            ],
            &[
                "enc", "--secret", TEST_SECRET_ALT, "--zrbcx", "--b32",
                "same_input",
            ],
        )
    });
    run_if(&mut r, ztier, "obz_enc_empty_plaintext", || {
        assert_failure(obz, &["enc", "-K", "--zrbcx", "--b32", ""])
    });
    r.record("obz_help", help_check(obz));

    r
}
