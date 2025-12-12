//! List pipelines command

use super::common::{get_github_username, matches_user, resolve_app_slug};
use crate::bitrise::BitriseClient;
use crate::cli::args::{OutputFormat, PipelinesArgs};
use crate::config::Config;
use crate::duration::parse_since;
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
    let app_slug = resolve_app_slug(args.app.as_deref(), config)?;

    // Resolve triggered_by filter (--me uses API to get current user + GitHub username)
    let me_filter: Option<(String, Option<String>)> = if args.me {
        let user = client.get_me().map_err(|e| {
            RepriseError::Config(format!(
                "Cannot determine current user for --me flag: {}. Use --triggered-by <username> instead.",
                e
            ))
        })?;
        let github_username = get_github_username();

        // Warn if GitHub username not configured (webhook-triggered builds won't match)
        if github_username.is_none() && format != OutputFormat::Json {
            eprintln!(
                "hint: GitHub username not configured. Webhook-triggered builds may not be matched.\n\
                 hint: Run: git config --global github.user YOUR_GITHUB_USERNAME\n"
            );
        }

        Some((user.data.username, github_username))
    } else {
        None
    };

    let triggered_by_filter = args.triggered_by.clone();

    // Status filter needs to be applied client-side (API doesn't support it)
    let status_filter = args.status.map(|s| s.to_api_code());

    // Fetch extra pipelines when filtering client-side to ensure we have enough results
    // Cap at 50 (API maximum)
    let needs_client_filter =
        me_filter.is_some() || triggered_by_filter.is_some() || status_filter.is_some();
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

    // Parse --since threshold if provided
    let since_threshold = args
        .since
        .as_ref()
        .map(|s| parse_since(s))
        .transpose()?;

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

            // Filter by --me flag (match both Bitrise username and webhook-github/<github-username>)
            if let Some((ref bitrise_username, ref github_username)) = me_filter {
                if !p
                    .triggered_by
                    .as_ref()
                    .map(|t| matches_user(t, bitrise_username, github_username.as_deref()))
                    .unwrap_or(false)
                {
                    return false;
                }
            }

            // Filter by --triggered-by flag (case-insensitive partial match)
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

            // Filter by --since threshold
            if let Some(threshold) = since_threshold {
                if let Some(triggered_at) = p.triggered_at {
                    if triggered_at < threshold {
                        return false;
                    }
                } else {
                    return false; // No triggered_at, exclude
                }
            }

            true
        })
        .take(args.limit as usize)
        .collect();

    output::format_pipelines(&pipelines, format)
}
