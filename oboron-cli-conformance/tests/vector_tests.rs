//! Thin wrapper invoking the library's `run_ob_vectors`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{binary_available, run_ob_vectors, Config};

#[test]
fn test_all_vectors() {
    let cfg = Config::from_path();
    if !binary_available(&cfg.ob) {
        eprintln!(
            "SKIPPED: `ob` not found on $PATH — ob conformance NOT validated. \
             Install ob, or run `oboron-cli-conformance --ob <path>`."
        );
        return;
    }
    run_ob_vectors(&cfg).assert_success();
}
