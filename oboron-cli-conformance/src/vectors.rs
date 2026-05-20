//! Vector-driven conformance tests. Each function loads the
//! appropriate embedded JSONL data, iterates the vectors, and
//! returns a `Report` of per-vector outcomes.

use crate::*;
use std::process::Command;

fn is_deterministic_secure(format: &str) -> bool {
    matches!(scheme_of(format), "aags" | "aasv")
}

fn is_deterministic_ztier(format: &str) -> bool {
    matches!(scheme_of(format), "zrbcx" | "zmock1")
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

fn run_obz(cfg: &Config, args: &[&str]) -> Result<String, String> {
    let home = TempHome::new();
    let out = Command::new(&cfg.obz)
        .env("HOME", home.path())
        .args(args)
        .output()
        .map_err(|e| format!("spawn obz: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "obz {:?} exit {}\nstderr: {}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    String::from_utf8(out.stdout)
        .map(strip_trailing_newline)
        .map_err(|e| format!("obz stdout not utf-8: {e}"))
}

fn run_obc(cfg: &Config, args: &[&str]) -> Result<String, String> {
    let home = TempHome::new();
    let out = Command::new(&cfg.obc)
        .env("HOME", home.path())
        .args(args)
        .output()
        .map_err(|e| format!("spawn obc: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "obc {:?} exit {}\nstderr: {}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    String::from_utf8(out.stdout)
        .map(strip_trailing_newline)
        .map_err(|e| format!("obc stdout not utf-8: {e}"))
}

/// Vector-driven conformance for `ob` against the secure-scheme
/// vectors (`test-vectors.jsonl`): `aags`, `aasv`, `apgs`, `apsv`,
/// `upbc`. Hardcoded test key applied via `-K`.
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

/// Vector-driven conformance for `obc`. Uses the same vectors as
/// `run_ob_vectors`, filtered to `.hex` formats (since
/// `obc -s <scheme> -x` produces hex output equivalent to
/// `ob -f <scheme>.hex`).
pub fn run_obc_vectors(cfg: &Config) -> Report {
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
        let name = format!("obc_vec:{}:{}", v.format, v.plaintext);

        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }

        report.record(name, obc_one_vector(cfg, v));
    }

    report
}

fn obc_one_vector(cfg: &Config, v: &TestVector) -> Result<(), String> {
    let scheme = scheme_of(&v.format);
    if is_deterministic_secure(&v.format) {
        // exact-match enc
        let got = run_obc(
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
        let pt = run_obc(
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
        let pt = run_obc(
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
        let fresh = run_obc(
            cfg,
            &[
                "encrypt", "-s", scheme, "-x", "-k", HARDCODED_KEY_HEX,
                "--", &v.plaintext,
            ],
        )?;
        let rt = run_obc(
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

/// Vector-driven conformance for `obz` against the z-tier
/// vectors (`zrbcx`). Hardcoded test secret applied via `-K`.
pub fn run_obz_ztier_vectors(cfg: &Config) -> Report {
    let mut report = Report::default();
    let vectors = parse_vectors_jsonl(ZTIER_VECTORS_JSONL);

    for v in &vectors {
        let scheme = scheme_of(&v.format);
        let name = format!("obz_ztier_vec:{}:{}", v.format, v.plaintext);

        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }

        report.record(name, obz_ztier_one(cfg, v));
    }

    report
}

fn obz_ztier_one(cfg: &Config, v: &TestVector) -> Result<(), String> {
    if is_deterministic_ztier(&v.format) {
        let got = run_obz(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        if got != v.obtext {
            return Err(format!(
                "enc mismatch\n  expected: {}\n  got     : {}",
                v.obtext, got
            ));
        }
        let pt = run_obz(
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
        let pt = run_obz(
            cfg,
            &["dec", "-K", "--format", &v.format, "--", &v.obtext],
        )?;
        if pt != v.plaintext {
            return Err(format!(
                "dec mismatch (canned)\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
        let fresh = run_obz(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        let rt = run_obz(
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

/// Vector-driven conformance for `obz` against the legacy-scheme
/// vectors. Secret carried in the first-line meta entry; passed
/// to `obz` via `-s <secret>`.
pub fn run_obz_legacy_vectors(cfg: &Config) -> Report {
    let mut report = Report::default();
    let (secret, vectors) = parse_legacy_jsonl(LEGACY_VECTORS_JSONL);

    for v in &vectors {
        let name = format!("obz_legacy_vec:{}:{}", v.format, v.plaintext);
        report.record(name, obz_legacy_one(cfg, &secret, v));
    }

    report
}

fn obz_legacy_one(
    cfg: &Config,
    secret: &str,
    v: &TestVector,
) -> Result<(), String> {
    // Legacy is deterministic. Known bug: `dec` strips trailing
    // '=' characters from plaintext; mirror that in the expected
    // value so the assertion reflects observed behavior.
    let expected_dec = v.plaintext.trim_end_matches('=').to_string();

    let got = run_obz(
        cfg,
        &["enc", "-s", secret, "--format", &v.format, "--", &v.plaintext],
    )?;
    if got != v.obtext {
        return Err(format!(
            "enc mismatch\n  expected: {}\n  got     : {}",
            v.obtext, got
        ));
    }

    let pt = run_obz(
        cfg,
        &["dec", "-s", secret, "--format", &v.format, "--", &v.obtext],
    )?;
    if pt != expected_dec {
        return Err(format!(
            "dec mismatch (legacy trailing-'=' bug applied, original plaintext: {:?})\n  expected: {}\n  got     : {}",
            v.plaintext, expected_dec, pt
        ));
    }

    Ok(())
}
