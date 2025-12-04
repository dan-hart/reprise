//! Trigger build command

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::bitrise::BitriseClient;
use crate::cli::args::{OutputFormat, TriggerArgs};
use crate::config::Config;
use crate::error::Result;

/// Handle the trigger command
pub fn trigger(
    client: &BitriseClient,
    config: &Config,
    args: &TriggerArgs,
    format: OutputFormat,
) -> Result<String> {
    // Get app slug from args or default
    let app_slug = args
        .app
        .as_ref()
        .or(config.defaults.app_slug.as_ref())
        .ok_or_else(|| {
            crate::error::RepriseError::Config(
                "No app specified. Use --app or set a default with 'reprise app set'".to_string(),
            )
        })?;

    // Build trigger params
    let params = crate::bitrise::TriggerParams {
        branch: args.branch.clone(),
        workflow_id: args.workflow.clone(),
        commit_message: args.message.clone(),
        environments: args.env.clone(),
    };

    // Trigger the build
    let build = client.trigger_build(app_slug, params)?;

    // Print initial status (to stderr so stdout can be piped)
    if format == OutputFormat::Pretty {
        eprintln!(
            "{} Build #{} triggered",
            "✓".green(),
            build.build_number.to_string().bold()
        );
        eprintln!("  Slug:     {}", build.slug.dimmed());
        eprintln!("  Branch:   {}", build.branch);
        eprintln!("  Workflow: {}", build.triggered_workflow);
        eprintln!(
            "\nView at: https://app.bitrise.io/build/{}",
            build.slug
        );
    }

    // Wait for build to complete if requested
    if args.wait {
        return wait_for_build(client, app_slug, &build.slug, args.interval, args.notify, format);
    }

    match format {
        OutputFormat::Pretty => Ok(String::new()), // Already printed above
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&build)?;
            Ok(json)
        }
    }
}

/// Wait for a build to complete
fn wait_for_build(
    client: &BitriseClient,
    app_slug: &str,
    build_slug: &str,
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
        eprintln!("\n{} Waiting for build to complete (Ctrl+C to stop)...", "->".cyan());
    }

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            if format == OutputFormat::Pretty {
                eprintln!("\n{} Interrupted - build continues in background", "!".yellow());
                eprintln!("  View at: https://app.bitrise.io/build/{}", build_slug);
            }
            return Ok(String::new());
        }

        thread::sleep(Duration::from_secs(interval_secs));

        let build = client.get_build(app_slug, build_slug)?;

        if !build.data.is_running() {
            // Build finished
            if send_notification {
                crate::notify::build_completed(&build.data, None);
            }

            return match format {
                OutputFormat::Pretty => {
                    let status_msg = match build.data.status {
                        1 => format!("\n{} Build completed successfully!", "✓".green()),
                        2 => format!("\n{} Build failed", "✗".red()),
                        3 => format!("\n{} Build aborted", "!".yellow()),
                        _ => format!("\n{} Build finished", "->".cyan()),
                    };

                    let mut output = status_msg;
                    output.push_str(&format!("\n  Duration: {}", build.data.duration_display()));

                    if let Some(ref reason) = build.data.abort_reason {
                        output.push_str(&format!("\n  Reason:   {}", reason));
                    }

                    Ok(output)
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&build.data)?;
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
