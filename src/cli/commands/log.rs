use std::fs;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

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
    let app_slug = args
        .app
        .as_deref()
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    // Handle follow mode
    if args.follow {
        return follow_log(client, app_slug, &args.slug, args.interval, args.notify, format);
    }

    // Fetch the full log
    let log_content = client.get_full_log(app_slug, &args.slug)?;

    if log_content.is_empty() {
        return Err(RepriseError::LogNotAvailable(
            "Log content is empty or not yet available.".to_string(),
        ));
    }

    // Apply --tail if specified
    let output = if let Some(tail_lines) = args.tail {
        let lines: Vec<&str> = log_content.lines().collect();
        let start = lines.len().saturating_sub(tail_lines);
        lines[start..].join("\n")
    } else {
        log_content.clone()
    };

    // Save to file if --save specified
    if let Some(ref path) = args.save {
        fs::write(path, &log_content)?;
        if format == OutputFormat::Pretty {
            eprintln!("Log saved to: {}", path);
        }
    }

    // Return appropriate output
    match format {
        OutputFormat::Pretty => Ok(highlight_log_content(&output)),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "build_slug": args.slug,
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
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .ok(); // Ignore error if handler already set

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

        // Get new lines since last fetch (use get() to prevent panic if log shrinks)
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
