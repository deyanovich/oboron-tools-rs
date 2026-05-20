//! Cross-implementation conformance test suite for the oboron
//! protocol CLI surface (`ob`, `obz`, `obcrypt`).
//!
//! Two entry points:
//!
//! - **Library**: `cargo test -p oboron-cli-conformance` invokes the
//!   `tests/*.rs` integration suites, which delegate to the lib's
//!   `run_*` functions using `Config::from_path()` (binaries
//!   resolved via `$PATH`).
//! - **Binary**: `cargo install oboron-cli-conformance` produces
//!   `oboron-cli-conformance`, a CLI driver that takes
//!   `--ob <path>` / `--obz <path>` / `--obc <path>` overrides
//!   and runs the same `run_*` functions, intended for
//!   alternative-language implementations to validate their
//!   `ob`/`obz`/`obcrypt` against the canonical vectors.

use serde::Deserialize;
use std::path::PathBuf;

pub mod smoke;
pub mod vectors;

pub use smoke::{run_ob_smoke, run_obz_smoke};
pub use vectors::{
    run_ob_vectors, run_obc_vectors, run_obz_legacy_vectors,
    run_obz_ztier_vectors,
};

/// Canonical hardcoded test key (hex). Matches
/// `oboron::HARDCODED_KEY_HEX`. Inlined so the conformance suite
/// has no dep on the implementation under test. The spec defines
/// this value in `CLI.md §8`.
pub const HARDCODED_KEY_HEX: &str = concat!(
    "38128463", "3d02ea5f", "35df8596", "b5cc4218",
    "31006046", "8e8b4654", "55a41517", "4ea6e966",
    "a9f48eec", "4ba446dd", "fc8b7858", "7895356f",
    "45a75a1a", "b7419454", "dd9f7aa8", "a95dbdd5",
);

/// Embedded vector files — included at compile time so the
/// `cargo install`-ed binary works without external file lookups.
pub const TEST_VECTORS_JSONL: &str =
    include_str!("../tests/vectors/test-vectors.jsonl");
pub const ZTIER_VECTORS_JSONL: &str =
    include_str!("../tests/vectors/ztier-test-vectors.jsonl");
pub const LEGACY_VECTORS_JSONL: &str =
    include_str!("../tests/vectors/legacy-test-vectors.jsonl");

/// Runtime configuration shared by every `run_*` function.
#[derive(Debug, Clone)]
pub struct Config {
    pub ob: PathBuf,
    pub obz: PathBuf,
    pub obc: PathBuf,
    pub schemes: SchemeFilter,
}

impl Config {
    /// Default config: resolve `ob`/`obz`/`obc` via `$PATH`. Used
    /// by both `cargo test` (where binaries are expected to be
    /// pre-installed) and the conformance binary (when no
    /// override flags are passed).
    pub fn from_path() -> Self {
        Self {
            ob: "ob".into(),
            obz: "obz".into(),
            obc: "obc".into(),
            schemes: SchemeFilter::all(),
        }
    }

    pub fn with_ob(mut self, ob: PathBuf) -> Self {
        self.ob = ob;
        self
    }
    pub fn with_obz(mut self, obz: PathBuf) -> Self {
        self.obz = obz;
        self
    }
    pub fn with_obc(mut self, obc: PathBuf) -> Self {
        self.obc = obc;
        self
    }
    pub fn with_schemes(mut self, schemes: SchemeFilter) -> Self {
        self.schemes = schemes;
        self
    }
}

/// Which schemes to exercise. Default: everything compiled in
/// (controlled by the `aags`/`aasv`/`apgs`/`apsv`/`upbc`/`ztier`
/// crate features).
#[derive(Debug, Clone, Copy)]
pub struct SchemeFilter {
    pub aags: bool,
    pub aasv: bool,
    pub apgs: bool,
    pub apsv: bool,
    pub upbc: bool,
    pub zrbcx: bool,
    pub legacy: bool,
}

impl SchemeFilter {
    pub fn all() -> Self {
        Self {
            aags: cfg!(feature = "aags"),
            aasv: cfg!(feature = "aasv"),
            apgs: cfg!(feature = "apgs"),
            apsv: cfg!(feature = "apsv"),
            upbc: cfg!(feature = "upbc"),
            zrbcx: cfg!(feature = "ztier"),
            legacy: cfg!(feature = "ztier"),
        }
    }

    pub fn enabled(&self, scheme: &str) -> bool {
        match scheme {
            "aags" => self.aags,
            "aasv" => self.aasv,
            "apgs" => self.apgs,
            "apsv" => self.apsv,
            "upbc" => self.upbc,
            "zrbcx" => self.zrbcx,
            "legacy" => self.legacy,
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
    pub fn record(
        &mut self,
        name: impl Into<String>,
        outcome: Result<(), String>,
    ) {
        let status = match outcome {
            Ok(()) => TestStatus::Pass,
            Err(e) => TestStatus::Fail(e),
        };
        self.results.push(TestResult { name: name.into(), status });
    }

    pub fn skip(
        &mut self,
        name: impl Into<String>,
        reason: impl Into<String>,
    ) {
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

    /// Panic with a multi-line failure report if any tests
    /// failed. Used by the `tests/*.rs` thin wrappers.
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

// ------------- internal helpers shared by submodules -------------

#[derive(Debug, Deserialize)]
pub(crate) struct TestVector {
    pub format: String,
    pub plaintext: String,
    pub obtext: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MetaEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub secret: Option<String>,
}

pub(crate) fn parse_vectors_jsonl(data: &str) -> Vec<TestVector> {
    data.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            // skip meta lines silently — only legacy file has one
            if l.contains("\"type\":\"meta\"")
                || l.contains("\"type\": \"meta\"")
            {
                None
            } else {
                Some(
                    serde_json::from_str(l).expect("parse vector"),
                )
            }
        })
        .collect()
}

pub(crate) fn parse_legacy_jsonl(data: &str) -> (String, Vec<TestVector>) {
    let mut lines = data.lines().filter(|l| !l.trim().is_empty());
    let first = lines.next().expect("empty legacy vectors file");
    let (secret, extra) =
        if let Ok(meta) = serde_json::from_str::<MetaEntry>(first) {
            if meta.entry_type == "meta" {
                (
                    meta.secret.expect("meta entry missing secret"),
                    None,
                )
            } else {
                (
                    String::new(),
                    Some(
                        serde_json::from_str::<TestVector>(first)
                            .expect("parse first vector"),
                    ),
                )
            }
        } else {
            (
                String::new(),
                Some(
                    serde_json::from_str::<TestVector>(first)
                        .expect("parse first vector"),
                ),
            )
        };
    let mut vectors: Vec<TestVector> = extra.into_iter().collect();
    vectors.extend(lines.map(|l| {
        serde_json::from_str::<TestVector>(l).expect("parse vector")
    }));
    (secret, vectors)
}

pub(crate) fn strip_trailing_newline(s: String) -> String {
    if let Some(s) = s.strip_suffix('\n') {
        s.strip_suffix('\r').unwrap_or(s).to_string()
    } else {
        s
    }
}

pub(crate) fn scheme_of(format: &str) -> &str {
    format.split('.').next().unwrap_or("")
}

/// Per-test scratch HOME dir for sandboxing `~/.oboron/`. Cleaned
/// up on drop.
pub(crate) struct TempHome {
    path: PathBuf,
}

impl TempHome {
    pub(crate) fn new() -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir()
            .join(format!("oboron-conf-{id}-{}", std::process::id()));
        std::fs::create_dir_all(&path).expect("create temp home");
        Self { path }
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
