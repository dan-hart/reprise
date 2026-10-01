use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::error::{RepriseError, Result};

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
        let home = Self::home_dir()?;
        let root = home.join(".reprise");

        Ok(Self {
            config_file: root.join("config.toml"),
            root,
        })
    }

    pub fn home_dir() -> Result<PathBuf> {
        #[cfg(windows)]
        let user_profile = std::env::var_os("USERPROFILE");
        #[cfg(not(windows))]
        let user_profile = None;
        resolve_home(std::env::var_os("HOME"), user_profile)
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

fn resolve_home(home: Option<OsString>, user_profile: Option<OsString>) -> Result<PathBuf> {
    home.filter(|value| !value.is_empty())
        .or_else(|| user_profile.filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .ok_or_else(|| {
            RepriseError::Config(
                "Set HOME or USERPROFILE on Windows to locate your configuration.".into(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_override_takes_precedence() {
        assert_eq!(
            resolve_home(Some("override".into()), Some("native".into())).unwrap(),
            PathBuf::from("override")
        );
    }

    #[test]
    fn windows_native_home_without_home_variable() {
        assert_eq!(
            resolve_home(None, Some("native".into())).unwrap(),
            PathBuf::from("native")
        );
        assert_eq!(
            resolve_home(Some("".into()), Some("native".into())).unwrap(),
            PathBuf::from("native")
        );
    }

    #[test]
    fn missing_home_reports_configuration_error() {
        assert!(resolve_home(None, None).is_err());
    }
}
