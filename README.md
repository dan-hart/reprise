# reprise

A Rust CLI for working with [Bitrise](https://bitrise.io): find builds, investigate failures, watch releases, and download artifacts from your terminal.

[![CI](https://github.com/dan-hart/reprise/actions/workflows/ci.yml/badge.svg)](https://github.com/dan-hart/reprise/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

Unofficial community project; not affiliated with or endorsed by Bitrise. Uses the public Bitrise API.

## A typical session

```bash
reprise builds --current-branch --status failed
reprise diagnose --latest --current-branch --status failed
reprise log --latest --current-branch --status failed --tail 50
```

Illustrative diagnosis output (actual values depend on your build):

```text
Build diagnosis for #1234
Status: failed
Workflow: primary
Likely category: tests
First error: Test Case 'CheckoutTests.testPayment' failed
Next step: Open the full log and inspect the first failing test.
```

## Install

```sh
brew install dan-hart/tap/reprise
# Or:
cargo install --git https://github.com/dan-hart/reprise --tag v0.2.0 --locked
```

Prebuilt binaries and checksums are available on [GitHub Releases](https://github.com/dan-hart/reprise/releases). If switching from Cargo to Homebrew, check `which reprise` before uninstalling your old Cargo copy.

## Get started in five commands

Get a personal access token from [Bitrise security settings](https://app.bitrise.io/me/profile#/security), then:

```bash
reprise config init
reprise apps
reprise app set example-app
reprise doctor
reprise builds
```

`config init` prompts for your token. Alternatively, set `BITRISE_TOKEN` in your environment. `--token` overrides environment and configured credentials. Do not commit tokens to your repository.

## Everyday commands

```bash
# Filter builds for your PR
reprise builds --pr 123 --status failed
# Watch elapsed worker time and estimated progress
reprise builds --watch --elapsed --progress
# Trigger and receive a completion notification
reprise trigger --workflow primary --current-branch --wait --notify
# Download the latest successful app package
reprise artifacts --latest --status success --filter '*.ipa' --download ./artifacts
# Use a profile for this invocation
reprise --profile work builds
# Install shell completions; does not modify your shell rc
reprise completions zsh --install
```

Builds are individual workflow executions. Pipelines coordinate multiple workflows/stages: use `pipeline watch` to track the whole pipeline. Progress is an estimate based on recent completed workflow durations, not a measurement from Bitrise.

## Learn more

- [Cookbook: complete workflows and expected results](docs/cookbook.md)
- [Configuration, profiles, project defaults, and completions](docs/configuration.md)
- [Generated command reference](docs/cli-reference.md)
- [Networking, filtering, output, and troubleshooting](docs/behavior.md)
- [Development and verification](docs/development.md)
- [Changes in v0.2.0](CHANGELOG.md)
- [Security policy](SECURITY.md)

Run `reprise --help` or `reprise builds --help` for command help. Aliases include `b` (builds), `l` (log), `p` (pipeline), and `pl` (pipelines).

## Contributing

Open an issue or pull request on [GitHub](https://github.com/dan-hart/reprise). Build with `cargo build`, then run the checks in [development docs](docs/development.md). Licensed under GPL-3.0-only; see [LICENSE](LICENSE).
