//! App list caching

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::bitrise::App;
use crate::cache::CacheEntryStatus;
use crate::error::Result;

/// Default cache TTL: 5 minutes
const DEFAULT_TTL_SECS: u64 = 300;

/// Cached app list with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CachedApps {
    /// When the cache was created
    cached_at: u64,
    /// The cached app list
    apps: Vec<App>,
}

/// App list cache manager
pub struct AppCache {
    cache_file: PathBuf,
    ttl: Duration,
}

impl AppCache {
    /// Create a new app cache
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            cache_file: cache_dir.join("apps.json"),
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        }
    }

    /// Create with custom TTL
    pub fn with_ttl(cache_dir: &Path, ttl_secs: u64) -> Self {
        Self {
            cache_file: cache_dir.join("apps.json"),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Try to get apps from cache
    pub fn get(&self) -> Option<Vec<App>> {
        let data = std::fs::read_to_string(&self.cache_file).ok()?;
        let cached: CachedApps = serde_json::from_str(&data).ok()?;

        // Check if cache is still valid
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        let age = now.saturating_sub(cached.cached_at);
        if age > self.ttl.as_secs() {
            return None;
        }

        Some(cached.apps)
    }

    /// Store apps in cache
    pub fn set(&self, apps: &[App]) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.cache_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let cached = CachedApps {
            cached_at: now,
            apps: apps.to_vec(),
        };

        let json = serde_json::to_string_pretty(&cached)?;
        std::fs::write(&self.cache_file, json)?;

        Ok(())
    }

    /// Clear the app cache
    pub fn clear(&self) -> Result<()> {
        if self.cache_file.exists() {
            std::fs::remove_file(&self.cache_file)?;
        }
        Ok(())
    }

    /// Get cache status
    pub fn status(&self) -> CacheEntryStatus {
        if !self.cache_file.exists() {
            return CacheEntryStatus {
                exists: false,
                age_secs: None,
                count: None,
            };
        }

        let data = match std::fs::read_to_string(&self.cache_file) {
            Ok(d) => d,
            Err(_) => {
                return CacheEntryStatus {
                    exists: true,
                    age_secs: None,
                    count: None,
                }
            }
        };

        let cached: CachedApps = match serde_json::from_str(&data) {
            Ok(c) => c,
            Err(_) => {
                return CacheEntryStatus {
                    exists: true,
                    age_secs: None,
                    count: None,
                }
            }
        };

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let age = now.saturating_sub(cached.cached_at);

        CacheEntryStatus {
            exists: true,
            age_secs: Some(age),
            count: Some(cached.apps.len()),
        }
    }
}
