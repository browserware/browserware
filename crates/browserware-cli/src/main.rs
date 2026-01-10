//! brw - Smart browser routing CLI

use clap::{Parser, Subcommand};

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
    Browsers,
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
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(if cli.verbose {
                tracing::Level::DEBUG.into()
            } else {
                tracing::Level::INFO.into()
            }),
        )
        .init();

    match cli.command {
        Commands::Browsers => {
            println!("Browser detection not yet implemented (Milestone 1)");
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
