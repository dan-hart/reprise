# Command reference

Generated from Clap. Regenerate with `cargo run --example generate_reference`. Do not edit by hand.

## reprise

```text
A fast, feature-rich CLI for Bitrise.

Written in Rust, reprise makes it easy to interact with Bitrise CI/CD from your terminal.

Features:
  - List and filter apps, builds, and pipelines
  - Stream live build logs in real-time
  - Trigger builds and pipelines with custom parameters
  - Download build artifacts
  - Desktop notifications when builds complete
  - Parse Bitrise URLs directly for quick access

Usage: reprise [OPTIONS] <COMMAND>

Commands:
  apps         List all accessible Bitrise apps
  app          Show or set the default app
  builds       List builds for the default or specified app
  build        Show details of a specific build
  log          View build logs
  config       Manage configuration
  yml          Get or update an app's bitrise.yml
  trigger      Trigger a new build
  artifacts    List or download build artifacts
  abort        Abort a running build
  url          Parse a Bitrise URL or generate URLs from slugs
  pipelines    List pipelines for the default or specified app
  pipeline     Show or manage a specific pipeline
  doctor       Run environment and configuration diagnostics
  diagnose     Diagnose a failed or suspicious build
  compare      Compare two builds side by side
  view         Manage saved views for builds and pipelines
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Quick Start:
  1. Set your token:  export BITRISE_TOKEN=your_token
  2. List your apps:  reprise apps
  3. Set default app: reprise app set my-app
  4. Check setup:     reprise doctor
  5. View builds:     reprise builds

Environment Variables:
  BITRISE_TOKEN    API token (can also use --token flag)
  NO_COLOR         Disable colored output when set

Aliases:
  Many commands have short aliases: builds (b), log (l, logs),
  app (a), pipelines (pl), pipeline (p), artifacts (art)

Cookbook: https://github.com/dan-hart/reprise/blob/main/docs/cookbook.md
Documentation: https://github.com/dan-hart/reprise

```

## reprise apps

```text
List all accessible Bitrise apps

Usage: reprise apps [OPTIONS]

Options:
  -f, --filter <TEXT>
          Filter apps by name (case-insensitive partial match)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -l, --limit <N>
          Maximum number of apps to return
          
          [default: 50]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise apps                    List all apps
  reprise apps --filter ios       Filter apps containing 'ios'
  reprise apps --filter "My App"  Filter by partial name match
  reprise apps --limit 10         Show only first 10 apps
  reprise apps -o json            Output as JSON for scripting
  reprise apps -o json | jq '.[0].slug'  Get first app's slug

```

## reprise app

```text
Show or set the default app

Usage: reprise app [OPTIONS] [COMMAND]

Commands:
  set   Set the default app for future commands
  show  Show the currently configured default app
  help  Print this message or the help of the given subcommand(s)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise app                     Show current default app
  reprise app show                Same as above
  reprise a                       Short alias
  reprise app set abc123def456    Set default app by slug
  reprise app set "My App"        Set default app by name (exact match)
  reprise app set ios             Set first app matching 'ios'

The default app is used by builds, trigger, log, and other commands
when the --app flag is not specified. The slug is the unique identifier
found in your Bitrise app URL: app.bitrise.io/app/<slug>

```

## reprise app set

```text
Set the default app for future commands

Usage: reprise app set [OPTIONS] [APP]

Arguments:
  [APP]
          App slug or name to set as default

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise app set abc123def456       Set by app slug
  reprise app set "My iOS App"       Set by exact app name
  reprise app set ios                Find first app matching 'ios'

Finding Your App Slug:
  The slug is in the Bitrise URL: app.bitrise.io/app/<slug>
  Or use 'reprise apps' to list all apps with their slugs.

The default app is saved to your config file and used by
commands like 'builds', 'trigger', and 'log' when no
--app flag is provided.

```

## reprise app show

```text
Show the currently configured default app

Usage: reprise app show [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Example:
  reprise app show                   Display default app info
  reprise app                        Same as 'app show'

Shows the app slug and name. If no default is set, you'll be
prompted to set one. Use 'reprise app set' to change it.

```

## reprise builds

```text
List builds for the default or specified app

Usage: reprise builds [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --with-search-metadata
          Include scanned/capped search metadata in a JSON envelope

  -a, --app <APP>
          App slug (overrides default app)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -s, --status <STATUS>
          Filter by build status (running, success, failed, aborted)

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

  -b, --branch <BRANCH>
          Filter by branch name (exact match)

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --current-branch
          Use the current git branch as the branch filter

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -w, --workflow <WORKFLOW>
          Filter by workflow name (exact match)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

      --workflow-contains <TEXT>
          Filter by workflow name (substring match, case-insensitive)

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --triggered-by <USER>
          Filter by user who triggered (partial match, case-insensitive)

      --me
          Show only builds triggered by you (matches Bitrise user and webhook-github/<github-user>)

  -v, --verbose
          Verbose mode - show debug information including API requests

      --since <DURATION>
          Show builds since a time (e.g., 1h, 30m, 2d, 1w, today, yesterday, this-week, 2025-01-15)

      --pr <NUMBER>
          Filter by pull request number

  -l, --limit <N>
          Maximum number of builds to return
          
          [default: 25]

      --elapsed
          Show elapsed worker time for running builds

      --average
          Show average duration for each workflow from recent completed builds

      --progress
          Show estimated progress for running builds from workflow averages

      --watch
          Watch mode - continuously refresh the build list

      --interval <SECS>
          Refresh interval in seconds for watch mode (default: 10)
          
          [default: 10]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise builds                  List recent builds
  reprise b                       Short alias
  reprise builds --status failed  Show only failed builds
  reprise builds -s running       Show running builds
  reprise builds --branch main    Filter by branch
  reprise builds --workflow deploy Filter by workflow
  reprise builds --me             Show only my builds
  reprise builds --triggered-by alice  Show builds triggered by 'alice'
  reprise builds --pr 1234        Show builds for PR #1234
  reprise builds --limit 50       Show more builds
  reprise builds --app other-app  Use different app
  reprise builds --elapsed        Show running worker time
  reprise builds --average        Show workflow duration averages
  reprise builds --progress       Show running build progress estimates
  reprise builds -o json          Output as JSON

Filtering:
  Use --me to show only builds you triggered (requires API auth).
  Use --triggered-by for partial username match (case-insensitive).
  Use --pr to filter by pull request number.
  Combine multiple filters: --status failed --branch main --me

Status Icons (in pretty output):
  [running]  Build is currently in progress
  [success]  Build completed successfully
  [failed]   Build failed
  [aborted]  Build was manually aborted

```

## reprise build

```text
Show details of a specific build

Usage: reprise build [OPTIONS] [SLUG]

Arguments:
  [SLUG]
          Build slug (unique ID from Bitrise URL or 'builds' output)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --latest
          Resolve the target build from the latest build matching the supplied filters

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

  -b, --branch <BRANCH>
          Filter latest resolution by branch name

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -w, --workflow <WORKFLOW>
          Filter latest resolution by workflow name

  -s, --status <STATUS>
          Filter latest resolution by status

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --pr <NUMBER>
          Filter latest resolution by pull request number

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

      --current-branch
          Resolve branch from the current git checkout

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -f, --follow
          Stream live log output for running builds

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --logs
          Dump the full build log to stdout

  -v, --verbose
          Verbose mode - show debug information including API requests

      --artifacts
          List build artifacts (files produced by the build)

      --interval <SECS>
          Polling interval in seconds when following (1-60 recommended)
          
          [default: 3]

  -n, --notify
          Send desktop notification when build completes (with --follow)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise build abc123            Show build details
  reprise build abc123 -o json    Output as JSON
  reprise build abc123 --app xyz  Specify app explicitly
  reprise build --latest --status failed
  reprise build --latest --current-branch
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
  or in the 'builds' command output. Example: app.bitrise.io/build/<slug>

```

## reprise log

```text
View build logs

Usage: reprise log [OPTIONS] [SLUG]

Arguments:
  [SLUG]
          Build slug (unique ID from Bitrise URL or 'builds' output)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --latest
          Resolve the target build from the latest build matching the supplied filters

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

  -b, --branch <BRANCH>
          Filter latest resolution by branch name

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -w, --workflow <WORKFLOW>
          Filter latest resolution by workflow name

  -s, --status <STATUS>
          Filter latest resolution by status

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --pr <NUMBER>
          Filter latest resolution by pull request number

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

      --current-branch
          Resolve branch from the current git checkout

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -t, --tail <LINES>
          Show only last N lines of the log

      --save <PATH>
          Save log to file (creates or overwrites)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -f, --follow
          Follow log output (stream live for running builds)

      --interval <SECS>
          Polling interval in seconds when following (1-60 recommended)
          
          [default: 3]

  -n, --notify
          Send desktop notification when build completes (with --follow)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

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
  Pipe to 'less -R' for scrollable colored output.

```

## reprise config

```text
Manage configuration

Usage: reprise config [OPTIONS] <COMMAND>

Commands:
  show     Show current configuration values
  set      Set a configuration value
  path     Show configuration file path
  init     Initialize configuration interactively
  alias    Manage app aliases (shortcuts for app slugs)
  profile  Manage named Bitrise profiles
  help     Print this message or the help of the given subcommand(s)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

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
Use 'reprise config path' to see the exact location.

```

## reprise config show

```text
Show current configuration values

Usage: reprise config show [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Example:
  reprise config show                Display all config values

Shows your current configuration including API token (masked),
default app, and output preferences.

```

## reprise config set

```text
Set a configuration value

Usage: reprise config set [OPTIONS] <KEY> <VALUE>

Arguments:
  <KEY>
          Configuration key (api.token, defaults.app_slug, etc.)

  <VALUE>
          Value to set

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise config set api.token YOUR_TOKEN
  reprise config set defaults.app_slug abc123def456
  reprise config set defaults.app_name "My iOS App"
  reprise config set output.format json

Available Keys:
  api.token           Your Bitrise personal access token
  defaults.app_slug   Default app slug for commands
  defaults.app_name   Display name for default app
  output.format       Default output format (pretty or json)

Get your API token from: https://app.bitrise.io/me/profile#/security

```

## reprise config path

```text
Show configuration file path

Usage: reprise config path [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Example:
  reprise config path                Show where config is stored

The config file is in your system's standard config directory:
  macOS:   ~/Library/Application Support/reprise/config.toml
  Linux:   ~/.config/reprise/config.toml
  Windows: %APPDATA%\reprise\config.toml

```

## reprise config init

```text
Initialize configuration interactively

Usage: reprise config init [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Example:
  reprise config init                Start interactive setup

Walks you through setting up:
  1. Your Bitrise API token
  2. Default app selection
  3. Output format preference

This is the recommended way to get started with reprise.

```

## reprise config alias

```text
Manage app aliases (shortcuts for app slugs)

Usage: reprise config alias [OPTIONS] [NAME] [SLUG]

Arguments:
  [NAME]
          Alias name (e.g., \"ignite-ios\"). Omit to list all aliases

  [SLUG]
          App slug to associate with the alias. Omit to show current value

Options:
  -r, --remove
          Remove the alias instead of setting it

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise config alias                           List all aliases
  reprise config alias ignite-ios                Show alias value
  reprise config alias ignite-ios abc123def456   Set alias
  reprise config alias ignite-ios --remove       Remove alias

Aliases allow you to use short names instead of long app slugs:
  reprise builds --app ignite-ios    # Uses alias
  reprise builds --app abc123def456  # Uses slug directly

Aliases are stored in ~/.reprise/config.toml under [aliases].

```

## reprise config profile

```text
Manage named Bitrise profiles

Usage: reprise config profile [OPTIONS] [NAME]

Arguments:
  [NAME]
          Profile name. Omit to list all profiles

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --token <TOKEN>
          API token for the profile

      --app <APP>
          Default app slug for the profile

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --format <FORMAT>
          Output format preference for the profile

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --use
          Switch to this profile

  -r, --remove
          Remove this profile

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise config profile                   List profiles
  reprise config profile work --use        Switch to 'work'
  reprise config profile work --token XXX  Create/update a profile
  reprise config profile work --app abc123 Set default app for a profile
  reprise config profile work --remove     Delete a profile

```

## reprise yml

```text
Get or update an app's bitrise.yml

Usage: reprise yml [OPTIONS] <COMMAND>

Commands:
  get   Fetch the current bitrise.yml for an app
  set   Upload a new bitrise.yml for an app
  help  Print this message or the help of the given subcommand(s)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise yml get                          Print bitrise.yml for default app
  reprise yml get --app abc123             Print bitrise.yml for specific app
  reprise yml get --save ./bitrise.yml     Save bitrise.yml to file
  reprise yml set --file ./bitrise.yml     Upload bitrise.yml (auto-backs up current config)
  reprise yml set --file ./bitrise.yml --app abc123
  reprise yml set --file ./bitrise.yml --backup-dir ./backups

Safety:
  Before every upload, reprise fetches the current bitrise.yml and
  saves a timestamped backup copy so you can roll back quickly.

```

## reprise yml get

```text
Fetch the current bitrise.yml for an app

Usage: reprise yml get [OPTIONS]

Options:
  -a, --app <APP>
          App slug (overrides default app)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --save <PATH>
          Save output to file instead of stdout

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise yml get
  reprise yml get --app abc123def456
  reprise yml get --save ./bitrise.yml

Without --save, the yml content is printed to stdout.

```

## reprise yml set

```text
Upload a new bitrise.yml for an app

Usage: reprise yml set [OPTIONS] --file <PATH>

Options:
  -f, --file <PATH>
          Path to bitrise.yml file to upload

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -a, --app <APP>
          App slug (overrides default app)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --backup-dir <DIR>
          Backup directory (defaults to ~/.reprise/backups/bitrise-yml/<app-slug>/)

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise yml set --file ./bitrise.yml
  reprise yml set --file ./bitrise.yml --app abc123def456
  reprise yml set --file ./bitrise.yml --backup-dir ./my-backups

Before uploading, reprise ALWAYS saves the current bitrise.yml
as a timestamped backup copy.

```

## reprise trigger

```text
Trigger a new build

Usage: reprise trigger [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -w, --workflow <WORKFLOW>
          Workflow name to run (as defined in bitrise.yml)

  -b, --branch <BRANCH>
          Branch to build (defaults to repo's default branch)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --current-branch
          Use the current git branch for the build

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -a, --app <APP>
          App slug (overrides default)

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -m, --message <MESSAGE>
          Commit message for the build (shown in Bitrise UI)

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --env <KEY=VALUE>
          Environment variables in KEY=VALUE format (repeatable)

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

      --wait
          Wait for build to complete before returning

  -n, --notify
          Send desktop notification when build completes (with --wait)

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --interval <SECS>
          Polling interval in seconds when waiting (1-60 recommended)
          
          [default: 10]

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise trigger -w primary              Trigger primary workflow
  reprise trigger -w deploy -b main       Build main branch with deploy workflow
  reprise trigger -w ci --env MY_VAR=foo  Pass environment variable
  reprise trigger -w ci --env A=1 --env B=2  Multiple env vars
  reprise trigger -w primary --wait       Wait for build to complete
  reprise trigger -w primary --wait -n    Wait with desktop notification
  reprise trigger -w primary --app xyz    Trigger for specific app
  reprise trigger -w deploy -m "Deploy v1.0"  Add commit message

Options:
  If --branch is not specified, the repository's default branch is used.
  Use --wait to block until the build completes. Combine with --notify
  for a desktop notification when done. Adjust --interval for polling.

Environment Variables:
  Use --env KEY=VALUE to pass environment variables to the build.
  Can be specified multiple times for multiple variables.

```

## reprise artifacts

```text
List or download build artifacts

Usage: reprise artifacts [OPTIONS] [SLUG]

Arguments:
  [SLUG]
          Build slug (unique ID from Bitrise URL or 'builds' output)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --latest
          Resolve the target build from the latest build matching the supplied filters

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

  -b, --branch <BRANCH>
          Filter latest resolution by branch name

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -w, --workflow <WORKFLOW>
          Filter latest resolution by workflow name

  -s, --status <STATUS>
          Filter latest resolution by status

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --pr <NUMBER>
          Filter latest resolution by pull request number

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

      --current-branch
          Resolve branch from the current git checkout

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -d, --download [<DIR>]
          Download artifacts to directory (current dir if no path given)

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --open
          Open the first matching artifact in the default browser

  -v, --verbose
          Verbose mode - show debug information including API requests

      --copy-url
          Copy the first matching artifact download URL to the clipboard

  -f, --filter <PATTERN>
          Filter artifacts by glob pattern (e.g., "*.ipa", "test-*")

      --exclude <PATTERN>
          Exclude artifacts matching glob pattern (e.g., "*.dSYM*")

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise artifacts abc123                List artifacts for build
  reprise art abc123                      Short alias
  reprise artifacts abc123 --download     Download all to current directory
  reprise artifacts abc123 -d ./output    Download to specific directory
  reprise artifacts abc123 -d ~/Downloads Download to home directory
  reprise artifacts abc123 -o json        List as JSON

Filtering:
  reprise artifacts abc123 --filter "*.ipa"       Only IPA files
  reprise artifacts abc123 -f "test-*"            Files starting with test-
  reprise artifacts abc123 --exclude "*.dSYM*"    Exclude dSYM files
  reprise artifacts abc123 -f "*.ipa" -d .        Download only IPAs
  reprise artifacts abc123 -f "*.apk" --exclude "*-debug*"  APKs except debug

Downloading:
  Without -d/--download, artifacts are listed but not downloaded.
  With -d, matching artifacts are downloaded to the specified directory
  (or current directory if no path given). Existing files are overwritten.

```

## reprise abort

```text
Abort a running build

Usage: reprise abort [OPTIONS] <SLUG>

Arguments:
  <SLUG>
          Build slug (unique ID from Bitrise URL or 'builds' output)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -r, --reason <REASON>
          Reason for aborting (shown in Bitrise UI)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -y, --yes
          Skip confirmation prompt

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise abort abc123                    Abort build (with confirmation)
  reprise abort abc123 -y                 Skip confirmation prompt
  reprise abort abc123 -r "Wrong branch"  Abort with reason
  reprise abort abc123 --app xyz          Specify app explicitly

Confirmation:
  By default, you'll be prompted to confirm before aborting.
  Use -y/--yes to skip the confirmation (useful for scripts).
  The abort reason is optional but helps with debugging.

```

## reprise url

```text
Parse a Bitrise URL or generate URLs from slugs

Usage: reprise url [OPTIONS] [URL]

Arguments:
  [URL]
          Bitrise URL to parse (app, build, or pipeline URL)

Options:
      --build <SLUG>
          Generate URL for a build slug (instead of parsing a URL)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --app <SLUG>
          Generate URL for an app slug (instead of parsing a URL)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --pipeline <ID>
          Generate URL for a pipeline (requires --app-slug for the app context)

      --app-slug <SLUG>
          App slug for pipeline URL generation (required with --pipeline)

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -b, --browser
          Open URL in default browser

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -w, --watch
          Watch build/pipeline progress until completion

      --interval <SECS>
          Polling interval in seconds when watching/following (default: 5)
          
          [default: 5]

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -n, --notify
          Send desktop notification when build/pipeline completes

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --set-default
          Set this app as the default (only for app URLs)

  -v, --verbose
          Verbose mode - show debug information including API requests

      --logs
          Dump the full build log (only for build URLs)

  -f, --follow
          Stream live log output for running builds (only for build URLs)

      --artifacts
          List build artifacts (only for build URLs)

      --abort
          Abort the build (only for build URLs)

      --reason <TEXT>
          Reason for aborting (with --abort)

  -y, --yes
          Skip abort confirmation prompt (with --abort)

      --retry
          Retry/rebuild the build with same parameters (only for build URLs)

      --wait
          Wait for retry build to complete (with --retry)

      --download <DIR>
          Download artifacts to directory (only for build URLs)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Parse URL Examples:
  reprise url https://app.bitrise.io/build/abc123           Show build status
  reprise url https://app.bitrise.io/app/xyz789             Show app info
  reprise url https://app.bitrise.io/app/xyz/pipelines/123  Show pipeline status
  reprise url <url> --browser                                Open URL in browser
  reprise url <url> --watch                                  Watch build/pipeline progress
  reprise url <url> --watch --notify                         Watch with notification

Generate URL Examples:
  reprise url --build abc123                  Generate build URL
  reprise url --app xyz789                    Generate app URL
  reprise url --pipeline p123 --app-slug xyz  Generate pipeline URL
  reprise url --build abc123 --browser        Generate and open in browser

Build URL View Actions:
  reprise url <build-url> --logs         Dump the full build log
  reprise url <build-url> --follow       Stream live log output (for running builds)
  reprise url <build-url> --artifacts    List build artifacts

Build URL Actions (Modify):
  reprise url <build-url> --abort            Abort running build
  reprise url <build-url> --abort -y         Abort without confirmation
  reprise url <build-url> --abort --reason "text"  Abort with reason
  reprise url <build-url> --retry            Rebuild with same parameters
  reprise url <build-url> --retry --wait     Rebuild and wait for completion
  reprise url <build-url> --download .       Download artifacts to directory

App URL Actions:
  reprise url <app-url> --set-default    Set this app as your default

Tips:
  Copy a URL from Bitrise and paste it here to quickly view status,
  check logs, abort, retry, or download artifacts without setting up app context.
  Use --watch to monitor a running build until completion. Add --notify
  to receive a desktop notification when the build or pipeline completes.

```

## reprise pipelines

```text
List pipelines for the default or specified app

Usage: reprise pipelines [OPTIONS]

Options:
  -a, --app <APP>
          App slug (overrides default app)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

  -s, --status <STATUS>
          Filter by pipeline status (running, success, failed, aborted)

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

  -b, --branch <BRANCH>
          Filter by branch name (exact match)

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --current-branch
          Use the current git branch as the branch filter

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --triggered-by <USER>
          Filter by user who triggered (partial match, case-insensitive)

      --me
          Show only pipelines triggered by you (matches Bitrise user and webhook-github/<github-user>)

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

      --since <DURATION>
          Show pipelines since a time (e.g., 1h, 30m, 2d, 1w, today, yesterday, this-week, 2025-01-15)

  -l, --limit <N>
          Maximum number of pipelines to return
          
          [default: 25]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

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
  to see individual workflow executions within a pipeline.

```

## reprise pipeline

```text
Show or manage a specific pipeline

Usage: reprise pipeline [OPTIONS] [ID] [COMMAND]

Commands:
  show     Show pipeline details and stage status
  trigger  Trigger a new pipeline run
  abort    Abort a running pipeline
  rebuild  Rebuild a pipeline (full or partial)
  watch    Watch pipeline progress until completion
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [ID]
          Pipeline ID (from 'pipelines' command or Bitrise URL)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise pipeline abc123                          Show pipeline details
  reprise p abc123                                 Short alias
  reprise pipeline show abc123                     Explicit show command
  reprise pipeline trigger my-pipeline             Trigger a pipeline
  reprise pipeline trigger deploy --branch main    Trigger with branch
  reprise pipeline trigger ci --env VERSION=1.0   Trigger with env var
  reprise pipeline abort abc123                    Abort running pipeline
  reprise pipeline abort abc123 -r "Wrong config"  Abort with reason
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

Use 'reprise pipeline <subcommand> --help' for subcommand details.

```

## reprise pipeline show

```text
Show pipeline details and stage status

Usage: reprise pipeline show [OPTIONS] [ID]

Arguments:
  [ID]
          Pipeline ID (from 'pipelines' command or Bitrise URL)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise pipeline show abc123       Show pipeline details
  reprise pipeline show abc123 -o json  Output as JSON
  reprise pipeline show abc123 --app xyz  Specify app

Displays pipeline information including:
  - Pipeline name and ID
  - Current status and duration
  - Branch and commit info
  - Stage breakdown with individual workflow status

```

## reprise pipeline trigger

```text
Trigger a new pipeline run

Usage: reprise pipeline trigger [OPTIONS] <NAME>

Arguments:
  <NAME>
          Pipeline name to trigger (as defined in bitrise.yml)

Options:
  -b, --branch <BRANCH>
          Branch to build (defaults to repo's default branch)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -a, --app <APP>
          App slug (overrides default)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --env <KEY=VALUE>
          Environment variables in KEY=VALUE format (repeatable)

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --wait
          Wait for pipeline to complete before returning

  -n, --notify
          Send desktop notification when pipeline completes (with --wait)

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --interval <SECS>
          Polling interval in seconds when waiting (default: 10)
          
          [default: 10]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

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
  Use --env KEY=VALUE to pass variables. Can be repeated.

```

## reprise pipeline abort

```text
Abort a running pipeline

Usage: reprise pipeline abort [OPTIONS] [ID]

Arguments:
  [ID]
          Pipeline ID to abort

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -r, --reason <REASON>
          Reason for aborting (shown in Bitrise UI)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -y, --yes
          Skip confirmation prompt

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise pipeline abort abc123
  reprise pipeline abort abc123 -y          Skip confirmation
  reprise pipeline abort abc123 -r "Wrong config"

Confirmation:
  By default, you'll be prompted to confirm. Use -y to skip.
  The abort reason is optional but helps with debugging.

```

## reprise pipeline rebuild

```text
Rebuild a pipeline (full or partial)

Usage: reprise pipeline rebuild [OPTIONS] [ID]

Arguments:
  [ID]
          Pipeline ID to rebuild

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --partial
          Only rebuild failed stages and their dependents

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --wait
          Wait for pipeline to complete before returning

  -n, --notify
          Send desktop notification when pipeline completes (with --wait)

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --interval <SECS>
          Polling interval in seconds when waiting (default: 10)
          
          [default: 10]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise pipeline rebuild abc123            Full rebuild
  reprise pipeline rebuild abc123 --partial  Rebuild failed stages only
  reprise pipeline rebuild abc123 --wait     Wait for completion
  reprise pipeline rebuild abc123 --partial --wait --notify

Rebuild Modes:
  Full rebuild (default): Re-runs all stages from the beginning.
  Partial rebuild (--partial): Only re-runs failed stages and
  their dependents, skipping already-successful stages.

Partial rebuilds are faster and preserve successful work.

```

## reprise pipeline watch

```text
Watch pipeline progress until completion

Usage: reprise pipeline watch [OPTIONS] [ID]

Arguments:
  [ID]
          Pipeline ID to watch

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --interval <SECS>
          Polling interval in seconds (default: 5)
          
          [default: 5]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -n, --notify
          Send desktop notification when pipeline completes

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise pipeline watch abc123
  reprise pipeline watch abc123 --notify
  reprise pipeline watch abc123 --interval 10

Monitors the pipeline and displays live status updates.
Press Ctrl+C to stop watching (pipeline continues running).

Use --notify to receive a desktop notification when the
pipeline completes (success, failure, or abort).

```

## reprise doctor

```text
Run environment and configuration diagnostics

Usage: reprise doctor [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise doctor
  reprise doctor -o json

Checks configuration, active profile, token availability, default app,
git context, and API connectivity when credentials are present.
Prints the report before returning nonzero for actionable failures.
Next: reprise config init, then reprise apps and reprise app set <slug>.

```

## reprise diagnose

```text
Diagnose a failed or suspicious build

Usage: reprise diagnose [OPTIONS] [SLUG]

Arguments:
  [SLUG]
          Build slug (unique ID from Bitrise URL or 'builds' output)

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --latest
          Diagnose the latest build matching the supplied filters

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

  -b, --branch <BRANCH>
          Filter latest resolution by branch name

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

  -w, --workflow <WORKFLOW>
          Filter latest resolution by workflow name

  -s, --status <STATUS>
          Filter latest resolution by status

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --pr <NUMBER>
          Filter latest resolution by pull request number

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

      --current-branch
          Resolve branch from the current git checkout

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise diagnose abc123
  reprise diagnose --latest --status failed
  reprise diagnose --latest --current-branch

Diagnosis includes concrete failure signals, confidence, numbered log context,
and explicit unavailable evidence. Confidence describes a signal, not a root cause.

Next: reprise log --latest --status failed --tail 80
      reprise compare <failed-build> <successful-build>

```

## reprise compare

```text
Compare two builds side by side

Usage: reprise compare [OPTIONS] <LEFT> <RIGHT>

Arguments:
  <LEFT>
          Left-hand build slug

  <RIGHT>
          Right-hand build slug

Options:
  -a, --app <APP>
          App slug (overrides default)

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise compare abc123 def456
  reprise compare abc123 def456 --app my-app

Comparison highlights differences in status, duration, branch,
workflow, commit, trigger source, pull request, and artifacts.

```

## reprise view

```text
Manage saved views for builds and pipelines

Usage: reprise view [OPTIONS] <COMMAND>

Commands:
  list    List saved views
  save    Save a view definition
  show    Show a saved view definition
  run     Run a saved view
  remove  Remove a saved view
  help    Print this message or the help of the given subcommand(s)

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise view list
  reprise view save failures --kind builds --status failed --branch main
  reprise view run failures
  reprise view remove failures

Saved views let you persist common filters and re-run them quickly.

```

## reprise view list

```text
List saved views

Usage: reprise view list [OPTIONS]

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## reprise view save

```text
Save a view definition

Usage: reprise view save [OPTIONS] --kind <KIND> <NAME>

Arguments:
  <NAME>
          View name

Options:
      --kind <KIND>
          View kind
          
          [possible values: builds, pipelines]

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

  -a, --app <APP>
          App slug (or alias)

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

  -s, --status <STATUS>
          Status filter

          Possible values:
          - running: Build is currently running
          - success: Build completed successfully
          - failed:  Build failed
          - aborted: Build was aborted

  -b, --branch <BRANCH>
          Branch filter

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

  -w, --workflow <WORKFLOW>
          Workflow filter (build views only)

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

      --triggered-by <TRIGGERED_BY>
          Triggered-by filter

      --me
          Match items triggered by the authenticated user

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

      --since <SINCE>
          Since filter

      --pr <NUMBER>
          Pull request filter (build views only)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -l, --limit <LIMIT>
          Limit

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## reprise view show

```text
Show a saved view definition

Usage: reprise view show [OPTIONS] <NAME>

Arguments:
  <NAME>
          View name

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## reprise view run

```text
Run a saved view

Usage: reprise view run [OPTIONS] <NAME>

Arguments:
  <NAME>
          View name

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## reprise view remove

```text
Remove a saved view

Usage: reprise view remove [OPTIONS] <NAME>

Arguments:
  <NAME>
          View name

Options:
      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## reprise completions

```text
Generate shell completions

Usage: reprise completions [OPTIONS] <SHELL>

Arguments:
  <SHELL>
          Shell to generate completions for
          
          [possible values: bash, elvish, fish, powershell, zsh]

Options:
      --install
          Install into a shell completion directory without editing shell startup files

      --timeout <TIMEOUT>
          Per-GET attempt timeout in seconds
          
          [default: 30]

      --directory <DIRECTORY>
          Override the installation directory

      --retries <RETRIES>
          Retry transient GET failures (mutations are never retried)
          
          [default: 2]

      --download-timeout <DOWNLOAD_TIMEOUT>
          Artifact and saved-log transfer timeout in seconds
          
          [default: 600]

      --refresh-workflows <REFRESH_WORKFLOWS>
          Explicitly refresh local workflow suggestions from a bitrise.yml file

      --search-limit <SEARCH_LIMIT>
          Maximum builds scanned for local filters and latest selectors
          
          [default: 500]

      --token <TOKEN>
          Bitrise API token (overrides config file and BITRISE_TOKEN env var)
          
          [env: BITRISE_TOKEN]

      --profile <PROFILE>
          Temporary named profile (does not change the saved active profile)

  -o, --output <OUTPUT>
          Output format: 'pretty' for human-readable, 'json' for scripting

          Possible values:
          - pretty: Colored, human-readable output
          - json:   JSON output for scripting
          
          [default: pretty]

  -q, --quiet
          Quiet mode - suppress non-essential output (progress indicators, hints)

  -v, --verbose
          Verbose mode - show debug information including API requests

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  reprise completions bash > ~/.bash_completion.d/reprise
  reprise completions zsh > ~/.zsh/completions/_reprise
  reprise completions fish > ~/.config/fish/completions/reprise.fish
  reprise completions powershell > reprise.ps1

Installation:
  Bash:   Source the file in your .bashrc
  Zsh:    Place in a directory in your $fpath, then run 'compinit'
  Fish:   Place in ~/.config/fish/completions/
  PowerShell: Dot-source with . ./reprise.ps1

Convenience:
  reprise completions zsh --install
  reprise completions zsh --refresh-workflows ./bitrise.yml

Installation prints activation instructions and never edits shell startup files.
Value suggestions read local configuration and cached workflows without API calls.

```

