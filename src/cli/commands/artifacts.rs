//! Artifacts command

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::bitrise::BitriseClient;
use crate::cli::args::{ArtifactsArgs, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Sanitize a filename to prevent path traversal attacks
///
/// Removes path separators and parent directory references,
/// keeping only the base filename with safe characters.
fn sanitize_filename(name: &str) -> Result<String> {
    // Get just the filename, stripping any path components
    // Explicitly reject if we can't extract a valid filename (don't fall back to original)
    let base_name = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            RepriseError::InvalidArgument(format!(
                "Cannot extract safe filename from: {}",
                name
            ))
        })?;

    // Reject if still contains path traversal sequences
    if base_name.contains("..") || base_name.contains('/') || base_name.contains('\\') {
        return Err(RepriseError::InvalidArgument(format!(
            "Unsafe artifact filename rejected: {}",
            name
        )));
    }

    // Reject empty or hidden files
    if base_name.is_empty() || base_name.starts_with('.') {
        return Err(RepriseError::InvalidArgument(format!(
            "Invalid artifact filename: {}",
            name
        )));
    }

    Ok(base_name.to_string())
}

/// Handle the artifacts command
pub fn artifacts(
    client: &BitriseClient,
    config: &Config,
    args: &ArtifactsArgs,
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

    // List artifacts
    let response = client.list_artifacts(app_slug, &args.slug)?;

    if response.data.is_empty() {
        return match format {
            OutputFormat::Pretty => Ok("No artifacts found for this build.".dimmed().to_string()),
            OutputFormat::Json => Ok(serde_json::to_string_pretty(&response.data)?),
        };
    }

    // Handle download if requested
    if let Some(ref dir_opt) = args.download {
        let download_dir = match dir_opt {
            Some(path) => PathBuf::from(path),
            None => std::env::current_dir()?,
        };

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&download_dir)?;

        let mut downloaded = Vec::new();

        for artifact in &response.data {
            // Get artifact with download URL
            let artifact_detail =
                client.get_artifact(app_slug, &args.slug, &artifact.slug)?;

            if let Some(ref url) = artifact_detail.data.expiring_download_url {
                // Sanitize filename to prevent path traversal
                let safe_filename = sanitize_filename(&artifact.title)?;
                let file_path = download_dir.join(&safe_filename);

                if format == OutputFormat::Pretty {
                    eprint!("Downloading {}... ", safe_filename);
                }

                client.download_artifact(url, &file_path)?;

                if format == OutputFormat::Pretty {
                    eprintln!("{}", "done".green());
                }

                downloaded.push(safe_filename);
            }
        }

        return match format {
            OutputFormat::Pretty => {
                Ok(format!(
                    "\n{} Downloaded {} artifact(s) to {}",
                    "✓".green(),
                    downloaded.len(),
                    download_dir.display()
                ))
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "downloaded": downloaded,
                    "directory": download_dir.to_string_lossy(),
                });
                Ok(serde_json::to_string_pretty(&json)?)
            }
        };
    }

    // Just list artifacts
    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!(
                "{} ({} artifact{})\n\n",
                "Build Artifacts".bold(),
                response.data.len(),
                if response.data.len() == 1 { "" } else { "s" }
            ));

            for artifact in &response.data {
                output.push_str(&format!(
                    "  {} {}\n",
                    "•".cyan(),
                    artifact.title.bold()
                ));
                output.push_str(&format!(
                    "    Slug: {}\n",
                    artifact.slug.dimmed()
                ));
                output.push_str(&format!(
                    "    Size: {}\n",
                    artifact.size_display()
                ));
                if let Some(ref artifact_type) = artifact.artifact_type {
                    output.push_str(&format!("    Type: {}\n", artifact_type));
                }
                output.push('\n');
            }

            Ok(output.trim_end().to_string())
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&response.data)?),
    }
}
