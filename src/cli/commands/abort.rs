//! Abort build command

use colored::Colorize;

use crate::bitrise::BitriseClient;
use crate::cli::args::{AbortArgs, OutputFormat};
use crate::config::Config;
use crate::error::Result;

/// Handle the abort command
pub fn abort(
    client: &BitriseClient,
    config: &Config,
    args: &AbortArgs,
    format: OutputFormat,
) -> Result<String> {
    // Get app slug from args or default
    let app_slug = args
        .app
        .as_ref()
        .map(|s| s.as_str())
        .or(config.defaults.app_slug.as_deref())
        .ok_or_else(|| {
            crate::error::RepriseError::Config(
                "No app specified. Use --app or set a default with 'reprise app set'".to_string(),
            )
        })?;

    // Get the build first to show info
    let build = client.get_build(app_slug, &args.slug)?;

    // Check if build is running
    if !build.data.is_running() {
        return match format {
            OutputFormat::Pretty => Ok(format!(
                "{} Build #{} is not running (status: {})",
                "!".yellow(),
                build.data.build_number,
                build.data.status_text
            )),
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "error": "Build is not running",
                    "build_number": build.data.build_number,
                    "status": build.data.status_text,
                });
                Ok(serde_json::to_string_pretty(&json)?)
            }
        };
    }

    // Abort the build
    client.abort_build(app_slug, &args.slug, args.reason.as_deref())?;

    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!(
                "{} Build #{} aborted\n",
                "✓".green(),
                build.data.build_number.to_string().bold()
            ));
            output.push_str(&format!("  Workflow: {}\n", build.data.triggered_workflow));
            output.push_str(&format!("  Branch:   {}\n", build.data.branch));
            if let Some(ref reason) = args.reason {
                output.push_str(&format!("  Reason:   {}", reason));
            }
            Ok(output.trim_end().to_string())
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "status": "aborted",
                "build_number": build.data.build_number,
                "build_slug": args.slug,
                "reason": args.reason,
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}
