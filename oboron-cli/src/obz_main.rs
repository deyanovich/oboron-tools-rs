//! CLI application for oboron z-tier schemes (obfuscation only)
//!
//! ⚠️ WARNING: This tool provides NO cryptographic security.
//! Use only for obfuscation, never for actual encryption.

mod obz_completions;
mod obz_config;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use oboron::{Encoding, Format, Scheme};
use obz_config::Config;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "obz")]
#[command(version, about = "Z-tier obfuscation tool (NOT SECURE)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct SchemeFlags {
    /// Use legacy scheme (Base32-based obfuscation)
    #[cfg(feature = "legacy")]
    #[arg(short = 'l', long)]
    legacy: bool,

    /// Use zrbcx scheme (XOR-based obfuscation)
    #[cfg(feature = "zrbcx")]
    #[arg(short = 'r', long)]
    zrbcx: bool,

    /// Use zmock1 scheme (testing, identity)
    #[cfg(feature = "zmock")]
    #[arg(long, hide = true)]
    zmock1: bool,
}

impl SchemeFlags {
    fn to_scheme(&self) -> Result<Option<Scheme>> {
        let mut count = 0;
        let mut scheme = None;

        #[cfg(feature = "legacy")]
        if self.legacy {
            count += 1;
            scheme = Some(Scheme::Legacy);
        }
        #[cfg(feature = "zrbcx")]
        if self.zrbcx {
            count += 1;
            scheme = Some(Scheme::Zrbcx);
        }
        #[cfg(feature = "zmock")]
        if self.zmock1 {
            count += 1;
            scheme = Some(Scheme::Zmock1);
        }

        if count > 1 {
            anyhow::bail!("Only one scheme flag can be specified at a time");
        }

        Ok(scheme)
    }

    fn is_set(&self) -> bool {
        #[cfg(feature = "legacy")]
        if self.legacy {
            return true;
        }
        #[cfg(feature = "zrbcx")]
        if self.zrbcx {
            return true;
        }
        #[cfg(feature = "zmock")]
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
            anyhow::bail!("Only one encoding flag can be specified at a time");
        }

        Ok(encoding)
    }

    fn is_set(&self) -> bool {
        self.c32 || self.b32 || self.b64 || self.hex
    }
}

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
            anyhow::bail!("Cannot use --format together with scheme flags");
        }
        if format_str.is_some() && encoding_flags.is_set() {
            anyhow::bail!("Cannot use --format together with encoding flags");
        }

        if let Some(fmt_str) = format_str {
            let format = Format::from_str(&fmt_str).map_err(|e| anyhow::anyhow!("{}", e))?;
            validate_ztier_scheme(format.scheme())?;
            return Ok(Self {
                scheme: format.scheme(),
                encoding: format.encoding(),
            });
        }

        let scheme = get_scheme(scheme_flags.to_scheme()?, config)?;
        // Legacy has a fixed encoding (B32/RFC lowercase) — encoding flags are incompatible
        #[cfg(feature = "legacy")]
        if scheme == Scheme::Legacy {
            if encoding_flags.is_set() {
                anyhow::bail!(
                    "--legacy is incompatible with encoding flags (--b32, --b64, --hex, --c32): \
                     legacy has a fixed encoding"
                );
            }
            return Ok(Self {
                scheme,
                encoding: Encoding::B32,
            });
        }
        let encoding = get_encoding(encoding_flags.to_encoding()?, config)?;

        Ok(Self { scheme, encoding })
    }
}

impl std::fmt::Display for FormatSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use Format::Display so Legacy emits "legacy" (not "legacy.b32")
        write!(f, "{}", Format::new(self.scheme, self.encoding))
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt+encode a plaintext string
    #[command(visible_alias = "e")]
    Enc {
        /// Plaintext string (reads from stdin if not provided)
        text: Option<String>,

        /// Secret key (43 base64 chars)
        #[arg(short = 's', long, conflicts_with = "profile", conflicts_with = "keyless")]
        secret: Option<String>,

        /// Use named secret profile
        #[arg(short, long, conflicts_with = "secret", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "secret", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "zrbcx.b64")
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

        /// Secret key (43 base64 chars)
        #[arg(short = 's', long, conflicts_with = "profile", conflicts_with = "keyless")]
        secret: Option<String>,

        /// Use named secret profile
        #[arg(short, long, conflicts_with = "secret", conflicts_with = "keyless")]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long, conflicts_with = "secret", conflicts_with = "profile")]
        keyless: bool,

        /// Format specification (e.g., "zrbcx.b64")
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
        /// Name for the secret profile (default: "default")
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

    /// Manage secret profiles
    #[command(visible_alias = "p")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Output the secret key
    #[command(visible_alias = "s")]
    Secret {
        /// Secret key (43 base64 chars)
        #[arg(short = 's', long)]
        secret: Option<String>,

        /// Use named secret profile
        #[arg(short, long)]
        profile: Option<String>,

        /// Use hardcoded key (INSECURE - testing only)
        #[arg(short = 'K', long)]
        keyless: bool,

        /// Output secret as hex instead of base64
        #[arg(short = 'x', long)]
        hex: bool,
    },

    /// Generate shell completion script
    Completion {
        #[command(subcommand)]
        shell: obz_completions::Shell,
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

        /// Secret key (43 base64 chars)
        #[arg(short = 's', long)]
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

    /// Set secret for a profile
    Set {
        /// Profile name
        name: String,

        /// Secret key (43 base64 chars)
        #[arg(short = 's', long)]
        secret: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Enc {
            text,
            secret,
            profile,
            keyless,
            format,
            scheme,
            encoding,
        } => {
            let cfg = obz_config::load_config().ok();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            enc_command(text, secret, profile, keyless, format_spec, cfg)
        }

        Commands::Dec {
            text,
            secret,
            profile,
            keyless,
            format,
            scheme,
            encoding,
        } => {
            let cfg = obz_config::load_config().ok();
            let scheme_is_explicit = scheme.is_set() || format.is_some();
            let format_spec = FormatSpec::parse(format, &scheme, &encoding, cfg.as_ref())?;
            dec_command(
                text,
                secret,
                profile,
                keyless,
                format_spec,
                scheme_is_explicit,
                cfg,
            )
        }

        Commands::Init { name } => obz_config::init_command(&name),

        Commands::Config { command, keyless } => match command {
            Some(ConfigCommands::Show) | None => {
                obz_config::config_show_command(keyless)
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
            ProfileCommands::List => obz_config::profile_list_command(),
            ProfileCommands::Show { name } => obz_config::profile_show_command(name.as_deref()),
            ProfileCommands::Activate { name } => obz_config::profile_activate_command(&name),
            ProfileCommands::Create { name, secret } => {
                obz_config::profile_create_command(&name, secret.as_deref())
            }
            ProfileCommands::Delete { name } => obz_config::profile_delete_command(&name),
            ProfileCommands::Rename { old_name, new_name } => {
                obz_config::profile_rename_command(&old_name, &new_name)
            }
            ProfileCommands::Set { name, secret } => {
                obz_config::profile_set_command(&name, secret.as_deref())
            }
        },

        Commands::Secret {
            secret,
            profile,
            keyless,
            hex,
        } => secret_command(secret, profile, keyless, hex),

        Commands::Completion { shell } => {
            obz_completions::generate_completion(shell);
            Ok(())
        }
    }
}

fn enc_command(
    text: Option<String>,
    secret: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    cfg: Option<Config>,
) -> Result<()> {
    let text = get_text_input(text)?;
    let format = format_spec.to_string();

    if keyless {
        let obz = oboron::ztier::Obz::new_keyless(&format)?;
        let encd = obz.enc(&text)?;
        println!("{}", encd);
    } else {
        let b64_secret = get_secret(secret.as_ref(), profile.as_deref(), cfg.as_ref())?;
        let obz = oboron::ztier::Obz::new(&format, &b64_secret)?;
        let encd = obz.enc(&text)?;
        println!("{}", encd);
    }

    Ok(())
}

fn dec_command(
    text: Option<String>,
    secret: Option<String>,
    profile: Option<String>,
    keyless: bool,
    format_spec: FormatSpec,
    scheme_is_explicit: bool,
    cfg: Option<Config>,
) -> Result<()> {
    let text = get_text_input(text)?;
    let format = format_spec.to_string();

    if keyless {
        let obz = oboron::ztier::Obz::new_keyless(&format)?;
        let decd = if scheme_is_explicit {
            obz.dec(&text)?
        } else {
            obz.autodec(&text)?
        };
        println!("{}", decd);
    } else {
        let b64_secret = get_secret(secret.as_ref(), profile.as_deref(), cfg.as_ref())?;
        let obz = oboron::ztier::Obz::new(&format, &b64_secret)?;
        let decd = if scheme_is_explicit {
            obz.dec(&text)?
        } else {
            obz.autodec(&text)?
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
    let mut config = obz_config::load_config().unwrap_or(Config {
        profile: "default".to_string(),
        scheme: "zrbcx".to_string(),
        encoding: "c32".to_string(),
    });

    if let Some(scheme) = scheme_override {
        validate_ztier_scheme(scheme)?;
        config.scheme = scheme.to_string();
    }

    if let Some(encoding) = encoding_override {
        config.encoding = encoding.to_string();
    }

    if let Some(p) = profile {
        config.profile = p;
    }

    obz_config::save_config(&config)?;

    println!("✓ Configuration updated");
    println!("  Profile:  {}", config.profile);
    println!("  Scheme:   {}", config.scheme);
    println!("  Encoding: {}", config.encoding);

    Ok(())
}

fn secret_command(
    secret: Option<String>,
    profile: Option<String>,
    keyless: bool,
    hex: bool,
) -> Result<()> {
    use data_encoding::{BASE64URL_NOPAD, HEXLOWER};

    if keyless {
        if hex {
            let secret_bytes = oboron::HARDCODED_KEY_BYTES;
            let hex_secret = HEXLOWER.encode(&secret_bytes[0..32]);
            println!("{}", hex_secret);
        } else {
            let secret_bytes = oboron::HARDCODED_KEY_BYTES;
            let b64_secret = BASE64URL_NOPAD.encode(&secret_bytes[0..32]);
            println!("{}", b64_secret);
        }
        return Ok(());
    }

    let cfg = obz_config::load_config().ok();

    if let Some(s) = secret {
        if hex {
            let secret_bytes = BASE64URL_NOPAD
                .decode(s.as_bytes())
                .context("Failed to decode base64 secret")?;
            let hex_secret = HEXLOWER.encode(&secret_bytes);
            println!("{}", hex_secret);
        } else {
            println!("{}", s);
        }
    } else if let Some(prof) = profile
        .as_deref()
        .or_else(|| cfg.as_ref().map(|c| c.profile.as_str()))
    {
        let profile = obz_config::load_profile(prof)?;
        if let Some(s) = &profile.secret {
            println!(
                "{}",
                if hex {
                    let secret_bytes = BASE64URL_NOPAD.decode(s.as_bytes())?;
                    HEXLOWER.encode(&secret_bytes)
                } else {
                    s.clone()
                }
            );
        } else {
            anyhow::bail!("Profile '{}' has no secret", prof);
        }
    } else if let Ok(env_secret) = std::env::var("OBORON_SECRET") {
        println!(
            "{}",
            if hex {
                let secret_bytes = BASE64URL_NOPAD
                    .decode(env_secret.as_bytes())
                    .context("Failed to decode base64 secret")?;
                HEXLOWER.encode(&secret_bytes)
            } else {
                env_secret
            }
        );
    } else {
        anyhow::bail!(
            "No secret specified: provide --secret, set $OBORON_SECRET, use --profile, or run 'obz init'"
        );
    }

    Ok(())
}

fn get_secret(
    secret: Option<&String>,
    profile: Option<&str>,
    config: Option<&Config>,
) -> Result<String> {
    // 1. Explicit --secret flag
    if let Some(secret_str) = secret {
        validate_base64_secret(secret_str)?;
        return Ok(secret_str.clone());
    }

    // 2. Environment variable
    if let Ok(env_secret) = std::env::var("OBORON_SECRET") {
        validate_base64_secret(&env_secret)?;
        return Ok(env_secret);
    }

    // 3-4. Profile (explicit or default from config)
    let profile_name = profile.or_else(|| config.map(|c| c.profile.as_str()));

    if let Some(name) = profile_name {
        let profile = obz_config::load_profile(name)?;
        if let Some(s) = &profile.secret {
            validate_base64_secret(s)?;
            return Ok(s.clone());
        }
        anyhow::bail!("Profile '{}' has no secret", name);
    }

    Err(anyhow::anyhow!(
        "No secret specified: provide --secret, set $OBORON_SECRET, use --profile, or run 'obz init'"
    ))
}

fn validate_base64_secret(secret_str: &str) -> Result<()> {
    if secret_str.len() != 43 {
        return Err(anyhow::anyhow!(
            "Secret must be 43 base64 chars, got {} chars",
            secret_str.len()
        ));
    }

    use data_encoding::BASE64URL_NOPAD;
    let secret_bytes = BASE64URL_NOPAD
        .decode(secret_str.as_bytes())
        .context("Invalid secret base64 encoding")?;

    if secret_bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Secret must decode to 32 bytes, got {} bytes",
            secret_bytes.len()
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
    if let Some(scheme) = scheme_override {
        validate_ztier_scheme(scheme)?;
        return Ok(scheme);
    }

    if let Some(cfg) = config {
        let scheme = Scheme::from_str(&cfg.scheme).map_err(|e| anyhow::anyhow!("{}", e))?;
        validate_ztier_scheme(scheme)?;
        return Ok(scheme);
    }

    Err(anyhow::anyhow!(
        "scheme not specified: run 'obz init' or use a scheme flag or --format"
    ))
}

fn get_encoding(encoding_override: Option<Encoding>, config: Option<&Config>) -> Result<Encoding> {
    if let Some(encoding) = encoding_override {
        return Ok(encoding);
    }

    if let Some(cfg) = config {
        return Encoding::from_str(&cfg.encoding).map_err(|e| anyhow::anyhow!("{}", e));
    }

    Err(anyhow::anyhow!(
        "encoding not specified: run 'obz init' or use an encoding flag or --format"
    ))
}

fn validate_ztier_scheme(scheme: Scheme) -> Result<()> {
    match scheme {
        #[cfg(feature = "zrbcx")]
        Scheme::Zrbcx => Ok(()),
        #[cfg(feature = "zmock")]
        Scheme::Zmock1 => Ok(()),
        #[cfg(feature = "legacy")]
        Scheme::Legacy => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Invalid z-tier scheme: {}.  Use zrbcx, zmock1, or legacy",
            scheme.as_str()
        )),
    }
}
