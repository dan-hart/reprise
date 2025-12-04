//! Cache module for reducing API calls
//!
//! Provides local caching for frequently accessed data like app lists.

mod apps;

pub use apps::AppCache;

use std::path::Path;

use crate::error::Result;

/// Clear all cached data
pub fn clear_all(cache_dir: &Path) -> Result<()> {
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir)?;
        std::fs::create_dir_all(cache_dir)?;
    }
    Ok(())
}

/// Get cache status information
pub fn status(cache_dir: &Path) -> CacheStatus {
    let apps = AppCache::new(cache_dir).status();

    CacheStatus { apps }
}

/// Overall cache status
#[derive(Debug)]
pub struct CacheStatus {
    pub apps: CacheEntryStatus,
}

/// Status of a single cache entry
#[derive(Debug)]
pub struct CacheEntryStatus {
    pub exists: bool,
    pub age_secs: Option<u64>,
    pub count: Option<usize>,
}
