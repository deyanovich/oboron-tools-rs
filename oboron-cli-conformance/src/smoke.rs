//! Smoke / interface tests for `ob`. The `run_ob_smoke`
//! function exercises the CLI's flag parsing, scheme selection,
//! encoding defaults, keyless/explicit-key handling, roundtrip
//! correctness, and error paths.
//!
//! Mirrors the per-test structure of the original assert_cmd
//! suite; the per-test functions are private, the per-binary
//! runner (`run_ob_smoke`) is the public medium-grained entry
//! point used by both the `tests/*.rs` wrappers and the
//! `oboron-cli-conformance` binary.

use crate::*;
use std::path::Path;
use std::process::Command;

// 128-char hex (64 bytes) — valid placeholder key shape.
const TEST_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_KEY_HEX_ALT: &str =
    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

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
    run_if(&mut r, s.dsiv, "ob_enc_keyless_dsiv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--dsiv", "--b32", "test123"])
    });
    run_if(&mut r, s.psiv, "ob_enc_keyless_psiv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--psiv", "--b32", "test123"])
    });
    run_if(&mut r, s.dgcmsiv, "ob_enc_keyless_dgcmsiv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--dgcmsiv", "--b32", "test123"])
    });
    run_if(&mut r, s.pgcmsiv, "ob_enc_keyless_pgcmsiv", || {
        assert_success_nonempty(ob, &["enc", "-K", "--pgcmsiv", "--b32", "test123"])
    });

    // ---------- enc with explicit --key ----------
    run_if(&mut r, s.dsiv, "ob_enc_explicit_key_dsiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_HEX, "--dsiv", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.dgcmsiv, "ob_enc_explicit_key_dgcmsiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_HEX, "--dgcmsiv", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.pgcmsiv, "ob_enc_explicit_key_pgcmsiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_HEX, "--pgcmsiv", "--b32", "test_data"],
        )
    });
    run_if(&mut r, s.psiv, "ob_enc_explicit_key_psiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "--key", TEST_KEY_HEX, "--psiv", "--b32", "test_data"],
        )
    });

    // ---------- enc-dec roundtrip per scheme (keyless, b32) ----------
    run_if(&mut r, s.dsiv, "ob_enc_dec_roundtrip_dsiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--dsiv", "--b32", "hello_world"],
            &["dec", "-K", "--dsiv", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.dgcmsiv, "ob_enc_dec_roundtrip_dgcmsiv", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc",
                "--key",
                TEST_KEY_HEX_ALT,
                "--dgcmsiv",
                "--b32",
                "hello_world",
            ],
            &["dec", "--key", TEST_KEY_HEX_ALT, "--dgcmsiv", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.pgcmsiv, "ob_enc_dec_roundtrip_pgcmsiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--pgcmsiv", "--b32", "hello_world"],
            &["dec", "-K", "--pgcmsiv", "--b32"],
            "hello_world",
        )
    });
    run_if(&mut r, s.psiv, "ob_enc_dec_roundtrip_psiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--psiv", "--b32", "hello_world"],
            &["dec", "-K", "--psiv", "--b32"],
            "hello_world",
        )
    });

    // ---------- enc with all schemes / all encodings ----------
    let all_schemes =
        s.dgcmsiv && s.dsiv && s.pgcmsiv && s.psiv;
    run_if(&mut r, all_schemes, "ob_enc_all_schemes", || {
        for scheme in ["--dgcmsiv", "--dsiv", "--pgcmsiv", "--psiv"] {
            assert_success_nonempty(
                ob,
                &["enc", "-K", scheme, "--b32", "test"],
            )?;
        }
        Ok(())
    });
    run_if(&mut r, s.dsiv, "ob_enc_all_encodings", || {
        for enc in ["--b32", "--b64", "--hex"] {
            assert_success_nonempty(
                ob,
                &["enc", "-K", "--dsiv", enc, "test"],
            )?;
        }
        Ok(())
    });

    // ---------- short-alias scheme flags ----------
    run_if(&mut r, s.dsiv, "ob_enc_short_alias_dsiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "-K", "-s", "--b32", "test123"],
        )
    });
    run_if(&mut r, s.psiv, "ob_enc_short_alias_psiv", || {
        assert_success_nonempty(
            ob,
            &["enc", "-K", "-S", "--b32", "test123"],
        )
    });

    // ---------- invalid keys / missing args ----------
    run_if(&mut r, s.dsiv, "ob_enc_invalid_key_too_short", || {
        assert_failure(
            ob,
            &["enc", "--key", "TOOSHORT", "--dsiv", "--b32", "hello"],
        )
    });
    run_if(&mut r, s.dsiv, "ob_enc_invalid_key_empty", || {
        assert_failure(
            ob,
            &["enc", "--key", "", "--dsiv", "--b32", "hello"],
        )
    });
    run_if(&mut r, s.dsiv, "ob_dec_garbage_input", || {
        assert_failure(
            ob,
            &["dec", "-K", "--dsiv", "--b32", "notvalidobtext"],
        )
    });
    run_if(&mut r, s.dsiv, "ob_enc_missing_plaintext", || {
        assert_failure(ob, &["enc", "-K", "--dsiv", "--b32"])
    });

    // ---------- roundtrip with explicit key ----------
    run_if(&mut r, s.dsiv, "ob_enc_dec_roundtrip_explicit_key_dsiv", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc", "--key", TEST_KEY_HEX, "--dsiv", "--b32",
                "hello_key_world",
            ],
            &["dec", "--key", TEST_KEY_HEX, "--dsiv", "--b32"],
            "hello_key_world",
        )
    });
    run_if(&mut r, s.psiv, "ob_enc_dec_roundtrip_explicit_key_psiv", || {
        enc_then_dec_contains(
            ob,
            &[
                "enc", "--key", TEST_KEY_HEX, "--psiv", "--b32",
                "hello_key_world",
            ],
            &["dec", "--key", TEST_KEY_HEX, "--psiv", "--b32"],
            "hello_key_world",
        )
    });

    // ---------- roundtrip with different encodings ----------
    run_if(&mut r, s.dsiv, "ob_enc_dec_roundtrip_b64_dsiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--dsiv", "--b64", "hello_b64"],
            &["dec", "-K", "--dsiv", "--b64"],
            "hello_b64",
        )
    });
    run_if(&mut r, s.dsiv, "ob_enc_dec_roundtrip_hex_dsiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "--dsiv", "--hex", "hello_hex"],
            &["dec", "-K", "--dsiv", "--hex"],
            "hello_hex",
        )
    });

    // ---------- dec short alias ----------
    run_if(&mut r, s.dsiv, "ob_dec_short_alias_dsiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "-s", "--b32", "hello_alias_s"],
            &["dec", "-K", "-s", "--b32"],
            "hello_alias_s",
        )
    });
    run_if(&mut r, s.psiv, "ob_dec_short_alias_psiv", || {
        enc_then_dec_contains(
            ob,
            &["enc", "-K", "-S", "--b32", "hello_alias_S"],
            &["dec", "-K", "-S", "--b32"],
            "hello_alias_S",
        )
    });

    // ---------- different keys produce different output ----------
    run_if(&mut r, s.dsiv, "ob_enc_different_keys_differ", || {
        enc_with_two_keys_differ(
            ob,
            &["enc", "--key", TEST_KEY_HEX, "--dsiv", "--b32", "same_input"],
            &["enc", "--key", TEST_KEY_HEX_ALT, "--dsiv", "--b32", "same_input"],
        )
    });

    // ---------- empty plaintext (rejected) ----------
    run_if(&mut r, s.dsiv, "ob_enc_empty_plaintext_dsiv", || {
        assert_failure(ob, &["enc", "-K", "--dsiv", "--b32", ""])
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
