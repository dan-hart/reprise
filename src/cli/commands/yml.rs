use chrono::Utc;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use crate::bitrise::BitriseClient;
use crate::cli::args::{OutputFormat, YmlArgs, YmlCommands};
use crate::cli::commands::common::resolve_app_slug;
use crate::config::{Config, Paths};
use crate::error::Result;

/// Handle bitrise.yml get/set operations.
pub fn yml(
    client: &BitriseClient,
    config: &Config,
    args: &YmlArgs,
    format: OutputFormat,
) -> Result<String> {
    match &args.command {
        YmlCommands::Get { app, save } => {
            yml_get(client, config, app.as_deref(), save.as_deref(), format)
        }
        YmlCommands::Set {
            file,
            app,
            backup_dir,
        } => yml_set(
            client,
            config,
            file,
            app.as_deref(),
            backup_dir.as_deref(),
            format,
        ),
    }
}

fn yml_get(
    client: &BitriseClient,
    config: &Config,
    app: Option<&str>,
    save: Option<&str>,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = resolve_app_slug(app, config)?;
    let content = client.get_bitrise_yml(app_slug)?;

    match save {
        Some(path) => {
            fs::write(path, &content)?;
            match format {
                OutputFormat::Pretty => Ok(format!(
                    "{} Saved bitrise.yml for {} to {}",
                    "✓".green(),
                    app_slug.bold(),
                    path
                )),
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "success": true,
                        "app_slug": app_slug,
                        "saved_to": path,
                    });
                    Ok(serde_json::to_string_pretty(&result)?)
                }
            }
        }
        None => match format {
            OutputFormat::Pretty => Ok(content),
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "app_slug": app_slug,
                    "content": content,
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
        },
    }
}

fn yml_set(
    client: &BitriseClient,
    config: &Config,
    file: &str,
    app: Option<&str>,
    backup_dir: Option<&str>,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = resolve_app_slug(app, config)?;
    let upload_content = fs::read_to_string(file)?;

    // Safety guarantee: always back up the current remote config before upload.
    let current_content = client.get_bitrise_yml(app_slug)?;
    let backup_root = resolve_backup_dir(app_slug, backup_dir)?;
    fs::create_dir_all(&backup_root)?;

    let backup_path = backup_root.join(format!("bitrise-{}.yml", timestamp_utc()));
    fs::write(&backup_path, current_content)?;

    let response = client.update_bitrise_yml(app_slug, &upload_content)?;

    match format {
        OutputFormat::Pretty => {
            let mut out = String::new();
            out.push_str(&format!(
                "{} Saved backup: {}\n",
                "✓".green(),
                backup_path.display()
            ));
            out.push_str(&format!(
                "{} Updated bitrise.yml for app {}",
                "✓".green(),
                app_slug.bold()
            ));

            if let Some(warnings) = response.get("warnings") {
                if warnings != &serde_json::Value::Null {
                    out.push_str(&format!(
                        "\n{} Bitrise warnings: {}",
                        "!".yellow(),
                        warnings
                    ));
                }
            }

            Ok(out)
        }
        OutputFormat::Json => {
            let result = serde_json::json!({
                "success": true,
                "app_slug": app_slug,
                "uploaded_from": file,
                "backup_path": backup_path,
                "response": response,
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

fn resolve_backup_dir(app_slug: &str, backup_dir: Option<&str>) -> Result<PathBuf> {
    match backup_dir {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => {
            let paths = Paths::new()?;
            Ok(paths
                .root
                .join("backups")
                .join("bitrise-yml")
                .join(app_slug))
        }
    }
}

fn timestamp_utc() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_resolve_backup_dir_custom_path() {
        let path = resolve_backup_dir("app-123", Some("/tmp/my-backups")).unwrap();
        assert_eq!(path, Path::new("/tmp/my-backups"));
    }

    #[test]
    fn test_timestamp_format_is_compact_utc() {
        let ts = timestamp_utc();
        assert_eq!(ts.len(), 16);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }
}
