use clap::Parser;
use reprise::cli::args::Cli;
use reprise::config::Config;
use std::path::Path;

#[test]
fn documentation_examples_are_valid_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut count = 0;
    for file in [
        "README.md",
        "docs/cookbook.md",
        "docs/configuration.md",
        "docs/behavior.md",
    ] {
        let source = std::fs::read_to_string(root.join(file)).expect("document exists");
        let mut bash = false;
        for line in source.lines() {
            if line.starts_with("```") {
                bash = !bash && matches!(line, "```bash" | "```sh");
                continue;
            }
            if bash && line.starts_with("reprise ") {
                let words = shell_words::split(line).expect("valid shell quoting");
                let end = words
                    .iter()
                    .position(|s| matches!(s.as_str(), "|" | ">" | ">>"))
                    .unwrap_or(words.len());
                Cli::try_parse_from(&words[..end])
                    .unwrap_or_else(|error| panic!("{file}: {line}\n{error}"));
                count += 1;
            }
        }
    }
    assert!(count >= 25, "cookbook should cover realistic workflows");
}

#[test]
fn example_user_configuration_selects_profile_at_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source =
        std::fs::read_to_string(root.join("examples/config.toml")).expect("example exists");
    let config: Config = toml::from_str(&source).expect("valid user configuration");
    assert_eq!(config.active_profile.as_deref(), Some("work"));
    assert!(!config.aliases.contains_key("active_profile"));
    assert_eq!(config.require_default_app().unwrap(), "example-work-app");
}

#[test]
fn generated_reference_exists() {
    let reference = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/cli-reference.md"),
    )
    .expect("generated reference exists");
    assert!(reference.contains("## reprise pipeline watch"));
    assert!(reference.contains("## reprise config profile"));
}

#[test]
fn generated_reference_matches_clap() {
    let actual = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/cli-reference.md"),
    )
    .expect("reference exists");
    assert_eq!(
        actual,
        reprise::cli::reference::markdown(),
        "run cargo run --example generate_reference"
    );
}

#[test]
fn example_project_configuration_is_token_free_and_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(root.join("examples/project.toml"))
        .expect("project example exists");
    let config: reprise::config::project::ProjectConfig =
        toml::from_str(&source).expect("valid strict project configuration");
    assert_eq!(config.workflow.as_deref(), Some("primary"));
    assert!(config.views.contains_key("failures"));
}

#[test]
fn documentation_local_links_resolve() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        "README.md",
        "docs/cookbook.md",
        "docs/configuration.md",
        "docs/behavior.md",
        "docs/development.md",
    ];
    for file in files {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        for link in source.split("](").skip(1) {
            let target = link.split(')').next().unwrap().split('#').next().unwrap();
            if !target.is_empty() && !target.contains("://") {
                assert!(
                    path.parent().unwrap().join(target).exists(),
                    "{file}: broken link {target}"
                );
            }
        }
    }
}
