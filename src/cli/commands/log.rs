use std::fs;

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
        OutputFormat::Pretty => Ok(output),
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
