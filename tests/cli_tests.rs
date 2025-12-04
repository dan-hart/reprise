//! CLI integration tests
//!
//! Tests that don't require API access

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command for the reprise binary
fn reprise() -> Command {
    Command::cargo_bin("reprise").unwrap()
}

#[test]
fn test_help() {
    reprise()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A fast, feature-rich CLI for Bitrise"));
}

#[test]
fn test_version() {
    reprise()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("reprise"));
}

#[test]
fn test_apps_help() {
    reprise()
        .args(["apps", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List all accessible Bitrise apps"))
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_builds_help() {
    reprise()
        .args(["builds", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List builds"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--workflow"));
}

#[test]
fn test_trigger_help() {
    reprise()
        .args(["trigger", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Trigger a new build"))
        .stdout(predicate::str::contains("--workflow"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--env"));
}

#[test]
fn test_config_help() {
    reprise()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage configuration"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("path"));
}

#[test]
fn test_cache_help() {
    reprise()
        .args(["cache", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage local cache"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("clear"));
}

#[test]
fn test_config_path() {
    reprise()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".reprise/config.toml"));
}

#[test]
fn test_cache_status() {
    reprise()
        .args(["cache", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache Status"));
}

#[test]
fn test_invalid_command() {
    reprise()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_global_flags() {
    // Test that global flags are available
    reprise()
        .args(["--output", "json", "config", "path"])
        .assert()
        .success();

    // Test quiet and verbose are mutually exclusive
    reprise()
        .args(["--quiet", "--verbose", "config", "path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_trigger_requires_workflow() {
    reprise()
        .arg("trigger")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--workflow"));
}

#[test]
fn test_env_var_parsing() {
    // Valid env var format
    reprise()
        .args(["trigger", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("KEY=VALUE"));
}

#[test]
fn test_output_format_options() {
    reprise()
        .args(["--output", "pretty", "config", "path"])
        .assert()
        .success();

    reprise()
        .args(["--output", "json", "config", "path"])
        .assert()
        .success();

    reprise()
        .args(["--output", "invalid", "config", "path"])
        .assert()
        .failure();
}

#[test]
fn test_aliases() {
    // Test command aliases work
    reprise()
        .args(["b", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List builds"));

    reprise()
        .args(["a", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default app"));

    reprise()
        .args(["l", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("build logs"));
}
