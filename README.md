# reprise

A fast, feature-rich CLI for [Bitrise](https://bitrise.io).

> **Note:** This is an unofficial, community-maintained project and is not affiliated with, endorsed by, or supported by Bitrise. It uses the public [Bitrise API](https://api-docs.bitrise.io/) to provide CLI functionality. For official Bitrise tools and support, please visit [bitrise.io](https://bitrise.io).

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/reprise.svg)](https://crates.io/crates/reprise)
[![Build Status](https://img.shields.io/github/actions/workflow/status/dan-hart/reprise/ci.yml?branch=main)](https://github.com/dan-hart/reprise/actions)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/dan-hart/reprise/pulls)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/codedbydan)

## Features

- **Fast** - Written in Rust for maximum performance
- **Easy authentication** - Inline token support via flag or environment variable
- **Flexible output** - Pretty terminal output by default, JSON for automation
- **Smart defaults** - Set a default app to skip repetitive flags
- **Build management** - List, filter, and inspect builds with ease
- **Pipeline support** - Full pipeline management including trigger, watch, abort, and rebuild
- **Log viewing** - View, tail, follow, and save build logs with syntax highlighting
- **Smart filtering** - Filter builds and pipelines by status, branch, workflow, or creator (`--me`)
- **URL integration** - Paste any Bitrise URL to instantly view status, logs, or artifacts

## Installation

### Homebrew (Recommended)

```bash
brew install dan-hart/tap/reprise
```

### Cargo

```bash
cargo install reprise
```

### Binary Releases

Download the latest binary for your platform from [GitHub Releases](https://github.com/dan-hart/reprise/releases).

### Switching from Cargo to Homebrew

If you previously installed via Cargo and want to switch to Homebrew:

```bash
# Install via Homebrew
brew install dan-hart/tap/reprise

# Verify Homebrew version is active
which reprise  # Should show /opt/homebrew/bin/reprise

# Remove the Cargo version
cargo uninstall reprise
```

## Quick Start

### 1. Authenticate

You can authenticate in three ways (in order of priority):

```bash
# Option 1: Inline flag (highest priority)
reprise --token YOUR_TOKEN apps

# Option 2: Environment variable
export BITRISE_TOKEN=YOUR_TOKEN
reprise apps

# Option 3: Config file (persistent)
reprise config init
```

### 2. Set a Default App

```bash
# List your apps
reprise apps

# Set default app by slug or name
reprise app set my-app-slug
```

### 3. View Builds

```bash
# List recent builds
reprise builds

# Filter by status
reprise builds --status failed

# View a specific build
reprise build abc123

# View build logs
reprise log abc123
```

## Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `reprise apps` | | List all accessible Bitrise apps |
| `reprise app show` | `a show` | Show current default app |
| `reprise app set <slug>` | `a set` | Set default app |
| `reprise builds` | `b` | List builds for an app |
| `reprise build <slug>` | | Show build details |
| `reprise log <slug>` | `logs`, `l` | View build logs |
| `reprise trigger` | | Trigger a new build |
| `reprise artifacts` | `art` | List or download build artifacts |
| `reprise abort <slug>` | | Abort a running build |
| `reprise pipelines` | `pl` | List pipelines for an app |
| `reprise pipeline show <id>` | `p show` | Show pipeline details |
| `reprise pipeline trigger <name>` | `p trigger` | Trigger a new pipeline |
| `reprise pipeline watch <id>` | `p watch` | Watch pipeline progress |
| `reprise pipeline abort <id>` | `p abort` | Abort a running pipeline |
| `reprise pipeline rebuild <id>` | `p rebuild` | Rebuild a pipeline |
| `reprise url <url>` | | Parse and interact with Bitrise URLs |
| `reprise config init` | | Interactive configuration setup |
| `reprise config show` | | Display current configuration |
| `reprise config set` | | Set a configuration value |
| `reprise config path` | | Show config file location |

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--token <TOKEN>` | | Bitrise API token (overrides config) |
| `--output <FORMAT>` | `-o` | Output format: `pretty` (default) or `json` |
| `--quiet` | `-q` | Minimal output |
| `--verbose` | `-v` | Show debug information |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

## Configuration

Configuration is stored in `~/.reprise/config.toml`:

```toml
[api]
token = "your_bitrise_api_token"

[defaults]
app_slug = "your-default-app"
app_name = "Your App Name"

[output]
format = "pretty"  # or "json"
```

### Getting Your API Token

1. Go to [Bitrise Account Settings](https://app.bitrise.io/me/profile#/security)
2. Scroll to "Personal Access Tokens"
3. Generate a new token with appropriate permissions

## JSON Output

All commands support JSON output for scripting and automation:

```bash
# Get builds as JSON
reprise builds --output json

# Pipe to jq for processing
reprise builds -o json | jq '.[] | select(.status == "failed")'
```

## Examples

### List Failed Builds on a Branch

```bash
reprise builds --status failed --branch main --limit 10
```

### Save Build Log to File

```bash
reprise log abc123 --save build.log
```

### View Last 50 Lines of a Log

```bash
reprise log abc123 --tail 50
```

### Filter Apps by Name

```bash
reprise apps --filter "ios"
```

### Use with Different App (Override Default)

```bash
reprise builds --app other-app-slug
```

### Filter Builds by Creator

```bash
# Show only your builds
reprise builds --me

# Show builds by a specific user
reprise builds --triggered-by alice
```

### Work with Bitrise URLs

```bash
# View build status from URL
reprise url https://app.bitrise.io/build/abc123

# View build logs from URL
reprise url https://app.bitrise.io/build/abc123 --logs

# Follow live log output
reprise url https://app.bitrise.io/build/abc123 --follow

# List artifacts from build URL
reprise url https://app.bitrise.io/build/abc123 --artifacts

# Set default app from URL
reprise url https://app.bitrise.io/app/xyz789 --set-default

# Watch build progress with notifications
reprise url https://app.bitrise.io/build/abc123 --watch --notify
```

### Pipeline Management

```bash
# List pipelines
reprise pipelines

# Show only your pipelines
reprise pipelines --me

# Trigger a pipeline
reprise pipeline trigger my-pipeline --branch main

# Watch pipeline progress
reprise pipeline watch abc123 --notify

# Rebuild failed workflows only
reprise pipeline rebuild abc123 --partial
```

## Development

### Prerequisites

- Rust 1.70 or later
- A Bitrise account with API access

### Building from Source

```bash
git clone https://github.com/dan-hart/reprise.git
cd reprise
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Code Style

This project uses standard Rust formatting:

```bash
cargo fmt
cargo clippy
```

## Troubleshooting

### Common Issues

#### "API token not configured"

You need to authenticate first. Choose one of these options:

```bash
# Set up persistent configuration
reprise config init

# Or use environment variable
export BITRISE_TOKEN=your_token_here

# Or provide token inline
reprise --token your_token_here apps
```

#### "No default app configured"

Set a default app to avoid specifying `--app` on every command:

```bash
# List your apps to find the slug
reprise apps

# Set the default
reprise app set your-app-slug
```

#### `--me` flag not matching webhook-triggered builds

The `--me` flag matches both your Bitrise username and GitHub webhook patterns (`webhook-github/<username>`). If webhook-triggered builds aren't showing up:

```bash
# Configure your GitHub username
git config --global github.user YOUR_GITHUB_USERNAME
```

#### Permission denied errors (401/403)

- Verify your API token is valid and not expired
- Check that the token has the required permissions for the operation
- Regenerate your token at [Bitrise Security Settings](https://app.bitrise.io/me/profile#/security)

#### Rate limiting

The Bitrise API has rate limits. If you're hitting limits:

- Reduce polling frequency with `--interval` (default: 5 seconds)
- Use `--limit` to fetch fewer results
- Wait a few minutes before retrying

### Exit Codes

reprise uses standard Unix exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Usage/argument error |
| 65 | Data parsing error |
| 66 | Resource not found (app, build, etc.) |
| 69 | Service unavailable / network error |
| 74 | I/O error |
| 77 | Permission denied |
| 78 | Configuration error |

### Getting Help

```bash
# General help
reprise --help

# Command-specific help
reprise builds --help
reprise pipeline --help
```

## Security

- API tokens are stored in `~/.reprise/config.toml` (outside any repository)
- Tokens are masked in output (only first/last 4 characters shown)
- git-secrets is configured to prevent accidental credential commits

See [SECURITY.md](SECURITY.md) for more details.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Inspired by other great Rust CLIs like [ripgrep](https://github.com/BurntSushi/ripgrep), [bat](https://github.com/sharkdp/bat), and [gh](https://github.com/cli/cli)
