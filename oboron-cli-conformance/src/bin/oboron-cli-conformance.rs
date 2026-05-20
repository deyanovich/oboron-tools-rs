//! Cross-implementation conformance harness for the oboron
//! protocol CLI surface. Spawns the binaries (`ob`, `obz`,
//! `obc`) end-to-end and asserts behavior against the canonical
//! test vectors.
//!
//! Implementers of `ob` / `obz` / `obcrypt` in other languages
//! point this tool at their binaries to validate conformance:
//!
//! ```text
//! cargo install oboron-cli-conformance
//! oboron-cli-conformance --ob ./my-ob --obz ./my-obz --obc ./my-obc
//! ```
//!
//! Or, if all three binaries are on `$PATH`, no arguments are
//! needed:
//!
//! ```text
//! oboron-cli-conformance
//! ```

use clap::{Parser, ValueEnum};
use oboron_cli_conformance::{
    run_ob_smoke, run_ob_vectors, run_obc_vectors,
    run_obz_legacy_vectors, run_obz_smoke, run_obz_ztier_vectors,
    Config, Report, TestStatus,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "oboron-cli-conformance",
    version,
    about = "Conformance test runner for ob/obz/obcrypt CLIs"
)]
struct Cli {
    /// Path to the `ob` binary. Defaults to `ob` on `$PATH`.
    #[arg(long, value_name = "PATH")]
    ob: Option<PathBuf>,

    /// Path to the `obz` binary. Defaults to `obz` on `$PATH`.
    #[arg(long, value_name = "PATH")]
    obz: Option<PathBuf>,

    /// Path to the `obcrypt` (or legacy `obc`) binary. Defaults
    /// to `obc` on `$PATH`.
    #[arg(long, value_name = "PATH")]
    obc: Option<PathBuf>,

    /// Restrict to specific test suites. Repeatable. Defaults
    /// to all suites.
    #[arg(long, value_enum)]
    suite: Vec<Suite>,

    /// Print each individual test result. Default: only
    /// summary + failures.
    #[arg(long)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Suite {
    ObSmoke,
    ObVectors,
    ObcVectors,
    ObzSmoke,
    ObzZtierVectors,
    ObzLegacyVectors,
}

const ALL_SUITES: &[Suite] = &[
    Suite::ObSmoke,
    Suite::ObVectors,
    Suite::ObcVectors,
    Suite::ObzSmoke,
    Suite::ObzZtierVectors,
    Suite::ObzLegacyVectors,
];

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut cfg = Config::from_path();
    if let Some(p) = cli.ob {
        cfg = cfg.with_ob(p);
    }
    if let Some(p) = cli.obz {
        cfg = cfg.with_obz(p);
    }
    if let Some(p) = cli.obc {
        cfg = cfg.with_obc(p);
    }

    let suites: &[Suite] = if cli.suite.is_empty() {
        ALL_SUITES
    } else {
        &cli.suite
    };

    let mut overall = Report::default();

    for suite in suites {
        let (label, sub) = match suite {
            Suite::ObSmoke => ("ob smoke", run_ob_smoke(&cfg)),
            Suite::ObVectors => ("ob vectors", run_ob_vectors(&cfg)),
            Suite::ObcVectors => {
                ("obc vectors", run_obc_vectors(&cfg))
            }
            Suite::ObzSmoke => ("obz smoke", run_obz_smoke(&cfg)),
            Suite::ObzZtierVectors => (
                "obz ztier vectors",
                run_obz_ztier_vectors(&cfg),
            ),
            Suite::ObzLegacyVectors => (
                "obz legacy vectors",
                run_obz_legacy_vectors(&cfg),
            ),
        };
        println!(
            "[{label}] {} pass, {} fail, {} skip",
            sub.passed(),
            sub.failed(),
            sub.skipped(),
        );
        if cli.verbose {
            for r in &sub.results {
                match &r.status {
                    TestStatus::Pass => println!("  PASS  {}", r.name),
                    TestStatus::Skipped(why) => {
                        println!("  SKIP  {} — {why}", r.name)
                    }
                    TestStatus::Fail(why) => println!(
                        "  FAIL  {}\n        {}",
                        r.name,
                        why.replace('\n', "\n        ")
                    ),
                }
            }
        } else {
            for r in &sub.results {
                if let TestStatus::Fail(why) = &r.status {
                    println!(
                        "  FAIL  {}\n        {}",
                        r.name,
                        why.replace('\n', "\n        ")
                    );
                }
            }
        }
        overall.merge(sub);
    }

    println!();
    println!(
        "TOTAL: {} pass, {} fail, {} skip",
        overall.passed(),
        overall.failed(),
        overall.skipped(),
    );

    if overall.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
