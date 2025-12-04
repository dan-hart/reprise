use serde::{Deserialize, Serialize};
use std::fs;

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
}

/// API-related configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Bitrise API token
    pub token: Option<String>,
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
        fs::write(&paths.config_file, contents)?;
        Ok(())
    }

    /// Get the API token or return an error with instructions
    pub fn require_token(&self) -> Result<&str> {
        self.api.token.as_deref().ok_or_else(|| {
            RepriseError::config_missing(
                "API token not configured. Run 'reprise config init' to set up.",
            )
        })
    }

    /// Get the default app slug or return an error with instructions
    pub fn require_default_app(&self) -> Result<&str> {
        self.defaults.app_slug.as_deref().ok_or(RepriseError::NoDefaultApp)
    }

    /// Set the default app
    pub fn set_default_app(&mut self, slug: String, name: Option<String>) {
        self.defaults.app_slug = Some(slug);
        self.defaults.app_name = name;
    }

    /// Set the API token
    pub fn set_token(&mut self, token: String) {
        self.api.token = Some(token);
    }
}
