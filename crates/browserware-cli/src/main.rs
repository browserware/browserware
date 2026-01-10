//! brw - Smart browser routing CLI

use clap::{Parser, Subcommand};

use browserware_detect::{Browser, BrowserFamily, detect_browsers, detect_default_browser};

#[derive(Parser)]
#[command(name = "brw")]
#[command(author, version, about = "Smart browser routing CLI", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format
    #[arg(short, long, global = true, default_value = "table")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Table,
    Json,
    Plain,
}

#[derive(Subcommand)]
enum Commands {
    /// List detected browsers
    Browsers {
        /// Filter by browser family (chromium, firefox, webkit)
        #[arg(short = 'F', long)]
        family: Option<String>,
    },
    /// List profiles for a browser
    Profiles {
        /// Browser ID or name
        browser: String,
    },
    /// Open URL(s) with routing rules
    Open {
        /// URLs to open
        urls: Vec<String>,
        /// Override browser selection
        #[arg(short, long)]
        browser: Option<String>,
        /// Override profile selection
        #[arg(short, long)]
        profile: Option<String>,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Register as default browser
    Register,
    /// Unregister as default browser
    Unregister,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration file
    Edit,
    /// Validate configuration
    Check,
}

fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(if cli.verbose {
                tracing::Level::DEBUG.into()
            } else {
                tracing::Level::WARN.into()
            }),
        )
        .init();

    match cli.command {
        Commands::Browsers { family } => {
            cmd_browsers(cli.format, family.as_deref());
        }
        Commands::Profiles { browser } => {
            println!("Profile listing for '{browser}' not yet implemented (Milestone 2)");
        }
        Commands::Open {
            urls,
            browser,
            profile,
        } => {
            println!("Opening URLs: {urls:?}");
            if let Some(b) = browser {
                println!("  Browser: {b}");
            }
            if let Some(p) = profile {
                println!("  Profile: {p}");
            }
            println!("Full routing not yet implemented (Milestone 4)");
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => println!("Config show not yet implemented"),
            ConfigAction::Edit => println!("Config edit not yet implemented"),
            ConfigAction::Check => println!("Config check not yet implemented"),
        },
        Commands::Register => {
            println!("Register not yet implemented (Milestone 5)");
        }
        Commands::Unregister => {
            println!("Unregister not yet implemented (Milestone 5)");
        }
    }
}

/// Execute the browsers command
fn cmd_browsers(format: OutputFormat, family_filter: Option<&str>) {
    // Parse family filter if provided
    let family = family_filter.map(|filter| {
        let Some(f) = parse_browser_family(filter) else {
            // Error message already printed by parse_browser_family
            std::process::exit(1);
        };
        f
    });

    // Get the default browser for marking
    let default_browser = detect_default_browser();
    let default_id = default_browser.as_ref().map(|b| b.id.0.as_str());

    // Detect browsers, optionally filtered by family
    let browsers: Vec<Browser> = family.map_or_else(detect_browsers, |f| {
        browserware_detect::detect_browsers_by_family(f)
    });

    // Output based on format
    match format {
        OutputFormat::Table => print_browsers_table(&browsers, default_id),
        OutputFormat::Json => print_browsers_json(&browsers, default_id),
        OutputFormat::Plain => print_browsers_plain(&browsers, default_id),
    }
}

/// Parse a browser family from string
fn parse_browser_family(s: &str) -> Option<BrowserFamily> {
    let family = match s.to_lowercase().as_str() {
        "chromium" | "chrome" => BrowserFamily::Chromium,
        "firefox" | "gecko" => BrowserFamily::Firefox,
        "webkit" | "safari" => BrowserFamily::WebKit,
        "other" => BrowserFamily::Other,
        _ => {
            eprintln!("Unknown browser family: {s}");
            eprintln!("Valid families: chromium, firefox, webkit, other");
            return None;
        }
    };
    Some(family)
}

/// Print browsers in table format
fn print_browsers_table(browsers: &[Browser], default_id: Option<&str>) {
    if browsers.is_empty() {
        println!("No browsers detected.");
        return;
    }

    // Calculate column widths (add 2 to ID for "* " prefix on default)
    let id_width = browsers
        .iter()
        .map(|b| b.id.0.len())
        .max()
        .unwrap_or(2)
        .max(2)
        + 2; // Space for "* " prefix
    let name_width = browsers
        .iter()
        .map(|b| b.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let family_width = 8; // "chromium" is longest
    let version_width = browsers
        .iter()
        .map(|b| b.version.as_ref().map_or(1, String::len))
        .max()
        .unwrap_or(7)
        .max(7);

    // Print header
    println!(
        "{:id_width$}  {:name_width$}  {:family_width$}  {:version_width$}",
        "ID",
        "NAME",
        "FAMILY",
        "VERSION",
        id_width = id_width,
        name_width = name_width,
        family_width = family_width,
        version_width = version_width,
    );
    println!(
        "{:-<id_width$}  {:-<name_width$}  {:-<family_width$}  {:-<version_width$}",
        "",
        "",
        "",
        "",
        id_width = id_width,
        name_width = name_width,
        family_width = family_width,
        version_width = version_width,
    );

    // Print browsers
    for browser in browsers {
        let is_default = default_id == Some(browser.id.0.as_str());
        let id_display = if is_default {
            format!("* {}", browser.id)
        } else {
            format!("  {}", browser.id)
        };
        let version = browser.version.as_deref().unwrap_or("-");
        let family = browser.family().to_string();

        println!(
            "{:id_width$}  {:name_width$}  {:family_width$}  {:version_width$}",
            id_display,
            browser.name,
            family,
            version,
            id_width = id_width,
            name_width = name_width,
            family_width = family_width,
            version_width = version_width,
        );
    }

    println!();
    println!("{} browser(s) detected", browsers.len());
}

/// Print browsers in JSON format
fn print_browsers_json(browsers: &[Browser], default_id: Option<&str>) {
    #[derive(serde::Serialize)]
    struct BrowserOutput<'a> {
        browsers: &'a [Browser],
        default: Option<&'a str>,
        count: usize,
    }

    let output = BrowserOutput {
        browsers,
        default: default_id,
        count: browsers.len(),
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Error serializing to JSON: {e}"),
    }
}

/// Print browsers in plain format (one per line)
fn print_browsers_plain(browsers: &[Browser], default_id: Option<&str>) {
    for browser in browsers {
        let is_default = default_id == Some(browser.id.0.as_str());
        let default_marker = if is_default { " (default)" } else { "" };
        println!("{}{default_marker}", browser.id);
    }
}
