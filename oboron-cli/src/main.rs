//! CLI application for the authenticated oboron core schemes
//! (`dsiv`, `psiv`, `dgcmsiv`, `pgcmsiv`).

mod completions;
mod config;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use config::Config;
use oboron::{Encoding, Format, Scheme};
use std::io::{self, Read};

/// Protocol specification version implemented by this binary.
const PROTOCOL_VERSION: &str = "1.0";
/// CLI specification version implemented by this binary.
const CLI_VERSION: &str = "1.0";
/// Implementation name reported in the `--version` line.
const IMPL_NAME: &str = "oboron-tools-rs";

/// The single-line `--version` output required by the CLI spec:
/// `ob <implementation> <version> protocol=<v> cli=<v>`.
fn version_line() -> String {
    format!(
        "ob {} {} protocol={} cli={}",
        IMPL_NAME,
        env!("CARGO_PKG_VERSION"),
        PROTOCOL_VERSION,
        CLI_VERSION,
    )
}

/// Uniform stderr message for every `dec` failure (CLI.md §8).
const DEC_FAILURE_MSG: &str = "dec: invalid obtext";

/// Print a usage error to stderr and exit with status `2` (CLI.md §8) —
/// invalid/conflicting flags, malformed format, wrong argument count.
/// Mirrors clap's own exit code for argument-parsing errors.
fn usage_error(msg: impl std::fmt::Display) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2);
}

/// Report a uniform `dec` failure and exit `1` (CLI.md §8). Every
/// decode/length/authentication/UTF-8/empty-plaintext failure collapses
/// to one message so `dec` cannot become a decryption oracle: the cause
/// MUST NOT be distinguishable, since it could leak information about
/// secret input.
fn dec_failure() -> ! {
    eprintln!("{DEC_FAILURE_MSG}");
    std::process::exit(1);
}

#[derive(Parser)]
#[command(name = "ob")]
#[command(
    about = "Authenticated string-in/string-out encryption with obtext encoding",
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
    /// Use dsiv scheme (deterministic AES-SIV)
    #[cfg(feature = "dsiv")]
    #[arg(short = 's', long)]
    dsiv: bool,

    /// Use psiv scheme (probabilistic AES-SIV)
    #[cfg(feature = "psiv")]
    #[arg(short = 'S', long)]
    psiv: bool,

    /// Use dgcmsiv scheme (deterministic AES-GCM-SIV)
    #[cfg(feature = "dgcmsiv")]
    #[arg(short = 'g', long)]
    dgcmsiv: bool,

    /// Use pgcmsiv scheme (probabilistic AES-GCM-SIV)
    #[cfg(feature = "pgcmsiv")]
    #[arg(short = 'G', long)]
    pgcmsiv: bool,

    /// Use mock1 scheme (testing, identity)
    #[cfg(feature = "mock")]
    #[arg(long, hide = true)]
    mock1: bool,

    /// Use mock2 scheme (testing, string reversal)
    #[cfg(feature = "mock")]
    #[arg(long, hide = true)]
    mock2: bool,
}

impl SchemeFlags {
    fn to_scheme(&self) -> Result<Option<Scheme>> {
        let mut count = 0;
        let mut scheme = None;

        #[cfg(feature = "dsiv")]
        if self.dsiv {
            count += 1;
            scheme = Some(Scheme::Dsiv);
        }
        #[cfg(feature = "psiv")]
        if self.psiv {
            count += 1;
            scheme = Some(Scheme::Psiv);
        }
        #[cfg(feature = "dgcmsiv")]
        if self.dgcmsiv {
            count += 1;
            scheme = Some(Scheme::Dgcmsiv);
        }
        #[cfg(feature = "pgcmsiv")]
        if self.pgcmsiv {
            count += 1;
            scheme = Some(Scheme::Pgcmsiv);
        }
        #[cfg(feature = "mock")]
        if self.mock1 {
            count += 1;
            scheme = Some(Scheme::Mock1);
        }
        #[cfg(feature = "mock")]
        if self.mock2 {
            count += 1;
            scheme = Some(Scheme::Mock2);
        }

        if count > 1 {
            usage_error("only one scheme flag may be specified");
        }

        Ok(scheme)
    }

    fn is_set(&self) -> bool {
        #[cfg(feature = "dsiv")]
        if self.dsiv {
            return true;
        }
        #[cfg(feature = "psiv")]
        if self.psiv {
            return true;
        }
        #[cfg(feature = "dgcmsiv")]
        if self.dgcmsiv {
            return true;
        }
        #[cfg(feature = "pgcmsiv")]
        if self.pgcmsiv {
            return true;
        }
        #[cfg(feature = "mock")]
        if self.mock1 {
            return true;
        }
        #[cfg(feature = "mock")]
        if self.mock2 {
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
    /// Convert flags to Option<Encoding>, returning error if multiple are set
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

    /// Check if any encoding flag is set
    fn is_set(&self) -> bool {
        self.c32 || self.b32 || self.b64 || self.hex
    }
}

/// Combined format specification (scheme + encoding)
#[derive(Debug)]
struct FormatSpec {
    scheme: Scheme,
    encoding: Encoding,
}

impl FormatSpec {
    /// Parse format from --format string, scheme flags, encoding flags, and config
    /// Validates that --format doesn't conflict with individual flags
    fn parse(
        format_str: Option<String>,
        scheme_flags: &SchemeFlags,
        encoding_flags: &EncodingFlags,
        config: Option<&Config>,
    ) -> Result<Self> {
        // Check for conflicts between --format and individual flags
        if format_str.is_some() && scheme_flags.is_set() {
            usage_error("--format cannot be combined with scheme flags");
        }
        if format_str.is_some() && encoding_flags.is_set() {
            usage_error("--format cannot be combined with encoding flags");
        }

        // Parse --format if provided. A malformed or unknown format
        // identifier is a usage error (CLI.md §5, §8).
        if let Some(fmt_str) = format_str {
            let format = Format::from_str(&fmt_str)
                .unwrap_or_else(|e| usage_error(format!("invalid format '{fmt_str}': {e}")));
            validate_secure_scheme(format.scheme())?;
            return Ok(Self {
                scheme: format.scheme(),
                encoding: format.encoding(),
            });
        }

        // Otherwise get scheme and encoding from flags or config
        let scheme = get_scheme(scheme_flags.to_scheme()?, config)?;
        let encoding = get_encoding(encoding_flags.to_encoding()?, config)?;

        Ok(Self { scheme, encoding })
    }
}

impl std::fmt::Display for FormatSpec {
    /// Format as a format string (e.g., "dsiv.b64")
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

        /// Encryption key (128 hex chars); conflicts with --keyless
        #[arg(short, long, conflicts_with = "profile", conflicts_with = "keyless")]
        key: Option<String>,

        /// Use named key profile
        #[arg(short, long, conflicts_with = "key", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use the fixed public test key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "key", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "dsiv.b64", "dgcmsiv.b32").
        /// Cannot be combined with scheme or encoding flags
        #[arg(short, long)]
        format: Option<String>,

        /// Disable CLI line framing (no stdin newline strip, no stdout newline)
        #[arg(short = '0', long)]
        raw: bool,

        /// Scheme selection
        #[command(flatten)]
        scheme: SchemeFlags,

        /// Encoding selection
        #[command(flatten)]
        encoding: EncodingFlags,
    },

    /// Decode+decrypt an obtext string
    #[command(visible_alias = "d")]
    Dec {
        /// Obtext string (reads from stdin if not provided)
        text: Option<String>,

        /// Encryption key (128 hex chars); conflicts with --keyless
        #[arg(short, long, conflicts_with = "profile", conflicts_with = "keyless")]
        key: Option<String>,

        /// Use named key profile
        #[arg(short, long, conflicts_with = "key", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use the fixed public test key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "key", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "dsiv.b64", "dgcmsiv.b32").
        /// Cannot be combined with scheme or encoding flags
        #[arg(short, long)]
        format: Option<String>,

        /// Disable CLI line framing (no stdin newline strip, no stdout newline)
        #[arg(short = '0', long)]
        raw: bool,

        /// Scheme selection
        #[command(flatten)]
        scheme: SchemeFlags,

        /// Encoding selection
        #[command(flatten)]
        encoding: EncodingFlags,
    },

    /// Initialize configuration with random profile
    #[command(visible_alias = "i")]
    Init {
        /// Name for the key profile (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// Manage configuration
    #[command(visible_alias = "c")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,
    },

    /// Manage key profiles
    #[command(visible_alias = "p")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Output the encryption key (canonical 128-char hex)
    #[command(visible_alias = "k")]
    Key {
        /// Use named key profile
        #[arg(short, long)]
        profile: Option<String>,

        /// Use the fixed public test key (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,

        /// Accepted no-op: hex is the only key output format.
        #[arg(short = 'x', long)]
        hex: bool,
    },

    /// Generate a fresh random key and print it (does not touch any profile)
    Keygen,

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
        /// Scheme selection
        #[command(flatten)]
        scheme: SchemeFlags,

        /// Encoding selection
        #[command(flatten)]
        encoding: EncodingFlags,

        /// Set default key profile
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all key profiles
    #[command(visible_alias = "l")]
    List,
    /// Show a specific key profile
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
    /// Create a new key profile
    #[command(visible_alias = "c")]
    Create {
        /// Profile name
        name: String,

        /// Encryption key (128 hex chars)
        #[arg(short, long)]
        key: Option<String>,
    },
    /// Delete a key profile
    #[command(visible_alias = "d")]
    Delete {
        /// Profile name
        name: String,
    },
    /// Rename a key profile
    #[command(visible_alias = "r")]
    #[command(visible_alias = "mv")]
    Rename {
        /// Current profile name
        old_name: String,

        /// New profile name
        new_name: String,
    },
    /// Set key for a profile
    Set {
        /// Profile name
        name: String,

        /// Encryption key (128 hex chars)
        #[arg(short, long)]
        key: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // --version is a global option: it must work before or after any
    // command name and must not require a key, config, profile, or
    // stdin. Handle it first and exit 0.
    if cli.version {
        println!("{}", version_line());
        return Ok(());
    }

    // With --version handled, a subcommand is required.
    let command = match cli.command {
        Some(c) => c,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            std::process::exit(2);
        }
    };

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

    match command {
        Commands::Enc {
            text,
            key,
            profile,
            keyless,
            format,
            raw,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            enc_command(text, key, profile, keyless, format_spec, raw, cfg)
        }

        Commands::Dec {
            text,
            key,
            profile,
            keyless,
            format,
            raw,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            dec_command(text, key, profile, keyless, format_spec, raw, cfg)
        }

        Commands::Init { name } => config::init_command(&name),

        Commands::Config { command, keyless } => match command {
            Some(ConfigCommands::Show) | None => {
                config::config_show_command(keyless)
            }
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
            ProfileCommands::Create { name, key } => {
                config::profile_create_command(&name, key.as_deref())
            }
            ProfileCommands::Delete { name } => config::profile_delete_command(&name),
            ProfileCommands::Rename { old_name, new_name } => {
                config::profile_rename_command(&old_name, &new_name)
            }
            ProfileCommands::Set { name, key } => {
                config::profile_set_command(&name, key.as_deref())
            }
        },

        Commands::Key {
            profile,
            keyless,
            hex,
        } => key_command(profile, keyless, hex),

        Commands::Keygen => {
            // Convenience: print a fresh canonical-hex key. Does not
            // create or modify any profile.
            println!("{}", oboron::generate_key());
            Ok(())
        }

        Commands::Completion { shell } => {
            completions::generate_completion(shell);
            Ok(())
        }
    }
}

/// Write `s` to stdout, appending a single `\n` in default framing and
/// nothing in `--raw` framing (CLI.md §7).
fn write_output(s: &str, raw: bool) -> Result<()> {
    use std::io::Write;
    let mut out = io::stdout();
    if raw {
        out.write_all(s.as_bytes())?;
    } else {
        out.write_all(s.as_bytes())?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}

fn enc_command(
    text: Option<String>,
    key: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    raw: bool,
    cfg: Option<Config>,
) -> Result<()> {
    // Get text from argument or stdin
    let text = get_text_input(text, raw)?;

    // Create format
    let format = format_spec.to_string();

    // Get ob instance
    let encd = if keyless {
        oboron::Ob::new_keyless(&format)?.enc(&text)?
    } else {
        let hex_key = get_key(key.as_ref(), profile.as_deref(), cfg.as_ref())?;
        oboron::Ob::new(&format, &hex_key)?.enc(&text)?
    };
    write_output(&encd, raw)
}

fn dec_command(
    text: Option<String>,
    key: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    raw: bool,
    cfg: Option<Config>,
) -> Result<()> {
    let format = format_spec.to_string();

    // Build the cipher first. Missing/invalid key and format problems
    // are operation/usage errors with their own messages (CLI.md §6,
    // §8) and are NOT part of the uniform dec contract. The scheme is
    // supplied by the resolved format — `dec` never auto-detects.
    let ob = if keyless {
        oboron::Ob::new_keyless(&format)?
    } else {
        let hex_key = get_key(key.as_ref(), profile.as_deref(), cfg.as_ref())?;
        oboron::Ob::new(&format, &hex_key)?
    };

    // From here every failure is on (secret) obtext input and MUST
    // collapse to one uniform message + exit 1 (CLI.md §8).
    let input = read_dec_input(text, raw);
    match ob.dec(&input) {
        Ok(plaintext) => write_output(&plaintext, raw),
        Err(_) => dec_failure(),
    }
}

/// Read the obtext input for `dec`. Framing matches `enc` (§7): a
/// positional `[TEXT]` is used exactly; otherwise stdin is read and, in
/// default mode, one trailing line ending is stripped. Unlike `enc`,
/// any failure to obtain non-empty valid-UTF-8 input collapses to the
/// uniform dec failure (§8) — an empty or non-UTF-8 obtext is just
/// invalid obtext, and reporting *why* would make `dec` an oracle.
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
            Some("dsiv".to_string()),
            Some("c32".to_string()),
        )
    });

    if let Some(scheme) = scheme_override {
        validate_secure_scheme(scheme)?;
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

fn key_command(profile: Option<String>, keyless: bool, _hex: bool) -> Result<()> {
    // Resolve the key as canonical 128-char hex from whichever source
    // applies, then emit. Hex is the only key output format; `--hex` is
    // an accepted no-op.
    let hex_key = if keyless {
        oboron::HARDCODED_KEY_HEX.to_string()
    } else {
        let cfg = config::load_config().ok();
        let active_profile_name = cfg.as_ref().and_then(|c| c.profile.as_deref());

        if let Some(prof) = profile.as_deref().or(active_profile_name) {
            oboron_cli_core::load_profile_key_as_hex(prof)?
        } else if let Ok(env_key) = std::env::var("OBORON_KEY") {
            require_hex_key(&env_key).context("invalid $OBORON_KEY")?
        } else {
            anyhow::bail!(
                "No key specified: provide --profile, set $OBORON_KEY, or run 'ob init'"
            );
        }
    };

    println!("{hex_key}");
    Ok(())
}

/// Validate that `key` is a canonical oboron key — exactly 128
/// lowercase hex characters — and return it. The CLI accepts no other
/// key form (CLI.md §6).
fn require_hex_key(key: &str) -> Result<String> {
    let is_canonical =
        key.len() == 128 && key.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !is_canonical {
        anyhow::bail!("key must be exactly 128 lowercase hex characters");
    }
    Ok(key.to_string())
}

fn get_key(key: Option<&String>, profile: Option<&str>, config: Option<&Config>) -> Result<String> {
    // 1. Explicit --key flag (128-char hex only).
    if let Some(key_str) = key {
        return require_hex_key(key_str).context("invalid --key");
    }

    // 2. Environment variable (128-char hex only).
    if let Ok(env_key) = std::env::var("OBORON_KEY") {
        return require_hex_key(&env_key).context("invalid $OBORON_KEY");
    }

    // 3-4. Profile (explicit --profile or default from config). This is
    // a convenience feature outside the CLI spec; the stored profile key
    // is already normalized to canonical hex by the core.
    let profile_name = profile.or_else(|| config.and_then(|c| c.profile.as_deref()));

    if let Some(name) = profile_name {
        return oboron_cli_core::load_profile_key_as_hex(name);
    }

    Err(anyhow::anyhow!(
        "No key specified: provide --key, set $OBORON_KEY, use --profile, or run 'ob init'"
    ))
}

/// Read the plaintext/obtext input for `enc` / `dec`.
///
/// If `[TEXT]` is supplied as a positional argument it is used exactly:
/// stdin is not read and no trailing newline is stripped. Otherwise all
/// of stdin is read as UTF-8. In default (non-raw) framing exactly one
/// trailing line ending is removed — `\r\n` if present, else a lone
/// `\n`; in `--raw` framing nothing is stripped. The resulting input
/// must be non-empty (CLI.md §7).
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
    // Explicit flag takes precedence.
    if let Some(scheme) = scheme_override {
        validate_secure_scheme(scheme)?;
        return Ok(scheme);
    }

    // Then any persisted config default (convenience feature).
    if let Some(cfg) = config {
        if let Some(scheme_str) = cfg.scheme.as_deref() {
            let scheme = Scheme::from_str(scheme_str).map_err(|e| anyhow::anyhow!("{}", e))?;
            validate_secure_scheme(scheme)?;
            return Ok(scheme);
        }
    }

    // Otherwise the built-in default scheme (CLI.md §5): dsiv.
    Scheme::from_str("dsiv").map_err(|e| anyhow::anyhow!("{}", e))
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
    // Built-in default encoding (CLI.md §5): c32.
    Ok(Encoding::C32)
}

/// Accept only the authenticated core schemes (plus mock under the
/// testing feature). Every scheme oboron now exposes is authenticated,
/// so this is a thin guard kept for symmetry with the config path.
fn validate_secure_scheme(_scheme: Scheme) -> Result<()> {
    Ok(())
}
