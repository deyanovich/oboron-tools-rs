//! Thin wrapper invoking the library's `run_obu_vectors`. The same
//! logic is exposed through the `obu-cli-conformance` binary for
//! cross-language implementers. Requires the `obu` binary on `$PATH`
//! (or run the binary directly with `--obu <path>`); skips loudly if
//! `obu` is not installed.

use obu_cli_conformance::{binary_available, run_obu_vectors, ObuConfig};

#[test]
fn test_all_obu_vectors() {
    let cfg = ObuConfig::from_path();
    if !binary_available(&cfg.obu) {
        eprintln!(
            "SKIPPED: `obu` not found on $PATH — obu conformance NOT validated. \
             Install obu, or run `obu-cli-conformance --obu <path>`."
        );
        return;
    }
    run_obu_vectors(&cfg).assert_success();
}
