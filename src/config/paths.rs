use std::fs;
use std::path::PathBuf;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::error::Result;

/// Manages paths for Reprise configuration and data
#[derive(Debug, Clone)]
pub struct Paths {
    /// Root configuration directory (~/.reprise)
    pub root: PathBuf,
    /// Configuration file path (~/.reprise/config.toml)
    pub config_file: PathBuf,
}

impl Paths {
    /// Create a new Paths instance using the user's home directory
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")?;
        let root = PathBuf::from(home).join(".reprise");

        Ok(Self {
            config_file: root.join("config.toml"),
            root,
        })
    }

    /// Ensure the configuration directory exists with proper permissions
    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;

        // Set restrictive permissions on directories (700 = owner only)
        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(&self.root, perms)?;
        }

        Ok(())
    }

    /// Check if the config file exists
    pub fn config_exists(&self) -> bool {
        self.config_file.exists()
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            root: PathBuf::from(".reprise"),
            config_file: PathBuf::from(".reprise/config.toml"),
        })
    }
}
