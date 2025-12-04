//! Cache management commands

use colored::Colorize;

use crate::cache;
use crate::cli::args::{CacheArgs, CacheCommands, OutputFormat};
use crate::config::Paths;
use crate::error::Result;

/// Handle cache commands
pub fn handle(args: &CacheArgs, format: OutputFormat) -> Result<String> {
    let paths = Paths::new()?;

    match &args.command {
        CacheCommands::Status => status(&paths, format),
        CacheCommands::Clear => clear(&paths, format),
    }
}

fn status(paths: &Paths, format: OutputFormat) -> Result<String> {
    let status = cache::status(&paths.cache_dir);

    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!("{}\n", "Cache Status".bold()));
            output.push_str(&format!("Location: {}\n\n", paths.cache_dir.display()));

            // Apps cache
            output.push_str(&"Apps Cache:\n".dimmed().to_string());
            if status.apps.exists {
                if let Some(count) = status.apps.count {
                    output.push_str(&format!("  Entries: {}\n", count));
                }
                if let Some(age) = status.apps.age_secs {
                    let age_str = format_age(age);
                    let fresh = age < 300; // 5 minutes
                    if fresh {
                        output.push_str(&format!("  Age: {} {}\n", age_str, "(fresh)".green()));
                    } else {
                        output.push_str(&format!("  Age: {} {}\n", age_str, "(stale)".yellow()));
                    }
                }
            } else {
                output.push_str(&format!("  {}\n", "Not cached".dimmed()));
            }

            Ok(output.trim_end().to_string())
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "cache_dir": paths.cache_dir.to_string_lossy(),
                "apps": {
                    "exists": status.apps.exists,
                    "age_secs": status.apps.age_secs,
                    "count": status.apps.count,
                }
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

fn clear(paths: &Paths, format: OutputFormat) -> Result<String> {
    cache::clear_all(&paths.cache_dir)?;

    match format {
        OutputFormat::Pretty => Ok(format!("{} Cache cleared", "✓".green())),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "status": "cleared"
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

/// Format age in human-readable form
fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
