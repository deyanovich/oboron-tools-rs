//! Thin wrapper invoking the library's `run_obz_ztier_vectors`.
//! The same logic is exposed through the
//! `oboron-cli-conformance` binary for cross-language
//! implementers.

use oboron_cli_conformance::{run_obz_ztier_vectors, Config};

#[test]
fn test_all_vectors() {
    let cfg = Config::from_path();
    run_obz_ztier_vectors(&cfg).assert_success();
}
