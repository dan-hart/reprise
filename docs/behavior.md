# Filtering, networking, and diagnostics

## Build filtering

`--limit` controls how many matching builds are returned. Reprise follows build-history cursors to apply local filters such as PR, creator, workflow substring, and date. A search may stop before the entire history is examined; a capped search is not proof that no older match exists.

Use `--search-limit` to bound the records scanned. Pretty output warns about a cap; `--with-search-metadata` exposes search details in a JSON envelope. Default JSON build output remains an array for compatibility. Latest-build selectors use bounded history search too.

```bash
reprise builds --pr 123 --status failed --limit 10 --search-limit 500
reprise builds --pr 123 --with-search-metadata -o json
```

## Network controls

Read requests may retry transient failures and rate limits with bounded backoff. Authentication/permission failures are not retried. Mutation requests, including build/pipeline triggers, aborts, and YAML uploads, are never automatically retried.

```bash
reprise --timeout 45 --retries 2 builds
reprise --download-timeout 900 artifacts example-build --download ./artifacts
```

`--timeout` is the timeout per API attempt; retries can increase the total command duration. Download timeout is separate so large files do not inherit a short API timeout. A server's excessive `Retry-After` delay is reported instead of retrying earlier than requested.

Identity and workflow-history data are cached briefly within a running client. Caches are scoped to that client and app; they do not persist credentials or live build state to disk. Timing history is reused during dashboard refreshes. Independent comparison reads run concurrently with a bounded number of workers.

## Transfers and logs

Artifact downloads stream into a unique file beside the destination, then replace the destination after success. Interrupted/failed downloads clean up temporary files and leave an existing destination intact. Progress goes to terminal stderr rather than contaminating JSON stdout.

`log --save FILE --tail N` saves the full log and prints only its tail. Full-log display and diagnosis may still require the complete log in memory; streaming is useful when saving a large log with a bounded tail.

```bash
reprise log example-build --save ./build.log --tail 50
```

## Output and exit codes

Use `-o json` for automation. Errors and transfer progress go to stderr. Commands do not prompt for ambiguous selections in JSON/noninteractive contexts.

| Code | Meaning |
| --- | --- |
| 0 | Command succeeded |
| 2 | Invalid usage/arguments |
| 65 | Invalid response data |
| 66 | Resource not found |
| 69 | Network/service failure |
| 74 | I/O failure |
| 77 | Permission/authentication failure |
| 78 | Configuration failure |

`doctor` prints its structured report before returning a failed-check status. The absence of a user config file is optional if other configuration supplies the required values. Missing token/default app and API failures are actionable checks.

## Failure diagnosis

Diagnosis highlights concrete compiler, test, signing, or dependency signals, their line numbers, and nearby context. Confidence describes confidence in a log signal's category, not proof of the build's root cause. Generic build failures retain low confidence; successful zero-count summaries are not treated as errors.

Step names come from explicit supported log markers. Logs without those markers may have no step name. Missing/unavailable log and artifact requests retain their error reasons. An unavailable artifact response is not an empty artifact list.

```bash
reprise diagnose --latest --status failed --current-branch
reprise doctor -o json
```

For a suspicious diagnosis, inspect the complete log and compare a build with the same workflow and inputs. For API authentication failures, update the token through `reprise config init`; for permission failures, check account access and token scopes. Increase polling intervals if rate limited.
