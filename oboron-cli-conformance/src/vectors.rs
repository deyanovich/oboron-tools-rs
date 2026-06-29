//! Vector-driven conformance tests. Each function loads the
//! appropriate embedded JSONL data, iterates the vectors, and
//! returns a `Report` of per-vector outcomes.

use crate::*;
use std::process::Command;

fn is_deterministic_secure(format: &str) -> bool {
    matches!(scheme_of(format), "dgcmsiv" | "dsiv")
}

fn run_ob(cfg: &Config, args: &[&str]) -> Result<String, String> {
    let home = TempHome::new();
    let out = Command::new(&cfg.ob)
        .env("HOME", home.path())
        .args(args)
        .output()
        .map_err(|e| format!("spawn ob: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "ob {:?} exit {}\nstderr: {}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    String::from_utf8(out.stdout)
        .map(strip_trailing_newline)
        .map_err(|e| format!("ob stdout not utf-8: {e}"))
}

fn run_obcrypt(cfg: &Config, args: &[&str]) -> Result<String, String> {
    let home = TempHome::new();
    let out = Command::new(&cfg.obcrypt)
        .env("HOME", home.path())
        .args(args)
        .output()
        .map_err(|e| format!("spawn obcrypt: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "obcrypt {:?} exit {}\nstderr: {}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    String::from_utf8(out.stdout)
        .map(strip_trailing_newline)
        .map_err(|e| format!("obcrypt stdout not utf-8: {e}"))
}

/// Vector-driven conformance for `ob` against the core-scheme
/// vectors (`test-vectors.jsonl`): `dgcmsiv`, `dsiv`, `pgcmsiv`,
/// `psiv`. Hardcoded test key applied via `-K`.
pub fn run_ob_vectors(cfg: &Config) -> Report {
    let mut report = Report::default();
    let vectors = parse_vectors_jsonl(TEST_VECTORS_JSONL);

    for v in &vectors {
        let scheme = scheme_of(&v.format);
        let name = format!("ob_vec:{}:{}", v.format, v.plaintext);

        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }

        report.record(name, ob_one_vector(cfg, v));
    }

    report
}

fn ob_one_vector(cfg: &Config, v: &TestVector) -> Result<(), String> {
    if is_deterministic_secure(&v.format) {
        // exact-match enc
        let got = run_ob(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        if got != v.obtext {
            return Err(format!(
                "enc mismatch\n  expected: {}\n  got     : {}",
                v.obtext, got
            ));
        }
        // exact-match dec
        let pt = run_ob(
            cfg,
            &["dec", "-K", "--format", &v.format, "--", &v.obtext],
        )?;
        if pt != v.plaintext {
            return Err(format!(
                "dec mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
    } else {
        // probabilistic: dec the canned obtext, then enc → dec roundtrip
        let pt = run_ob(
            cfg,
            &["dec", "-K", "--format", &v.format, "--", &v.obtext],
        )?;
        if pt != v.plaintext {
            return Err(format!(
                "dec mismatch (canned)\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
        let fresh = run_ob(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        let rt = run_ob(
            cfg,
            &["dec", "-K", "--format", &v.format, "--", &fresh],
        )?;
        if rt != v.plaintext {
            return Err(format!(
                "roundtrip mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, rt
            ));
        }
    }
    Ok(())
}

/// Vector-driven conformance for `obcrypt`. Uses the same vectors as
/// `run_ob_vectors`, filtered to `.hex` formats (since
/// `obcrypt -s <scheme> -x` produces hex output equivalent to
/// `ob -f <scheme>.hex`).
pub fn run_obcrypt_vectors(cfg: &Config) -> Report {
    let mut report = Report::default();
    let vectors: Vec<&TestVector> = parse_vectors_jsonl(TEST_VECTORS_JSONL)
        .into_iter()
        .filter(|v| v.format.ends_with(".hex"))
        .collect::<Vec<_>>()
        .leak()
        .iter()
        .collect();

    for v in &vectors {
        let scheme = scheme_of(&v.format);
        let name = format!("obcrypt_vec:{}:{}", v.format, v.plaintext);

        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }

        report.record(name, obcrypt_one_vector(cfg, v));
    }

    report
}

#[derive(Debug, serde::Deserialize)]
struct NegativeVector {
    op: String,
    format: String,
    input: String,
    #[serde(default)]
    #[allow(dead_code)]
    reason: Option<String>,
}

/// Negative-vector conformance for `ob` (CLI.md §10): each input **MUST**
/// fail its `op` as an operation failure — exit status `1`. Exit `2`
/// (usage) or `0` (success) is a conformance failure. This also pins
/// the uniform `dec`-failure contract (§8): a non-canonical encoding and
/// an authentication failure both surface as the same operation failure.
/// Covers core schemes only; obu negatives belong to the obu suite.
pub fn run_ob_negative(cfg: &Config) -> Report {
    let mut report = Report::default();
    let vectors: Vec<NegativeVector> = NEGATIVE_TEST_VECTORS_JSONL
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse negative vector"))
        .collect();

    for nv in &vectors {
        let scheme = scheme_of(&nv.format);
        let name = format!("ob_neg:{}:{}:{}", nv.op, nv.format, nv.input);
        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }
        report.record(name, ob_one_negative(cfg, nv));
    }
    report
}

fn ob_one_negative(cfg: &Config, nv: &NegativeVector) -> Result<(), String> {
    let home = TempHome::new();
    let out = Command::new(&cfg.ob)
        .env("HOME", home.path())
        .args([
            nv.op.as_str(),
            "-K",
            "--format",
            &nv.format,
            "--",
            &nv.input,
        ])
        .output()
        .map_err(|e| format!("spawn ob: {e}"))?;
    match out.status.code() {
        Some(1) => Ok(()),
        other => Err(format!(
            "expected exit 1 (operation failure), got {other:?}\nstderr: {}",
            String::from_utf8_lossy(&out.stderr),
        )),
    }
}

fn obcrypt_one_vector(cfg: &Config, v: &TestVector) -> Result<(), String> {
    let scheme = scheme_of(&v.format);
    if is_deterministic_secure(&v.format) {
        // exact-match enc
        let got = run_obcrypt(
            cfg,
            &[
                "encrypt", "-s", scheme, "-x", "-k", HARDCODED_KEY_HEX,
                "--", &v.plaintext,
            ],
        )?;
        if got != v.obtext {
            return Err(format!(
                "encrypt mismatch\n  expected: {}\n  got     : {}",
                v.obtext, got
            ));
        }
        // dec
        let pt = run_obcrypt(
            cfg,
            &[
                "decrypt", "-s", scheme, "-X", "-k", HARDCODED_KEY_HEX,
                "--", &v.obtext,
            ],
        )?;
        if pt != v.plaintext {
            return Err(format!(
                "decrypt mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
    } else {
        // probabilistic
        let pt = run_obcrypt(
            cfg,
            &[
                "decrypt", "-s", scheme, "-X", "-k", HARDCODED_KEY_HEX,
                "--", &v.obtext,
            ],
        )?;
        if pt != v.plaintext {
            return Err(format!(
                "decrypt mismatch (canned)\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
        let fresh = run_obcrypt(
            cfg,
            &[
                "encrypt", "-s", scheme, "-x", "-k", HARDCODED_KEY_HEX,
                "--", &v.plaintext,
            ],
        )?;
        let rt = run_obcrypt(
            cfg,
            &[
                "decrypt", "-s", scheme, "-X", "-k", HARDCODED_KEY_HEX,
                "--", &fresh,
            ],
        )?;
        if rt != v.plaintext {
            return Err(format!(
                "roundtrip mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, rt
            ));
        }
    }
    Ok(())
}
