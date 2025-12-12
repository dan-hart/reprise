//! CLI integration tests
//!
//! Tests that don't require API access

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command for the reprise binary
fn reprise() -> Command {
    Command::new(env!("CARGO_BIN_EXE_reprise"))
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
fn test_config_path() {
    reprise()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".reprise/config.toml"));
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

// ─────────────────────────────────────────────────────────────────────────────
// Additional Command Help Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_abort_help() {
    reprise()
        .args(["abort", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Abort"))
        .stdout(predicate::str::contains("--reason"))
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn test_artifacts_help() {
    reprise()
        .args(["artifacts", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("artifacts"))
        .stdout(predicate::str::contains("--download"));
}

#[test]
fn test_artifacts_alias() {
    reprise()
        .args(["art", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("artifacts"));
}

#[test]
fn test_log_help() {
    reprise()
        .args(["log", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("logs"))
        .stdout(predicate::str::contains("--tail"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("--save"));
}

#[test]
fn test_logs_alias() {
    reprise()
        .args(["logs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("logs"));
}

#[test]
fn test_url_help() {
    reprise()
        .args(["url", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("URL"))
        .stdout(predicate::str::contains("--browser"))
        .stdout(predicate::str::contains("--set-default"));
}

#[test]
fn test_pipelines_help() {
    reprise()
        .args(["pipelines", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelines"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--branch"));
}

#[test]
fn test_pipelines_alias() {
    reprise()
        .args(["pl", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelines"));
}

#[test]
fn test_pipeline_help() {
    reprise()
        .args(["pipeline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipeline"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("trigger"))
        .stdout(predicate::str::contains("abort"))
        .stdout(predicate::str::contains("rebuild"))
        .stdout(predicate::str::contains("watch"));
}

#[test]
fn test_pipeline_alias() {
    reprise()
        .args(["p", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipeline"));
}

#[test]
fn test_app_help() {
    reprise()
        .args(["app", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default app"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"));
}

#[test]
fn test_build_help() {
    reprise()
        .args(["build", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("--logs"))
        .stdout(predicate::str::contains("--artifacts"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Required Argument Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_abort_requires_build_slug() {
    reprise()
        .arg("abort")
        .assert()
        .failure()
        .stderr(predicate::str::contains("SLUG"));
}

#[test]
fn test_build_requires_build_slug() {
    reprise()
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("SLUG"));
}

#[test]
fn test_log_requires_build_slug() {
    reprise()
        .arg("log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("SLUG"));
}

#[test]
fn test_artifacts_requires_build_slug() {
    reprise()
        .arg("artifacts")
        .assert()
        .failure()
        .stderr(predicate::str::contains("SLUG"));
}

#[test]
fn test_url_requires_url_arg() {
    reprise()
        .arg("url")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL"));
}

#[test]
fn test_pipeline_show_requires_id() {
    reprise()
        .args(["pipeline", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ID"));
}

#[test]
fn test_pipeline_trigger_requires_name() {
    reprise()
        .args(["pipeline", "trigger"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("NAME"));
}

#[test]
fn test_pipeline_abort_requires_id() {
    reprise()
        .args(["pipeline", "abort"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ID"));
}

#[test]
fn test_pipeline_rebuild_requires_id() {
    reprise()
        .args(["pipeline", "rebuild"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ID"));
}

#[test]
fn test_pipeline_watch_requires_id() {
    reprise()
        .args(["pipeline", "watch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ID"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Config Subcommand Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_config_set_requires_key_value() {
    reprise()
        .args(["config", "set"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("KEY"));
}

#[test]
fn test_config_set_requires_value() {
    reprise()
        .args(["config", "set", "api.token"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("VALUE"));
}

#[test]
fn test_app_set_requires_app_arg() {
    reprise()
        .args(["app", "set"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("APP"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Output Format Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_json_output_format_config_show() {
    // config show with json should output valid JSON structure
    reprise()
        .args(["--output", "json", "config", "show"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Flag Combination Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quiet_mode() {
    reprise()
        .args(["--quiet", "config", "path"])
        .assert()
        .success();
}

#[test]
fn test_verbose_mode() {
    reprise()
        .args(["--verbose", "config", "path"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Filter Option Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_apps_filter_option() {
    // Just verify the filter option is accepted
    reprise()
        .args(["apps", "--filter", "test", "--help"])
        .assert()
        .success();
}

#[test]
fn test_apps_limit_option() {
    reprise()
        .args(["apps", "--limit", "5", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_status_filter() {
    reprise()
        .args(["builds", "--status", "success", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_branch_filter() {
    reprise()
        .args(["builds", "--branch", "main", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_workflow_filter() {
    reprise()
        .args(["builds", "--workflow", "primary", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_me_filter() {
    reprise()
        .args(["builds", "--me", "--help"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// New Feature Tests (v0.1.7)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_completions_help() {
    reprise()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("shell completions"))
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("zsh"))
        .stdout(predicate::str::contains("fish"));
}

#[test]
fn test_completions_bash() {
    reprise()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_reprise"));
}

#[test]
fn test_completions_zsh() {
    reprise()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

#[test]
fn test_completions_fish() {
    reprise()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_url_generate_build() {
    reprise()
        .args(["url", "--build", "abc123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://app.bitrise.io/build/abc123"));
}

#[test]
fn test_url_generate_app() {
    reprise()
        .args(["url", "--app", "xyz789"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://app.bitrise.io/app/xyz789"));
}

#[test]
fn test_url_generate_pipeline_requires_app_slug() {
    reprise()
        .args(["url", "--pipeline", "p123"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--app-slug"));
}

#[test]
fn test_url_generate_pipeline() {
    reprise()
        .args(["url", "--pipeline", "p123", "--app-slug", "myapp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://app.bitrise.io/app/myapp/pipelines/p123"));
}

#[test]
fn test_builds_since_option() {
    reprise()
        .args(["builds", "--since", "1h", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_workflow_contains_option() {
    reprise()
        .args(["builds", "--workflow-contains", "test", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_watch_option() {
    reprise()
        .args(["builds", "--watch", "--help"])
        .assert()
        .success();
}

#[test]
fn test_builds_watch_with_interval() {
    reprise()
        .args(["builds", "--watch", "--interval", "5", "--help"])
        .assert()
        .success();
}

#[test]
fn test_pipelines_since_option() {
    reprise()
        .args(["pipelines", "--since", "today", "--help"])
        .assert()
        .success();
}
