//! Pipeline command with subcommands

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::bitrise::{BitriseClient, Pipeline, PipelineTriggerParams};
use crate::cli::args::{OutputFormat, PipelineArgs, PipelineCommands};
use crate::config::Config;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the pipeline command
pub fn pipeline(
    client: &BitriseClient,
    config: &Config,
    args: &PipelineArgs,
    format: OutputFormat,
) -> Result<String> {
    match &args.command {
        Some(PipelineCommands::Show { id, app }) => {
            pipeline_show(client, config, id, app.as_deref(), format)
        }
        Some(PipelineCommands::Trigger {
            name,
            branch,
            app,
            env,
            wait,
            notify,
            interval,
        }) => pipeline_trigger(
            client,
            config,
            name,
            branch.as_deref(),
            app.as_deref(),
            env,
            *wait,
            *notify,
            *interval,
            format,
        ),
        Some(PipelineCommands::Abort {
            id,
            app,
            reason,
            yes,
        }) => pipeline_abort(
            client,
            config,
            id,
            app.as_deref(),
            reason.as_deref(),
            *yes,
            format,
        ),
        Some(PipelineCommands::Rebuild {
            id,
            app,
            partial,
            wait,
            notify,
            interval,
        }) => pipeline_rebuild(
            client,
            config,
            id,
            app.as_deref(),
            *partial,
            *wait,
            *notify,
            *interval,
            format,
        ),
        Some(PipelineCommands::Watch {
            id,
            app,
            interval,
            notify,
        }) => pipeline_watch(client, config, id, app.as_deref(), *interval, *notify, format),
        None => {
            // If no subcommand but ID provided, show pipeline details
            if let Some(ref id) = args.id {
                pipeline_show(client, config, id, None, format)
            } else {
                Err(RepriseError::InvalidArgument(
                    "Please provide a pipeline ID or use a subcommand (trigger, abort, rebuild, watch)".to_string(),
                ))
            }
        }
    }
}

/// Show pipeline details
fn pipeline_show(
    client: &BitriseClient,
    config: &Config,
    pipeline_id: &str,
    app: Option<&str>,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = app
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    let response = client.get_pipeline(app_slug, pipeline_id)?;
    output::format_pipeline(&response.into_pipeline(), format)
}

/// Trigger a new pipeline
#[allow(clippy::too_many_arguments)]
fn pipeline_trigger(
    client: &BitriseClient,
    config: &Config,
    name: &str,
    branch: Option<&str>,
    app: Option<&str>,
    env: &[(String, String)],
    wait: bool,
    send_notification: bool,
    interval_secs: u64,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = app
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    let params = PipelineTriggerParams {
        pipeline_id: name.to_string(),
        branch: branch.map(String::from),
        environments: env.to_vec(),
    };

    let pipeline = client.trigger_pipeline(app_slug, params)?;

    // Print initial status (to stderr so stdout can be piped)
    if format == OutputFormat::Pretty {
        eprintln!(
            "{} Pipeline triggered",
            "✓".green(),
        );
        eprintln!("  ID:       {}", pipeline.id.dimmed());
        eprintln!("  Pipeline: {}", pipeline.pipeline_id);
        let branch = pipeline.get_branch();
        if !branch.is_empty() {
            eprintln!("  Branch:   {}", branch);
        }
        eprintln!(
            "\nView at: https://app.bitrise.io/app/{}/pipelines/{}",
            app_slug, pipeline.id
        );
    }

    // Wait for pipeline to complete if requested
    if wait {
        return wait_for_pipeline(
            client,
            app_slug,
            &pipeline.id,
            interval_secs,
            send_notification,
            format,
        );
    }

    match format {
        OutputFormat::Pretty => Ok(String::new()), // Already printed above
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&pipeline)?;
            Ok(json)
        }
    }
}

/// Abort a running pipeline
fn pipeline_abort(
    client: &BitriseClient,
    config: &Config,
    pipeline_id: &str,
    app: Option<&str>,
    reason: Option<&str>,
    skip_confirmation: bool,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = app
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    // Confirm unless --yes flag is provided
    if !skip_confirmation {
        eprint!(
            "Are you sure you want to abort pipeline {}? [y/N] ",
            pipeline_id
        );
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("Aborted.".to_string());
        }
    }

    client.abort_pipeline(app_slug, pipeline_id, reason)?;

    match format {
        OutputFormat::Pretty => {
            Ok(format!(
                "{} Pipeline {} aborted",
                "✓".green(),
                pipeline_id.bold()
            ))
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "status": "aborted",
                "pipeline_id": pipeline_id,
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Rebuild a pipeline
#[allow(clippy::too_many_arguments)]
fn pipeline_rebuild(
    client: &BitriseClient,
    config: &Config,
    pipeline_id: &str,
    app: Option<&str>,
    partial: bool,
    wait: bool,
    send_notification: bool,
    interval_secs: u64,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = app
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    let pipeline = client.rebuild_pipeline(app_slug, pipeline_id, partial)?;

    // Print initial status (to stderr so stdout can be piped)
    if format == OutputFormat::Pretty {
        let rebuild_type = if partial { "partial rebuild" } else { "full rebuild" };
        eprintln!(
            "{} Pipeline {} triggered",
            "✓".green(),
            rebuild_type
        );
        eprintln!("  ID:       {}", pipeline.id.dimmed());
        eprintln!("  Pipeline: {}", pipeline.pipeline_id);
        eprintln!(
            "\nView at: https://app.bitrise.io/app/{}/pipelines/{}",
            app_slug, pipeline.id
        );
    }

    // Wait for pipeline to complete if requested
    if wait {
        return wait_for_pipeline(
            client,
            app_slug,
            &pipeline.id,
            interval_secs,
            send_notification,
            format,
        );
    }

    match format {
        OutputFormat::Pretty => Ok(String::new()), // Already printed above
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&pipeline)?;
            Ok(json)
        }
    }
}

/// Watch pipeline progress
fn pipeline_watch(
    client: &BitriseClient,
    config: &Config,
    pipeline_id: &str,
    app: Option<&str>,
    interval_secs: u64,
    send_notification: bool,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = app
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    // Initial display
    if format == OutputFormat::Pretty {
        eprintln!("{} Watching pipeline {} (Ctrl+C to stop)...", "->".cyan(), pipeline_id);
    }

    wait_for_pipeline(
        client,
        app_slug,
        pipeline_id,
        interval_secs,
        send_notification,
        format,
    )
}

/// Fetch pipeline with retry logic for transient server errors
fn get_pipeline_with_retry(
    client: &BitriseClient,
    app_slug: &str,
    pipeline_id: &str,
    max_retries: u32,
) -> Result<Pipeline> {
    let mut attempt = 0;
    loop {
        match client.get_pipeline(app_slug, pipeline_id) {
            Ok(response) => return Ok(response.into_pipeline()),
            Err(e) => {
                // Only retry on 5xx server errors
                let should_retry =
                    matches!(&e, RepriseError::Api { status, .. } if *status >= 500);

                if should_retry && attempt < max_retries {
                    attempt += 1;
                    let backoff = Duration::from_secs(1 << (attempt - 1)); // 1, 2, 4, 8, 16s
                    thread::sleep(backoff);
                    continue;
                }
                return Err(e);
            }
        }
    }
}

/// Wait for a pipeline to complete
fn wait_for_pipeline(
    client: &BitriseClient,
    app_slug: &str,
    pipeline_id: &str,
    interval_secs: u64,
    send_notification: bool,
    format: OutputFormat,
) -> Result<String> {
    // Set up signal handler for graceful Ctrl+C handling
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .ok(); // Ignore error if handler already set

    if format == OutputFormat::Pretty {
        eprintln!(
            "\n{} Waiting for pipeline to complete (Ctrl+C to stop)...",
            "->".cyan()
        );
    }

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            if format == OutputFormat::Pretty {
                eprintln!(
                    "\n{} Interrupted - pipeline continues in background",
                    "!".yellow()
                );
                eprintln!(
                    "  View at: https://app.bitrise.io/app/{}/pipelines/{}",
                    app_slug, pipeline_id
                );
            }
            return Ok(String::new());
        }

        thread::sleep(Duration::from_secs(interval_secs));

        let pipeline = get_pipeline_with_retry(client, app_slug, pipeline_id, 5)?;

        if !pipeline.is_running() {
            // Pipeline finished
            if send_notification {
                notify_pipeline_completed(&pipeline);
            }

            return match format {
                OutputFormat::Pretty => {
                    let status_msg = match pipeline.status {
                        1 => format!("\n{} Pipeline completed successfully!", "✓".green()),
                        2 => format!("\n{} Pipeline failed", "✗".red()),
                        3 => format!("\n{} Pipeline aborted", "!".yellow()),
                        _ => format!("\n{} Pipeline finished", "->".cyan()),
                    };

                    let mut output = status_msg;
                    output.push_str(&format!("\n  Duration: {}", pipeline.duration_display()));

                    if let Some(ref reason) = pipeline.abort_reason {
                        output.push_str(&format!("\n  Reason:   {}", reason));
                    }

                    // Show workflow statuses
                    if !pipeline.workflows.is_empty() {
                        output.push_str("\n\n  Workflows:");
                        for wf in &pipeline.workflows {
                            let wf_status = match wf.status {
                                1 => "✓".green(),
                                2 => "✗".red(),
                                3 => "○".dimmed(),
                                _ => "?".dimmed(),
                            };
                            output.push_str(&format!("\n    {} {}", wf_status, wf.name));
                        }
                    }

                    Ok(output)
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&pipeline)?;
                    Ok(json)
                }
            };
        }

        // Still running - show progress
        if format == OutputFormat::Pretty {
            eprint!(".");
        }
    }
}

/// Send desktop notification when pipeline completes
fn notify_pipeline_completed(pipeline: &crate::bitrise::Pipeline) {
    let (title, body) = match pipeline.status {
        1 => ("Pipeline Succeeded", format!("Pipeline {} completed successfully", pipeline.pipeline_id)),
        2 => ("Pipeline Failed", format!("Pipeline {} failed", pipeline.pipeline_id)),
        3 => ("Pipeline Aborted", format!("Pipeline {} was aborted", pipeline.pipeline_id)),
        _ => ("Pipeline Finished", format!("Pipeline {} finished", pipeline.pipeline_id)),
    };

    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(&body)
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
    {
        eprintln!("Failed to send notification: {}", e);
    }
}
