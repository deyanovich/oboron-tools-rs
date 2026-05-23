use clap::{Arg, Command, Subcommand};
use clap_complete::{generate, Shell as ClapShell};
use std::io;

#[derive(Subcommand, Clone)]
pub enum Shell {
    /// Generate bash completion script
    Bash,
    /// Generate zsh completion script
    Zsh,
    /// Generate fish completion script
    Fish,
    /// Generate PowerShell completion script
    Powershell,
}

pub fn generate_completion(shell: Shell) {
    let mut cmd = build_cli();
    let bin_name = "obz";

    match shell {
        Shell::Bash => generate(ClapShell::Bash, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Fish => generate(ClapShell::Fish, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Powershell => generate(ClapShell::PowerShell, &mut cmd, bin_name, &mut io::stdout()),
    }
}

fn build_cli() -> Command {
    Command::new("obz")
        .about("Z-tier obfuscation tool (NOT SECURE)")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(vec![
            Command::new("enc")
                .visible_alias("e")
                .about("Encrypt+encode a plaintext string")
                .arg(Arg::new("text").help("Plaintext string (reads from stdin if not provided)"))
                .arg(Arg::new("secret").short('s').long("secret").help("Secret key").conflicts_with("profile").conflicts_with("keyless"))
                .arg(Arg::new("profile").short('p').long("profile").help("Use named secret profile").conflicts_with("secret").conflicts_with("keyless"))
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use hardcoded key (INSECURE - testing only)").conflicts_with("secret").conflicts_with("profile"))
                .arg(Arg::new("format").short('f').long("format").help("Format specification (e.g., \"zrbcx.b64\")")),
            Command::new("dec")
                .visible_alias("d")
                .about("Decode+decrypt an obtext string")
                .arg(Arg::new("text").help("Obtext string (reads from stdin if not provided)"))
                .arg(Arg::new("secret").short('s').long("secret").help("Secret key").conflicts_with("profile").conflicts_with("keyless"))
                .arg(Arg::new("profile").short('p').long("profile").help("Use named secret profile").conflicts_with("secret").conflicts_with("keyless"))
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use hardcoded key (INSECURE - testing only)").conflicts_with("secret").conflicts_with("profile"))
                .arg(Arg::new("format").short('f').long("format").help("Format specification (e.g., \"zrbcx.b64\")")),
            Command::new("init")
                .visible_alias("i")
                .about("Initialize configuration with random profile")
                .arg(Arg::new("name").default_value("default").help("Name for the secret profile")),
            Command::new("config")
                .visible_alias("c")
                .about("Manage configuration")
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use hardcoded key (INSECURE - testing only)"))
                .subcommands(vec![
                    Command::new("show").about("Show current configuration"),
                    Command::new("set")
                        .about("Set configuration values")
                        .arg(Arg::new("profile").short('p').long("profile").help("Set default secret profile")),
                ]),
            Command::new("profile")
                .visible_alias("p")
                .about("Manage secret profiles")
                .subcommands(vec![
                    Command::new("list").visible_alias("l").about("List all secret profiles"),
                    Command::new("show").visible_alias("g").about("Show a specific secret profile")
                        .arg(Arg::new("name").help("Profile name")),
                    Command::new("activate").visible_alias("a").about("Set a profile as the default")
                        .arg(Arg::new("name").required(true).help("Profile name")),
                    Command::new("create").visible_alias("c").about("Create a new secret profile")
                        .arg(Arg::new("name").required(true).help("Profile name"))
                        .arg(Arg::new("secret").short('s').long("secret").help("Secret key (43 base64 chars)")),
                    Command::new("delete").visible_alias("d").about("Delete a secret profile")
                        .arg(Arg::new("name").required(true).help("Profile name")),
                    Command::new("rename").visible_alias("r").about("Rename a secret profile")
                        .arg(Arg::new("old_name").required(true).help("Current profile name"))
                        .arg(Arg::new("new_name").required(true).help("New profile name")),
                    Command::new("set").about("Set secret for a profile")
                        .arg(Arg::new("name").required(true).help("Profile name"))
                        .arg(Arg::new("secret").short('s').long("secret").help("Secret key (43 base64 chars)")),
                ]),
            Command::new("secret")
                .visible_alias("s")
                .about("Output the secret key")
                .arg(Arg::new("secret").short('s').long("secret").help("Secret key"))
                .arg(Arg::new("profile").short('p').long("profile").help("Use named secret profile"))
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use hardcoded key (INSECURE - testing only)"))
                .arg(Arg::new("hex").short('x').long("hex").action(clap::ArgAction::SetTrue).help("Output secret as hex instead of base64")),
            Command::new("completion")
                .about("Generate shell completion script")
                .subcommands(vec![
                    Command::new("bash").about("Generate bash completion script"),
                    Command::new("zsh").about("Generate zsh completion script"),
                    Command::new("fish").about("Generate fish completion script"),
                    Command::new("powershell").about("Generate PowerShell completion script"),
                ]),
        ])
}
