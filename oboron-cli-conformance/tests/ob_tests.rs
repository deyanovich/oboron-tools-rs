//! Thin wrapper invoking the library's `run_ob_smoke`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{run_ob_smoke, Config};

#[test]
fn test_ob_smoke() {
    let cfg = Config::from_path();
    run_ob_smoke(&cfg).assert_success();
}
