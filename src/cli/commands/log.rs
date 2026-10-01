use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use colored::Colorize;

use super::common::{
    is_interrupted, resolve_app_slug, resolve_build_slug, setup_interrupt_handler,
};
use crate::bitrise::BitriseClient;
use crate::cli::args::{LogArgs, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Handle the log command
pub fn log(
    client: &BitriseClient,
    config: &Config,
    args: &LogArgs,
    format: OutputFormat,
) -> Result<String> {
    // Resolve app slug from args or config default
    let app_slug = resolve_app_slug(args.app.as_deref(), config)?;
    let build_slug = resolve_build_slug(
        client,
        app_slug,
        args.slug.as_deref(),
        args.latest,
        args.branch.as_deref(),
        args.workflow.as_deref(),
        args.status,
        args.pr,
        args.current_branch,
        format,
    )?;

    // Handle follow mode
    if args.follow {
        return follow_log(
            client,
            app_slug,
            &build_slug,
            args.interval,
            args.notify,
            format,
        );
    }

    let output = if let Some(path) = &args.save {
        let path = std::path::Path::new(path);
        if let Some(tail) = args.tail {
            client.save_log_tail(app_slug, &build_slug, path, tail)?
        } else {
            client.save_log(app_slug, &build_slug, path)?;
            fs::read_to_string(path)?
        }
    } else {
        let content = client.get_full_log(app_slug, &build_slug)?;
        if content.is_empty() {
            return Err(RepriseError::LogNotAvailable(
                "Log content is empty or not yet available.".into(),
            ));
        }
        if let Some(tail) = args.tail {
            content
                .lines()
                .rev()
                .take(tail)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            content
        }
    };
    if let Some(path) = &args.save {
        if format == OutputFormat::Pretty {
            eprintln!("Log saved to: {}", path);
        }
    }

    // Return appropriate output
    match format {
        OutputFormat::Pretty => Ok(highlight_log_content(&output)),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "build_slug": build_slug,
                "log": output,
                "lines": output.lines().count()
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Follow log output for a running build
fn follow_log(
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
    let interrupted = setup_interrupt_handler();

    if format == OutputFormat::Pretty {
        eprintln!("{} Following build log (Ctrl+C to stop)...\n", "->".cyan());
    }

    loop {
        // Check for interrupt
        if is_interrupted(&interrupted) {
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

        let line_count = log_content.lines().count();
        if line_count < last_line_count {
            last_line_count = 0;
        }
        for line in log_content.lines().skip(last_line_count) {
            match format {
                OutputFormat::Pretty => writeln!(stdout, "{}", highlight_log_line(line))?,
                OutputFormat::Json => writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&serde_json::json!({ "line": line }))?
                )?,
            }
        }
        stdout.flush()?;
        last_line_count = line_count;

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

    // Return empty string since we've already printed everything
    Ok(String::new())
}

/// Highlight log lines based on content
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
