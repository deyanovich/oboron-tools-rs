//! Thin wrapper invoking the library's `run_obcrypt_vectors`. The
//! same logic is exposed through the `oboron-cli-conformance`
//! binary for cross-language implementers.

use oboron_cli_conformance::{binary_available, run_obcrypt_vectors, Config};

#[test]
fn test_obcrypt_all_hex_vectors() {
    let cfg = Config::from_path();
    if !binary_available(&cfg.obcrypt) {
        eprintln!(
            "SKIPPED: `obcrypt` not found on $PATH — obcrypt conformance NOT \
             validated. Install obcrypt, or run `oboron-cli-conformance \
             --obcrypt <path>`."
        );
        return;
    }
    run_obcrypt_vectors(&cfg).assert_success();
}
