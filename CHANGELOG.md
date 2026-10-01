# Changelog

## 0.2.0

- Shorter getting-started README, task-oriented cookbook, valid configuration examples, and generated command reference with CI drift checks.
- Token-free repository defaults in `.reprise.toml`, workflow defaults, project saved views, and temporary `--profile` selection.
- Shell completion installation and local value suggestions for aliases, profiles, saved views, and explicitly cached workflows.
- Paginated build filtering with bounded search and explicit search metadata.
- Cached identity and timing history, and concurrent independent build-comparison reads.
- Streamed artifact transfers through temporary files and lower log-processing allocations.
- Configurable read retries/timeouts and actionable doctor reports with failed-check exit status.
- Failure diagnosis based on concrete signals, confidence, step markers, line context, and explicit unavailable evidence.
- Native release archives for Linux (Intel/ARM), macOS (Intel/Apple Silicon), and Windows (Intel), with SHA-256 checksums and commit provenance.

## 0.1.11 — 2026-08-19

- Fix pipeline triggering to use the build-trigger endpoint.
- Handle queued and unknown pipeline statuses during watches.
- Refresh locked dependencies for security advisories.
