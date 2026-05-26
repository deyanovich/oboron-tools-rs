//! CLI application for oboron secure schemes (a-tier and u-tier)

mod completions;
mod config;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use config::Config;
use oboron::{Encoding, Format, Scheme};
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "ob")]
#[command(version, about = "Reversible hash-like references (secure schemes)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct SchemeFlags {
    /// Use upbc scheme (probabilistic unauthenticated)
    #[cfg(feature = "upbc")]
    #[arg(short = 'u', long, alias = "21p")]
    upbc: bool,

    /// Use aags scheme (deterministic AES-GCM-SIV)
    #[cfg(feature = "aags")]
    #[arg(short = 'g', long, alias = "31")]
    aags: bool,

    /// Use apgs scheme (probabilistic AES-GCM-SIV)
    #[cfg(feature = "apgs")]
    #[arg(short = 'G', long, alias = "31p")]
    apgs: bool,

    /// Use aasv scheme (deterministic AES-SIV)
    #[cfg(feature = "aasv")]
    #[arg(short = 's', long, alias = "32")]
    aasv: bool,

    /// Use apsv scheme (probabilistic AES-SIV)
    #[cfg(feature = "apsv")]
    #[arg(short = 'S', long, alias = "32p")]
    apsv: bool,

    /// Use mock1 scheme (testing, identity)
    #[cfg(feature = "mock")]
    #[arg(long, alias = "70", hide = true)]
    mock1: bool,

    /// Use mock2 scheme (testing, string reversal)
    #[cfg(feature = "mock")]
    #[arg(long, alias = "71", hide = true)]
    mock2: bool,
}

impl SchemeFlags {
    fn to_scheme(&self) -> Result<Option<Scheme>> {
        let mut count = 0;
        let mut scheme = None;

        #[cfg(feature = "upbc")]
        if self.upbc {
            count += 1;
            scheme = Some(Scheme::Upbc);
        }
        #[cfg(feature = "aags")]
        if self.aags {
            count += 1;
            scheme = Some(Scheme::Aags);
        }
        #[cfg(feature = "apgs")]
        if self.apgs {
            count += 1;
            scheme = Some(Scheme::Apgs);
        }
        #[cfg(feature = "aasv")]
        if self.aasv {
            count += 1;
            scheme = Some(Scheme::Aasv);
        }
        #[cfg(feature = "apsv")]
        if self.apsv {
            count += 1;
            scheme = Some(Scheme::Apsv);
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
            anyhow::bail!("Only one scheme flag can be specified at a time");
        }

        Ok(scheme)
    }

    fn is_set(&self) -> bool {
        #[cfg(feature = "upbc")]
        if self.upbc {
            return true;
        }
        #[cfg(feature = "aags")]
        if self.aags {
            return true;
        }
        #[cfg(feature = "apgs")]
        if self.apgs {
            return true;
        }
        #[cfg(feature = "aasv")]
        if self.aasv {
            return true;
        }
        #[cfg(feature = "apsv")]
        if self.apsv {
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
            anyhow::bail!("Only one encoding flag can be specified at a time");
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
            anyhow::bail!("Cannot use --format together with scheme flags");
        }
        if format_str.is_some() && encoding_flags.is_set() {
            anyhow::bail!("Cannot use --format together with encoding flags");
        }

        // Parse --format if provided
        if let Some(fmt_str) = format_str {
            let format = Format::from_str(&fmt_str).map_err(|e| anyhow::anyhow!("{}", e))?;
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
    /// Format as a format string (e.g., "zrbcx.b64")
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

        /// Encryption key (128 hex chars, or legacy 86-char base64; for non-ztier schemes)
        #[arg(short, long, conflicts_with = "profile", conflicts_with = "keyless")]
        key: Option<String>,

        /// Use named key profile
        #[arg(short, long, conflicts_with = "key", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "key", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "zrbcx.b64", "aags.b32")
        /// Cannot be combined with scheme or encoding flags
        #[arg(short, long)]
        format: Option<String>,

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

        /// Encryption key (128 hex chars, or legacy 86-char base64; for non-ztier schemes)
        #[arg(short, long, conflicts_with = "profile", conflicts_with = "keyless")]
        key: Option<String>,

        /// Use named key profile
        #[arg(short, long, conflicts_with = "key", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "key", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "zrbcx.b64", "aags.b32")
        /// Cannot be combined with scheme or encoding flags
        #[arg(short, long)]
        format: Option<String>,

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

    /// Output the encryption key (canonical hex by default)
    #[command(visible_alias = "k")]
    Key {
        /// Use named key profile
        #[arg(short, long)]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,

        /// Deprecated no-op: hex is now the default output format.
        #[arg(short = 'x', long)]
        hex: bool,

        /// Output the key as legacy base64 (deprecated; removed before oboron 1.0)
        #[arg(short = 'B', long, conflicts_with = "hex")]
        base64: bool,
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

        /// Encryption key (128 hex chars, or legacy 86-char base64)
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

        /// Encryption key (128 hex chars, or legacy 86-char base64)
        #[arg(short, long)]
        key: Option<String>,
    },
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
        Commands::Enc {
            text,
            key,
            profile,
            keyless,
            format,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            enc_command(text, key, profile, keyless, format_spec, cfg)
        }

        Commands::Dec {
            text,
            key,
            profile,
            keyless,
            format,
            scheme,
            encoding,
        } => {
            let cfg = config::load_config().ok();
            let scheme_is_explicit = scheme.is_set() || format.is_some();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            dec_command(text, key, profile, keyless, format_spec, scheme_is_explicit, cfg)
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
            base64,
        } => key_command(profile, keyless, hex, base64),

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

fn enc_command(
    text: Option<String>,
    key: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    cfg: Option<Config>,
) -> Result<()> {
    // Get text from argument or stdin
    let text = get_text_input(text)?;

    // Create format
    let format = format_spec.to_string();

    // Get ob instance
    if keyless {
        let ob = oboron::Ob::new_keyless(&format)?;
        let encd = ob.enc(&text)?;
        println!("{}", encd);
    } else {
        let b64_key = get_key(key.as_ref(), profile.as_deref(), cfg.as_ref())?;
        let ob = oboron::Ob::new(&format, &b64_key)?;
        let encd = ob.enc(&text)?;
        println!("{}", encd);
    }

    Ok(())
}

fn dec_command(
    text: Option<String>,
    key: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    scheme_is_explicit: bool,
    cfg: Option<Config>,
) -> Result<()> {
    // Get text from argument or stdin
    let text = get_text_input(text)?;

    // Create format
    let format = format_spec.to_string();

    // Get ob instance and decode
    if keyless {
        let ob = oboron::Ob::new_keyless(&format)?;
        let decd = if scheme_is_explicit {
            ob.dec(&text)?
        } else {
            ob.autodec(&text)?
        };
        println!("{}", decd);
    } else {
        let b64_key = get_key(key.as_ref(), profile.as_deref(), cfg.as_ref())?;
        let ob = oboron::Ob::new(&format, &b64_key)?;
        let decd = if scheme_is_explicit {
            ob.dec(&text)?
        } else {
            ob.autodec(&text)?
        };
        println!("{}", decd);
    }

    Ok(())
}

fn config_set_command(
    scheme_override: Option<Scheme>,
    encoding_override: Option<Encoding>,
    profile: Option<String>,
) -> Result<()> {
    let mut config = config::load_config().unwrap_or_else(|_| Config {
        profile: Some("default".to_string()),
        scheme: Some("aasv".to_string()),
        encoding: Some("c32".to_string()),
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

fn key_command(
    profile: Option<String>,
    keyless: bool,
    _hex: bool,
    base64: bool,
) -> Result<()> {
    // Resolve the key as canonical hex from whichever source applies,
    // then emit. Hex is the canonical form and the default; `--hex` is
    // an accepted no-op, `--base64` opts into the deprecated form.
    let hex_key = if keyless {
        oboron::HARDCODED_KEY_HEX.to_string()
    } else {
        let cfg = config::load_config().ok();
        let active_profile_name = cfg.as_ref().and_then(|c| c.profile.as_deref());

        if let Some(prof) = profile.as_deref().or(active_profile_name) {
            // Migrate any legacy base64 profile in place and print a
            // notice, exactly like enc / dec / config show / profile show.
            oboron_cli_core::commands::load_profile_key_with_notice(prof)?
        } else if let Ok(env_key) = std::env::var("OBORON_KEY") {
            oboron_cli_core::normalize_key_to_hex(&env_key).context("invalid $OBORON_KEY")?
        } else {
            anyhow::bail!(
                "No key specified: provide --profile, set $OBORON_KEY, or run 'ob init'"
            );
        }
    };

    if base64 {
        warn_base64_output();
        let key_bytes = hex::decode(&hex_key).context("decode key")?;
        println!("{}", data_encoding::BASE64URL_NOPAD.encode(&key_bytes));
    } else {
        println!("{hex_key}");
    }

    Ok(())
}

fn warn_base64_output() {
    eprintln!(
        "warning: base64 key output is deprecated and will be removed before \
         oboron 1.0;"
    );
    eprintln!("         hex is the canonical key format and the default for 'ob key'.");
}

fn get_key(key: Option<&String>, profile: Option<&str>, config: Option<&Config>) -> Result<String> {
    // 1. Explicit --key flag
    if let Some(key_str) = key {
        let (hex, fmt) = oboron_cli_core::normalize_key_classify(key_str).context("invalid --key")?;
        if fmt == oboron_cli_core::KeyFormat::LegacyBase64 {
            warn_base64_via("--key");
        }
        return Ok(hex);
    }

    // 2. Environment variable
    if let Ok(env_key) = std::env::var("OBORON_KEY") {
        let (hex, fmt) =
            oboron_cli_core::normalize_key_classify(&env_key).context("invalid $OBORON_KEY")?;
        if fmt == oboron_cli_core::KeyFormat::LegacyBase64 {
            warn_base64_via("$OBORON_KEY");
        }
        return Ok(hex);
    }

    // 3-4. Profile (explicit --profile or default from config)
    let profile_name = profile.or_else(|| config.and_then(|c| c.profile.as_deref()));

    if let Some(name) = profile_name {
        return oboron_cli_core::commands::load_profile_key_with_notice(name);
    }

    Err(anyhow::anyhow!(
        "No key specified: provide --key, set $OBORON_KEY, use --profile, or run 'ob init'"
    ))
}

fn warn_base64_via(source: &str) {
    eprintln!(
        "warning: {source} was given as legacy base64. base64 keys are deprecated \
         and will be removed before oboron 1.0;"
    );
    eprintln!("         pass a 128-character hex key instead. The base64 key was accepted.");
}

#[allow(dead_code)]
fn validate_base64_key(key_str: &str) -> Result<()> {
    // Kept for any remaining callers (e.g. obz_main); new code uses
    // oboron_cli_core::normalize_key_to_hex which accepts both formats.
    if key_str.len() != 86 {
        return Err(anyhow::anyhow!(
            "Key must be 86 base64 chars, got {} chars",
            key_str.len()
        ));
    }
    use data_encoding::BASE64URL_NOPAD;
    let key_bytes = BASE64URL_NOPAD
        .decode(key_str.as_bytes())
        .context("Invalid key base64 encoding")?;
    if key_bytes.len() != 64 {
        return Err(anyhow::anyhow!(
            "Key must decode to 64 bytes, got {} bytes",
            key_bytes.len()
        ));
    }

    Ok(())
}

fn get_text_input(text: Option<String>) -> Result<String> {
    match text {
        Some(t) => Ok(t),
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("failed to read from stdin")?;
            let trimmed = buffer.trim();
            if trimmed.is_empty() {
                anyhow::bail!("no input provided");
            }
            Ok(trimmed.to_string())
        }
    }
}

fn get_scheme(scheme_override: Option<Scheme>, config: Option<&Config>) -> Result<Scheme> {
    // Explicit flag takes precedence
    if let Some(scheme) = scheme_override {
        validate_secure_scheme(scheme)?;
        return Ok(scheme);
    }

    // Fall back to config
    if let Some(cfg) = config {
        if let Some(scheme_str) = cfg.scheme.as_deref() {
            let scheme = Scheme::from_str(scheme_str).map_err(|e| anyhow::anyhow!("{}", e))?;
            validate_secure_scheme(scheme)?;
            return Ok(scheme);
        }
    }

    Err(anyhow::anyhow!(
        "scheme not specified: run 'ob init' or use a scheme flag or --format"
    ))
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
    Err(anyhow::anyhow!(
        "encoding not specified: run 'ob init' or use an encoding flag or --format"
    ))
}

fn validate_secure_scheme(scheme: Scheme) -> Result<()> {
    match scheme {
        #[cfg(feature = "aags")]
        Scheme::Aags => Ok(()),
        #[cfg(feature = "apgs")]
        Scheme::Apgs => Ok(()),
        #[cfg(feature = "aasv")]
        Scheme::Aasv => Ok(()),
        #[cfg(feature = "apsv")]
        Scheme::Apsv => Ok(()),
        #[cfg(feature = "upbc")]
        Scheme::Upbc => Ok(()),
        #[cfg(feature = "mock")]
        Scheme::Mock1 => Ok(()),
        #[cfg(feature = "mock")]
        Scheme::Mock2 => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Invalid secure scheme: {}.  Use ob for secure schemes (aags, aasv, etc.) or obz for z-tier schemes",
            scheme.as_str()
        )),
    }
}
