//! Shell scripts with local-only runtime value suggestions.
use super::args::{Cli, CompletionsArgs};
use crate::{
    config::{Config, Paths},
    error::{RepriseError, Result},
};
use clap::CommandFactory;
use clap_complete::Shell;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// This private protocol runs before argument parsing and never creates an API client.
pub fn handle_query() -> Result<bool> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("--__complete") {
        return Ok(false);
    }
    let json_words: Vec<String>;
    let words = if args.get(1).map(String::as_str) == Some("--words-json") {
        json_words = serde_json::from_str(args.get(2).map(String::as_str).unwrap_or("[]"))?;
        &json_words[..]
    } else {
        &args[1..]
    };
    let mut config = Config::load()?;
    let completed = &words[..words.len().saturating_sub(1)];
    let mut profile = None;
    for (index, word) in completed.iter().enumerate() {
        if let Some(name) = word.strip_prefix("--profile=") {
            profile = Some(name);
        } else if word == "--profile" {
            if let Some(name) = completed
                .get(index + 1)
                .filter(|name| !name.starts_with('-'))
            {
                profile = Some(name.as_str());
            }
        }
    }
    config.apply_context(&std::env::current_dir()?, profile)?;
    for value in values(&config, words, &Paths::new()?)? {
        println!("{value}");
    }
    Ok(true)
}

fn values(config: &Config, words: &[String], paths: &Paths) -> Result<Vec<String>> {
    let previous = words.iter().rev().nth(1).map(String::as_str).unwrap_or("");
    let mut values: Vec<String> = match previous {
        "--profile" => config.profiles.keys().cloned().collect(),
        "--app" | "-a" => {
            let mut aliases: Vec<_> = config.aliases.keys().cloned().collect();
            let profile = config
                .selected_profile
                .as_ref()
                .or(config.active_profile.as_ref())
                .and_then(|n| config.profiles.get(n));
            if let Some(profile) = profile {
                aliases.extend(profile.aliases.keys().cloned());
            }
            aliases
        }
        "--workflow" | "-w" => {
            let mut workflows: Vec<String> = match fs::read(paths.root.join("workflows.json")) {
                Ok(bytes) => serde_json::from_slice(&bytes)?,
                Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
                Err(e) => return Err(e.into()),
            };
            if let Some(default) = config.default_workflow() {
                workflows.push(default.to_string());
            }
            workflows
        }
        "run" | "show" | "remove" if words.iter().any(|w| w == "view") => config
            .list_views()
            .into_iter()
            .map(|(name, _)| name.clone())
            .collect(),
        "profile" if words.iter().any(|w| w == "config") => {
            config.profiles.keys().cloned().collect()
        }
        _ => Vec::new(),
    };
    let prefix = words.last().map(String::as_str).unwrap_or("");
    values.retain(|v| v.starts_with(prefix) && !v.contains(['\n', '\r', '\0']));
    values.sort();
    values.dedup();
    Ok(values)
}

fn refresh_workflows(file: &Path, paths: &Paths) -> Result<()> {
    let yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&fs::read_to_string(file)?)
        .map_err(|e| RepriseError::InvalidArgument(format!("Invalid {}: {e}", file.display())))?;
    let workflows = yaml
        .get("workflows")
        .and_then(serde_yaml_ng::Value::as_mapping)
        .ok_or_else(|| {
            RepriseError::InvalidArgument(format!(
                "{} must contain a workflows mapping",
                file.display()
            ))
        })?;
    let names: Vec<_> = workflows
        .keys()
        .filter_map(serde_yaml_ng::Value::as_str)
        .collect();
    paths.ensure_dirs()?;
    fs::write(
        paths.root.join("workflows.json"),
        serde_json::to_vec(&names)?,
    )?;
    Ok(())
}

pub fn run(args: &CompletionsArgs) -> Result<()> {
    if let Some(file) = &args.refresh_workflows {
        refresh_workflows(file, &Paths::new()?)?;
    }
    let script = generate(args.shell);
    if args.install {
        let dir = match &args.directory {
            Some(dir) => dir.clone(),
            None => default_directory(args.shell)?,
        };
        fs::create_dir_all(&dir)?;
        let path = dir.join(filename(args.shell));
        fs::write(&path, script)?;
        println!("Installed completions: {}", path.display());
        let quote = |path: &Path| format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
        match args.shell {
            Shell::Bash => println!("Activate in this shell: source {}", quote(&path)),
            Shell::Zsh => println!(
                "Activate in this shell: fpath=({} $fpath); autoload -Uz compinit; compinit",
                quote(&dir)
            ),
            Shell::Fish => println!("Activate in this shell: source {}", quote(&path)),
            Shell::PowerShell => println!(
                "Activate in this shell: . '{}'",
                path.to_string_lossy().replace('\'', "''")
            ),
            _ => {}
        }
    } else {
        io::stdout().write_all(script.as_bytes())?;
    }
    Ok(())
}

fn filename(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "reprise",
        Shell::Zsh => "_reprise",
        Shell::Fish => "reprise.fish",
        Shell::PowerShell => "reprise.ps1",
        _ => "reprise.elv",
    }
}

fn default_directory(shell: Shell) -> Result<PathBuf> {
    let home = crate::config::Paths::home_dir()?;
    match shell {
        Shell::Bash => Ok(home.join(".local/share/bash-completion/completions")),
        Shell::Zsh => Ok(home.join(".zsh/completions")),
        Shell::Fish => Ok(home.join(".config/fish/completions")),
        Shell::PowerShell => Err(RepriseError::InvalidArgument("PowerShell installation requires --directory; dot-source the installed reprise.ps1 from your profile.".into())),
        _ => Err(RepriseError::InvalidArgument("This shell requires an explicit --directory.".into())),
    }
}

pub fn generate(shell: Shell) -> String {
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "reprise", &mut buffer);
    let script = String::from_utf8_lossy(&buffer).into_owned();
    match shell {
        Shell::Bash => script.replacen("    local i cur prev opts cmd", r#"    local value
    COMPREPLY=()
    while IFS= read -r value; do COMPREPLY+=("$value"); done < <(command reprise --__complete "${COMP_WORDS[@]:0:COMP_CWORD}" "${COMP_WORDS[COMP_CWORD]}" 2>/dev/null)
    if (( ${#COMPREPLY[@]} )); then
        compopt -o filenames 2>/dev/null || true
        return 0
    fi
    local i cur prev opts cmd"#, 1),
        Shell::Zsh => script.replacen("    typeset -A opt_args", r#"    local -a reprise_values
    reprise_values=("${(@f)$(command reprise --__complete "${words[@]:0:$((CURRENT-1))}" "${words[CURRENT]}" 2>/dev/null)}")
    if (( ${#reprise_values[@]} )) && [[ -n ${reprise_values[1]} ]]; then
        compadd -- "${reprise_values[@]}"
        return 0
    fi
    typeset -A opt_args"#, 1),
        Shell::Fish => format!("{script}\nfunction __reprise_values\n    set -l words (commandline -opc)\n    set -l current (commandline -ct)\n    command reprise --__complete $words \"$current\" 2>/dev/null\nend\ncomplete -c reprise -f -a '(__reprise_values)'\ncomplete -c reprise -l app -s a -r -f -a '(__reprise_values)'\ncomplete -c reprise -l profile -r -f -a '(__reprise_values)'\ncomplete -c reprise -l workflow -s w -r -f -a '(__reprise_values)'\n"),
        Shell::PowerShell => script.replacen("    $commandElements = $commandAst.CommandElements", r#"    $currentStart = $cursorPosition - $wordToComplete.Length
    $words = @($commandAst.CommandElements | Where-Object { $_.Extent.EndOffset -le $currentStart } | ForEach-Object { $_.Extent.Text.Trim("'", '"') })
    $words += $wordToComplete.Trim("'", '"')
    $wordsJson = ConvertTo-Json -InputObject $words -Compress
    $values = @(& reprise --__complete --words-json $wordsJson 2>$null)
    if ($values.Count -gt 0) {
        foreach ($value in $values) {
            $quoted = "'" + $value.Replace("'", "''") + "'"
            [CompletionResult]::new($quoted, $value, [CompletionResultType]::ParameterValue, $value)
        }
        return
    }
    $commandElements = $commandAst.CommandElements"#, 1),
        _ => script,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProfileConfig;
    #[test]
    fn suggestions_are_local_sorted_and_preserve_spaces_without_tokens() {
        let temp = tempfile::tempdir().unwrap();
        let paths = Paths {
            root: temp.path().to_path_buf(),
            config_file: temp.path().join("config.toml"),
        };
        let mut config = Config::default();
        config.api.token = Some("never-emit-this".into());
        config.aliases.insert("ios app".into(), "slug".into());
        config
            .profiles
            .insert("work profile".into(), ProfileConfig::default());
        let words = |flag: &str| vec!["reprise".into(), "builds".into(), flag.into(), "".into()];
        assert_eq!(
            values(&config, &words("--app"), &paths).unwrap(),
            ["ios app"]
        );
        assert_eq!(
            values(&config, &words("--profile"), &paths).unwrap(),
            ["work profile"]
        );
        assert!(values(&config, &words("--token"), &paths)
            .unwrap()
            .is_empty());
        let file = temp.path().join("bitrise.yml");
        fs::write(&file, "workflows:\n  'test ios': {}\n  deploy: {}\n").unwrap();
        refresh_workflows(&file, &paths).unwrap();
        assert_eq!(
            values(&config, &words("--workflow"), &paths).unwrap(),
            ["deploy", "test ios"]
        );
    }
    #[test]
    fn generated_scripts_use_native_shell_syntax_and_dynamic_query() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let script = generate(shell);
            assert!(script.contains("--__complete"), "{shell}");
            assert!(!script.contains("never-emit-this"));
        }
        assert!(generate(Shell::PowerShell).contains("[CompletionResult]::new($quoted"));
        assert!(generate(Shell::Fish).contains("commandline -ct"));
    }
}
