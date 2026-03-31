# Reprise CLI Roadmap Design

**Date:** 2026-03-30

## Goal

Expand `reprise` from a capable Bitrise utility into a faster day-to-day operator CLI by adding:

- Interactive selection for common resource lookups
- "Latest" shortcuts that remove the need to paste build IDs
- Smarter artifact workflows
- Git-aware shortcuts
- Named profiles and saved views
- Better watch/dashboard experiences
- Diagnostics and analysis commands

## Rollout

### Phase 1: Workflow-first

Focus on daily speed with minimal architectural risk.

- Interactive picker mode
- Latest shortcuts
- Smarter artifacts
- Git-aware shortcuts

### Phase 2: Operational maturity

Add durable workflow state and operator polish.

- Multi-context config profiles
- Saved queries and named views
- `doctor` command
- Richer watch dashboard

### Phase 3: Analysis features

Add higher-level interpretation features built on the stronger foundation from phases 1 and 2.

- Failure triage / diagnosis
- Build comparison

## Feature Decisions

### 1. Interactive picker mode

Initial support should be added where it removes the most friction:

- `reprise app set`
- `reprise build`
- `reprise log`
- `reprise artifacts`
- `reprise pipeline` show/watch/abort/rebuild

Behavior:

- When the positional slug/id is omitted in pretty mode and stdin/stdout are TTYs, fetch a recent list and present a numbered picker.
- The picker should stay simple and dependency-light: numbered options, one prompt, one selection.
- JSON mode should never become interactive.

### 2. Latest shortcuts

Add a shared "latest build resolver" used by `build`, `log`, and `artifacts`.

Behavior:

- Support `--latest` plus filters like branch, workflow, PR, and status.
- Support `--current-branch` as a Git-aware branch source.
- Reuse the same resolver in artifact workflows and analysis commands.

### 3. Smarter artifacts

Extend the existing artifact command so it is more action-oriented.

Behavior:

- Allow artifact lookup from `--latest`
- Add `--open` for the first matching artifact
- Add `--copy-url` for the first matching artifact
- Preserve existing list/download behavior

### 4. Git-aware shortcuts

Use local git state to reduce required flags.

Behavior:

- `--current-branch` resolves to `git rev-parse --abbrev-ref HEAD`
- Commands that resolve "latest" can use the current branch automatically when explicitly requested
- If no app is set and the current repo name matches an alias, resolve that alias as a convenience

### 5. Multi-context config profiles

Add named profiles for separate Bitrise accounts, orgs, or app defaults.

Behavior:

- Config supports an `active_profile`
- Profiles carry token/defaults/output/aliases
- Existing top-level config remains valid and acts as the fallback/default context

### 6. Saved queries / named views

Store reusable filters for common workflows.

Behavior:

- Add a `view` command with `list`, `save`, `show`, `run`, and `remove`
- First version targets `builds` and `pipelines`
- Views store serialized filter values and run through the existing command formatters

### 7. Richer watch dashboard

Improve the current watch modes without adding a heavyweight TUI dependency.

Behavior:

- Add a concise summary header with counts, current app, filter summary, and last refresh time
- Highlight changes between refreshes in watch mode
- Improve pipeline watch/build follow headers so long-running sessions are easier to scan

### 8. Failure triage

Add a `diagnose` command for builds.

Behavior:

- Fetch build metadata, log content, and artifacts
- Heuristically identify likely failure category
- Surface first error, last error, possible next step, and useful follow-up commands

### 9. Build comparison

Add a `compare` command for two builds.

Behavior:

- Compare status, duration, branch, workflow, commit, trigger source, pull request, and artifact names
- Pretty output emphasizes deltas
- JSON output returns a structured diff

### 10. Doctor command

Add a setup and environment validation command.

Behavior:

- Check config file presence
- Check active profile/default app/token availability
- Check API reachability with authenticated user lookup
- Check git-aware prerequisites like current branch and optional GitHub username

## Architecture Notes

- Shared selection/resolution logic should live in new helper functions under `src/cli/commands/common.rs` or a dedicated helper module.
- Avoid adding a full-screen terminal UI dependency in the first pass.
- Keep JSON mode deterministic and non-interactive.
- Keep current config schema backward compatible by layering profile-aware accessors rather than rewriting all command logic.
- Prefer adding focused helpers for "resolve build target", "resolve artifact target", and "read git context" instead of embedding logic in each command.

## Testing Strategy

- Unit tests for new config/profile/view helpers
- Unit tests for git-aware branch resolution and selection helper validation
- Unit tests for diagnose heuristics and compare diff generation
- CLI help/parse coverage in `tests/cli_tests.rs`
- Keep `cargo test` as the primary verification gate
