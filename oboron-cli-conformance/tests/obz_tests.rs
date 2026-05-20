//! Thin wrapper invoking the library's `run_obz_smoke`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{run_obz_smoke, Config};

#[test]
fn test_obz_smoke() {
    let cfg = Config::from_path();
    run_obz_smoke(&cfg).assert_success();
}
