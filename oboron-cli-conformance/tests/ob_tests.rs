//! Thin wrapper invoking the library's `run_ob_smoke`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{binary_available, run_ob_smoke, Config};

#[test]
fn test_ob_smoke() {
    let cfg = Config::from_path();
    if !binary_available(&cfg.ob) {
        eprintln!(
            "SKIPPED: `ob` not found on $PATH — ob conformance NOT validated. \
             Install ob, or run `oboron-cli-conformance --ob <path>`."
        );
        return;
    }
    run_ob_smoke(&cfg).assert_success();
}
