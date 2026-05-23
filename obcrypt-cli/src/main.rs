//! `obcrypt` — bytes-in / bytes-out symmetric encryption CLI
//! (cryptographic core of the oboron protocol).
//!
//! See `obcrypt --help` for usage. The CLI is a thin wrapper around the
//! `obcrypt` library plus profile / config management for the shared
//! `~/.oboron/` directory (via [`oboron_cli_core`]).

mod completions;
mod config;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use obcrypt::{Key, Scheme};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "obcrypt")]
#[command(
    version,
    about = "Bytes-in/bytes-out symmetric encryption (obcrypt / oboron protocol)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt plaintext bytes under a scheme
    #[command(visible_alias = "e")]
    Encrypt {
        /// Plaintext (reads stdin if absent). Raw bytes by default;
        /// with -X/--hex-in, treated as a hex string and decoded first.
        text: Option<String>,

        /// Hex-encode the ciphertext on output, for safe terminal
        /// display. Without this, ciphertext is written as raw bytes.
        #[arg(short = 'x', long)]
        hex: bool,

        /// Decode the plaintext input as hex before encrypting.
        /// For binary plaintext that's easier to type/paste as hex.
        #[arg(short = 'X', long = "hex-in")]
        hex_in: bool,

        #[command(flatten)]
        common: CommonOpts,
    },

    /// Decrypt ciphertext bytes (scheme auto-detects from trailing marker if not given)
    #[command(visible_alias = "d")]
    Decrypt {
        /// Ciphertext (reads stdin if absent). Raw bytes by default;
        /// with -X/--hex-in, treated as a hex string and decoded first.
        text: Option<String>,

        /// Hex-encode the plaintext on output, for safe terminal display.
        /// Without this, plaintext is written as raw bytes (pipe-friendly
        /// but may garble a terminal if it contains non-printable bytes).
        #[arg(short = 'x', long)]
        hex: bool,

        /// Decode the ciphertext input as hex before decrypting.
        /// Convenient when piping in is awkward and you have the
        /// ciphertext as a hex string from a previous `encrypt -x`.
        #[arg(short = 'X', long = "hex-in")]
        hex_in: bool,

        #[command(flatten)]
        common: CommonOpts,
    },

    /// Generate a fresh random 128-character hex key
    #[command(visible_alias = "k")]
    Keygen,

    /// Initialize configuration with a fresh profile
    #[command(visible_alias = "i")]
    Init {
        /// Profile name (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// Manage configuration
    #[command(visible_alias = "c")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },

    /// Manage key profiles
    #[command(visible_alias = "p")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Print the active profile's key
    Key {
        /// Use named profile (default: active profile from config)
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// Generate shell completion script
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set configuration defaults
    Set {
        /// Default scheme (aasv / aags / apsv / apgs / upbc)
        #[arg(short, long)]
        scheme: Option<String>,
        /// Active profile name
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all profiles
    #[command(visible_alias = "l")]
    List,
    /// Show a profile's key (defaults to active)
    #[command(visible_alias = "s")]
    #[command(visible_alias = "g")]
    #[command(visible_alias = "get")]
    Show { name: Option<String> },
    /// Activate a profile
    #[command(visible_alias = "a")]
    #[command(visible_alias = "use")]
    Activate { name: String },
    /// Create a new profile (random key unless --key supplied)
    #[command(visible_alias = "c")]
    Create {
        name: String,
        /// Key (128 hex chars, or legacy 86-char base64)
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Delete a profile (cannot delete the active one)
    #[command(visible_alias = "d")]
    Delete { name: String },
    /// Rename a profile
    #[command(visible_alias = "r")]
    #[command(visible_alias = "mv")]
    Rename { old_name: String, new_name: String },
    /// Replace the key on an existing profile
    Set {
        name: String,
        #[arg(short, long)]
        key: String,
    },
}

#[derive(Args, Debug)]
struct CommonOpts {
    /// Scheme to use (encrypt: required if no default in config;
    /// decrypt: optional, auto-detects from the trailing marker)
    #[arg(short, long, value_enum)]
    scheme: Option<SchemeArg>,

    /// Encryption key (128 hex chars, or legacy 86-char base64)
    #[arg(short, long, conflicts_with = "profile")]
    key: Option<String>,

    /// Use named key profile from `~/.oboron/profiles/<NAME>.json`
    #[arg(short, long, conflicts_with = "key")]
    profile: Option<String>,

    /// Read input from file instead of TEXT/stdin
    #[arg(long, value_name = "PATH")]
    in_file: Option<PathBuf>,

    /// Write output to file instead of stdout
    #[arg(long, value_name = "PATH")]
    out_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum SchemeArg {
    /// Deterministic AES-SIV (canonical default)
    #[cfg(feature = "aasv")]
    Aasv,
    /// Deterministic AES-GCM-SIV
    #[cfg(feature = "aags")]
    Aags,
    /// Probabilistic AES-SIV
    #[cfg(feature = "apsv")]
    Apsv,
    /// Probabilistic AES-GCM-SIV
    #[cfg(feature = "apgs")]
    Apgs,
    /// Probabilistic AES-CBC (unauthenticated)
    #[cfg(feature = "upbc")]
    Upbc,
}

impl From<SchemeArg> for Scheme {
    fn from(s: SchemeArg) -> Self {
        match s {
            #[cfg(feature = "aasv")]
            SchemeArg::Aasv => Scheme::Aasv,
            #[cfg(feature = "aags")]
            SchemeArg::Aags => Scheme::Aags,
            #[cfg(feature = "apsv")]
            SchemeArg::Apsv => Scheme::Apsv,
            #[cfg(feature = "apgs")]
            SchemeArg::Apgs => Scheme::Apgs,
            #[cfg(feature = "upbc")]
            SchemeArg::Upbc => Scheme::Upbc,
        }
    }
}

fn parse_scheme_name(name: &str) -> Result<Scheme> {
    name.parse::<Scheme>()
        .map_err(|_| anyhow!("unknown scheme '{name}'"))
}

fn trim_trailing_ws(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &bytes[..end]
}

/// Decode a hex-encoded byte buffer, trimming trailing whitespace
/// (typical when reading from stdin or a file).
fn decode_hex_input(buf: &[u8], label: &str) -> Result<Vec<u8>> {
    hex::decode(trim_trailing_ws(buf)).with_context(|| format!("invalid hex {label}"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // One-time migration of the legacy `~/.ob/` config dir from
    // older oboron-cli releases. No-op on fresh installs and on
    // every subsequent invocation.
    if let Some(notice) =
        oboron_cli_core::migration::ensure_config_root_migrated()?
    {
        eprintln!(
            "notice: migrated legacy config dir {} → {}",
            notice.from.display(),
            notice.to.display(),
        );
        if notice.symlink_created {
            eprintln!(
                "        left a {} → {} symlink for backward compatibility \
                 with any older binary still installed.",
                notice.from.display(),
                notice.to.display(),
            );
        }
    }

    match cli.command {
        Commands::Encrypt {
            text,
            hex,
            hex_in,
            common,
        } => run_encrypt(text, hex, hex_in, common),
        Commands::Decrypt {
            text,
            hex,
            hex_in,
            common,
        } => run_decrypt(text, hex, hex_in, common),
        Commands::Keygen => run_keygen(),
        Commands::Init { name } => config::init_command(&name),
        Commands::Config { command } => match command {
            Some(ConfigCommands::Show) | None => config::config_show_command(),
            Some(ConfigCommands::Set { scheme, profile }) => {
                config::config_set_command(scheme, profile)
            }
        },
        Commands::Profile { command } => match command {
            ProfileCommands::List => config::profile_list_command(),
            ProfileCommands::Show { name } => config::profile_show_command(name.as_deref()),
            ProfileCommands::Activate { name } => config::profile_activate_command(&name),
            ProfileCommands::Create { name, key } => {
                config::profile_create_command(&name, key.as_deref())
            }
            ProfileCommands::Delete { name } => config::profile_delete_command(&name),
            ProfileCommands::Rename { old_name, new_name } => {
                config::profile_rename_command(&old_name, &new_name)
            }
            ProfileCommands::Set { name, key } => config::profile_set_command(&name, &key),
        },
        Commands::Key { profile } => config::key_command(profile.as_deref()),
        Commands::Completions { shell } => {
            completions::generate_completion::<Cli>(shell);
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// encrypt / decrypt / keygen
// ---------------------------------------------------------------------------

fn run_encrypt(
    text: Option<String>,
    hex_output: bool,
    hex_input: bool,
    opts: CommonOpts,
) -> Result<()> {
    let key = resolve_key(&opts)?;
    let scheme = resolve_scheme_for_encrypt(&opts)?;

    let raw_input = read_input(text, opts.in_file.as_deref())?;
    let plaintext = if hex_input {
        decode_hex_input(&raw_input, "plaintext")?
    } else {
        raw_input
    };

    let payload = obcrypt::encrypt(&plaintext, scheme, &key)
        .map_err(|e| anyhow!("encrypt failed: {e}"))?;

    if hex_output {
        // Ciphertext bytes encoded as hex for terminal-safe display.
        let encoded = hex::encode(&payload);
        write_output(encoded.as_bytes(), opts.out_file.as_deref(), false)
    } else {
        // Default: raw ciphertext bytes (pipe-friendly, no trailing newline).
        write_output(&payload, opts.out_file.as_deref(), true)
    }
}

fn run_decrypt(
    text: Option<String>,
    hex_output: bool,
    hex_input: bool,
    opts: CommonOpts,
) -> Result<()> {
    let key = resolve_key(&opts)?;
    let scheme = opts.scheme.map(Scheme::from);

    let raw_input = read_input(text, opts.in_file.as_deref())?;
    let payload = if hex_input {
        decode_hex_input(&raw_input, "ciphertext")?
    } else {
        raw_input
    };

    let plaintext = match scheme {
        Some(s) => obcrypt::decrypt_as(&payload, s, &key)
            .map_err(|e| anyhow!("decrypt failed: {e}"))?,
        None => obcrypt::decrypt(&payload, &key)
            .map_err(|e| anyhow!("decrypt failed: {e}"))?,
    };

    if hex_output {
        // Plaintext bytes encoded as hex for terminal-safe display.
        let encoded = hex::encode(&plaintext);
        write_output(encoded.as_bytes(), opts.out_file.as_deref(), false)
    } else {
        // Default: raw plaintext bytes (pipe-friendly, no trailing newline).
        write_output(&plaintext, opts.out_file.as_deref(), true)
    }
}

fn run_keygen() -> Result<()> {
    println!("{}", obcrypt::generate_key().to_hex());
    Ok(())
}

// ---------------------------------------------------------------------------
// Key + scheme resolution
// ---------------------------------------------------------------------------

fn resolve_key(opts: &CommonOpts) -> Result<Key> {
    let hex = if let Some(direct) = opts.key.as_deref() {
        let (hex, fmt) = oboron_cli_core::normalize_key_classify(direct)
            .context("invalid key passed via --key")?;
        if fmt == oboron_cli_core::KeyFormat::LegacyBase64 {
            warn_base64_via_key();
        }
        hex
    } else if let Some(name) = opts.profile.as_deref() {
        oboron_cli_core::commands::load_profile_key_with_notice(name)?
    } else {
        let cfg = config::load_config()?.ok_or_else(|| {
            anyhow!(
                "no key given (--key / --profile) and no ~/.oboron/config.json found; \
                 run 'obcrypt init' to create one"
            )
        })?;
        let profile_name = cfg.profile.as_deref().ok_or_else(|| {
            anyhow!(
                "config.json has no active profile; \
                 set one with 'obcrypt config set --profile <NAME>'"
            )
        })?;
        oboron_cli_core::commands::load_profile_key_with_notice(profile_name)?
    };
    Key::from_hex(&hex).map_err(|e| anyhow!("failed to parse key as hex: {e:?}"))
}

fn warn_base64_via_key() {
    eprintln!(
        "warning: --key was given as legacy base64. base64 keys are deprecated \
         and will be removed before oboron 1.0;"
    );
    eprintln!(
        "         pass a 128-character hex key instead. The base64 key was accepted."
    );
}

fn resolve_scheme_for_encrypt(opts: &CommonOpts) -> Result<Scheme> {
    if let Some(s) = opts.scheme {
        return Ok(s.into());
    }
    if let Some(cfg) = config::load_config()? {
        if let Some(name) = cfg.scheme {
            return parse_scheme_name(&name)
                .with_context(|| format!("default scheme '{name}' from config"));
        }
    }
    bail!("no --scheme given and no default in ~/.oboron/config.json")
}

// ---------------------------------------------------------------------------
// I/O
// ---------------------------------------------------------------------------

fn read_input(text: Option<String>, in_file: Option<&std::path::Path>) -> Result<Vec<u8>> {
    if let Some(t) = text {
        return Ok(t.into_bytes());
    }
    if let Some(path) = in_file {
        return fs::read(path).with_context(|| format!("read {}", path.display()));
    }
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).context("read stdin")?;
    Ok(buf)
}

fn write_output(
    bytes: &[u8],
    out_file: Option<&std::path::Path>,
    raw_bytes_on_stdout: bool,
) -> Result<()> {
    if let Some(path) = out_file {
        fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
        return Ok(());
    }
    let stdout = io::stdout();
    let mut h = stdout.lock();
    h.write_all(bytes).context("write stdout")?;
    if !raw_bytes_on_stdout {
        h.write_all(b"\n").context("write stdout")?;
    }
    Ok(())
}
