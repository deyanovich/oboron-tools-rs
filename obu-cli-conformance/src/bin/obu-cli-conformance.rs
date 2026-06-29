//! Cross-implementation conformance harness for the **obu** CLI
//! surface. Spawns the `obu` binary end-to-end and asserts behavior
//! against the canonical obu test vectors.
//!
//! Implementers of `obu` in other languages point this tool at their
//! binary to validate conformance:
//!
//! ```text
//! cargo install obu-cli-conformance
//! obu-cli-conformance --obu ./my-obu
//! ```
//!
//! Or, if `obu` is on `$PATH`, no arguments are needed:
//!
//! ```text
//! obu-cli-conformance
//! ```
//!
//! The obu layer is unauthenticated; this suite is kept separate from
//! the authenticated `oboron-cli-conformance` suite by design.

use clap::Parser;
use obu_cli_conformance::{run_obu_vectors, ObuConfig, Report, TestStatus};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "obu-cli-conformance",
    version,
    about = "Conformance test runner for the obu CLI (unauthenticated layer)"
)]
struct Cli {
    /// Path to the `obu` binary. Defaults to `obu` on `$PATH`.
    #[arg(long, value_name = "PATH")]
    obu: Option<PathBuf>,

    /// Print each individual test result. Default: only summary +
    /// failures.
    #[arg(long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut cfg = ObuConfig::from_path();
    if let Some(p) = cli.obu {
        cfg = cfg.with_obu(p);
    }

    let report: Report = run_obu_vectors(&cfg);

    println!(
        "[obu vectors] {} pass, {} fail, {} skip",
        report.passed(),
        report.failed(),
        report.skipped(),
    );

    for r in &report.results {
        match &r.status {
            TestStatus::Pass => {
                if cli.verbose {
                    println!("  PASS  {}", r.name);
                }
            }
            TestStatus::Skipped(why) => {
                if cli.verbose {
                    println!("  SKIP  {} — {why}", r.name);
                }
            }
            TestStatus::Fail(why) => println!(
                "  FAIL  {}\n        {}",
                r.name,
                why.replace('\n', "\n        ")
            ),
        }
    }

    println!();
    println!(
        "TOTAL: {} pass, {} fail, {} skip",
        report.passed(),
        report.failed(),
        report.skipped(),
    );

    if report.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
