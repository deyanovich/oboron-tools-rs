//! Cross-implementation conformance harness for the oboron
//! protocol CLI surface. Spawns the binaries (`ob`, `obcrypt`)
//! end-to-end and asserts behavior against the canonical test
//! vectors.
//!
//! Implementers of `ob` / `obcrypt` in other languages point this
//! tool at their binaries to validate conformance:
//!
//! ```text
//! cargo install oboron-cli-conformance
//! oboron-cli-conformance --ob ./my-ob --obcrypt ./my-obcrypt
//! ```
//!
//! Or, if both binaries are on `$PATH`, no arguments are
//! needed:
//!
//! ```text
//! oboron-cli-conformance
//! ```

use clap::{Parser, ValueEnum};
use oboron_cli_conformance::{
    run_ob_negative, run_ob_smoke, run_ob_vectors, run_obcrypt_vectors, Config,
    Report, TestStatus,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "oboron-cli-conformance",
    version,
    about = "Conformance test runner for ob/obcrypt CLIs"
)]
struct Cli {
    /// Path to the `ob` binary. Defaults to `ob` on `$PATH`.
    #[arg(long, value_name = "PATH")]
    ob: Option<PathBuf>,

    /// Path to the `obcrypt` binary. Defaults to `obcrypt` on
    /// `$PATH`.
    #[arg(long, value_name = "PATH")]
    obcrypt: Option<PathBuf>,

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
    ObNegative,
    ObcryptVectors,
}

const ALL_SUITES: &[Suite] = &[
    Suite::ObSmoke,
    Suite::ObVectors,
    Suite::ObNegative,
    Suite::ObcryptVectors,
];

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut cfg = Config::from_path();
    if let Some(p) = cli.ob {
        cfg = cfg.with_ob(p);
    }
    if let Some(p) = cli.obcrypt {
        cfg = cfg.with_obcrypt(p);
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
            Suite::ObNegative => ("ob negative", run_ob_negative(&cfg)),
            Suite::ObcryptVectors => {
                ("obcrypt vectors", run_obcrypt_vectors(&cfg))
            }
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
