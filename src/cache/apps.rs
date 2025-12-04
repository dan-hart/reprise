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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitrise::Owner;
    use tempfile::TempDir;

    /// Create a test app for caching
    fn make_test_app(slug: &str, title: &str) -> App {
        App {
            slug: slug.to_string(),
            title: title.to_string(),
            project_type: Some("ios".to_string()),
            provider: Some("github".to_string()),
            repo_owner: Some("testowner".to_string()),
            repo_slug: Some("testrepo".to_string()),
            repo_url: Some("https://github.com/test/repo".to_string()),
            is_disabled: false,
            status: 1,
            is_public: false,
            owner: Owner {
                account_type: "user".to_string(),
                name: "Test User".to_string(),
                slug: "user-slug".to_string(),
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Constructor Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_new_creates_cache_with_default_ttl() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        assert_eq!(cache.ttl, Duration::from_secs(DEFAULT_TTL_SECS));
        assert!(cache.cache_file.ends_with("apps.json"));
    }

    #[test]
    fn test_with_ttl_creates_cache_with_custom_ttl() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::with_ttl(temp_dir.path(), 60);

        assert_eq!(cache.ttl, Duration::from_secs(60));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Get/Set Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_returns_none_when_no_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        assert!(cache.get().is_none());
    }

    #[test]
    fn test_set_and_get_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![
            make_test_app("app1", "App One"),
            make_test_app("app2", "App Two"),
        ];

        cache.set(&apps).unwrap();
        let retrieved = cache.get().unwrap();

        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].slug, "app1");
        assert_eq!(retrieved[0].title, "App One");
        assert_eq!(retrieved[1].slug, "app2");
        assert_eq!(retrieved[1].title, "App Two");
    }

    #[test]
    fn test_set_creates_cache_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("nested").join("cache");
        let cache = AppCache::new(&cache_dir);

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();

        assert!(cache_dir.exists());
        assert!(cache.cache_file.exists());
    }

    #[test]
    fn test_set_empty_list() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        cache.set(&[]).unwrap();
        let retrieved = cache.get().unwrap();

        assert!(retrieved.is_empty());
    }

    #[test]
    fn test_get_returns_none_when_expired() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::with_ttl(temp_dir.path(), 1);

        // Manually write an old cache entry (timestamp from the past)
        let old_cached = serde_json::json!({
            "cached_at": 1000000000_u64,  // Year 2001 - definitely expired
            "apps": [{"slug": "app1", "title": "App One", "project_type": "ios",
                      "provider": "github", "repo_owner": "test", "repo_slug": "test",
                      "repo_url": "https://github.com/test/repo", "is_disabled": false,
                      "status": 1, "isPublic": false,
                      "owner": {"account_type": "user", "name": "Test", "slug": "test"}}]
        });
        std::fs::create_dir_all(temp_dir.path()).unwrap();
        std::fs::write(&cache.cache_file, old_cached.to_string()).unwrap();

        // Should return None because cache is expired
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_get_returns_apps_when_not_expired() {
        let temp_dir = TempDir::new().unwrap();
        // Create cache with very long TTL
        let cache = AppCache::with_ttl(temp_dir.path(), 3600);

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();

        // Should return apps because TTL is not expired
        assert!(cache.get().is_some());
    }

    #[test]
    fn test_set_overwrites_existing_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps1 = vec![make_test_app("app1", "App One")];
        cache.set(&apps1).unwrap();

        let apps2 = vec![
            make_test_app("app2", "App Two"),
            make_test_app("app3", "App Three"),
        ];
        cache.set(&apps2).unwrap();

        let retrieved = cache.get().unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].slug, "app2");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Clear Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_clear_removes_cache_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();
        assert!(cache.cache_file.exists());

        cache.clear().unwrap();
        assert!(!cache.cache_file.exists());
    }

    #[test]
    fn test_clear_when_no_cache_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        // Should not error when cache file doesn't exist
        cache.clear().unwrap();
    }

    #[test]
    fn test_get_after_clear_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();
        cache.clear().unwrap();

        assert!(cache.get().is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Status Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_status_when_no_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let status = cache.status();
        assert!(!status.exists);
        assert!(status.age_secs.is_none());
        assert!(status.count.is_none());
    }

    #[test]
    fn test_status_when_cache_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![
            make_test_app("app1", "App One"),
            make_test_app("app2", "App Two"),
            make_test_app("app3", "App Three"),
        ];
        cache.set(&apps).unwrap();

        let status = cache.status();
        assert!(status.exists);
        assert!(status.age_secs.is_some());
        assert_eq!(status.count, Some(3));
    }

    #[test]
    fn test_status_age_is_reasonable() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();

        let status = cache.status();
        // Age should be very small (just created)
        assert!(status.age_secs.unwrap() < 5);
    }

    #[test]
    fn test_status_with_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        // Write invalid JSON to cache file
        std::fs::write(&cache.cache_file, "invalid json").unwrap();

        let status = cache.status();
        assert!(status.exists);
        assert!(status.age_secs.is_none());
        assert!(status.count.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Serialization Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cache_file_is_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let apps = vec![make_test_app("app1", "App One")];
        cache.set(&apps).unwrap();

        let contents = std::fs::read_to_string(&cache.cache_file).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(parsed.get("cached_at").is_some());
        assert!(parsed.get("apps").is_some());
    }

    #[test]
    fn test_cache_preserves_app_fields() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AppCache::new(temp_dir.path());

        let mut app = make_test_app("test-slug", "Test App");
        app.project_type = Some("android".to_string());
        app.is_disabled = true;
        app.is_public = true;

        cache.set(&[app]).unwrap();
        let retrieved = cache.get().unwrap();

        assert_eq!(retrieved[0].slug, "test-slug");
        assert_eq!(retrieved[0].title, "Test App");
        assert_eq!(retrieved[0].project_type, Some("android".to_string()));
        assert!(retrieved[0].is_disabled);
        assert!(retrieved[0].is_public);
        assert_eq!(retrieved[0].owner.name, "Test User");
    }
}
