//! Token-free, runtime-only project defaults.
use super::SavedView;
use crate::error::{RepriseError, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub app: Option<String>,
    pub workflow: Option<String>,
    pub profile: Option<String>,
    #[serde(default)]
    pub views: HashMap<String, SavedView>,
}

/// Find the closest context, including the git root, without crossing that root.
pub fn discover(cwd: &Path) -> Result<Option<ProjectConfig>> {
    for dir in cwd.ancestors() {
        let path = dir.join(".reprise.toml");
        if path.exists() {
            let contents = fs::read_to_string(&path).map_err(|e| {
                RepriseError::Config(format!("Cannot read {}: {e}", path.display()))
            })?;
            let project: ProjectConfig = toml::from_str(&contents).map_err(|error: toml::de::Error| {
                // TOML Display diagnostics include source lines, which may contain credentials.
                let location = error.span().map(|span| {
                    let line = contents.bytes().take(span.start).filter(|byte| *byte == b'\n').count() + 1;
                    format!(" at line {line}")
                }).unwrap_or_default();
                RepriseError::Config(format!("Invalid {}{location}. Expected token-free project settings with keys: app, workflow, profile, views. Store credentials in user config or BITRISE_TOKEN.", path.display()))
            })?;
            for (key, value) in [
                ("app", &project.app),
                ("workflow", &project.workflow),
                ("profile", &project.profile),
            ] {
                if value.as_ref().is_some_and(|v| v.trim().is_empty()) {
                    return Err(RepriseError::Config(format!(
                        "{}: {key} cannot be empty",
                        path.display()
                    )));
                }
            }
            return Ok(Some(project));
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProfileConfig};
    #[test]
    fn nearest_ancestor_stops_at_git_directory_or_worktree_file() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".reprise.toml"), "app = 'outside'").unwrap();
        let repo = temp.path().join("repo");
        let nested = repo.join("sub/deeper");
        fs::create_dir_all(&nested).unwrap();
        fs::write(repo.join(".git"), "gitdir: elsewhere").unwrap();
        assert!(discover(&nested).unwrap().is_none());
        fs::write(repo.join(".reprise.toml"), "app = 'root'").unwrap();
        assert_eq!(
            discover(&nested).unwrap().unwrap().app.as_deref(),
            Some("root")
        );
        fs::write(repo.join("sub/.reprise.toml"), "app = 'nearest'").unwrap();
        assert_eq!(
            discover(&nested).unwrap().unwrap().app.as_deref(),
            Some("nearest")
        );
    }
    #[test]
    fn profile_edit_recovery_ignores_unknown_project_profile_and_retains_explicit_selection() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".reprise.toml"), "profile = 'missing'\n").unwrap();
        let mut config = Config::default();
        assert!(config.apply_context(temp.path(), None).is_err());
        config.apply_profile_edit_context(None).unwrap();
        assert!(config.project.is_none());
        config
            .profiles
            .insert("existing".into(), ProfileConfig::default());
        config.apply_profile_edit_context(Some("existing")).unwrap();
        config.set_token("updated".into());
        assert_eq!(
            config.profiles["existing"].api.token.as_deref(),
            Some("updated")
        );
        assert!(config.active_profile.is_none());
        assert!(config.apply_profile_edit_context(Some("unknown")).is_err());
    }

    #[test]
    fn temporary_context_is_not_serialized_and_mutations_target_selected_profile() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".reprise.toml"), "app = 'project'\nworkflow = 'project-flow'\nprofile = 'secondary'\n[views.local]\nkind = 'builds'\n").unwrap();
        let mut config = Config::default();
        config.defaults.app_slug = Some("root-app".into());
        config
            .profiles
            .insert("secondary".into(), ProfileConfig::default());
        config.apply_context(temp.path(), None).unwrap();
        assert_eq!(config.require_default_app().unwrap(), "project");
        assert_eq!(config.default_workflow(), Some("project-flow"));
        assert!(config.get_view("local").is_some());
        config.set_token("new-token".into());
        let saved: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
        assert!(saved.active_profile.is_none());
        assert!(saved.project.is_none());
        assert!(saved.get_view("local").is_none());
        assert_eq!(saved.require_default_app().unwrap(), "root-app");
        assert_eq!(
            saved.profiles["secondary"].api.token.as_deref(),
            Some("new-token")
        );
    }
}
