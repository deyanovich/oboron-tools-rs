//! CLI for the oboron obu layer — the unauthenticated schemes `upcbc`
//! and `zdcbc`. NOT authenticated; never use for sensitive data.
//!
//! Mirrors the core `ob` CLI (profiles, config, format strings), but
//! uses a 256-bit secret (`--secret` / `OBORON_SECRET`, 64 hex chars)
//! and its own `~/.obu/` config directory.

mod completions;
mod config;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use config::Config;
use obu::{Encoding, Format, Scheme};
use std::io::{self, Read};

/// Protocol specification version implemented by this binary.
const PROTOCOL_VERSION: &str = "1.0";
/// CLI specification version implemented by this binary.
const CLI_VERSION: &str = "1.0";
/// Implementation name reported in the `--version` line.
const IMPL_NAME: &str = "oboron-tools-rs";

/// The single-line `--version` output:
/// `obu <implementation> <version> protocol=<v> cli=<v>`.
fn version_line() -> String {
    format!(
        "obu {} {} protocol={} cli={}",
        IMPL_NAME,
        env!("CARGO_PKG_VERSION"),
        PROTOCOL_VERSION,
        CLI_VERSION,
    )
}

/// Uniform stderr message for every `dec` failure (OBU spec §6.4 / §2.1).
const DEC_FAILURE_MSG: &str = "dec: invalid obtext";

/// Print a usage error to stderr and exit `2` — the exit-code contract
/// is inherited from the core Oboron CLI spec (§8).
fn usage_error(msg: impl std::fmt::Display) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2);
}

/// Report a uniform `dec` failure and exit `1`. The obu spec **SHOULD**
/// report all decode/length/UTF-8/empty failures through one message so
/// `dec` does not become a distinguishing oracle (OBU spec §2.1, §6.4);
/// we collapse them here for parity with the core `ob` CLI.
fn dec_failure() -> ! {
    eprintln!("{DEC_FAILURE_MSG}");
    std::process::exit(1);
}

#[derive(Parser)]
#[command(name = "obu")]
#[command(
    about = "Unauthenticated string-in/string-out codecs with obtext encoding (NOT secure)",
    long_about = None,
    disable_version_flag = true
)]
struct Cli {
    /// Print version information and exit.
    #[arg(short = 'V', long, global = true)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args, Debug)]
struct SchemeFlags {
    /// Use upcbc scheme (probabilistic AES-256-CBC, unauthenticated)
    #[cfg(feature = "upcbc")]
    #[arg(short = 'u', long)]
    upcbc: bool,

    /// Use zdcbc scheme (deterministic AES-128-CBC, obfuscation only)
    #[cfg(feature = "zdcbc")]
    #[arg(short = 'z', long)]
    zdcbc: bool,

    /// Use zmock1 scheme (testing, identity)
    #[cfg(feature = "mock")]
    #[arg(long, hide = true)]
    zmock1: bool,
}

impl SchemeFlags {
    fn to_scheme(&self) -> Result<Option<Scheme>> {
        let mut count = 0;
        let mut scheme = None;

        #[cfg(feature = "upcbc")]
        if self.upcbc {
            count += 1;
            scheme = Some(Scheme::Upcbc);
        }
        #[cfg(feature = "zdcbc")]
        if self.zdcbc {
            count += 1;
            scheme = Some(Scheme::Zdcbc);
        }
        #[cfg(feature = "mock")]
        if self.zmock1 {
            count += 1;
            scheme = Some(Scheme::Zmock1);
        }

        if count > 1 {
            usage_error("only one scheme flag may be specified");
        }
        Ok(scheme)
    }

    fn is_set(&self) -> bool {
        #[cfg(feature = "upcbc")]
        if self.upcbc {
            return true;
        }
        #[cfg(feature = "zdcbc")]
        if self.zdcbc {
            return true;
        }
        #[cfg(feature = "mock")]
        if self.zmock1 {
            return true;
        }
        false
    }
}

#[derive(Args, Debug)]
struct EncodingFlags {
    /// Use c32 encoding
    #[arg(short = 'c', long, alias = "base32crockford")]
    c32: bool,
    /// Use b32 encoding
    #[arg(short = 'b', long, alias = "base32rfc")]
    b32: bool,
    /// Use b64 encoding
    #[arg(short = 'B', long, alias = "base64")]
    b64: bool,
    /// Use hex encoding
    #[arg(short = 'x', long)]
    hex: bool,
}

impl EncodingFlags {
    fn to_encoding(&self) -> Result<Option<Encoding>> {
        let mut count = 0;
        let mut encoding = None;
        if self.c32 {
            count += 1;
            encoding = Some(Encoding::C32);
        }
        if self.b32 {
            count += 1;
            encoding = Some(Encoding::B32);
        }
        if self.b64 {
            count += 1;
            encoding = Some(Encoding::B64);
        }
        if self.hex {
            count += 1;
            encoding = Some(Encoding::Hex);
        }
        if count > 1 {
            usage_error("only one encoding flag may be specified");
        }
        Ok(encoding)
    }

    fn is_set(&self) -> bool {
        self.c32 || self.b32 || self.b64 || self.hex
    }
}

/// Combined format specification (scheme + encoding).
#[derive(Debug)]
struct FormatSpec {
    scheme: Scheme,
    encoding: Encoding,
}

impl FormatSpec {
    fn parse(
        format_str: Option<String>,
        scheme_flags: &SchemeFlags,
        encoding_flags: &EncodingFlags,
        config: Option<&Config>,
    ) -> Result<Self> {
        if format_str.is_some() && scheme_flags.is_set() {
            usage_error("--format cannot be combined with scheme flags");
        }
        if format_str.is_some() && encoding_flags.is_set() {
            usage_error("--format cannot be combined with encoding flags");
        }

        if let Some(fmt_str) = format_str {
            let format = Format::from_str(&fmt_str)
                .unwrap_or_else(|e| usage_error(format!("invalid format '{fmt_str}': {e}")));
            return Ok(Self {
                scheme: format.scheme(),
                encoding: format.encoding(),
            });
        }

        let scheme = get_scheme(scheme_flags.to_scheme()?, config)?;
        let encoding = get_encoding(encoding_flags.to_encoding()?, config)?;
        Ok(Self { scheme, encoding })
    }
}

impl std::fmt::Display for FormatSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.scheme.as_str(), self.encoding.as_str())
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt+encode a plaintext string
    #[command(visible_alias = "e")]
    Enc {
        /// Plaintext string (reads from stdin if not provided)
        text: Option<String>,

        /// Secret (64 hex chars); conflicts with --profile / --keyless.
        /// No short flag (to avoid clashing with -s elsewhere).
        #[arg(long, conflicts_with = "profile", conflicts_with = "keyless")]
        secret: Option<String>,

        /// Use named secret profile
        #[arg(short, long, conflicts_with = "secret", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use the fixed public test secret (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "secret", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "upcbc.b64", "zdcbc.c32").
        /// Cannot be combined with scheme or encoding flags
        #[arg(short, long)]
        format: Option<String>,

        /// Disable CLI line framing (no stdin newline strip, no stdout newline)
        #[arg(short = '0', long)]
        raw: bool,

        #[command(flatten)]
        scheme: SchemeFlags,
        #[command(flatten)]
        encoding: EncodingFlags,
    },

    /// Decode+decrypt an obtext string
    #[command(visible_alias = "d")]
    Dec {
        /// Obtext string (reads from stdin if not provided)
        text: Option<String>,

        /// Secret (64 hex chars); conflicts with --profile / --keyless.
        #[arg(long, conflicts_with = "profile", conflicts_with = "keyless")]
        secret: Option<String>,

        /// Use named secret profile
        #[arg(short, long, conflicts_with = "secret", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use the fixed public test secret (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "secret", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "upcbc.b64", "zdcbc.c32").
        #[arg(short, long)]
        format: Option<String>,

        /// Disable CLI line framing (no stdin newline strip, no stdout newline)
        #[arg(short = '0', long)]
        raw: bool,

        #[command(flatten)]
        scheme: SchemeFlags,
        #[command(flatten)]
        encoding: EncodingFlags,
    },

    /// Generate a fresh random 256-bit secret and print it
    Secretgen,

    /// Initialize configuration with a random secret profile
    #[command(visible_alias = "i")]
    Init {
        /// Name for the secret profile (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// Manage configuration
    #[command(visible_alias = "c")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,

        /// Use the fixed public test secret (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,
    },

    /// Manage secret profiles
    #[command(visible_alias = "p")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Output the secret (canonical 64-char hex)
    #[command(visible_alias = "s")]
    Secret {
        /// Use named secret profile
        #[arg(short, long)]
        profile: Option<String>,

        /// Use the fixed public test secret (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,

        /// Accepted no-op: hex is the only secret output format.
        #[arg(short = 'x', long)]
        hex: bool,
    },

    /// Generate shell completion script
    Completion {
        #[command(subcommand)]
        shell: completions::Shell,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set configuration values
    Set {
        #[command(flatten)]
        scheme: SchemeFlags,
        #[command(flatten)]
        encoding: EncodingFlags,
        /// Set default secret profile
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all secret profiles
    #[command(visible_alias = "l")]
    List,
    /// Show a specific secret profile
    #[command(visible_alias = "get")]
    #[command(visible_alias = "g")]
    Show {
        /// Profile name (shows default if not provided)
        name: Option<String>,
    },
    /// Set a profile as the default
    #[command(visible_alias = "a")]
    #[command(visible_alias = "use")]
    Activate {
        /// Profile name
        name: String,
    },
    /// Create a new secret profile
    #[command(visible_alias = "c")]
    Create {
        /// Profile name
        name: String,
        /// Secret (64 hex chars)
        #[arg(long)]
        secret: Option<String>,
    },
    /// Delete a secret profile
    #[command(visible_alias = "d")]
    Delete {
        /// Profile name
        name: String,
    },
    /// Rename a secret profile
    #[command(visible_alias = "r")]
    #[command(visible_alias = "mv")]
    Rename {
        /// Current profile name
        old_name: String,
        /// New profile name
        new_name: String,
    },
    /// Set the secret for a profile
    Set {
        /// Profile name
        name: String,
        /// Secret (64 hex chars)
        #[arg(long)]
        secret: Option<String>,
    },
}

fn main() -> Result<()> {
    // Install the obu environment (the `~/.obu/` config dir and 64-hex
    // secrets) before any config/profile/key operation runs.
    oboron_cli_core::set_env(oboron_cli_core::CliEnv::OBU);

    let cli = Cli::parse();

    // --version is global: works before/after any command name and needs
    // no secret, config, profile, or stdin. Handle it first and exit 0.
    if cli.version {
        println!("{}", version_line());
        return Ok(());
    }

    let command = match cli.command {
        Some(c) => c,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            std::process::exit(2);
        }
    };

    match command {
        Commands::Enc {
            text,
            secret,
            profile,
            keyless,
            format,
            raw,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            enc_command(text, secret, profile, keyless, format_spec, raw, cfg)
        }

        Commands::Dec {
            text,
            secret,
            profile,
            keyless,
            format,
            raw,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            dec_command(text, secret, profile, keyless, format_spec, raw, cfg)
        }

        Commands::Secretgen => {
            println!("{}", obu::generate_secret());
            Ok(())
        }

        Commands::Init { name } => config::init_command(&name),

        Commands::Config { command, keyless } => match command {
            Some(ConfigCommands::Show) | None => config::config_show_command(keyless),
            Some(ConfigCommands::Set {
                scheme,
                encoding,
                profile,
            }) => {
                let scheme_override = scheme.to_scheme()?;
                let encoding_override = encoding.to_encoding()?;
                config_set_command(scheme_override, encoding_override, profile)
            }
        },

        Commands::Profile { command } => match command {
            ProfileCommands::List => config::profile_list_command(),
            ProfileCommands::Show { name } => config::profile_show_command(name.as_deref()),
            ProfileCommands::Activate { name } => config::profile_activate_command(&name),
            ProfileCommands::Create { name, secret } => {
                config::profile_create_command(&name, secret.as_deref())
            }
            ProfileCommands::Delete { name } => config::profile_delete_command(&name),
            ProfileCommands::Rename { old_name, new_name } => {
                config::profile_rename_command(&old_name, &new_name)
            }
            ProfileCommands::Set { name, secret } => {
                config::profile_set_command(&name, secret.as_deref())
            }
        },

        Commands::Secret {
            profile,
            keyless,
            hex,
        } => secret_command(profile, keyless, hex),

        Commands::Completion { shell } => {
            completions::generate_completion(shell);
            Ok(())
        }
    }
}

/// Write `s` to stdout, appending a single `\n` in default framing and
/// nothing in `--raw` framing.
fn write_output(s: &str, raw: bool) -> Result<()> {
    use std::io::Write;
    let mut out = io::stdout();
    out.write_all(s.as_bytes())?;
    if !raw {
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}

fn enc_command(
    text: Option<String>,
    secret: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    raw: bool,
    cfg: Option<Config>,
) -> Result<()> {
    let text = get_text_input(text, raw)?;
    let format = format_spec.to_string();

    let encd = if keyless {
        obu::Obu::new_keyless(&format)?.enc(&text)?
    } else {
        let hex_secret = get_secret(secret.as_ref(), profile.as_deref(), cfg.as_ref())?;
        obu::Obu::new(&format, &hex_secret)?.enc(&text)?
    };
    write_output(&encd, raw)
}

fn dec_command(
    text: Option<String>,
    secret: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    raw: bool,
    cfg: Option<Config>,
) -> Result<()> {
    let format = format_spec.to_string();

    // Build the codec first. Missing/invalid secret and format problems
    // keep their own messages; the scheme is supplied by the resolved
    // format — `dec` never auto-detects (OBU spec §6.2).
    let obu = if keyless {
        obu::Obu::new_keyless(&format)?
    } else {
        let hex_secret = get_secret(secret.as_ref(), profile.as_deref(), cfg.as_ref())?;
        obu::Obu::new(&format, &hex_secret)?
    };

    // Every failure past this point collapses to one uniform message +
    // exit 1 so `dec` is not a distinguishing oracle (OBU spec §2.1).
    let input = read_dec_input(text, raw);
    match obu.dec(&input) {
        Ok(plaintext) => write_output(&plaintext, raw),
        Err(_) => dec_failure(),
    }
}

/// Read the obtext input for `dec` (framing as in `enc`, §7). Any
/// failure to obtain non-empty valid-UTF-8 input collapses to the
/// uniform dec failure rather than a distinguishing message.
fn read_dec_input(text: Option<String>, raw: bool) -> String {
    let s = match text {
        Some(t) => t,
        None => {
            let mut bytes = Vec::new();
            if io::stdin().read_to_end(&mut bytes).is_err() {
                dec_failure();
            }
            if !raw {
                if bytes.ends_with(b"\r\n") {
                    bytes.truncate(bytes.len() - 2);
                } else if bytes.ends_with(b"\n") {
                    bytes.truncate(bytes.len() - 1);
                }
            }
            match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => dec_failure(),
            }
        }
    };
    if s.is_empty() {
        dec_failure();
    }
    s
}

fn config_set_command(
    scheme_override: Option<Scheme>,
    encoding_override: Option<Encoding>,
    profile: Option<String>,
) -> Result<()> {
    let mut config = config::load_config().unwrap_or_else(|_| {
        Config::new(
            Some("default".to_string()),
            Some("upcbc".to_string()),
            Some("c32".to_string()),
        )
    });

    if let Some(scheme) = scheme_override {
        config.scheme = Some(scheme.to_string());
    }
    if let Some(encoding) = encoding_override {
        config.encoding = Some(encoding.to_string());
    }
    if let Some(p) = profile {
        config.profile = Some(p);
    }

    config::save_config(&config)?;
    println!("✓ Configuration updated");
    if let Some(p) = &config.profile {
        println!("  Profile:  {p}");
    }
    if let Some(s) = &config.scheme {
        println!("  Scheme:   {s}");
    }
    if let Some(e) = &config.encoding {
        println!("  Encoding: {e}");
    }
    Ok(())
}

fn secret_command(profile: Option<String>, keyless: bool, _hex: bool) -> Result<()> {
    let hex_secret = if keyless {
        config::KEYLESS_SECRET_HEX.to_string()
    } else {
        let cfg = config::load_config().ok();
        let active_profile_name = cfg.as_ref().and_then(|c| c.profile.as_deref());

        if let Some(prof) = profile.as_deref().or(active_profile_name) {
            oboron_cli_core::load_profile_key_as_hex(prof)?
        } else if let Ok(env_secret) = std::env::var("OBORON_SECRET") {
            require_hex_secret(&env_secret).context("invalid $OBORON_SECRET")?
        } else {
            anyhow::bail!(
                "No secret specified: provide --profile, set $OBORON_SECRET, or run 'obu init'"
            );
        }
    };
    println!("{hex_secret}");
    Ok(())
}

/// Validate that `secret` is a canonical obu secret — exactly 64
/// lowercase hex characters — and return it (OBU-CLI spec §2.1).
fn require_hex_secret(secret: &str) -> Result<String> {
    let is_canonical = secret.len() == 64
        && secret
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !is_canonical {
        anyhow::bail!("secret must be exactly 64 lowercase hex characters");
    }
    Ok(secret.to_string())
}

fn get_secret(
    secret: Option<&String>,
    profile: Option<&str>,
    config: Option<&Config>,
) -> Result<String> {
    // 1. Explicit --secret flag (64-char hex only).
    if let Some(secret_str) = secret {
        return require_hex_secret(secret_str).context("invalid --secret");
    }

    // 2. Environment variable.
    if let Ok(env_secret) = std::env::var("OBORON_SECRET") {
        return require_hex_secret(&env_secret).context("invalid $OBORON_SECRET");
    }

    // 3-4. Profile (explicit --profile or default from config).
    let profile_name = profile.or_else(|| config.and_then(|c| c.profile.as_deref()));
    if let Some(name) = profile_name {
        return oboron_cli_core::load_profile_key_as_hex(name);
    }

    Err(anyhow::anyhow!(
        "No secret specified: provide --secret, set $OBORON_SECRET, use --profile, or run 'obu init'"
    ))
}

/// Read the plaintext/obtext input for `enc` / `dec` (same framing rules
/// as the core CLI).
fn get_text_input(text: Option<String>, raw: bool) -> Result<String> {
    let input = match text {
        Some(t) => t,
        None => {
            let mut bytes = Vec::new();
            io::stdin()
                .read_to_end(&mut bytes)
                .context("failed to read from stdin")?;
            if !raw {
                if bytes.ends_with(b"\r\n") {
                    bytes.truncate(bytes.len() - 2);
                } else if bytes.ends_with(b"\n") {
                    bytes.truncate(bytes.len() - 1);
                }
            }
            String::from_utf8(bytes).context("input is not valid UTF-8")?
        }
    };
    if input.is_empty() {
        anyhow::bail!("no input provided");
    }
    Ok(input)
}

fn get_scheme(scheme_override: Option<Scheme>, config: Option<&Config>) -> Result<Scheme> {
    if let Some(scheme) = scheme_override {
        return Ok(scheme);
    }
    if let Some(cfg) = config {
        if let Some(scheme_str) = cfg.scheme.as_deref() {
            return Scheme::from_str(scheme_str).map_err(|e| anyhow::anyhow!("{}", e));
        }
    }
    // Built-in default scheme (OBU-CLI spec §3): upcbc.
    Scheme::from_str("upcbc").map_err(|e| anyhow::anyhow!("{}", e))
}

fn get_encoding(encoding_override: Option<Encoding>, config: Option<&Config>) -> Result<Encoding> {
    if let Some(encoding) = encoding_override {
        return Ok(encoding);
    }
    if let Some(cfg) = config {
        if let Some(enc_str) = cfg.encoding.as_deref() {
            return Encoding::from_str(enc_str).map_err(|e| anyhow::anyhow!("{}", e));
        }
    }
    // Built-in default encoding: c32.
    Ok(Encoding::C32)
}
