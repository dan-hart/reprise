use colored::Colorize;

use crate::bitrise::BitriseClient;
use crate::cli::args::{AppArgs, AppCommands, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Handle the app set command
pub fn app_set(
    client: &BitriseClient,
    config: &mut Config,
    args: &AppArgs,
    format: OutputFormat,
) -> Result<String> {
    // Extract the app identifier from the Set command
    let app_identifier = match &args.command {
        Some(AppCommands::Set { app }) => app.as_str(),
        _ => return Err(RepriseError::InvalidArgument("Expected app set command".into())),
    };

    // Try to get the app - first as a slug, then search by name
    let app = match client.get_app(app_identifier) {
        Ok(response) => response.data,
        Err(_) => {
            // Try to find by name
            client
                .find_app_by_name(app_identifier)?
                .ok_or_else(|| RepriseError::AppNotFound(app_identifier.to_string()))?
        }
    };

    // Update config
    config.set_default_app(app.slug.clone(), Some(app.title.clone()));
    config.save()?;

    match format {
        OutputFormat::Pretty => Ok(format!(
            "{} Default app set to: {} ({})",
            "✓".green(),
            app.title.bold(),
            app.slug
        )),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "success": true,
                "app_slug": app.slug,
                "app_name": app.title
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Show the current default app
pub fn app_show(config: &Config, format: OutputFormat) -> Result<String> {
    match (&config.defaults.app_slug, &config.defaults.app_name) {
        (Some(slug), Some(name)) => match format {
            OutputFormat::Pretty => Ok(format!(
                "{}: {} ({})",
                "Default app".bold(),
                name,
                slug.dimmed()
            )),
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "app_slug": slug,
                    "app_name": name
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
        },
        (Some(slug), None) => match format {
            OutputFormat::Pretty => Ok(format!("{}: {}", "Default app".bold(), slug)),
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "app_slug": slug,
                    "app_name": null
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
        },
        _ => match format {
            OutputFormat::Pretty => Ok(format!(
                "{} No default app set. Use '{}' to set one.",
                "!".yellow(),
                "reprise app set <slug>".cyan()
            )),
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "app_slug": null,
                    "app_name": null
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
        },
    }
}
