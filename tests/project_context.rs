use std::fs;
use tempfile::TempDir;
#[test]
fn project_app_is_used_and_token_is_rejected() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".reprise.toml"), "app = 'project-app'\n").unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .current_dir(dir.path())
        .args(["app", "show", "-o", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("project-app"));
    fs::write(dir.path().join(".reprise.toml"), "token = 'secret'\n").unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .current_dir(dir.path())
        .args(["app", "show"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(".reprise.toml"));
}
#[test]
fn completions_install_supports_explicit_directory() {
    let dir = TempDir::new().unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .args(["completions", "zsh", "--install", "--directory"])
        .arg(dir.path())
        .assert()
        .success();
    assert!(dir.path().join("_reprise").exists());
}
#[test]
fn unknown_temporary_profile_has_actionable_error() {
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .args(["--profile", "does-not-exist", "app", "show"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Unknown profile"));
}

#[test]
fn trigger_requires_workflow_before_token_or_network() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .current_dir(dir.path())
        .env_remove("BITRISE_TOKEN")
        .args(["trigger"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("--workflow"));
}

#[cfg(unix)]
#[test]
fn bash_and_zsh_runtime_completions_preserve_spaces() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".reprise.toml"), "workflow = 'test ios'\n").unwrap();
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_reprise"));
    let path = format!(
        "{}:{}",
        binary.parent().unwrap().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    for (shell, command) in [
        ("bash", "source \"$1\"; COMP_WORDS=(reprise trigger --workflow \"\"); COMP_CWORD=3; _reprise reprise \"\" --workflow; printf '<%s>\\n' \"${COMPREPLY[@]}\""),
        ("zsh", "autoload -Uz compinit; compinit -D; source \"$1\"; function compadd { shift; printf '<%s>\\n' \"$@\"; }; words=(reprise trigger --workflow \"\"); CURRENT=4; _reprise"),
    ] {
        if std::process::Command::new(shell).arg("--version").output().is_err() { continue; }
        let generated = assert_cmd::cargo::cargo_bin_cmd!("reprise").args(["completions", shell]).output().unwrap();
        assert!(generated.status.success());
        let script = dir.path().join(format!("reprise.{shell}"));
        fs::write(&script, generated.stdout).unwrap();
        let result = std::process::Command::new(shell).current_dir(dir.path()).env("PATH", &path).args(["-c", command, "shell-test"]).arg(&script).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "<test ios>\n", "{shell}");
    }
}

#[test]
fn incomplete_profile_completion_does_not_validate_current_word() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".reprise.toml"), "workflow = 'local'\n").unwrap();
    for prefix in ["", "wo"] {
        assert_cmd::cargo::cargo_bin_cmd!("reprise")
            .current_dir(dir.path())
            .args(["--__complete", "reprise", "--profile", prefix])
            .assert()
            .success();
    }
}

#[test]
fn project_views_are_listed_as_json_and_cannot_be_mutated() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join(".reprise.toml"),
        "[views.project_only]\nkind = 'builds'\n",
    )
    .unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("reprise")
        .current_dir(dir.path())
        .args(["view", "list", "-o", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("project_only"));
    for args in [
        vec!["view", "remove", "project_only"],
        vec!["view", "save", "project_only", "--kind", "builds"],
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("reprise")
            .current_dir(dir.path())
            .args(args)
            .assert()
            .failure()
            .stderr(predicates::str::contains("edit .reprise.toml"));
    }
}

#[test]
fn fish_and_powershell_runtime_completions_preserve_spaces_when_available() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".reprise.toml"), "workflow = 'test ios'\n").unwrap();
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_reprise"));
    let mut paths = vec![binary.parent().unwrap().to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();
    for (executable, shell, flags, command) in [
        ("fish", "fish", vec!["-c"], "source \"$REPRISE_TEST_COMPLETION_FILE\"; complete -C 'reprise trigger --workflow '"),
        ("pwsh", "powershell", vec!["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"], ". $env:REPRISE_TEST_COMPLETION_FILE; $line = 'reprise trigger --workflow '; (TabExpansion2 $line $line.Length).CompletionMatches | ForEach-Object { $_.CompletionText }"),
        ("pwsh", "powershell", vec!["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"], ". $env:REPRISE_TEST_COMPLETION_FILE; $line = 'reprise trigger --workflow te --app trailing'; $cursor = 'reprise trigger --workflow te'.Length; (TabExpansion2 $line $cursor).CompletionMatches | ForEach-Object { $_.CompletionText }"),
    ] {
        if std::process::Command::new(executable).arg("--version").output().is_err() { continue; }
        let generated = assert_cmd::cargo::cargo_bin_cmd!("reprise").args(["completions", shell]).output().unwrap();
        assert!(generated.status.success());
        let script = dir.path().join(if shell == "fish" { "reprise.fish" } else { "reprise.ps1" });
        fs::write(&script, generated.stdout).unwrap();
        let result = std::process::Command::new(executable).current_dir(dir.path()).env("PATH", &path).env("REPRISE_TEST_COMPLETION_FILE", &script).args(flags).arg(command).output().unwrap();
        assert!(result.status.success(), "{shell}: {}", String::from_utf8_lossy(&result.stderr));
        assert!(String::from_utf8_lossy(&result.stdout).contains("test ios"), "{shell}: stdout={}, stderr={}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr));
    }
}

#[test]
fn project_parse_errors_never_echo_source_values() {
    let dir = TempDir::new().unwrap();
    for contents in [
        "token = 'SECRET_REPRISE_MARKER'\n",
        "app = ['SECRET_REPRISE_MARKER']\n",
        "app = 'SECRET_REPRISE_MARKER\n",
    ] {
        fs::write(dir.path().join(".reprise.toml"), contents).unwrap();
        let result = assert_cmd::cargo::cargo_bin_cmd!("reprise")
            .current_dir(dir.path())
            .args(["app", "show"])
            .output()
            .unwrap();
        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(".reprise.toml"));
        assert!(!stderr.contains("SECRET_REPRISE_MARKER"));
    }
}
