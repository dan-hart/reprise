use crate::bitrise::BitriseClient;
use crate::cli::args::{BuildsArgs, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the builds command
pub fn builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<String> {
    // Resolve app slug from args or config default
    let app_slug = args
        .app
        .as_deref()
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    // Resolve triggered_by filter (--me uses API to get current user)
    let triggered_by_filter = if args.me {
        let user = client.get_me().map_err(|e| {
            RepriseError::Config(format!(
                "Cannot determine current user for --me flag: {}. Use --triggered-by <username> instead.",
                e
            ))
        })?;
        Some(user.data.username)
    } else {
        args.triggered_by.clone()
    };

    // Convert status filter to API code
    let status = args.status.map(|s| s.to_api_code());

    // Fetch extra builds when filtering client-side to ensure we have enough results
    // Cap at 50 (API maximum)
    let fetch_limit = if triggered_by_filter.is_some() {
        args.limit.saturating_mul(4).min(50)
    } else {
        args.limit.min(50)
    };

    let response = client.list_builds(
        app_slug,
        status,
        args.branch.as_deref(),
        args.workflow.as_deref(),
        fetch_limit,
    )?;

    // Apply triggered_by filter client-side (case-insensitive partial match)
    let builds: Vec<_> = if let Some(ref user) = triggered_by_filter {
        let user_lower = user.to_lowercase();
        response
            .data
            .into_iter()
            .filter(|b| {
                b.triggered_by
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&user_lower))
                    .unwrap_or(false)
            })
            .take(args.limit as usize)
            .collect()
    } else {
        response.data.into_iter().take(args.limit as usize).collect()
    };

    output::format_builds(&builds, format)
}
