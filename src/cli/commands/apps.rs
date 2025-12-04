use crate::bitrise::BitriseClient;
use crate::cache::AppCache;
use crate::cli::args::{AppsArgs, OutputFormat};
use crate::config::Paths;
use crate::error::Result;
use crate::output;

/// Handle the apps command
pub fn apps(
    client: &BitriseClient,
    args: &AppsArgs,
    format: OutputFormat,
    use_cache: bool,
) -> Result<String> {
    let paths = Paths::new()?;
    let cache = AppCache::new(&paths.cache_dir);

    // Try cache first (if enabled and no filter - filtered results shouldn't be cached)
    let all_apps = if use_cache && args.filter.is_none() {
        if let Some(cached) = cache.get() {
            cached
        } else {
            let response = client.list_apps(args.limit)?;
            let _ = cache.set(&response.data); // Ignore cache write errors
            response.data
        }
    } else {
        let response = client.list_apps(args.limit)?;
        // Update cache even when bypassed (unless filtered)
        if args.filter.is_none() {
            let _ = cache.set(&response.data);
        }
        response.data
    };

    // Apply filter if provided
    let apps: Vec<_> = if let Some(ref filter) = args.filter {
        let filter_lower = filter.to_lowercase();
        all_apps
            .into_iter()
            .filter(|app| app.title.to_lowercase().contains(&filter_lower))
            .collect()
    } else {
        // Apply limit
        all_apps.into_iter().take(args.limit as usize).collect()
    };

    output::format_apps(&apps, format)
}
