use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::paths::Paths;
use crate::error::{RepriseError, Result};

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// API configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Default settings
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Output preferences
    #[serde(default)]
    pub output: OutputConfig,

    /// App aliases (short name -> app slug)
    #[serde(default)]
    pub aliases: HashMap<String, String>,

    /// Name of the active profile, if any
    #[serde(default)]
    pub active_profile: Option<String>,

    /// Named profiles for different Bitrise contexts
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    /// Saved views for common build/pipeline filters
    #[serde(default)]
    pub views: HashMap<String, SavedView>,
}

/// API-related configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Bitrise API token
    pub token: Option<String>,
}

/// A named profile with its own token, defaults, output, and aliases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

/// Default values for commands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default app slug
    pub app_slug: Option<String>,
    /// Default app name (for display)
    pub app_name: Option<String>,
}

/// Output formatting preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    #[serde(default = "default_format")]
    pub format: String,
}

/// Saved view kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SavedViewKind {
    Builds,
    Pipelines,
}

/// Saved view definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub kind: SavedViewKind,
    pub app: Option<String>,
    pub status: Option<String>,
    pub branch: Option<String>,
    pub workflow: Option<String>,
    pub triggered_by: Option<String>,
    #[serde(default)]
    pub me: bool,
    pub since: Option<String>,
    pub pr: Option<i64>,
    pub limit: Option<u32>,
}

fn default_format() -> String {
    "pretty".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
        }
    }
}

impl Config {
    fn current_profile(&self) -> Option<&ProfileConfig> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    fn current_profile_mut_or_create(&mut self) -> Option<&mut ProfileConfig> {
        let name = self.active_profile.clone()?;
        Some(self.profiles.entry(name).or_default())
    }

    fn effective_token(&self) -> Option<&str> {
        self.current_profile()
            .and_then(|profile| profile.api.token.as_deref())
            .or(self.api.token.as_deref())
    }

    fn effective_defaults(&self) -> &DefaultsConfig {
        self.current_profile()
            .map(|profile| &profile.defaults)
            .unwrap_or(&self.defaults)
    }

    fn effective_output(&self) -> &OutputConfig {
        self.current_profile()
            .map(|profile| &profile.output)
            .unwrap_or(&self.output)
    }

    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let paths = Paths::new()?;
        Self::load_from(&paths)
    }

    /// Load configuration from a specific paths instance
    pub fn load_from(paths: &Paths) -> Result<Self> {
        if !paths.config_exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&paths.config_file)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let paths = Paths::new()?;
        self.save_to(&paths)
    }

    /// Save configuration to a specific paths instance
    pub fn save_to(&self, paths: &Paths) -> Result<()> {
        paths.ensure_dirs()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&paths.config_file, &contents)?;

        // Set restrictive permissions on config file (contains API token)
        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&paths.config_file, perms)?;
        }

        Ok(())
    }

    /// Get the API token or return an error with instructions
    pub fn require_token(&self) -> Result<&str> {
        self.effective_token().ok_or_else(|| {
            RepriseError::config_missing(
                "API token not configured. Run 'reprise config init' to set up.",
            )
        })
    }

    /// Get the default app slug or return an error with instructions
    pub fn require_default_app(&self) -> Result<&str> {
        self.effective_defaults()
            .app_slug
            .as_deref()
            .ok_or(RepriseError::NoDefaultApp)
    }

    /// Set the default app
    pub fn set_default_app(&mut self, slug: String, name: Option<String>) {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.defaults.app_slug = Some(slug);
            profile.defaults.app_name = name;
        } else {
            self.defaults.app_slug = Some(slug);
            self.defaults.app_name = name;
        }
    }

    /// Set just the default app name, respecting the active profile.
    pub fn set_default_app_name(&mut self, name: String) {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.defaults.app_name = Some(name);
        } else {
            self.defaults.app_name = Some(name);
        }
    }

    /// Set the API token
    pub fn set_token(&mut self, token: String) {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.api.token = Some(token);
        } else {
            self.api.token = Some(token);
        }
    }

    /// Set an app alias
    pub fn set_alias(&mut self, name: String, slug: String) {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.aliases.insert(name, slug);
        } else {
            self.aliases.insert(name, slug);
        }
    }

    /// Remove an app alias
    pub fn remove_alias(&mut self, name: &str) -> Option<String> {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.aliases.remove(name)
        } else {
            self.aliases.remove(name)
        }
    }

    /// Get an app slug by alias name
    pub fn get_alias(&self, name: &str) -> Option<&str> {
        self.current_profile()
            .and_then(|profile| profile.aliases.get(name).map(|s| s.as_str()))
            .or_else(|| self.aliases.get(name).map(|s| s.as_str()))
    }

    /// Resolve an app slug, checking aliases first
    ///
    /// If the input matches an alias, returns the corresponding slug.
    /// Otherwise, returns the input unchanged.
    pub fn resolve_alias<'a>(&'a self, input: &'a str) -> &'a str {
        self.get_alias(input).unwrap_or(input)
    }

    /// Get the effective default app name, respecting the active profile.
    pub fn default_app_name(&self) -> Option<&str> {
        self.effective_defaults().app_name.as_deref()
    }

    /// Get the effective output format string.
    pub fn output_format(&self) -> &str {
        &self.effective_output().format
    }

    /// Set the output format, respecting the active profile.
    pub fn set_output_format(&mut self, format: String) {
        if let Some(profile) = self.current_profile_mut_or_create() {
            profile.output.format = format;
        } else {
            self.output.format = format;
        }
    }

    /// Set the active profile name.
    pub fn set_active_profile(&mut self, name: Option<String>) {
        self.active_profile = name;
    }

    /// Create or update a profile.
    pub fn upsert_profile(&mut self, name: String, profile: ProfileConfig) {
        self.profiles.insert(name, profile);
    }

    /// Remove a profile.
    pub fn remove_profile(&mut self, name: &str) -> Option<ProfileConfig> {
        if self.active_profile.as_deref() == Some(name) {
            self.active_profile = None;
        }
        self.profiles.remove(name)
    }

    /// Get a profile by name.
    pub fn get_profile(&self, name: &str) -> Option<&ProfileConfig> {
        self.profiles.get(name)
    }

    /// Set or update a saved view.
    pub fn set_view(&mut self, name: String, view: SavedView) {
        self.views.insert(name, view);
    }

    /// Get a saved view by name.
    pub fn get_view(&self, name: &str) -> Option<&SavedView> {
        self.views.get(name)
    }

    /// Remove a saved view.
    pub fn remove_view(&mut self, name: &str) -> Option<SavedView> {
        self.views.remove(name)
    }

    /// Return a sorted list of saved view names and definitions.
    pub fn list_views(&self) -> Vec<(&String, &SavedView)> {
        let mut views: Vec<_> = self.views.iter().collect();
        views.sort_by_key(|(name, _)| *name);
        views
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test Paths instance using a temp directory
    fn make_test_paths(temp_dir: &TempDir) -> Paths {
        let root = temp_dir.path().to_path_buf();
        Paths {
            config_file: root.join("config.toml"),
            root,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Default Value Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.api.token.is_none());
        assert!(config.defaults.app_slug.is_none());
        assert!(config.defaults.app_name.is_none());
        assert_eq!(config.output.format, "pretty");
    }

    #[test]
    fn test_output_config_default() {
        let output = OutputConfig::default();
        assert_eq!(output.format, "pretty");
    }

    #[test]
    fn test_active_profile_overrides_root_values() {
        let mut config = Config::default();
        config.api.token = Some("root-token".to_string());
        config.defaults.app_slug = Some("root-app".to_string());
        config.set_alias("root".to_string(), "root-slug".to_string());

        let mut profile = ProfileConfig::default();
        profile.api.token = Some("work-token".to_string());
        profile.defaults.app_slug = Some("work-app".to_string());
        profile
            .aliases
            .insert("work".to_string(), "work-slug".to_string());

        config.active_profile = Some("work".to_string());
        config.profiles.insert("work".to_string(), profile);

        assert_eq!(config.require_token().unwrap(), "work-token");
        assert_eq!(config.require_default_app().unwrap(), "work-app");
        assert_eq!(config.resolve_alias("work"), "work-slug");
        assert_eq!(config.resolve_alias("root"), "root-slug");
    }

    #[test]
    fn test_set_token_uses_active_profile() {
        let mut config = Config::default();
        config.active_profile = Some("work".to_string());

        config.set_token("work-token".to_string());

        assert_eq!(
            config.profiles["work"].api.token.as_deref(),
            Some("work-token")
        );
        assert!(config.api.token.is_none());
    }

    #[test]
    fn test_saved_view_roundtrip() {
        let mut config = Config::default();
        config.set_view(
            "failures".to_string(),
            SavedView {
                kind: SavedViewKind::Builds,
                app: Some("my-app".to_string()),
                status: Some("failed".to_string()),
                branch: Some("main".to_string()),
                workflow: None,
                triggered_by: None,
                me: false,
                since: Some("1d".to_string()),
                pr: None,
                limit: Some(10),
            },
        );

        let view = config.get_view("failures").unwrap();
        assert_eq!(view.kind, SavedViewKind::Builds);
        assert_eq!(view.status.as_deref(), Some("failed"));
        assert_eq!(view.branch.as_deref(), Some("main"));
        assert_eq!(config.list_views().len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Load/Save Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_load_returns_default_when_no_file() {
        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        let config = Config::load_from(&paths).unwrap();
        assert!(config.api.token.is_none());
        assert_eq!(config.output.format, "pretty");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        let mut config = Config::default();
        config.api.token = Some("test-token-123".to_string());
        config.defaults.app_slug = Some("my-app-slug".to_string());
        config.defaults.app_name = Some("My App".to_string());
        config.output.format = "json".to_string();

        config.save_to(&paths).unwrap();

        let loaded = Config::load_from(&paths).unwrap();
        assert_eq!(loaded.api.token, Some("test-token-123".to_string()));
        assert_eq!(loaded.defaults.app_slug, Some("my-app-slug".to_string()));
        assert_eq!(loaded.defaults.app_name, Some("My App".to_string()));
        assert_eq!(loaded.output.format, "json");
    }

    #[test]
    fn test_save_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        let config = Config::default();
        config.save_to(&paths).unwrap();

        assert!(paths.root.exists());
        assert!(paths.config_file.exists());
    }

    #[test]
    fn test_load_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        // Write a partial config (only api section)
        fs::create_dir_all(&paths.root).unwrap();
        fs::write(
            &paths.config_file,
            r#"
[api]
token = "partial-token"
"#,
        )
        .unwrap();

        let config = Config::load_from(&paths).unwrap();
        assert_eq!(config.api.token, Some("partial-token".to_string()));
        assert!(config.defaults.app_slug.is_none()); // defaults to None
        assert_eq!(config.output.format, "pretty"); // defaults to "pretty"
    }

    #[test]
    fn test_load_empty_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        fs::create_dir_all(&paths.root).unwrap();
        fs::write(&paths.config_file, "").unwrap();

        let config = Config::load_from(&paths).unwrap();
        assert!(config.api.token.is_none());
        assert_eq!(config.output.format, "pretty");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Require Methods Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_require_token_when_present() {
        let mut config = Config::default();
        config.api.token = Some("my-token".to_string());

        let token = config.require_token().unwrap();
        assert_eq!(token, "my-token");
    }

    #[test]
    fn test_require_token_when_missing() {
        let config = Config::default();
        let result = config.require_token();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("API token not configured"));
    }

    #[test]
    fn test_require_default_app_when_present() {
        let mut config = Config::default();
        config.defaults.app_slug = Some("app-123".to_string());

        let slug = config.require_default_app().unwrap();
        assert_eq!(slug, "app-123");
    }

    #[test]
    fn test_require_default_app_when_missing() {
        let config = Config::default();
        let result = config.require_default_app();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No default app"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Setter Methods Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_set_default_app_with_name() {
        let mut config = Config::default();
        config.set_default_app("app-slug".to_string(), Some("App Name".to_string()));

        assert_eq!(config.defaults.app_slug, Some("app-slug".to_string()));
        assert_eq!(config.defaults.app_name, Some("App Name".to_string()));
    }

    #[test]
    fn test_set_default_app_without_name() {
        let mut config = Config::default();
        config.set_default_app("app-slug".to_string(), None);

        assert_eq!(config.defaults.app_slug, Some("app-slug".to_string()));
        assert!(config.defaults.app_name.is_none());
    }

    #[test]
    fn test_set_token() {
        let mut config = Config::default();
        config.set_token("new-token".to_string());

        assert_eq!(config.api.token, Some("new-token".to_string()));
    }

    #[test]
    fn test_set_token_overwrites() {
        let mut config = Config::default();
        config.api.token = Some("old-token".to_string());
        config.set_token("new-token".to_string());

        assert_eq!(config.api.token, Some("new-token".to_string()));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Serialization Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_config_serializes_to_toml() {
        let mut config = Config::default();
        config.api.token = Some("test-token".to_string());
        config.defaults.app_slug = Some("test-app".to_string());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("token = \"test-token\""));
        assert!(toml_str.contains("app_slug = \"test-app\""));
    }

    #[test]
    fn test_config_deserializes_from_toml() {
        let toml_str = r#"
[api]
token = "my-api-token"

[defaults]
app_slug = "my-app"
app_name = "My Application"

[output]
format = "json"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api.token, Some("my-api-token".to_string()));
        assert_eq!(config.defaults.app_slug, Some("my-app".to_string()));
        assert_eq!(config.defaults.app_name, Some("My Application".to_string()));
        assert_eq!(config.output.format, "json");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // File Permissions Tests (Unix only)
    // ─────────────────────────────────────────────────────────────────────────

    #[cfg(unix)]
    #[test]
    fn test_save_sets_restrictive_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let paths = make_test_paths(&temp_dir);

        let mut config = Config::default();
        config.api.token = Some("secret-token".to_string());
        config.save_to(&paths).unwrap();

        let metadata = fs::metadata(&paths.config_file).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "Config file should have 0600 permissions");
    }
}
