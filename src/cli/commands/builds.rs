use super::common::{get_github_username, matches_user, resolve_app_slug};
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

    // Convert status filter to API code
    let status = args.status.map(|s| s.to_api_code());

    // Fetch extra builds when filtering client-side to ensure we have enough results
    // Cap at 50 (API maximum)
    let fetch_limit = if me_filter.is_some() || triggered_by_filter.is_some() {
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

    // Apply triggered_by filter client-side
    let builds: Vec<_> = if let Some((ref bitrise_username, ref github_username)) = me_filter {
        // --me flag: match both Bitrise username and webhook-github/<github-username>
        response
            .data
            .into_iter()
            .filter(|b| {
                b.triggered_by
                    .as_ref()
                    .map(|t| matches_user(t, bitrise_username, github_username.as_deref()))
                    .unwrap_or(false)
            })
            .take(args.limit as usize)
            .collect()
    } else if let Some(ref user) = triggered_by_filter {
        // --triggered-by flag: case-insensitive partial match (existing behavior)
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
