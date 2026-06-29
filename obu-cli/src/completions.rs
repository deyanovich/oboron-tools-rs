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
    let bin_name = "obu";

    match shell {
        Shell::Bash => generate(ClapShell::Bash, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Fish => generate(ClapShell::Fish, &mut cmd, bin_name, &mut io::stdout()),
        Shell::Powershell => generate(ClapShell::PowerShell, &mut cmd, bin_name, &mut io::stdout()),
    }
}

fn enc_dec_args(cmd: Command) -> Command {
    cmd.arg(Arg::new("text").help("Input string (reads from stdin if not provided)"))
        .arg(
            Arg::new("secret")
                .long("secret")
                .help("Secret (64 hex chars)")
                .conflicts_with("profile")
                .conflicts_with("keyless"),
        )
        .arg(
            Arg::new("profile")
                .short('p')
                .long("profile")
                .help("Use named secret profile")
                .conflicts_with("secret")
                .conflicts_with("keyless"),
        )
        .arg(
            Arg::new("keyless")
                .short('K')
                .long("keyless")
                .action(clap::ArgAction::SetTrue)
                .help("Use the fixed public test secret (INSECURE - testing only)")
                .conflicts_with("secret")
                .conflicts_with("profile"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help("Format specification (e.g., \"upcbc.b64\", \"zdcbc.c32\")"),
        )
        .arg(
            Arg::new("raw")
                .short('0')
                .long("raw")
                .action(clap::ArgAction::SetTrue)
                .help("Disable CLI line framing"),
        )
        .arg(Arg::new("upcbc").short('u').long("upcbc").action(clap::ArgAction::SetTrue).help("Use upcbc scheme"))
        .arg(Arg::new("zdcbc").short('z').long("zdcbc").action(clap::ArgAction::SetTrue).help("Use zdcbc scheme"))
        .arg(Arg::new("c32").short('c').long("c32").action(clap::ArgAction::SetTrue).help("Use c32 encoding"))
        .arg(Arg::new("b32").short('b').long("b32").action(clap::ArgAction::SetTrue).help("Use b32 encoding"))
        .arg(Arg::new("b64").short('B').long("b64").action(clap::ArgAction::SetTrue).help("Use b64 encoding"))
        .arg(Arg::new("hex").short('x').long("hex").action(clap::ArgAction::SetTrue).help("Use hex encoding"))
}

fn build_cli() -> Command {
    Command::new("obu")
        .about("Unauthenticated string-in/string-out codecs with obtext encoding (NOT secure)")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(vec![
            enc_dec_args(
                Command::new("enc")
                    .visible_alias("e")
                    .about("Encrypt+encode a plaintext string"),
            ),
            enc_dec_args(
                Command::new("dec")
                    .visible_alias("d")
                    .about("Decode+decrypt an obtext string"),
            ),
            Command::new("secretgen")
                .about("Generate a fresh random 256-bit secret and print it"),
            Command::new("init")
                .visible_alias("i")
                .about("Initialize configuration with a random secret profile")
                .arg(Arg::new("name").default_value("default").help("Name for the secret profile")),
            Command::new("config")
                .visible_alias("c")
                .about("Manage configuration")
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use the fixed public test secret (INSECURE - testing only)"))
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
                        .arg(Arg::new("secret").long("secret").help("Secret (64 hex chars)")),
                    Command::new("delete").visible_alias("d").about("Delete a secret profile")
                        .arg(Arg::new("name").required(true).help("Profile name")),
                    Command::new("rename").visible_alias("r").about("Rename a secret profile")
                        .arg(Arg::new("old_name").required(true).help("Current profile name"))
                        .arg(Arg::new("new_name").required(true).help("New profile name")),
                    Command::new("set").about("Set the secret for a profile")
                        .arg(Arg::new("name").required(true).help("Profile name"))
                        .arg(Arg::new("secret").long("secret").help("Secret (64 hex chars)")),
                ]),
            Command::new("secret")
                .visible_alias("s")
                .about("Output the secret (canonical 64-char hex)")
                .arg(Arg::new("profile").short('p').long("profile").help("Use named secret profile"))
                .arg(Arg::new("keyless").short('K').long("keyless").action(clap::ArgAction::SetTrue).help("Use the fixed public test secret (INSECURE - testing only)"))
                .arg(Arg::new("hex").short('x').long("hex").action(clap::ArgAction::SetTrue).help("Accepted no-op: hex is the only secret output format")),
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
