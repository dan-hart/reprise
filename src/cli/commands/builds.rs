use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Local;
use colored::Colorize;

use super::common::{current_git_branch, get_github_username, matches_user, resolve_app_slug};
use crate::bitrise::BitriseClient;
use crate::bitrise::Build;
use crate::cli::args::{BuildsArgs, OutputFormat};
use crate::config::Config;
use crate::duration::parse_since;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the builds command
pub fn builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<String> {
    // Watch mode: continuously refresh
    if args.watch {
        return watch_builds(client, config, args, format);
    }

    // Single fetch mode
    fetch_and_format_builds(client, config, args, format)
}

/// Watch builds continuously until interrupted
fn watch_builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<String> {
    let mut stdout = io::stdout();

    // Set up signal handler for graceful Ctrl+C handling
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .ok();

    if format == OutputFormat::Pretty {
        eprintln!(
            "{} Watching builds (Ctrl+C to stop, refreshing every {}s)...\n",
            "->".cyan(),
            args.interval
        );
    }

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            if format == OutputFormat::Pretty {
                eprintln!("\n{} Interrupted by user", "!".yellow());
            }
            break;
        }

        // Clear screen (ANSI escape code)
        if format == OutputFormat::Pretty {
            print!("\x1B[2J\x1B[1;1H");
            stdout.flush()?;
        }

        // Fetch and display builds
        match fetch_builds(client, config, args, format) {
            Ok(builds) => {
                if format == OutputFormat::Pretty {
                    println!("{}", render_build_dashboard(config, args, &builds)?);
                }

                let output = output::format_builds(&builds, format)?;
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "error".red(), e);
            }
        }

        stdout.flush()?;

        // Wait before next poll
        thread::sleep(Duration::from_secs(args.interval));
    }

    Ok(String::new())
}

/// Fetch builds (used by both single and watch modes)
fn fetch_builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<Vec<Build>> {
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
    let branch_filter = if args.current_branch {
        Some(current_git_branch()?)
    } else {
        args.branch.clone()
    };

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
        branch_filter.as_deref(),
        args.workflow.as_deref(),
        fetch_limit,
    )?;

    // Parse --since threshold if provided
    let since_threshold = args.since.as_ref().map(|s| parse_since(s)).transpose()?;

    // Apply client-side filters
    let workflow_contains_lower = args.workflow_contains.as_ref().map(|s| s.to_lowercase());

    // PR number filter
    let pr_filter = args.pr;

    let builds = if let Some((ref bitrise_username, ref github_username)) = me_filter {
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
            .filter(|b| {
                workflow_contains_lower
                    .as_ref()
                    .is_none_or(|pattern| b.triggered_workflow.to_lowercase().contains(pattern))
            })
            .filter(|b| since_threshold.is_none_or(|threshold| b.triggered_at >= threshold))
            .filter(|b| pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num)))
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
            .filter(|b| {
                workflow_contains_lower
                    .as_ref()
                    .is_none_or(|pattern| b.triggered_workflow.to_lowercase().contains(pattern))
            })
            .filter(|b| since_threshold.is_none_or(|threshold| b.triggered_at >= threshold))
            .filter(|b| pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num)))
            .take(args.limit as usize)
            .collect()
    } else {
        response
            .data
            .into_iter()
            .filter(|b| {
                workflow_contains_lower
                    .as_ref()
                    .is_none_or(|pattern| b.triggered_workflow.to_lowercase().contains(pattern))
            })
            .filter(|b| since_threshold.is_none_or(|threshold| b.triggered_at >= threshold))
            .filter(|b| pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num)))
            .take(args.limit as usize)
            .collect()
    };

    Ok(builds)
}

/// Fetch builds and format output (used by both single and watch modes)
fn fetch_and_format_builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<String> {
    let builds = fetch_builds(client, config, args, format)?;
    output::format_builds(&builds, format)
}

fn render_build_dashboard(config: &Config, args: &BuildsArgs, builds: &[Build]) -> Result<String> {
    let app_slug = resolve_app_slug(args.app.as_deref(), config)?;
    let running = builds.iter().filter(|build| build.status == 0).count();
    let success = builds.iter().filter(|build| build.status == 1).count();
    let failed = builds.iter().filter(|build| build.status == 2).count();
    let aborted = builds.iter().filter(|build| build.status == 3).count();

    let branch = if args.current_branch {
        current_git_branch().ok()
    } else {
        args.branch.clone()
    };

    let mut output = String::new();
    output.push_str("Build Dashboard\n");
    output.push_str("───────────────\n");
    output.push_str(&format!("App:      {}\n", app_slug));
    output.push_str(&format!("Updated:  {}\n", Local::now().format("%H:%M:%S")));
    output.push_str(&format!("Running:  {}\n", running));
    output.push_str(&format!("Success:  {}\n", success));
    output.push_str(&format!("Failed:   {}\n", failed));
    output.push_str(&format!("Aborted:  {}\n", aborted));
    if let Some(branch) = branch {
        output.push_str(&format!("Branch:   {}\n", branch));
    }
    if let Some(workflow) = args.workflow.as_deref() {
        output.push_str(&format!("Workflow: {}\n", workflow));
    }
    if let Some(pr) = args.pr {
        output.push_str(&format!("PR:       {}\n", pr));
    }
    output.push('\n');
    Ok(output)
}
