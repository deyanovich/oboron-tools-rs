//! Thin wrapper invoking the library's `run_ob_negative`. The same
//! logic is exposed through the `oboron-cli-conformance` binary for
//! cross-language implementers. Asserts every negative vector fails
//! `ob` with exit status 1 (CLI.md §10).

use oboron_cli_conformance::{binary_available, run_ob_negative, Config};

#[test]
fn test_all_negative_vectors() {
    let cfg = Config::from_path();
    if !binary_available(&cfg.ob) {
        eprintln!(
            "SKIPPED: `ob` not found on $PATH — negative conformance NOT \
             validated. Install ob, or run `oboron-cli-conformance --ob <path>`."
        );
        return;
    }
    run_ob_negative(&cfg).assert_success();
}
