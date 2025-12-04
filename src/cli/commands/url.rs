//! URL command - parse and interact with Bitrise URLs

use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::bitrise::{parse_bitrise_url, BitriseClient, BitriseUrl};
use crate::cli::args::{OutputFormat, UrlArgs};
use crate::config::Config;
use crate::error::{RepriseError, Result};
use crate::output;

/// Handle the url command
pub fn url(
    client: &BitriseClient,
    config: &Config,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // Parse the URL
    let parsed = parse_bitrise_url(&args.url)?;

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
            handle_app_url(client, &slug, args, format)
        }
        BitriseUrl::Pipeline { app_slug, pipeline_id } => {
            handle_pipeline_url(&app_slug, &pipeline_id, args, format)
        }
    }
}

/// Handle a build URL
fn handle_build_url(
    client: &BitriseClient,
    config: &Config,
    build_slug: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // For builds, we need the app slug - try to find it
    // First check if we have a default app configured
    let app_slug = config.defaults.app_slug.as_deref();

    // Try to get the build - if we have an app slug, use it
    // Otherwise, we'll need to search through apps
    let build = if let Some(slug) = app_slug {
        match client.get_build(slug, build_slug) {
            Ok(response) => response.data,
            Err(_) => {
                // Default app doesn't have this build, search others
                find_build_in_apps(client, build_slug)?
            }
        }
    } else {
        find_build_in_apps(client, build_slug)?
    };

    // Handle watch mode
    if args.watch && build.is_running() {
        return watch_build(client, config, build_slug, args.interval, args.notify, format);
    }

    // Show build info
    let mut output = output::format_build(&build, format)?;

    // Add URL to output in pretty mode
    if format == OutputFormat::Pretty {
        output.push_str(&format!("\n{} {}\n", "URL:".dimmed(), args.url));
    }

    Ok(output)
}

/// Search for a build across all accessible apps
fn find_build_in_apps(client: &BitriseClient, build_slug: &str) -> Result<crate::bitrise::Build> {
    // List apps and try to find the build
    let apps = client.list_apps(50)?;

    for app in &apps.data {
        if let Ok(response) = client.get_build(&app.slug, build_slug) {
            return Ok(response.data);
        }
    }

    Err(RepriseError::BuildNotFound(format!(
        "Build {} not found in any accessible app. Try setting a default app with 'reprise app set'.",
        build_slug
    )))
}

/// Watch a build until it completes
fn watch_build(
    client: &BitriseClient,
    config: &Config,
    build_slug: &str,
    interval_secs: u64,
    send_notification: bool,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = config.defaults.app_slug.as_deref();
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
        let build = if let Some(slug) = app_slug {
            match client.get_build(slug, build_slug) {
                Ok(response) => response.data,
                Err(_) => find_build_in_apps(client, build_slug)?
            }
        } else {
            find_build_in_apps(client, build_slug)?
        };

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
    app_slug: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    let app = client.get_app(app_slug)?;

    let mut output = output::format_app(&app.data, format)?;

    // Add URL to output in pretty mode
    if format == OutputFormat::Pretty && !args.browser {
        output.push_str(&format!("\n{} {}\n", "URL:".dimmed(), args.url));
    }

    Ok(output)
}

/// Handle a pipeline URL
fn handle_pipeline_url(
    app_slug: &str,
    pipeline_id: &str,
    args: &UrlArgs,
    format: OutputFormat,
) -> Result<String> {
    // Bitrise API v0.1 doesn't have pipeline endpoints
    // Show what we know and provide the URL

    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!("{}\n", "Pipeline".bold()));
            output.push_str(&"─".repeat(40));
            output.push('\n');
            output.push_str(&format!("App:      {}\n", app_slug));
            output.push_str(&format!("Pipeline: {}\n", pipeline_id));
            output.push_str(&format!(
                "\n{} Pipeline API not yet supported. View in browser:\n",
                "Note:".yellow()
            ));

            if !args.browser {
                output.push_str(&format!("{} {}\n", "URL:".dimmed(), args.url));
            }

            Ok(output)
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "type": "pipeline",
                "app_slug": app_slug,
                "pipeline_id": pipeline_id,
                "url": args.url,
                "note": "Pipeline API not yet supported"
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

/// Open a URL in the default browser
fn open_url_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| RepriseError::Io(e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| RepriseError::Io(e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .map_err(|e| RepriseError::Io(e))?;
    }

    Ok(())
}
