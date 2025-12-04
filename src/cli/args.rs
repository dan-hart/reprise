use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// A fast, feature-rich CLI for viewing build details from Bitrise
#[derive(Parser)]
#[command(name = "reprise")]
#[command(version, propagate_version = true)]
#[command(about = "A fast, feature-rich CLI for viewing build details from Bitrise")]
pub struct Cli {
    /// Output format for command results
    #[arg(short, long, value_enum, default_value = "pretty", global = true)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

/// Output format options
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Colored, human-readable output
    #[default]
    Pretty,
    /// JSON output for scripting
    Json,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// List all accessible Bitrise apps
    Apps(AppsArgs),

    /// Show or set the default app
    #[command(alias = "a")]
    App(AppArgs),

    /// List builds for the default or specified app
    #[command(alias = "b")]
    Builds(BuildsArgs),

    /// Show details of a specific build
    Build(BuildArgs),

    /// View build logs
    #[command(aliases = ["logs", "l"])]
    Log(LogArgs),

    /// Manage configuration
    Config(ConfigArgs),
}

/// Arguments for the apps command
#[derive(Args)]
pub struct AppsArgs {
    /// Filter apps by name
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Maximum number of apps to show
    #[arg(short, long, default_value = "50")]
    pub limit: u32,
}

/// Arguments for the app command
#[derive(Args)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: Option<AppCommands>,
}

/// App subcommands
#[derive(Subcommand)]
pub enum AppCommands {
    /// Set the default app
    Set {
        /// App slug or name
        app: String,
    },
    /// Show current default app
    Show,
}

/// Arguments for the builds command
#[derive(Args)]
pub struct BuildsArgs {
    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Filter by build status
    #[arg(short, long, value_enum)]
    pub status: Option<BuildStatusFilter>,

    /// Filter by branch name
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Filter by workflow name
    #[arg(short, long)]
    pub workflow: Option<String>,

    /// Maximum number of builds to show
    #[arg(short, long, default_value = "25")]
    pub limit: u32,
}

/// Build status filter options
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildStatusFilter {
    /// Build is currently running
    Running,
    /// Build completed successfully
    Success,
    /// Build failed
    Failed,
    /// Build was aborted
    Aborted,
}

impl BuildStatusFilter {
    /// Convert to Bitrise API status code
    pub fn to_api_code(self) -> i32 {
        match self {
            Self::Running => 0,
            Self::Success => 1,
            Self::Failed => 2,
            Self::Aborted => 3,
        }
    }
}

/// Arguments for the build command
#[derive(Args)]
pub struct BuildArgs {
    /// Build slug
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,
}

/// Arguments for the log command
#[derive(Args)]
pub struct LogArgs {
    /// Build slug
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Show only last N lines
    #[arg(short, long)]
    pub tail: Option<usize>,

    /// Save log to file
    #[arg(long)]
    pub save: Option<String>,
}

/// Arguments for the config command
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Config subcommands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., api.token)
        key: String,
        /// Value to set
        value: String,
    },
    /// Show configuration file path
    Path,
    /// Initialize configuration interactively
    Init,
}
