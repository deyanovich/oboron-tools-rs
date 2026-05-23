//! Shell completion script generation.

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn generate_completion<C: CommandFactory>(shell: Shell) {
    let mut cmd = C::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}
