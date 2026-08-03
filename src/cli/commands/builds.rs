use std::io::{self, Write};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::{Local, Utc};
use colored::Colorize;

use super::common::{get_github_username, matches_user, resolve_app_slug};
use crate::bitrise::BitriseClient;
use crate::cli::args::{BuildsArgs, OutputFormat};
use crate::config::Config;
use crate::duration::parse_since;
use crate::error::{RepriseError, Result};
use crate::output;

fn workflow_averages(history: &[crate::bitrise::Build]) -> HashMap<String, chrono::Duration> {
    let mut totals: HashMap<String, (chrono::Duration, i32)> = HashMap::new();

    for build in history {
        if let Some(duration) = build.duration().filter(|duration| {
            !build.is_running() && *duration >= chrono::Duration::zero()
        }) {
            let entry = totals
                .entry(build.triggered_workflow.clone())
                .or_insert((chrono::Duration::zero(), 0));
            entry.0 += duration;
            entry.1 += 1;
        }
    }

    totals
        .into_iter()
        .map(|(workflow, (total, count))| (workflow, total / count))
        .collect()
}

fn build_timing(
    build: &crate::bitrise::Build,
    averages: &HashMap<String, chrono::Duration>,
    now: chrono::DateTime<chrono::Utc>,
) -> output::BuildTiming {
    let elapsed = build.elapsed_at(now);
    let average = averages.get(&build.triggered_workflow).copied();
    let progress_percent = elapsed.zip(average).and_then(|(elapsed, average)| {
        (average > chrono::Duration::zero()).then(|| {
            ((elapsed.num_milliseconds() as f64 / average.num_milliseconds() as f64) * 100.0)
                .min(100.0) as u64
        })
    });

    output::BuildTiming {
        elapsed,
        average,
        progress_percent,
    }
}

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
        match fetch_and_format_builds(client, config, args, format) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "error".red(), e);
            }
        }

        // Show last update time in pretty mode
        if format == OutputFormat::Pretty {
            println!(
                "\n{} Last updated: {} (refreshing every {}s)",
                "->".dimmed(),
                Local::now().format("%H:%M:%S"),
                args.interval
            );
        }

        stdout.flush()?;

        // Wait before next poll
        thread::sleep(Duration::from_secs(args.interval));
    }

    Ok(String::new())
}

/// Fetch builds and format output (used by both single and watch modes)
fn fetch_and_format_builds(
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

    let history = if args.average || args.progress {
        Some(client.list_builds(app_slug, None, None, None, 50)?.data)
    } else {
        None
    };

    // Parse --since threshold if provided
    let since_threshold = args
        .since
        .as_ref()
        .map(|s| parse_since(s))
        .transpose()?;

    // Apply client-side filters
    let workflow_contains_lower = args.workflow_contains.as_ref().map(|s| s.to_lowercase());

    // PR number filter
    let pr_filter = args.pr;

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
            .filter(|b| {
                workflow_contains_lower.as_ref().is_none_or(|pattern| {
                    b.triggered_workflow.to_lowercase().contains(pattern)
                })
            })
            .filter(|b| {
                since_threshold.is_none_or(|threshold| b.triggered_at >= threshold)
            })
            .filter(|b| {
                pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num))
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
            .filter(|b| {
                workflow_contains_lower.as_ref().is_none_or(|pattern| {
                    b.triggered_workflow.to_lowercase().contains(pattern)
                })
            })
            .filter(|b| {
                since_threshold.is_none_or(|threshold| b.triggered_at >= threshold)
            })
            .filter(|b| {
                pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num))
            })
            .take(args.limit as usize)
            .collect()
    } else {
        response.data.into_iter()
            .filter(|b| {
                workflow_contains_lower.as_ref().is_none_or(|pattern| {
                    b.triggered_workflow.to_lowercase().contains(pattern)
                })
            })
            .filter(|b| {
                since_threshold.is_none_or(|threshold| b.triggered_at >= threshold)
            })
            .filter(|b| {
                pr_filter.is_none_or(|pr_num| b.pull_request_id == Some(pr_num))
            })
            .take(args.limit as usize)
            .collect()
    };

    if args.elapsed || args.average || args.progress {
        let averages = history
            .as_deref()
            .map(workflow_averages)
            .unwrap_or_default();
        let now = Utc::now();
        let timings = builds
            .iter()
            .map(|build| build_timing(build, &averages, now))
            .collect::<Vec<_>>();
        output::format_builds_with_timing(
            &builds,
            &timings,
            output::TimingOptions {
                elapsed: args.elapsed,
                average: args.average,
                progress: args.progress,
            },
            format,
        )
    } else {
        output::format_builds(&builds, format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::bitrise::Build;
    use mockito::Server;

    fn build(workflow: &str, started_at: Option<chrono::DateTime<Utc>>, finished_at: Option<chrono::DateTime<Utc>>) -> Build {
        Build {
            slug: "build".to_string(),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_on_worker_at: started_at,
            finished_at,
            status: 1,
            status_text: "success".to_string(),
            abort_reason: None,
            branch: "main".to_string(),
            build_number: 1,
            commit_hash: None,
            commit_message: None,
            tag: None,
            triggered_workflow: workflow.to_string(),
            triggered_by: None,
            stack_identifier: None,
            machine_type_id: None,
            pull_request_id: None,
            pull_request_target_branch: None,
            credit_cost: None,
        }
    }

    fn args_with_timing(average: bool, progress: bool) -> BuildsArgs {
        BuildsArgs {
            app: Some("test-app".to_string()),
            status: None,
            branch: None,
            workflow: None,
            workflow_contains: None,
            triggered_by: None,
            me: false,
            since: None,
            pr: None,
            limit: 25,
            elapsed: false,
            average,
            progress,
            watch: false,
            interval: 10,
        }
    }

    fn list_response(builds: &[Build]) -> String {
        serde_json::json!({
            "data": builds,
            "paging": { "total_item_count": builds.len(), "page_item_limit": 50, "next": null }
        })
        .to_string()
    }

    #[test]
    fn workflow_averages_uses_completed_durations_only() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let history = vec![
            build("primary", Some(start), Some(start + chrono::Duration::seconds(60))),
            build("primary", Some(start), Some(start + chrono::Duration::seconds(120))),
            build("primary", Some(start), None),
        ];

        assert_eq!(workflow_averages(&history)["primary"].num_seconds(), 90);
    }

    #[test]
    fn build_timing_caps_running_progress_at_one_hundred_percent() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let now = start + chrono::Duration::seconds(180);
        let mut running = build("primary", Some(start), None);
        running.status = 0;
        let averages = workflow_averages(&[build(
            "primary",
            Some(start),
            Some(start + chrono::Duration::seconds(60)),
        )]);

        let timing = build_timing(&running, &averages, now);

        assert_eq!(timing.elapsed.unwrap().num_seconds(), 180);
        assert_eq!(timing.average.unwrap().num_seconds(), 60);
        assert_eq!(timing.progress_percent, Some(100));
    }

    #[test]
    fn build_timing_is_unavailable_without_worker_start() {
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 12, 3, 0).unwrap();
        let mut running = build("primary", None, None);
        running.status = 0;
        let averages = workflow_averages(&[build(
            "primary",
            Some(now),
            Some(now + chrono::Duration::seconds(60)),
        )]);

        let timing = build_timing(&running, &averages, now);

        assert_eq!(timing.elapsed, None);
        assert_eq!(timing.progress_percent, None);
    }

    #[test]
    fn progress_fetches_unfiltered_history_once() {
        let mut server = Server::new();
        let main = server
            .mock("GET", "/apps/test-app/builds?limit=25")
            .with_status(200)
            .with_body(list_response(&[build("primary", None, None)]))
            .create();
        let history = server
            .mock("GET", "/apps/test-app/builds?limit=50")
            .with_status(200)
            .with_body(list_response(&[build(
                "primary",
                Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()),
                Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 1, 0).unwrap()),
            )]))
            .create();
        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();

        fetch_and_format_builds(
            &client,
            &Config::default(),
            &args_with_timing(false, true),
            OutputFormat::Json,
        )
        .unwrap();

        main.assert();
        history.assert();
    }

    #[test]
    fn elapsed_does_not_fetch_history() {
        let mut server = Server::new();
        let main = server
            .mock("GET", "/apps/test-app/builds?limit=25")
            .with_status(200)
            .with_body(list_response(&[build("primary", None, None)]))
            .create();
        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let mut args = args_with_timing(false, false);
        args.elapsed = true;

        fetch_and_format_builds(&client, &Config::default(), &args, OutputFormat::Json).unwrap();

        main.assert();
    }
}
