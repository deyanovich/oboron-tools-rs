//! Thin wrapper invoking the library's `run_obc_vectors`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{run_obc_vectors, Config};

#[test]
fn test_obc_all_hex_vectors() {
    let cfg = Config::from_path();
    run_obc_vectors(&cfg).assert_success();
}
