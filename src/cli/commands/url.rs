//! URL command - parse and interact with Bitrise URLs

use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::bitrise::{parse_bitrise_url, BitriseClient, BitriseUrl, Build};
use crate::cli::args::{OutputFormat, UrlArgs};
use crate::config::Config;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the url command
pub fn url(
    client: &BitriseClient,
    config: &mut Config,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // Parse the URL
    let parsed = parse_bitrise_url(&args.url)?;

    // Validate flags for URL type
    validate_flags_for_url_type(&parsed, args)?;

    // Open in browser if requested
    if args.browser {
        open_url_in_browser(&parsed.to_url())?;
        if format == OutputFormat::Pretty {
            return Ok(format!("{} Opened in browser: {}", "->".cyan(), parsed.to_url()));
        }
    }

    // Handle based on URL type
    match parsed {
        BitriseUrl::Build { slug } => {
            handle_build_url(client, config, &slug, args, format)
        }
        BitriseUrl::App { slug } => {
            handle_app_url(client, config, &slug, args, format)
        }
        BitriseUrl::Pipeline { app_slug, pipeline_id } => {
            handle_pipeline_url(client, &app_slug, &pipeline_id, args, format)
        }
    }
}

/// Validate that flags are appropriate for the URL type
fn validate_flags_for_url_type(parsed: &BitriseUrl, args: &UrlArgs) -> Result<()> {
    match parsed {
        BitriseUrl::Build { .. } => {
            if args.set_default {
                return Err(RepriseError::InvalidArgument(
                    "--set-default is only valid for app URLs".to_string(),
                ));
            }
        }
        BitriseUrl::App { .. } => {
            if args.logs {
                return Err(RepriseError::InvalidArgument(
                    "--logs is only valid for build URLs".to_string(),
                ));
            }
            if args.follow {
                return Err(RepriseError::InvalidArgument(
                    "--follow is only valid for build URLs".to_string(),
                ));
            }
            if args.artifacts {
                return Err(RepriseError::InvalidArgument(
                    "--artifacts is only valid for build URLs".to_string(),
                ));
            }
        }
        BitriseUrl::Pipeline { .. } => {
            if args.set_default {
                return Err(RepriseError::InvalidArgument(
                    "--set-default is only valid for app URLs".to_string(),
                ));
            }
            if args.logs {
                return Err(RepriseError::InvalidArgument(
                    "--logs is only valid for build URLs (pipelines contain multiple workflows)".to_string(),
                ));
            }
            if args.follow {
                return Err(RepriseError::InvalidArgument(
                    "--follow is only valid for build URLs (pipelines contain multiple workflows)".to_string(),
                ));
            }
            if args.artifacts {
                return Err(RepriseError::InvalidArgument(
                    "--artifacts is only valid for build URLs (pipelines contain multiple workflows)".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Handle a build URL
fn handle_build_url(
    client: &BitriseClient,
    config: &Config,
    build_slug: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // Find the build and get the app_slug it belongs to
    let (build, app_slug) = find_build_with_app(client, config, build_slug)?;

    // Handle --logs flag: dump the full build log
    if args.logs {
        return dump_build_log(client, &app_slug, build_slug, format);
    }

    // Handle --follow flag: stream live log output
    if args.follow {
        return follow_build_log(client, &app_slug, build_slug, args.interval, args.notify, format);
    }

    // Handle --artifacts flag: list build artifacts
    if args.artifacts {
        return list_build_artifacts(client, &app_slug, build_slug, format);
    }

    // Handle watch mode
    if args.watch && build.is_running() {
        return watch_build_with_app(client, &app_slug, build_slug, args.interval, args.notify, format);
    }

    // Show build info
    let mut output = output::format_build(&build, format)?;

    // Add URL to output in pretty mode
    if format == OutputFormat::Pretty {
        output.push_str(&format!("\n{} {}\n", "URL:".dimmed(), args.url));
    }

    Ok(output)
}

/// Find a build and return both the build and its app_slug
fn find_build_with_app(
    client: &BitriseClient,
    config: &Config,
    build_slug: &str,
) -> Result<(Build, String)> {
    // First try the default app if configured
    if let Some(app_slug) = config.defaults.app_slug.as_deref() {
        if let Ok(response) = client.get_build(app_slug, build_slug) {
            return Ok((response.data, app_slug.to_string()));
        }
    }

    // Search through all accessible apps
    let apps = client.list_apps(50)?;
    for app in &apps.data {
        if let Ok(response) = client.get_build(&app.slug, build_slug) {
            return Ok((response.data, app.slug.clone()));
        }
    }

    Err(RepriseError::BuildNotFound(format!(
        "Build {} not found in any accessible app. Try setting a default app with 'reprise app set'.",
        build_slug
    )))
}

/// Dump the full build log
fn dump_build_log(
    client: &BitriseClient,
    app_slug: &str,
    build_slug: &str,
    format: OutputFormat,
) -> Result<String> {
    let log_content = client.get_full_log(app_slug, build_slug)?;

    if log_content.is_empty() {
        return Err(RepriseError::LogNotAvailable(
            "Log content is empty or not yet available.".to_string(),
        ));
    }

    match format {
        OutputFormat::Pretty => Ok(highlight_log_content(&log_content)),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "build_slug": build_slug,
                "app_slug": app_slug,
                "log": log_content,
                "lines": log_content.lines().count()
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Stream live log output for a running build
fn follow_build_log(
    client: &BitriseClient,
    app_slug: &str,
    build_slug: &str,
    interval_secs: u64,
    send_notification: bool,
    format: OutputFormat,
) -> Result<String> {
    let mut last_line_count = 0;
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
            "{} Following build log (Ctrl+C to stop)...\n",
            "->".cyan()
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

        // Get build status to check if still running
        let build = client.get_build(app_slug, build_slug)?;

        // Try to get log content
        let log_content = match client.get_full_log(app_slug, build_slug) {
            Ok(content) => content,
            Err(_) => {
                // Log may not be available yet
                if build.data.is_running() {
                    thread::sleep(Duration::from_secs(interval_secs));
                    continue;
                }
                return Err(RepriseError::LogNotAvailable(
                    "Log content is not available.".to_string(),
                ));
            }
        };

        // Get new lines since last fetch
        let lines: Vec<&str> = log_content.lines().collect();
        let new_lines = lines.get(last_line_count..).unwrap_or_default();

        // Print new lines
        if !new_lines.is_empty() {
            for line in new_lines {
                match format {
                    OutputFormat::Pretty => {
                        writeln!(stdout, "{}", highlight_log_line(line))?;
                    }
                    OutputFormat::Json => {
                        let json = serde_json::json!({ "line": line });
                        writeln!(stdout, "{}", serde_json::to_string(&json)?)?;
                    }
                }
            }
            stdout.flush()?;
            last_line_count = lines.len();
        }

        // Check if build is done
        if !build.data.is_running() {
            if format == OutputFormat::Pretty {
                let status_msg = match build.data.status {
                    1 => format!("\n{} Build completed successfully", "✓".green()),
                    2 => format!("\n{} Build failed", "✗".red()),
                    3 => format!("\n{} Build aborted", "!".yellow()),
                    _ => format!("\n{} Build finished", "->".cyan()),
                };
                eprintln!("{}", status_msg);
            }

            // Send desktop notification if requested
            if send_notification {
                crate::notify::build_completed(&build.data, None);
            }

            break;
        }

        // Wait before next poll
        thread::sleep(Duration::from_secs(interval_secs));
    }

    Ok(String::new())
}

/// List build artifacts
fn list_build_artifacts(
    client: &BitriseClient,
    app_slug: &str,
    build_slug: &str,
    format: OutputFormat,
) -> Result<String> {
    let response = client.list_artifacts(app_slug, build_slug)?;

    if response.data.is_empty() {
        return match format {
            OutputFormat::Pretty => Ok(format!("{} No artifacts found for this build.", "!".yellow())),
            OutputFormat::Json => Ok(serde_json::to_string_pretty(&response.data)?),
        };
    }

    output::format_artifacts(&response.data, format)
}

/// Highlight a single log line based on content
fn highlight_log_line(line: &str) -> String {
    let line_lower = line.to_lowercase();

    // Error patterns (red)
    if line_lower.contains("error")
        || line_lower.contains("failed")
        || line_lower.contains("failure")
        || line_lower.contains("fatal")
        || line_lower.contains("exception")
        || line_lower.contains("panic")
        || line.starts_with("E ")
        || line.contains("[ERROR]")
        || line.contains("[error]")
    {
        return line.red().to_string();
    }

    // Warning patterns (yellow)
    if line_lower.contains("warning")
        || line_lower.contains("warn")
        || line.starts_with("W ")
        || line.contains("[WARN]")
        || line.contains("[warn]")
    {
        return line.yellow().to_string();
    }

    // Success patterns (green)
    if line_lower.contains("success")
        || line_lower.contains("passed")
        || line_lower.contains("completed")
        || line.contains("[OK]")
        || line.contains("BUILD SUCCESSFUL")
    {
        return line.green().to_string();
    }

    line.to_string()
}

/// Apply highlighting to full log content
fn highlight_log_content(content: &str) -> String {
    content
        .lines()
        .map(highlight_log_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Watch a build until it completes (with known app_slug)
fn watch_build_with_app(
    client: &BitriseClient,
    app_slug: &str,
    build_slug: &str,
    interval_secs: u64,
    send_notification: bool,
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
            "{} Watching build (Ctrl+C to stop)...\n",
            "->".cyan()
        );
    }

    let mut last_status = -1;

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            if format == OutputFormat::Pretty {
                eprintln!("\n{} Interrupted by user", "!".yellow());
            }
            break;
        }

        // Get build status
        let build = client.get_build(app_slug, build_slug)?.data;

        // Print status update if changed
        if build.status != last_status {
            let status_str = match build.status {
                0 => "RUNNING".yellow().bold(),
                1 => "SUCCESS".green().bold(),
                2 => "FAILED".red().bold(),
                3 => "ABORTED".red(),
                _ => "UNKNOWN".dimmed(),
            };

            match format {
                OutputFormat::Pretty => {
                    writeln!(
                        stdout,
                        "{} Build #{} - {} ({})",
                        "->".cyan(),
                        build.build_number,
                        status_str,
                        build.duration_display()
                    )?;
                }
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "build_number": build.build_number,
                        "status": build.status_text,
                        "duration": build.duration_display()
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&json)?)?;
                }
            }
            stdout.flush()?;
            last_status = build.status;
        }

        // Check if build is done
        if !build.is_running() {
            if format == OutputFormat::Pretty {
                let final_msg = match build.status {
                    1 => format!("\n{} Build completed successfully!", "✓".green()),
                    2 => format!("\n{} Build failed", "✗".red()),
                    3 => format!("\n{} Build aborted", "!".yellow()),
                    _ => format!("\n{} Build finished", "->".cyan()),
                };
                eprintln!("{}", final_msg);

                // Show build URL
                eprintln!(
                    "\n{} https://app.bitrise.io/build/{}",
                    "View:".dimmed(),
                    build.slug
                );
            }

            // Send desktop notification if requested
            if send_notification {
                crate::notify::build_completed(&build, None);
            }

            break;
        }

        // Wait before next poll
        thread::sleep(Duration::from_secs(interval_secs));
    }

    Ok(String::new())
}

/// Handle an app URL
fn handle_app_url(
    client: &BitriseClient,
    config: &mut Config,
    app_slug: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    let app = client.get_app(app_slug)?;

    // Handle --set-default flag
    if args.set_default {
        config.set_default_app(app.data.slug.clone(), Some(app.data.title.clone()));
        config.save()?;

        if format == OutputFormat::Pretty {
            let mut output = format!(
                "{} Default app set to: {} ({})\n\n",
                "->".green(),
                app.data.title.bold(),
                app.data.slug.dimmed()
            );
            output.push_str(&output::format_app(&app.data, format)?);
            return Ok(output);
        }
    }

    let mut output = output::format_app(&app.data, format)?;

    // Add URL to output in pretty mode
    if format == OutputFormat::Pretty && !args.browser {
        output.push_str(&format!("\n{} {}\n", "URL:".dimmed(), args.url));
    }

    Ok(output)
}

/// Handle a pipeline URL
fn handle_pipeline_url(
    client: &BitriseClient,
    app_slug: &str,
    pipeline_id: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // Get pipeline details
    let response = client.get_pipeline(app_slug, pipeline_id)?;
    let pipeline = response.into_pipeline();

    // Handle watch mode
    if args.watch && pipeline.is_running() {
        return watch_pipeline(client, app_slug, pipeline_id, args.interval, args.notify, format);
    }

    // Show pipeline info
    let mut output = output::format_pipeline(&pipeline, format)?;

    // Add URL to output in pretty mode
    if format == OutputFormat::Pretty && !args.browser {
        output.push_str(&format!("\n{} {}\n", "URL:".dimmed(), args.url));
    }

    Ok(output)
}

/// Watch a pipeline until it completes
fn watch_pipeline(
    client: &BitriseClient,
    app_slug: &str,
    pipeline_id: &str,
    interval_secs: u64,
    send_notification: bool,
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
            "{} Watching pipeline (Ctrl+C to stop)...\n",
            "->".cyan()
        );
    }

    let mut last_status = -1;

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            if format == OutputFormat::Pretty {
                eprintln!("\n{} Interrupted by user", "!".yellow());
            }
            break;
        }

        // Get pipeline status
        let response = client.get_pipeline(app_slug, pipeline_id)?;
        let pipeline = response.into_pipeline();

        // Print status update if changed
        if pipeline.status != last_status {
            let status_str = match pipeline.status {
                0 => "RUNNING".yellow().bold(),
                1 => "SUCCESS".green().bold(),
                2 => "FAILED".red().bold(),
                3 => "ABORTED".red(),
                _ => "UNKNOWN".dimmed(),
            };

            match format {
                OutputFormat::Pretty => {
                    writeln!(
                        stdout,
                        "{} Pipeline {} - {} ({})",
                        "->".cyan(),
                        pipeline.pipeline_id,
                        status_str,
                        pipeline.duration_display()
                    )?;

                    // Show workflow statuses
                    for wf in &pipeline.workflows {
                        let wf_status = match wf.status {
                            0 => "●".yellow(),
                            1 => "✓".green(),
                            2 => "✗".red(),
                            3 => "○".dimmed(),
                            _ => "?".dimmed(),
                        };
                        writeln!(stdout, "   {} {}", wf_status, wf.name)?;
                    }
                }
                OutputFormat::Json => {
                    let json = serde_json::json!({
                        "pipeline_id": pipeline.pipeline_id,
                        "status": pipeline.status_text,
                        "duration": pipeline.duration_display(),
                        "workflows": pipeline.workflows.iter().map(|wf| {
                            serde_json::json!({
                                "name": wf.name,
                                "status": wf.status_text
                            })
                        }).collect::<Vec<_>>()
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&json)?)?;
                }
            }
            stdout.flush()?;
            last_status = pipeline.status;
        }

        // Check if pipeline is done
        if !pipeline.is_running() {
            if format == OutputFormat::Pretty {
                let final_msg = match pipeline.status {
                    1 => format!("\n{} Pipeline completed successfully!", "✓".green()),
                    2 => format!("\n{} Pipeline failed", "✗".red()),
                    3 => format!("\n{} Pipeline aborted", "!".yellow()),
                    _ => format!("\n{} Pipeline finished", "->".cyan()),
                };
                eprintln!("{}", final_msg);

                // Show pipeline URL
                eprintln!(
                    "\n{} https://app.bitrise.io/app/{}/pipelines/{}",
                    "View:".dimmed(),
                    app_slug,
                    pipeline_id
                );
            }

            // Send desktop notification if requested
            if send_notification {
                notify_pipeline_completed(&pipeline);
            }

            break;
        }

        // Wait before next poll
        thread::sleep(Duration::from_secs(interval_secs));
    }

    Ok(String::new())
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

/// Open a URL in the default browser
fn open_url_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(RepriseError::Io)?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(RepriseError::Io)?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .map_err(RepriseError::Io)?;
    }

    Ok(())
}
