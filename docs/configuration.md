# Configuration and shell completion

## User settings

User configuration lives at `~/.reprise/config.toml`. The directory and file use owner-only permissions on Unix. Start with `reprise config init`; [examples/config.toml](../examples/config.toml) is a token-free reference.

Root keys such as `active_profile` must be above TOML table headers:

```toml
active_profile = "work"

[aliases]
ios = "example-ios-app"

[profiles.work.defaults]
app_slug = "example-work-app"
```

Profiles contain separate API tokens, default app, output settings, and aliases. CLI token > `BITRISE_TOKEN` > selected-profile token > root token. Explicit command flags override defaults.

```bash
reprise config profile work --app example-work-app
reprise --profile work builds
reprise config profile work --use
reprise config alias ios example-ios-app
```

`--profile` never changes the saved active profile. Profile-scoped settings and aliases changed with `--profile` update that profile. Saved views remain user-level settings, and named profile-management commands target their named profile. `config profile NAME --use` deliberately changes the persisted active profile.

## Repository defaults

Place `.reprise.toml` in your repository. Reprise searches from the current directory upward for the nearest file, stopping at a Git repository/worktree boundary. Outside a repository, the nearest ancestor project file is used. Unknown keys and credentials are rejected.

```toml
app = "example-work-app"
workflow = "primary"
profile = "work"

[views.failures]
kind = "builds"
status = "failed"
branch = "main"
```

The supported top-level keys are `app`, `workflow`, `profile`, and `views`. Profile names must exist in your user configuration. App aliases may be used as app defaults. Project views override same-named user views during reads; changes to user configuration do not persist project overlays. Explicit CLI profile selection overrides project profile selection, which overrides your persisted active profile.

```bash
reprise trigger --current-branch
reprise trigger --workflow release --current-branch
reprise view run failures
```

The first trigger uses the project workflow; the second overrides it. A trigger without either an explicit workflow or a default returns a helpful usage error.

## Completions

Static command and flag completions work without credentials. Runtime value suggestions read local aliases, profiles, saved views, and workflow names; they do not make API calls.

```bash
reprise completions zsh --install
reprise completions fish --install
reprise completions bash --directory ./completions --install
reprise completions powershell --directory ./completions --install
reprise completions zsh --refresh-workflows ./bitrise.yml
```

`--directory` overrides the installation directory. Reprise prints activation instructions and never edits your shell rc. Workflow names are refreshed explicitly from a local Bitrise YAML file. Fetch your current YAML first if needed:

```bash
reprise yml get --save ./bitrise.yml
reprise completions zsh --refresh-workflows ./bitrise.yml
```

For scripts or packaging, generate the completion script directly:

```bash
reprise completions zsh > ./_reprise
reprise completions bash > ./reprise.bash
```
