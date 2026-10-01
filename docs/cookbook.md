# Reprise cookbook

Replace example app/build slugs, PR numbers, and workflow names with your own. Output descriptions below are illustrative, not live API evidence. Run `reprise doctor` if configuration or authentication fails.

## Investigate a failed PR build

```bash
reprise builds --pr 123 --status failed --limit 10
reprise diagnose --latest --pr 123 --status failed
reprise log --latest --pr 123 --status failed --tail 80
reprise artifacts --latest --pr 123 --status failed
```

Expect matching builds, followed by build metadata and any detected failure evidence. Diagnosis may be uncertain; inspect the surrounding log before treating a match as the root cause. A missing log or artifact response is unavailable evidence, not proof that none exists.

## Watch a release

```bash
reprise trigger --workflow release --branch main --wait --notify
reprise builds --branch main --watch --elapsed --average --progress
reprise pipeline watch example-pipeline --notify
```

Use the first command to launch a workflow and wait for it. The dashboard shows multiple builds; interrupt it with Ctrl+C. Use the pipeline command when the release spans several workflows/stages. Elapsed time starts when a worker starts; queued builds may have no elapsed value. Progress estimates can reach 100% before the build completes.

## Download the latest successful IPA

```bash
reprise artifacts --latest --workflow release --status success --filter '*.ipa'
reprise artifacts --latest --workflow release --status success --filter '*.ipa' --download ./artifacts
reprise url https://app.bitrise.io/build/example-build --download ./artifacts
```

Expect the filtered artifact list, then files in `./artifacts`. An artifact is written through a temporary file; a failed transfer leaves an existing destination intact. The URL command supplies app/build context from the URL. Quotes prevent your shell from expanding the glob.

## Compare a failed build with a successful one

```bash
reprise builds --workflow primary --status success --limit 5
reprise compare failed-build successful-build
reprise compare failed-build successful-build -o json
```

Expect differences in status, workflow, branch, commit, duration, and artifact names. Pick builds with comparable workflows and inputs; a duration difference alone does not establish a regression.

## Switch projects without changing your active profile

```bash
reprise config profile work --app example-work-app
reprise --profile work builds
reprise --profile work app show
reprise config profile work --use
```

Set the profile token through local configuration or use `BITRISE_TOKEN`; avoid tokens in shell history. `--profile` applies only to that invocation. `config profile work --use` intentionally persists the selection. A repository can supply token-free defaults through `.reprise.toml`; see [configuration](configuration.md).

## Save a frequently used view

```bash
reprise view save failures --kind builds --status failed --branch main --since 1d
reprise view show failures
reprise view run failures
reprise view list
```

Expect a saved filter definition and matching builds. Project-defined views are also available, without copying them into your user configuration.

## Use shell completion

```bash
reprise completions zsh --install
reprise completions bash --install
reprise completions fish --install
reprise completions powershell --directory ./completions --install
reprise completions zsh --refresh-workflows ./bitrise.yml
```

Installation prints the path and shell activation instructions. Restart or reload your shell as directed. Workflow suggestions come from an explicit local YAML refresh; tab completion does not contact Bitrise. App aliases, saved views, and profiles come from local configuration.

## Inspect configuration and export automation-friendly output

```bash
reprise config path
reprise config show
reprise doctor -o json
reprise builds --status failed -o json
reprise build example-build -o json
reprise log example-build --save ./build.log --tail 40
```

Use JSON for scripts and stderr for diagnostics. `--save` stores the full log even when `--tail` limits terminal output. Check exit codes before processing output. Detailed conventions are in [behavior](behavior.md).
