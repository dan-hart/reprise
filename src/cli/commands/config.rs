use std::io::{self, Write};

use colored::Colorize;
use rpassword::read_password;

use crate::cli::args::{ConfigArgs, ConfigCommands, OutputFormat};
use crate::config::{Config, Paths};
use crate::error::{RepriseError, Result};

/// Safely truncate a string to show first and last n characters
/// Works correctly with multi-byte UTF-8 characters
fn mask_token(token: &str, visible_chars: usize) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() > visible_chars * 2 {
        let start: String = chars.iter().take(visible_chars).collect();
        let end: String = chars.iter().rev().take(visible_chars).rev().collect();
        format!("{}...{}", start, end)
    } else {
        "****".to_string()
    }
}

/// Handle the config command
pub fn config(
    config: &mut Config,
    args: &ConfigArgs,
    format: OutputFormat,
) -> Result<String> {
    match &args.command {
        ConfigCommands::Show => config_show(config, format),
        ConfigCommands::Set { key, value } => config_set(config, key, value, format),
        ConfigCommands::Path => config_path(format),
        ConfigCommands::Init => config_init(config, format),
        ConfigCommands::Alias { name, slug, remove } => {
            config_alias(config, name.as_deref(), slug.as_deref(), *remove, format)
        }
    }
}

/// Show current configuration
fn config_show(config: &Config, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!("{}\n", "Configuration".bold()));
            output.push_str(&"─".repeat(40));
            output.push('\n');

            // API section
            output.push_str(&format!("\n{}\n", "[api]".cyan()));
            let token_display = config
                .api
                .token
                .as_ref()
                .map(|t| mask_token(t, 4))
                .unwrap_or_else(|| "(not set)".dimmed().to_string());
            output.push_str(&format!("  token = {}\n", token_display));

            // Defaults section
            output.push_str(&format!("\n{}\n", "[defaults]".cyan()));
            output.push_str(&format!(
                "  app_slug = {}\n",
                config
                    .defaults
                    .app_slug
                    .as_deref()
                    .unwrap_or("(not set)")
            ));
            output.push_str(&format!(
                "  app_name = {}\n",
                config
                    .defaults
                    .app_name
                    .as_deref()
                    .unwrap_or("(not set)")
            ));

            // Output section
            output.push_str(&format!("\n{}\n", "[output]".cyan()));
            output.push_str(&format!("  format = {}\n", config.output.format));

            // Aliases section (if any exist)
            if !config.aliases.is_empty() {
                output.push_str(&format!("\n{}\n", "[aliases]".cyan()));
                let mut aliases: Vec<_> = config.aliases.iter().collect();
                aliases.sort_by_key(|(k, _)| *k);
                for (name, slug) in aliases {
                    output.push_str(&format!("  {} = {}\n", name, slug.dimmed()));
                }
            }

            Ok(output)
        }
        OutputFormat::Json => {
            // Don't expose the full token in JSON output either
            let mut safe_config = config.clone();
            if let Some(ref token) = safe_config.api.token {
                safe_config.api.token = Some(mask_token(token, 4));
            }
            Ok(serde_json::to_string_pretty(&safe_config)?)
        }
    }
}

/// Set a configuration value
fn config_set(config: &mut Config, key: &str, value: &str, format: OutputFormat) -> Result<String> {
    match key {
        "api.token" => {
            config.set_token(value.to_string());
            config.save()?;
        }
        "defaults.app_slug" => {
            config.defaults.app_slug = Some(value.to_string());
            config.save()?;
        }
        "defaults.app_name" => {
            config.defaults.app_name = Some(value.to_string());
            config.save()?;
        }
        "output.format" => {
            if value != "pretty" && value != "json" {
                return Err(RepriseError::InvalidArgument(
                    "output.format must be 'pretty' or 'json'".to_string(),
                ));
            }
            config.output.format = value.to_string();
            config.save()?;
        }
        _ => {
            return Err(RepriseError::InvalidArgument(format!(
                "Unknown config key: {}. Valid keys: api.token, defaults.app_slug, defaults.app_name, output.format",
                key
            )));
        }
    }

    match format {
        OutputFormat::Pretty => Ok(format!("{} Set {} = {}", "✓".green(), key, value)),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "success": true,
                "key": key,
                "value": value
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Show configuration file path
fn config_path(format: OutputFormat) -> Result<String> {
    let paths = Paths::new()?;

    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!("Config file: {}\n", paths.config_file.display()));
            output.push_str(&format!(
                "Exists: {}\n",
                if paths.config_exists() {
                    "yes".green()
                } else {
                    "no".yellow()
                }
            ));
            Ok(output)
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "path": paths.config_file.display().to_string(),
                "exists": paths.config_exists()
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

/// Initialize configuration interactively
fn config_init(config: &mut Config, format: OutputFormat) -> Result<String> {
    if format == OutputFormat::Json {
        return Err(RepriseError::InvalidArgument(
            "config init requires interactive mode (--output pretty)".to_string(),
        ));
    }

    println!("{}", "Reprise Configuration".bold());
    println!("{}", "─".repeat(40));
    println!();

    // Prompt for API token with hidden input (secure)
    print!("Enter your Bitrise API token: ");
    io::stdout().flush()?;

    let token = read_password().map_err(|e| {
        RepriseError::Io(io::Error::other(e.to_string()))
    })?;
    let token = token.trim().to_string();
    println!(); // Add newline since read_password doesn't

    if token.is_empty() {
        return Err(RepriseError::InvalidArgument(
            "API token cannot be empty".to_string(),
        ));
    }

    config.set_token(token);
    config.save()?;

    let paths = Paths::new()?;

    Ok(format!(
        "\n{} Configuration saved to: {}\n\nRun '{}' to see your apps.",
        "✓".green(),
        paths.config_file.display(),
        "reprise apps".cyan()
    ))
}

/// Handle alias operations: list, show, set, or remove
fn config_alias(
    config: &mut Config,
    name: Option<&str>,
    slug: Option<&str>,
    remove: bool,
    format: OutputFormat,
) -> Result<String> {
    match (name, slug, remove) {
        // List all aliases
        (None, None, false) => {
            if config.aliases.is_empty() {
                return match format {
                    OutputFormat::Pretty => Ok("No aliases configured.\n\nSet one with: reprise config alias <name> <slug>".dimmed().to_string()),
                    OutputFormat::Json => Ok(serde_json::to_string_pretty(&config.aliases)?),
                };
            }

            match format {
                OutputFormat::Pretty => {
                    let mut output = String::new();
                    output.push_str(&format!("{}\n", "App Aliases".bold()));
                    output.push_str(&"─".repeat(50));
                    output.push('\n');

                    let mut aliases: Vec<_> = config.aliases.iter().collect();
                    aliases.sort_by_key(|(k, _)| *k);
                    for (alias_name, alias_slug) in aliases {
                        output.push_str(&format!(
                            "  {} {} {}\n",
                            alias_name.cyan(),
                            "→".dimmed(),
                            alias_slug
                        ));
                    }
                    Ok(output)
                }
                OutputFormat::Json => Ok(serde_json::to_string_pretty(&config.aliases)?),
            }
        }

        // Show specific alias
        (Some(alias_name), None, false) => {
            match config.get_alias(alias_name) {
                Some(alias_slug) => match format {
                    OutputFormat::Pretty => Ok(format!(
                        "{} {} {}",
                        alias_name.cyan(),
                        "→".dimmed(),
                        alias_slug
                    )),
                    OutputFormat::Json => {
                        let result = serde_json::json!({
                            "name": alias_name,
                            "slug": alias_slug
                        });
                        Ok(serde_json::to_string_pretty(&result)?)
                    }
                },
                None => Err(RepriseError::Config(format!(
                    "Alias '{}' not found. Use 'reprise config alias' to list all aliases.",
                    alias_name
                ))),
            }
        }

        // Remove alias
        (Some(alias_name), None, true) | (Some(alias_name), Some(_), true) => {
            match config.remove_alias(alias_name) {
                Some(old_slug) => {
                    config.save()?;
                    match format {
                        OutputFormat::Pretty => Ok(format!(
                            "{} Removed alias '{}' (was: {})",
                            "✓".green(),
                            alias_name,
                            old_slug.dimmed()
                        )),
                        OutputFormat::Json => {
                            let result = serde_json::json!({
                                "action": "removed",
                                "name": alias_name,
                                "previous_slug": old_slug
                            });
                            Ok(serde_json::to_string_pretty(&result)?)
                        }
                    }
                }
                None => Err(RepriseError::Config(format!(
                    "Alias '{}' not found",
                    alias_name
                ))),
            }
        }

        // Set alias
        (Some(alias_name), Some(alias_slug), false) => {
            let was_update = config.get_alias(alias_name).is_some();
            config.set_alias(alias_name.to_string(), alias_slug.to_string());
            config.save()?;

            match format {
                OutputFormat::Pretty => {
                    let action = if was_update { "Updated" } else { "Set" };
                    Ok(format!(
                        "{} {} alias: {} {} {}",
                        "✓".green(),
                        action,
                        alias_name.cyan(),
                        "→".dimmed(),
                        alias_slug
                    ))
                }
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "action": if was_update { "updated" } else { "created" },
                        "name": alias_name,
                        "slug": alias_slug
                    });
                    Ok(serde_json::to_string_pretty(&result)?)
                }
            }
        }

        // Invalid: remove flag without a name
        (None, _, true) => Err(RepriseError::InvalidArgument(
            "Alias name required with --remove flag".to_string(),
        )),

        // Invalid: slug without name
        (None, Some(_), false) => Err(RepriseError::InvalidArgument(
            "Alias name required when setting a slug".to_string(),
        )),
    }
}
