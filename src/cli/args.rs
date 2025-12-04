use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// A fast, feature-rich CLI for viewing build details from Bitrise
#[derive(Parser)]
#[command(name = "reprise")]
#[command(version, propagate_version = true)]
#[command(about = "A fast, feature-rich CLI for Bitrise")]
#[command(long_about = "A fast, feature-rich CLI for Bitrise.\n\n\
Written in Rust, reprise makes it easy to interact with Bitrise CI/CD \
from your terminal. List apps, view builds, stream logs, and more.")]
#[command(after_help = "\
Quick Start:
  1. Set your token:  export BITRISE_TOKEN=your_token
  2. List your apps:  reprise apps
  3. Set default app: reprise app set my-app
  4. View builds:     reprise builds

Environment Variables:
  BITRISE_TOKEN    API token (same as --token flag)

Documentation: https://github.com/dan-hart/reprise")]
pub struct Cli {
    /// Bitrise API token (overrides config file)
    #[arg(long, global = true, env = "BITRISE_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Output format for command results
    #[arg(short, long, value_enum, default_value = "pretty", global = true)]
    pub output: OutputFormat,

    /// Quiet mode - suppress non-essential output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Verbose mode - show debug information
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Bypass cache and fetch fresh data
    #[arg(long, global = true)]
    pub no_cache: bool,

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
    #[command(after_help = "\
Examples:
  reprise apps                    List all apps
  reprise apps --filter ios       Filter apps containing 'ios'
  reprise apps --limit 10         Show only 10 apps
  reprise apps -o json            Output as JSON")]
    Apps(AppsArgs),

    /// Show or set the default app
    #[command(alias = "a", after_help = "\
Examples:
  reprise app                     Show current default app
  reprise app show                Same as above
  reprise app set abc123          Set default app by slug
  reprise app set \"My App\"        Set default app by name")]
    App(AppArgs),

    /// List builds for the default or specified app
    #[command(alias = "b", after_help = "\
Examples:
  reprise builds                  List recent builds
  reprise builds --status failed  Show only failed builds
  reprise builds -s running       Show running builds
  reprise builds --branch main    Filter by branch
  reprise builds --workflow deploy Filter by workflow
  reprise builds --me             Show only my builds
  reprise builds --triggered-by alice  Show builds triggered by 'alice'
  reprise builds --limit 50       Show more builds
  reprise builds --app other-app  Use different app

Filtering:
  Use --me to show only builds you triggered (requires API auth).
  Use --triggered-by for builds by a specific user (partial match).
  Combine with --status, --branch, --workflow for precise filtering.")]
    Builds(BuildsArgs),

    /// Show details of a specific build
    #[command(after_help = "\
Examples:
  reprise build abc123            Show build details
  reprise build abc123 -o json    Output as JSON
  reprise build abc123 --app xyz  Specify app explicitly")]
    Build(BuildArgs),

    /// View build logs
    #[command(aliases = ["logs", "l"], after_help = "\
Examples:
  reprise log abc123              View full build log
  reprise log abc123 --tail 100   Show last 100 lines
  reprise log abc123 --save build.log  Save to file
  reprise log abc123 --follow     Stream live log output
  reprise log abc123 -f --notify  Follow with desktop notification
  reprise logs abc123             Alias for 'log'
  reprise l abc123                Short alias")]
    Log(LogArgs),

    /// Manage configuration
    #[command(after_help = "\
Examples:
  reprise config init             Interactive setup
  reprise config show             Display current config
  reprise config path             Show config file location
  reprise config set api.token YOUR_TOKEN  Set API token
  reprise config set defaults.app_slug abc123  Set default app")]
    Config(ConfigArgs),

    /// Manage local cache
    #[command(after_help = "\
Examples:
  reprise cache status            Show cache status
  reprise cache clear             Clear all cached data")]
    Cache(CacheArgs),

    /// Trigger a new build
    #[command(after_help = "\
Examples:
  reprise trigger -w primary              Trigger primary workflow
  reprise trigger -w deploy -b main       Build main branch with deploy workflow
  reprise trigger -w ci --env MY_VAR=foo  Pass environment variable
  reprise trigger -w primary --wait       Wait for build to complete
  reprise trigger -w primary --app xyz    Trigger for specific app")]
    Trigger(TriggerArgs),

    /// List or download build artifacts
    #[command(alias = "art", after_help = "\
Examples:
  reprise artifacts abc123                List artifacts for build
  reprise artifacts abc123 --download     Download all artifacts
  reprise artifacts abc123 -d ./output    Download to specific directory
  reprise art abc123                      Short alias")]
    Artifacts(ArtifactsArgs),

    /// Abort a running build
    #[command(after_help = "\
Examples:
  reprise abort abc123                    Abort build
  reprise abort abc123 -r \"Wrong branch\"  Abort with reason
  reprise abort abc123 --app xyz          Specify app explicitly")]
    Abort(AbortArgs),

    /// Parse and interact with a Bitrise URL
    #[command(after_help = "\
Examples:
  reprise url https://app.bitrise.io/build/abc123           Show build status
  reprise url https://app.bitrise.io/app/xyz789             Show app info
  reprise url https://app.bitrise.io/app/xyz/pipelines/123  Show pipeline status
  reprise url <url> --browser                                Open URL in browser
  reprise url <url> --watch                                  Watch build/pipeline progress

Build URL Actions:
  reprise url <build-url> --logs         Dump the full build log
  reprise url <build-url> --follow       Stream live log output (for running builds)
  reprise url <build-url> --artifacts    List build artifacts

App URL Actions:
  reprise url <app-url> --set-default    Set this app as your default

Tips:
  Copy a URL from Bitrise and paste it here to quickly view status,
  check logs, or download artifacts without setting up app context.
  Use --watch to monitor a running build until completion.")]
    Url(UrlArgs),

    /// List pipelines for the default or specified app
    #[command(alias = "pl", after_help = "\
Examples:
  reprise pipelines                  List recent pipelines
  reprise pipelines --status running Show running pipelines
  reprise pipelines --branch main    Filter by branch
  reprise pipelines --me             Show only my pipelines
  reprise pipelines --triggered-by bob  Show pipelines triggered by 'bob'
  reprise pipelines --limit 50       Show more pipelines
  reprise pl                         Short alias

Filtering:
  Use --me to show only pipelines you triggered (requires API auth).
  Use --triggered-by for pipelines by a specific user (partial match).
  Combine with --status and --branch for precise filtering.")]
    Pipelines(PipelinesArgs),

    /// Show or manage a specific pipeline
    #[command(alias = "p", after_help = "\
Examples:
  reprise pipeline abc123                          Show pipeline details
  reprise pipeline trigger my-pipeline             Trigger a pipeline
  reprise pipeline trigger deploy --branch main    Trigger with branch
  reprise pipeline abort abc123                    Abort running pipeline
  reprise pipeline rebuild abc123                  Rebuild a pipeline
  reprise pipeline rebuild abc123 --partial        Rebuild only failed workflows
  reprise pipeline watch abc123                    Watch pipeline progress
  reprise p abc123                                 Short alias")]
    Pipeline(PipelineArgs),
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

    /// Filter by user who triggered the build (partial match, case-insensitive)
    #[arg(long, value_name = "USER")]
    pub triggered_by: Option<String>,

    /// Show only builds triggered by the current authenticated user
    #[arg(long, conflicts_with = "triggered_by")]
    pub me: bool,

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

    /// Follow log output (stream live for running builds)
    #[arg(short, long)]
    pub follow: bool,

    /// Polling interval in seconds when following (default: 3)
    #[arg(long, default_value = "3")]
    pub interval: u64,

    /// Send desktop notification when build completes (with --follow)
    #[arg(short, long)]
    pub notify: bool,
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

/// Arguments for the cache command
#[derive(Args)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

/// Cache subcommands
#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache status and age
    Status,
    /// Clear all cached data
    Clear,
}

/// Arguments for the trigger command
#[derive(Args)]
pub struct TriggerArgs {
    /// Workflow to run
    #[arg(short, long)]
    pub workflow: String,

    /// Branch to build (defaults to repo's default branch)
    #[arg(short, long)]
    pub branch: Option<String>,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Commit message for the build
    #[arg(short, long)]
    pub message: Option<String>,

    /// Environment variables (KEY=VALUE format, can be specified multiple times)
    #[arg(long, value_parser = parse_env_var)]
    pub env: Vec<(String, String)>,

    /// Wait for build to complete
    #[arg(long)]
    pub wait: bool,

    /// Send desktop notification when build completes (with --wait)
    #[arg(short, long)]
    pub notify: bool,

    /// Polling interval in seconds when waiting (default: 10)
    #[arg(long, default_value = "10")]
    pub interval: u64,
}

/// Arguments for the artifacts command
#[derive(Args)]
pub struct ArtifactsArgs {
    /// Build slug
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Download artifacts to directory (default: current directory)
    #[arg(short, long)]
    pub download: Option<Option<String>>,
}

/// Arguments for the abort command
#[derive(Args)]
pub struct AbortArgs {
    /// Build slug
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Reason for aborting
    #[arg(short, long)]
    pub reason: Option<String>,

    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
}

/// Arguments for the url command
#[derive(Args)]
pub struct UrlArgs {
    /// Bitrise URL to parse (app, build, or pipeline URL)
    pub url: String,

    /// Open URL in default browser
    #[arg(short, long)]
    pub browser: bool,

    /// Watch build/pipeline progress until completion
    #[arg(short, long)]
    pub watch: bool,

    /// Polling interval in seconds when watching/following (default: 5)
    #[arg(long, default_value = "5")]
    pub interval: u64,

    /// Send desktop notification when build/pipeline completes
    #[arg(short, long)]
    pub notify: bool,

    /// Set this app as the default (only for app URLs)
    #[arg(long)]
    pub set_default: bool,

    /// Dump the full build log (only for build URLs)
    #[arg(long, conflicts_with_all = ["watch", "follow"])]
    pub logs: bool,

    /// Stream live log output for running builds (only for build URLs)
    #[arg(short, long, conflicts_with_all = ["watch", "logs"])]
    pub follow: bool,

    /// List build artifacts (only for build URLs)
    #[arg(long)]
    pub artifacts: bool,
}

/// Arguments for the pipelines command
#[derive(Args)]
pub struct PipelinesArgs {
    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Filter by pipeline status
    #[arg(short, long, value_enum)]
    pub status: Option<BuildStatusFilter>,

    /// Filter by branch name
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Filter by user who triggered the pipeline (partial match, case-insensitive)
    #[arg(long, value_name = "USER")]
    pub triggered_by: Option<String>,

    /// Show only pipelines triggered by the current authenticated user
    #[arg(long, conflicts_with = "triggered_by")]
    pub me: bool,

    /// Maximum number of pipelines to show
    #[arg(short, long, default_value = "25")]
    pub limit: u32,
}

/// Arguments for the pipeline command
#[derive(Args)]
pub struct PipelineArgs {
    /// Pipeline ID (for show command)
    pub id: Option<String>,

    #[command(subcommand)]
    pub command: Option<PipelineCommands>,
}

/// Pipeline subcommands
#[derive(Subcommand)]
pub enum PipelineCommands {
    /// Show pipeline details
    Show {
        /// Pipeline ID
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Trigger a new pipeline
    Trigger {
        /// Pipeline name to trigger
        name: String,

        /// Branch to build (defaults to repo's default branch)
        #[arg(short, long)]
        branch: Option<String>,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Environment variables (KEY=VALUE format, can be specified multiple times)
        #[arg(long, value_parser = parse_env_var)]
        env: Vec<(String, String)>,

        /// Wait for pipeline to complete
        #[arg(long)]
        wait: bool,

        /// Send desktop notification when pipeline completes (with --wait)
        #[arg(short, long)]
        notify: bool,

        /// Polling interval in seconds when waiting (default: 10)
        #[arg(long, default_value = "10")]
        interval: u64,
    },

    /// Abort a running pipeline
    Abort {
        /// Pipeline ID
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Reason for aborting
        #[arg(short, long)]
        reason: Option<String>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Rebuild a pipeline
    Rebuild {
        /// Pipeline ID
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Only rebuild failed and subsequent workflows
        #[arg(long)]
        partial: bool,

        /// Wait for pipeline to complete
        #[arg(long)]
        wait: bool,

        /// Send desktop notification when pipeline completes (with --wait)
        #[arg(short, long)]
        notify: bool,

        /// Polling interval in seconds when waiting (default: 10)
        #[arg(long, default_value = "10")]
        interval: u64,
    },

    /// Watch pipeline progress
    Watch {
        /// Pipeline ID
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Polling interval in seconds (default: 5)
        #[arg(long, default_value = "5")]
        interval: u64,

        /// Send desktop notification when pipeline completes
        #[arg(short, long)]
        notify: bool,
    },
}

/// Parse environment variable in KEY=VALUE format
fn parse_env_var(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid format: '{}'. Expected KEY=VALUE", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
