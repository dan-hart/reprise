# Development and verification

Use a current stable Rust toolchain. Build and run offline checks:

```sh
cargo build --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked -- -D warnings
cargo run --locked --example generate_reference -- --check
cargo audit
```

The tests use local HTTP mock servers; no Bitrise token is required. Documentation checks parse the command examples through Clap and load the TOML examples. Tests verify repository boundaries and that runtime project/profile overlays are not serialized into user settings.

After changing help or flags, regenerate the reference:

```sh
cargo run --locked --example generate_reference
```

Completion runtime tests run for installed Bash, Zsh, Fish, and PowerShell. CI installs/requires all four on Linux; local runs skip shells absent from the machine. Generated shell scripts must preserve values containing spaces and perform no API requests while completing.

## Release process

Update `Cargo.toml`, regenerate `Cargo.lock` and the reference, and update `CHANGELOG.md`. Verify source checks and `cargo build --locked --release`; create `vVERSION` only on the reviewed commit after CI passes.

The tag-triggered release workflow builds and tests native binaries on Linux (x86_64/aarch64), macOS (Intel/Apple Silicon), and Windows (x86_64). Each archive includes the license, documentation, and `build-info.json` with the source commit. It prepares a draft release only after all platform jobs succeed and produces `SHA256SUMS`. Review and publish the draft, then update the Homebrew formula's source tag and checksum.

Cargo publication additionally requires crates.io ownership and registry credentials. A GitHub release does not automatically publish a crate. Validate with `cargo publish --dry-run` before publishing through a configured registry account.

## Reproducible performance checks

```sh
cargo test --locked --lib performance_ -- --nocapture
```

These local HTTP fixtures assert request counts instead of relying on unstable wall-clock thresholds:

| Fixture | Verified request count |
| --- | --- |
| 100 paired identity/history lookups within the cache TTL | 2 HTTP requests |
| 3 dashboard refreshes with timing history | 4 HTTP requests |
| 2 independent comparisons | 8 HTTP requests |

Timing output is diagnostic only and depends on the host. This does not measure live Bitrise latency. Separate fixtures cover cache expiry, app/client isolation, and 100,000 chunked log fragments without recursively chained readers.

Run packaging checks without building real binaries:

```sh
python3 scripts/test_package_release.py
```

They package fake binaries in temporary directories solely to verify archive contents, linked documentation, and provenance. Release CI separately builds and smoke-tests the real binaries.
