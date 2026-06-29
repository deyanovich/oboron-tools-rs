//! Cross-implementation conformance suite for the **obu** CLI surface
//! (the `obu` binary) — the oboron family's unauthenticated /
//! obfuscation layer.
//!
//! This crate is deliberately kept separate from the authenticated
//! `oboron-cli-conformance` suite: the insecure obu layer is segregated
//! from the secure core everywhere else (its own `obu` library, its own
//! `obu` binary), and its conformance is segregated too. The only
//! overlap is the scheme-agnostic test-runner plumbing
//! ([`Report`] / [`TempHome`] / the JSONL parser), which carries no
//! cryptographic meaning and is duplicated here so each conformance
//! crate is self-contained.
//!
//! Two entry points:
//!
//! - **Library**: `cargo test -p obu-cli-conformance` invokes the
//!   `tests/*.rs` integration suites, which delegate to
//!   [`run_obu_vectors`] using [`ObuConfig::from_path`] (the `obu`
//!   binary resolved via `$PATH`).
//! - **Binary**: `cargo install obu-cli-conformance` produces
//!   `obu-cli-conformance`, a CLI driver that takes an `--obu <path>`
//!   override and runs the same vectors, intended for
//!   alternative-language implementers to validate their `obu`.

use serde::Deserialize;
use std::path::PathBuf;

/// Returns `true` if `bin` can be spawned (probes `<bin> --version`).
///
/// The `tests/*.rs` wrapper uses this to **skip loudly** when `obu` is
/// not installed, rather than report spurious spawn failures. CI is
/// expected to install `obu` (or invoke the `obu-cli-conformance`
/// binary with an explicit `--obu` path) so coverage is not skipped.
pub fn binary_available(bin: &std::path::Path) -> bool {
    std::process::Command::new(bin)
        .arg("--version")
        .output()
        .is_ok()
}

/// The fixed public test secret (OBU spec §6.4), in canonical hex (64
/// chars). The obu vectors bind to this secret; the harness drives it
/// via `obu --keyless`, so this constant is documentation only. It is
/// the first 32 bytes of the core CLI's fixed public test key.
pub const KEYLESS_SECRET_HEX: &str =
    "381284633d02ea5f35df8596b5cc4218310060468e8b465455a415174ea6e966";

/// Embedded obu vector file — included at compile time so the
/// `cargo install`-ed binary works without external file lookups.
pub const OBU_TEST_VECTORS_JSONL: &str =
    include_str!("../tests/vectors/obu-test-vectors.jsonl");

/// Runtime configuration for the obu conformance run.
#[derive(Debug, Clone)]
pub struct ObuConfig {
    /// Path to the `obu` binary.
    pub obu: PathBuf,
    /// Which obu schemes to exercise.
    pub schemes: ObuSchemeFilter,
}

impl ObuConfig {
    /// Default config: resolve `obu` via `$PATH`.
    pub fn from_path() -> Self {
        Self {
            obu: "obu".into(),
            schemes: ObuSchemeFilter::all(),
        }
    }

    pub fn with_obu(mut self, obu: PathBuf) -> Self {
        self.obu = obu;
        self
    }

    pub fn with_schemes(mut self, schemes: ObuSchemeFilter) -> Self {
        self.schemes = schemes;
        self
    }
}

/// Which obu schemes to exercise. Default: everything compiled in
/// (controlled by the `upcbc` / `zdcbc` crate features).
#[derive(Debug, Clone, Copy)]
pub struct ObuSchemeFilter {
    pub upcbc: bool,
    pub zdcbc: bool,
}

impl ObuSchemeFilter {
    pub fn all() -> Self {
        Self {
            upcbc: cfg!(feature = "upcbc"),
            zdcbc: cfg!(feature = "zdcbc"),
        }
    }

    pub fn enabled(&self, scheme: &str) -> bool {
        match scheme {
            "upcbc" => self.upcbc,
            "zdcbc" => self.zdcbc,
            _ => false,
        }
    }
}

/// Outcome of running a single named test.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Pass,
    Fail(String),
    Skipped(String),
}

/// Accumulating result set for a `run_*` invocation.
#[derive(Debug, Default)]
pub struct Report {
    pub results: Vec<TestResult>,
}

impl Report {
    pub fn record(&mut self, name: impl Into<String>, outcome: Result<(), String>) {
        let status = match outcome {
            Ok(()) => TestStatus::Pass,
            Err(e) => TestStatus::Fail(e),
        };
        self.results.push(TestResult {
            name: name.into(),
            status,
        });
    }

    pub fn skip(&mut self, name: impl Into<String>, reason: impl Into<String>) {
        self.results.push(TestResult {
            name: name.into(),
            status: TestStatus::Skipped(reason.into()),
        });
    }

    pub fn merge(&mut self, other: Report) {
        self.results.extend(other.results);
    }

    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Pass))
            .count()
    }

    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Fail(_)))
            .count()
    }

    pub fn skipped(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Skipped(_)))
            .count()
    }

    pub fn is_success(&self) -> bool {
        self.failed() == 0
    }

    /// Panic with a multi-line failure report if any tests failed. Used
    /// by the `tests/*.rs` thin wrappers.
    pub fn assert_success(&self) {
        if !self.is_success() {
            let mut msg = format!(
                "{} passed, {} failed, {} skipped\n",
                self.passed(),
                self.failed(),
                self.skipped()
            );
            for r in &self.results {
                if let TestStatus::Fail(reason) = &r.status {
                    msg.push_str(&format!(
                        "\n  FAIL [{}]\n    {}\n",
                        r.name,
                        reason.replace('\n', "\n    "),
                    ));
                }
            }
            panic!("{msg}");
        }
    }
}

// ------------- internal helpers -------------

#[derive(Debug, Deserialize)]
struct TestVector {
    format: String,
    plaintext: String,
    obtext: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
}

fn parse_vectors_jsonl(data: &str) -> Vec<TestVector> {
    data.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse vector"))
        .collect()
}

fn strip_trailing_newline(s: String) -> String {
    if let Some(s) = s.strip_suffix('\n') {
        s.strip_suffix('\r').unwrap_or(s).to_string()
    } else {
        s
    }
}

fn scheme_of(format: &str) -> &str {
    format.split('.').next().unwrap_or("")
}

/// `zdcbc` is deterministic (exact enc/dec match); `upcbc` is
/// probabilistic (dec the stored obtext, then enc → dec roundtrip).
fn is_deterministic(format: &str) -> bool {
    scheme_of(format) == "zdcbc"
}

/// Per-test scratch HOME dir, so the run never touches a real
/// `~/.obu/`. Cleaned up on drop.
struct TempHome {
    path: PathBuf,
}

impl TempHome {
    fn new() -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("obu-conf-{id}-{}", std::process::id()));
        std::fs::create_dir_all(&path).expect("create temp home");
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_obu(cfg: &ObuConfig, args: &[&str]) -> Result<String, String> {
    use std::process::Command;
    let home = TempHome::new();
    let out = Command::new(&cfg.obu)
        .env("HOME", home.path())
        .args(args)
        .output()
        .map_err(|e| format!("spawn obu: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "obu {:?} exit {}\nstderr: {}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    String::from_utf8(out.stdout)
        .map(strip_trailing_newline)
        .map_err(|e| format!("obu stdout not utf-8: {e}"))
}

/// Vector-driven conformance for `obu` against the obu vectors
/// (`obu-test-vectors.jsonl`): `upcbc` and `zdcbc` over every encoding.
/// The fixed public test secret is applied via `-K`.
pub fn run_obu_vectors(cfg: &ObuConfig) -> Report {
    let mut report = Report::default();
    let vectors = parse_vectors_jsonl(OBU_TEST_VECTORS_JSONL);

    for v in &vectors {
        let scheme = scheme_of(&v.format);
        let name = format!("obu_vec:{}:{}", v.format, v.plaintext);

        if !cfg.schemes.enabled(scheme) {
            report.skip(name, format!("scheme {scheme} disabled"));
            continue;
        }

        report.record(name, obu_one_vector(cfg, v));
    }

    report
}

fn obu_one_vector(cfg: &ObuConfig, v: &TestVector) -> Result<(), String> {
    if is_deterministic(&v.format) {
        // zdcbc: exact-match enc, then exact-match dec.
        let got = run_obu(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        if got != v.obtext {
            return Err(format!(
                "enc mismatch\n  expected: {}\n  got     : {}",
                v.obtext, got
            ));
        }
        let pt = run_obu(cfg, &["dec", "-K", "--format", &v.format, "--", &v.obtext])?;
        if pt != v.plaintext {
            return Err(format!(
                "dec mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
    } else {
        // upcbc: dec the stored obtext, then enc → dec roundtrip.
        let pt = run_obu(cfg, &["dec", "-K", "--format", &v.format, "--", &v.obtext])?;
        if pt != v.plaintext {
            return Err(format!(
                "dec mismatch (stored)\n  expected: {}\n  got     : {}",
                v.plaintext, pt
            ));
        }
        let fresh = run_obu(
            cfg,
            &["enc", "-K", "--format", &v.format, "--", &v.plaintext],
        )?;
        let rt = run_obu(cfg, &["dec", "-K", "--format", &v.format, "--", &fresh])?;
        if rt != v.plaintext {
            return Err(format!(
                "roundtrip mismatch\n  expected: {}\n  got     : {}",
                v.plaintext, rt
            ));
        }
    }
    Ok(())
}
