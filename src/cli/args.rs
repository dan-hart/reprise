use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use serde::{Deserialize, Serialize};

/// A fast, feature-rich CLI for interacting with Bitrise CI/CD
#[derive(Parser)]
#[command(name = "reprise")]
#[command(version, propagate_version = true)]
#[command(about = "A fast, feature-rich CLI for Bitrise")]
#[command(long_about = "A fast, feature-rich CLI for Bitrise.\n\n\
Written in Rust, reprise makes it easy to interact with Bitrise CI/CD \
from your terminal.\n\n\
Features:\n  \
- List and filter apps, builds, and pipelines\n  \
- Stream live build logs in real-time\n  \
- Trigger builds and pipelines with custom parameters\n  \
- Download build artifacts\n  \
- Desktop notifications when builds complete\n  \
- Parse Bitrise URLs directly for quick access\n  \
- Local caching for faster repeated queries")]
#[command(after_help = "\
Quick Start:
  1. Set your token:  export BITRISE_TOKEN=your_token
  2. List your apps:  reprise apps
  3. Set default app: reprise app set my-app
  4. View builds:     reprise builds

Environment Variables:
  BITRISE_TOKEN    API token (can also use --token flag)
  NO_COLOR         Disable colored output when set

Aliases:
  Many commands have short aliases: builds (b), log (l, logs),
  app (a), pipelines (pl), pipeline (p), artifacts (art)

Documentation: https://github.com/dan-hart/reprise")]
pub struct Cli {
    /// Bitrise API token (overrides config file and BITRISE_TOKEN env var)
    #[arg(long, global = true, env = "BITRISE_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Output format: 'pretty' for human-readable, 'json' for scripting
    #[arg(short, long, value_enum, default_value = "pretty", global = true)]
    pub output: OutputFormat,

    /// Quiet mode - suppress non-essential output (progress indicators, hints)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Verbose mode - show debug information including API requests
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Bypass the local cache and fetch fresh data from Bitrise API
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
  reprise apps --filter \"My App\"  Filter by partial name match
  reprise apps --limit 10         Show only first 10 apps
  reprise apps -o json            Output as JSON for scripting
  reprise apps -o json | jq '.[0].slug'  Get first app's slug

Caching:
  App list is cached locally. Use --no-cache to fetch fresh data,
  or 'reprise cache clear' to clear all cached data.")]
    Apps(AppsArgs),

    /// Show or set the default app
    #[command(alias = "a", after_help = "\
Examples:
  reprise app                     Show current default app
  reprise app show                Same as above
  reprise a                       Short alias
  reprise app set abc123def456    Set default app by slug
  reprise app set \"My App\"        Set default app by name (exact match)
  reprise app set ios             Set first app matching 'ios'

The default app is used by builds, trigger, log, and other commands
when the --app flag is not specified. The slug is the unique identifier
found in your Bitrise app URL: app.bitrise.io/app/<slug>")]
    App(AppArgs),

    /// List builds for the default or specified app
    #[command(alias = "b", after_help = "\
Examples:
  reprise builds                  List recent builds
  reprise b                       Short alias
  reprise builds --status failed  Show only failed builds
  reprise builds -s running       Show running builds
  reprise builds --branch main    Filter by branch
  reprise builds --workflow deploy Filter by workflow
  reprise builds --me             Show only my builds
  reprise builds --triggered-by alice  Show builds triggered by 'alice'
  reprise builds --limit 50       Show more builds
  reprise builds --app other-app  Use different app
  reprise builds -o json          Output as JSON

Filtering:
  Use --me to show only builds you triggered (requires API auth).
  Use --triggered-by for partial username match (case-insensitive).
  Combine multiple filters: --status failed --branch main --me

Status Icons (in pretty output):
  [running]  Build is currently in progress
  [success]  Build completed successfully
  [failed]   Build failed
  [aborted]  Build was manually aborted")]
    Builds(BuildsArgs),

    /// Show details of a specific build
    #[command(after_help = "\
Examples:
  reprise build abc123            Show build details
  reprise build abc123 -o json    Output as JSON
  reprise build abc123 --app xyz  Specify app explicitly
  reprise build abc123 --follow   Stream live log output
  reprise build abc123 -f --notify  Follow with desktop notification
  reprise build abc123 --logs     Dump the full build log
  reprise build abc123 --artifacts  List build artifacts

Following Builds:
  Use --follow (-f) to stream live log output for running builds.
  Add --notify (-n) to receive a desktop notification when complete.
  Adjust --interval to change polling frequency (default: 3 seconds).

Finding Build Slugs:
  The build slug is the unique ID shown in the Bitrise URL after /build/
  or in the 'builds' command output. Example: app.bitrise.io/build/<slug>")]
    Build(BuildArgs),

    /// View build logs
    #[command(aliases = ["logs", "l"], after_help = "\
Examples:
  reprise log abc123              View full build log
  reprise logs abc123             Alias for 'log'
  reprise l abc123                Short alias
  reprise log abc123 --tail 100   Show last 100 lines
  reprise log abc123 --tail 50 --follow  Follow with context
  reprise log abc123 --save build.log  Save log to file
  reprise log abc123 --follow     Stream live log output
  reprise log abc123 -f --notify  Follow with desktop notification
  reprise log abc123 --app other  View log from different app

Output:
  Logs include ANSI color codes from Bitrise. Colors display in
  terminals that support them. Use --save to capture raw output.
  Pipe to 'less -R' for scrollable colored output.")]
    Log(LogArgs),

    /// Manage configuration
    #[command(after_help = "\
Examples:
  reprise config init             Interactive setup wizard
  reprise config show             Display current configuration
  reprise config path             Show config file location
  reprise config set api.token YOUR_TOKEN  Set API token
  reprise config set defaults.app_slug abc123  Set default app

Configuration Keys:
  api.token           Your Bitrise API token
  defaults.app_slug   Default app slug for commands
  defaults.app_name   Default app display name
  output.format       Default output format (pretty/json)

The config file is stored in your system's config directory.
Use 'reprise config path' to see the exact location.")]
    Config(ConfigArgs),

    /// Manage local cache
    #[command(after_help = "\
Examples:
  reprise cache status            Show cache status and age
  reprise cache clear             Clear all cached data

What's Cached:
  The app list is cached to speed up repeated queries.
  Cache is stored in your system's cache directory.
  Use --no-cache on any command to bypass the cache.")]
    Cache(CacheArgs),

    /// Trigger a new build
    #[command(after_help = "\
Examples:
  reprise trigger -w primary              Trigger primary workflow
  reprise trigger -w deploy -b main       Build main branch with deploy workflow
  reprise trigger -w ci --env MY_VAR=foo  Pass environment variable
  reprise trigger -w ci --env A=1 --env B=2  Multiple env vars
  reprise trigger -w primary --wait       Wait for build to complete
  reprise trigger -w primary --wait -n    Wait with desktop notification
  reprise trigger -w primary --app xyz    Trigger for specific app
  reprise trigger -w deploy -m \"Deploy v1.0\"  Add commit message

Options:
  If --branch is not specified, the repository's default branch is used.
  Use --wait to block until the build completes. Combine with --notify
  for a desktop notification when done. Adjust --interval for polling.

Environment Variables:
  Use --env KEY=VALUE to pass environment variables to the build.
  Can be specified multiple times for multiple variables.")]
    Trigger(TriggerArgs),

    /// List or download build artifacts
    #[command(alias = "art", after_help = "\
Examples:
  reprise artifacts abc123                List artifacts for build
  reprise art abc123                      Short alias
  reprise artifacts abc123 --download     Download all to current directory
  reprise artifacts abc123 -d ./output    Download to specific directory
  reprise artifacts abc123 -d ~/Downloads Download to home directory
  reprise artifacts abc123 -o json        List as JSON

Downloading:
  Without -d/--download, artifacts are listed but not downloaded.
  With -d, all artifacts are downloaded to the specified directory
  (or current directory if no path given). Existing files are overwritten.")]
    Artifacts(ArtifactsArgs),

    /// Abort a running build
    #[command(after_help = "\
Examples:
  reprise abort abc123                    Abort build (with confirmation)
  reprise abort abc123 -y                 Skip confirmation prompt
  reprise abort abc123 -r \"Wrong branch\"  Abort with reason
  reprise abort abc123 --app xyz          Specify app explicitly

Confirmation:
  By default, you'll be prompted to confirm before aborting.
  Use -y/--yes to skip the confirmation (useful for scripts).
  The abort reason is optional but helps with debugging.")]
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
  reprise pl                         Short alias
  reprise pipelines --status running Show running pipelines
  reprise pipelines --status failed  Show failed pipelines
  reprise pipelines --branch main    Filter by branch
  reprise pipelines --me             Show only my pipelines
  reprise pipelines --triggered-by bob  Show pipelines triggered by 'bob'
  reprise pipelines --limit 50       Show more pipelines
  reprise pipelines -o json          Output as JSON

Filtering:
  Use --me to show only pipelines you triggered (requires API auth).
  Use --triggered-by for partial username match (case-insensitive).
  Combine multiple filters: --status running --branch main

Pipelines vs Builds:
  Pipelines orchestrate multiple workflows in stages. Use 'builds'
  to see individual workflow executions within a pipeline.")]
    Pipelines(PipelinesArgs),

    /// Show or manage a specific pipeline
    #[command(alias = "p", after_help = "\
Examples:
  reprise pipeline abc123                          Show pipeline details
  reprise p abc123                                 Short alias
  reprise pipeline show abc123                     Explicit show command
  reprise pipeline trigger my-pipeline             Trigger a pipeline
  reprise pipeline trigger deploy --branch main    Trigger with branch
  reprise pipeline trigger ci --env VERSION=1.0   Trigger with env var
  reprise pipeline abort abc123                    Abort running pipeline
  reprise pipeline abort abc123 -r \"Wrong config\"  Abort with reason
  reprise pipeline rebuild abc123                  Rebuild a pipeline
  reprise pipeline rebuild abc123 --partial        Rebuild only failed stages
  reprise pipeline watch abc123                    Watch pipeline progress
  reprise pipeline watch abc123 --notify           Watch with notification

Subcommands:
  show      Display pipeline details and stage status
  trigger   Start a new pipeline run
  abort     Cancel a running pipeline
  rebuild   Re-run a pipeline (full or partial)
  watch     Monitor pipeline progress until completion

Use 'reprise pipeline <subcommand> --help' for subcommand details.")]
    Pipeline(PipelineArgs),
}

/// Arguments for the apps command
#[derive(Args)]
pub struct AppsArgs {
    /// Filter apps by name (case-insensitive partial match)
    #[arg(short, long, value_name = "TEXT")]
    pub filter: Option<String>,

    /// Maximum number of apps to return
    #[arg(short, long, default_value = "50", value_name = "N")]
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
    /// Set the default app for future commands
    #[command(after_help = "\
Examples:
  reprise app set abc123def456       Set by app slug
  reprise app set \"My iOS App\"       Set by exact app name
  reprise app set ios                Find first app matching 'ios'

Finding Your App Slug:
  The slug is in the Bitrise URL: app.bitrise.io/app/<slug>
  Or use 'reprise apps' to list all apps with their slugs.

The default app is saved to your config file and used by
commands like 'builds', 'trigger', and 'log' when no
--app flag is provided.")]
    Set {
        /// App slug or name to set as default
        app: String,
    },

    /// Show the currently configured default app
    #[command(after_help = "\
Example:
  reprise app show                   Display default app info
  reprise app                        Same as 'app show'

Shows the app slug and name. If no default is set, you'll be
prompted to set one. Use 'reprise app set' to change it.")]
    Show,
}

/// Arguments for the builds command
#[derive(Args)]
pub struct BuildsArgs {
    /// App slug (overrides default app)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Filter by build status (running, success, failed, aborted)
    #[arg(short, long, value_enum)]
    pub status: Option<BuildStatusFilter>,

    /// Filter by branch name (exact match)
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Filter by workflow name (exact match)
    #[arg(short, long)]
    pub workflow: Option<String>,

    /// Filter by user who triggered (partial match, case-insensitive)
    #[arg(long, value_name = "USER")]
    pub triggered_by: Option<String>,

    /// Show only builds triggered by the current authenticated user
    #[arg(long, conflicts_with = "triggered_by")]
    pub me: bool,

    /// Maximum number of builds to return
    #[arg(short, long, default_value = "25", value_name = "N")]
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
    /// Build slug (unique ID from Bitrise URL or 'builds' output)
    #[arg(value_name = "SLUG")]
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Stream live log output for running builds
    #[arg(short, long, conflicts_with_all = ["logs", "artifacts"])]
    pub follow: bool,

    /// Dump the full build log to stdout
    #[arg(long, conflicts_with_all = ["follow", "artifacts"])]
    pub logs: bool,

    /// List build artifacts (files produced by the build)
    #[arg(long, conflicts_with_all = ["follow", "logs"])]
    pub artifacts: bool,

    /// Polling interval in seconds when following (1-60 recommended)
    #[arg(long, default_value = "3", value_name = "SECS")]
    pub interval: u64,

    /// Send desktop notification when build completes (with --follow)
    #[arg(short, long)]
    pub notify: bool,
}

/// Arguments for the log command
#[derive(Args)]
pub struct LogArgs {
    /// Build slug (unique ID from Bitrise URL or 'builds' output)
    #[arg(value_name = "SLUG")]
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Show only last N lines of the log
    #[arg(short, long, value_name = "LINES")]
    pub tail: Option<usize>,

    /// Save log to file (creates or overwrites)
    #[arg(long, value_hint = ValueHint::FilePath, value_name = "PATH")]
    pub save: Option<String>,

    /// Follow log output (stream live for running builds)
    #[arg(short, long)]
    pub follow: bool,

    /// Polling interval in seconds when following (1-60 recommended)
    #[arg(long, default_value = "3", value_name = "SECS")]
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
    /// Show current configuration values
    #[command(after_help = "\
Example:
  reprise config show                Display all config values

Shows your current configuration including API token (masked),
default app, and output preferences.")]
    Show,

    /// Set a configuration value
    #[command(after_help = "\
Examples:
  reprise config set api.token YOUR_TOKEN
  reprise config set defaults.app_slug abc123def456
  reprise config set defaults.app_name \"My iOS App\"
  reprise config set output.format json

Available Keys:
  api.token           Your Bitrise personal access token
  defaults.app_slug   Default app slug for commands
  defaults.app_name   Display name for default app
  output.format       Default output format (pretty or json)

Get your API token from: https://app.bitrise.io/me/profile#/security")]
    Set {
        /// Configuration key (api.token, defaults.app_slug, etc.)
        key: String,
        /// Value to set
        value: String,
    },

    /// Show configuration file path
    #[command(after_help = "\
Example:
  reprise config path                Show where config is stored

The config file is in your system's standard config directory:
  macOS:   ~/Library/Application Support/reprise/config.toml
  Linux:   ~/.config/reprise/config.toml
  Windows: %APPDATA%\\reprise\\config.toml")]
    Path,

    /// Initialize configuration interactively
    #[command(after_help = "\
Example:
  reprise config init                Start interactive setup

Walks you through setting up:
  1. Your Bitrise API token
  2. Default app selection
  3. Output format preference

This is the recommended way to get started with reprise.")]
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
    #[command(after_help = "\
Example:
  reprise cache status               Show cache info

Displays:
  - Cache location on disk
  - Last update timestamp
  - Number of cached items (apps)
  - Cache size

The cache stores your app list to speed up repeated commands.")]
    Status,

    /// Clear all cached data
    #[command(after_help = "\
Example:
  reprise cache clear                Delete all cached data

Removes all locally cached data. The cache will be rebuilt
automatically on the next command that needs it.

Use this if you're seeing stale data or after adding/removing
apps in Bitrise.")]
    Clear,
}

/// Arguments for the trigger command
#[derive(Args)]
pub struct TriggerArgs {
    /// Workflow name to run (as defined in bitrise.yml)
    #[arg(short, long)]
    pub workflow: String,

    /// Branch to build (defaults to repo's default branch)
    #[arg(short, long)]
    pub branch: Option<String>,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Commit message for the build (shown in Bitrise UI)
    #[arg(short, long)]
    pub message: Option<String>,

    /// Environment variables in KEY=VALUE format (repeatable)
    #[arg(long, value_name = "KEY=VALUE", value_parser = parse_env_var)]
    pub env: Vec<(String, String)>,

    /// Wait for build to complete before returning
    #[arg(long)]
    pub wait: bool,

    /// Send desktop notification when build completes (with --wait)
    #[arg(short, long)]
    pub notify: bool,

    /// Polling interval in seconds when waiting (1-60 recommended)
    #[arg(long, default_value = "10", value_name = "SECS")]
    pub interval: u64,
}

/// Arguments for the artifacts command
#[derive(Args)]
pub struct ArtifactsArgs {
    /// Build slug (unique ID from Bitrise URL or 'builds' output)
    #[arg(value_name = "SLUG")]
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Download all artifacts to directory (current dir if no path given)
    #[arg(short, long, value_hint = ValueHint::DirPath, value_name = "DIR")]
    pub download: Option<Option<String>>,
}

/// Arguments for the abort command
#[derive(Args)]
pub struct AbortArgs {
    /// Build slug (unique ID from Bitrise URL or 'builds' output)
    #[arg(value_name = "SLUG")]
    pub slug: String,

    /// App slug (overrides default)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Reason for aborting (shown in Bitrise UI)
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
    #[arg(value_hint = ValueHint::Url)]
    pub url: String,

    /// Open URL in default browser
    #[arg(short, long)]
    pub browser: bool,

    /// Watch build/pipeline progress until completion
    #[arg(short, long)]
    pub watch: bool,

    /// Polling interval in seconds when watching/following (default: 5)
    #[arg(long, default_value = "5", value_name = "SECS")]
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
    /// App slug (overrides default app)
    #[arg(short, long)]
    pub app: Option<String>,

    /// Filter by pipeline status (running, success, failed, aborted)
    #[arg(short, long, value_enum)]
    pub status: Option<BuildStatusFilter>,

    /// Filter by branch name (exact match)
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Filter by user who triggered (partial match, case-insensitive)
    #[arg(long, value_name = "USER")]
    pub triggered_by: Option<String>,

    /// Show only pipelines triggered by the current authenticated user
    #[arg(long, conflicts_with = "triggered_by")]
    pub me: bool,

    /// Maximum number of pipelines to return
    #[arg(short, long, default_value = "25", value_name = "N")]
    pub limit: u32,
}

/// Arguments for the pipeline command
#[derive(Args)]
pub struct PipelineArgs {
    /// Pipeline ID (from 'pipelines' command or Bitrise URL)
    #[arg(value_name = "ID")]
    pub id: Option<String>,

    #[command(subcommand)]
    pub command: Option<PipelineCommands>,
}

/// Pipeline subcommands
#[derive(Subcommand)]
pub enum PipelineCommands {
    /// Show pipeline details and stage status
    #[command(after_help = "\
Examples:
  reprise pipeline show abc123       Show pipeline details
  reprise pipeline show abc123 -o json  Output as JSON
  reprise pipeline show abc123 --app xyz  Specify app

Displays pipeline information including:
  - Pipeline name and ID
  - Current status and duration
  - Branch and commit info
  - Stage breakdown with individual workflow status")]
    Show {
        /// Pipeline ID (from 'pipelines' command or Bitrise URL)
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Trigger a new pipeline run
    #[command(after_help = "\
Examples:
  reprise pipeline trigger my-pipeline
  reprise pipeline trigger deploy --branch main
  reprise pipeline trigger ci --branch feature/xyz
  reprise pipeline trigger release --env VERSION=1.0.0
  reprise pipeline trigger ci --env A=1 --env B=2
  reprise pipeline trigger deploy --wait --notify

Options:
  If --branch is not specified, the repository's default branch is used.
  Use --wait to block until the pipeline completes.
  Add --notify for a desktop notification when done.

Environment Variables:
  Use --env KEY=VALUE to pass variables. Can be repeated.")]
    Trigger {
        /// Pipeline name to trigger (as defined in bitrise.yml)
        name: String,

        /// Branch to build (defaults to repo's default branch)
        #[arg(short, long)]
        branch: Option<String>,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Environment variables in KEY=VALUE format (repeatable)
        #[arg(long, value_name = "KEY=VALUE", value_parser = parse_env_var)]
        env: Vec<(String, String)>,

        /// Wait for pipeline to complete before returning
        #[arg(long)]
        wait: bool,

        /// Send desktop notification when pipeline completes (with --wait)
        #[arg(short, long)]
        notify: bool,

        /// Polling interval in seconds when waiting (default: 10)
        #[arg(long, default_value = "10", value_name = "SECS")]
        interval: u64,
    },

    /// Abort a running pipeline
    #[command(after_help = "\
Examples:
  reprise pipeline abort abc123
  reprise pipeline abort abc123 -y          Skip confirmation
  reprise pipeline abort abc123 -r \"Wrong config\"

Confirmation:
  By default, you'll be prompted to confirm. Use -y to skip.
  The abort reason is optional but helps with debugging.")]
    Abort {
        /// Pipeline ID to abort
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Reason for aborting (shown in Bitrise UI)
        #[arg(short, long)]
        reason: Option<String>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Rebuild a pipeline (full or partial)
    #[command(after_help = "\
Examples:
  reprise pipeline rebuild abc123            Full rebuild
  reprise pipeline rebuild abc123 --partial  Rebuild failed stages only
  reprise pipeline rebuild abc123 --wait     Wait for completion
  reprise pipeline rebuild abc123 --partial --wait --notify

Rebuild Modes:
  Full rebuild (default): Re-runs all stages from the beginning.
  Partial rebuild (--partial): Only re-runs failed stages and
  their dependents, skipping already-successful stages.

Partial rebuilds are faster and preserve successful work.")]
    Rebuild {
        /// Pipeline ID to rebuild
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Only rebuild failed stages and their dependents
        #[arg(long)]
        partial: bool,

        /// Wait for pipeline to complete before returning
        #[arg(long)]
        wait: bool,

        /// Send desktop notification when pipeline completes (with --wait)
        #[arg(short, long)]
        notify: bool,

        /// Polling interval in seconds when waiting (default: 10)
        #[arg(long, default_value = "10", value_name = "SECS")]
        interval: u64,
    },

    /// Watch pipeline progress until completion
    #[command(after_help = "\
Examples:
  reprise pipeline watch abc123
  reprise pipeline watch abc123 --notify
  reprise pipeline watch abc123 --interval 10

Monitors the pipeline and displays live status updates.
Press Ctrl+C to stop watching (pipeline continues running).

Use --notify to receive a desktop notification when the
pipeline completes (success, failure, or abort).")]
    Watch {
        /// Pipeline ID to watch
        id: String,

        /// App slug (overrides default)
        #[arg(short, long)]
        app: Option<String>,

        /// Polling interval in seconds (default: 5)
        #[arg(long, default_value = "5", value_name = "SECS")]
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
