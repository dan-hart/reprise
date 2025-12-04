//! List pipelines command

use crate::bitrise::BitriseClient;
use crate::cli::args::{OutputFormat, PipelinesArgs};
use crate::config::Config;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the pipelines command
pub fn pipelines(
    client: &BitriseClient,
    config: &Config,
    args: &PipelinesArgs,
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

    // Status filter needs to be applied client-side (API doesn't support it)
    let status_filter = args.status.map(|s| s.to_api_code());

    // Fetch extra pipelines when filtering client-side to ensure we have enough results
    // Cap at 50 (API maximum)
    let needs_client_filter = triggered_by_filter.is_some() || status_filter.is_some();
    let fetch_limit = if needs_client_filter {
        args.limit.saturating_mul(4).min(50)
    } else {
        args.limit.min(50)
    };

    let response = client.list_pipelines(
        app_slug,
        None, // Status filtering not supported by API, filter client-side
        args.branch.as_deref(),
        fetch_limit,
    )?;

    // Apply filters client-side
    let pipelines: Vec<_> = response
        .data
        .into_iter()
        .filter(|p| {
            // Filter by status if specified
            if let Some(status) = status_filter {
                if p.status != status {
                    return false;
                }
            }
            // Filter by triggered_by if specified (case-insensitive partial match)
            if let Some(ref user) = triggered_by_filter {
                let user_lower = user.to_lowercase();
                if !p
                    .triggered_by
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&user_lower))
                    .unwrap_or(false)
                {
                    return false;
                }
            }
            true
        })
        .take(args.limit as usize)
        .collect();

    output::format_pipelines(&pipelines, format)
}
