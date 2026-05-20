//! Thin wrapper invoking the library's `run_ob_vectors`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{run_ob_vectors, Config};

#[test]
fn test_all_vectors() {
    let cfg = Config::from_path();
    run_ob_vectors(&cfg).assert_success();
}
