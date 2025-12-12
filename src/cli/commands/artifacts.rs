//! Artifacts command

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::bitrise::{Artifact, BitriseClient};
use crate::cli::args::{ArtifactsArgs, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Match a filename against a simple glob pattern.
///
/// Supports:
/// - `*` matches any sequence of characters (except path separators)
/// - `?` matches any single character
/// - All other characters are matched literally (case-insensitive)
///
/// # Examples
/// ```ignore
/// assert!(matches_glob("app.ipa", "*.ipa"));
/// assert!(matches_glob("test-results.xml", "test-*"));
/// assert!(matches_glob("App.dSYM.zip", "*.dSYM*"));
/// assert!(!matches_glob("app.apk", "*.ipa"));
/// ```
fn matches_glob(name: &str, pattern: &str) -> bool {
    let name_lower = name.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // Convert glob pattern to regex
    let mut regex_pattern = String::from("^");
    for c in pattern_lower.chars() {
        match c {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            // Escape regex special characters
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(c);
            }
            _ => regex_pattern.push(c),
        }
    }
    regex_pattern.push('$');

    // Simple regex matching without pulling in the regex crate
    // For complex patterns, this may not work perfectly, but covers common cases
    simple_regex_match(&name_lower, &regex_pattern)
}

/// Simple regex matcher for glob-converted patterns.
/// Handles .* (match any) and . (match single) patterns.
fn simple_regex_match(text: &str, pattern: &str) -> bool {
    // Remove ^ and $ anchors for processing
    let pattern = pattern.trim_start_matches('^').trim_end_matches('$');

    fn matches_recursive(text: &str, pattern: &str) -> bool {
        if pattern.is_empty() {
            return text.is_empty();
        }

        // Handle .* (match any sequence)
        if let Some(rest_pattern) = pattern.strip_prefix(".*") {
            // Try matching .* against 0 or more characters
            for i in 0..=text.len() {
                if matches_recursive(&text[i..], rest_pattern) {
                    return true;
                }
            }
            return false;
        }

        // Handle \. (escaped dot - literal match)
        if let Some(rest_pattern) = pattern.strip_prefix("\\.") {
            if let Some(rest_text) = text.strip_prefix('.') {
                return matches_recursive(rest_text, rest_pattern);
            }
            return false;
        }

        // Handle . (match single character)
        if let Some(rest_pattern) = pattern.strip_prefix('.') {
            if !text.is_empty() {
                return matches_recursive(&text[1..], rest_pattern);
            }
            return false;
        }

        // Handle escaped characters
        if let Some(escaped_rest) = pattern.strip_prefix('\\') {
            if let Some(escaped) = escaped_rest.chars().next() {
                if let Some(rest_text) = text.strip_prefix(escaped) {
                    return matches_recursive(rest_text, &escaped_rest[escaped.len_utf8()..]);
                }
            }
            return false;
        }

        // Literal character match
        if let Some(first_char) = pattern.chars().next() {
            if let Some(rest_text) = text.strip_prefix(first_char) {
                return matches_recursive(rest_text, &pattern[first_char.len_utf8()..]);
            }
        }

        false
    }

    matches_recursive(text, pattern)
}

/// Filter artifacts based on filter and exclude patterns
fn filter_artifacts<'a>(
    artifacts: &'a [Artifact],
    filter: Option<&str>,
    exclude: Option<&str>,
) -> Vec<&'a Artifact> {
    artifacts
        .iter()
        .filter(|a| {
            // Include if matches filter (or no filter specified)
            filter.is_none_or(|pattern| matches_glob(&a.title, pattern))
        })
        .filter(|a| {
            // Exclude if matches exclude pattern
            exclude.is_none_or(|pattern| !matches_glob(&a.title, pattern))
        })
        .collect()
}

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
        .as_deref()
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

    // Apply filtering
    let filtered_artifacts = filter_artifacts(
        &response.data,
        args.filter.as_deref(),
        args.exclude.as_deref(),
    );

    if filtered_artifacts.is_empty() {
        let filter_msg = match (&args.filter, &args.exclude) {
            (Some(f), Some(e)) => format!("filter '{}' and exclude '{}'", f, e),
            (Some(f), None) => format!("filter '{}'", f),
            (None, Some(e)) => format!("exclude '{}'", e),
            (None, None) => "filters".to_string(),
        };
        return match format {
            OutputFormat::Pretty => Ok(format!(
                "No artifacts matched {}.\n\nTotal artifacts in build: {}",
                filter_msg,
                response.data.len()
            ).dimmed().to_string()),
            OutputFormat::Json => Ok(serde_json::to_string_pretty(&Vec::<&Artifact>::new())?),
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

        for artifact in &filtered_artifacts {
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
                let filter_note = if args.filter.is_some() || args.exclude.is_some() {
                    format!(" (filtered from {} total)", response.data.len())
                } else {
                    String::new()
                };
                Ok(format!(
                    "\n{} Downloaded {} artifact(s){} to {}",
                    "✓".green(),
                    downloaded.len(),
                    filter_note,
                    download_dir.display()
                ))
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "downloaded": downloaded,
                    "directory": download_dir.to_string_lossy(),
                    "total_artifacts": response.data.len(),
                });
                Ok(serde_json::to_string_pretty(&json)?)
            }
        };
    }

    // Just list artifacts
    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            let filter_note = if args.filter.is_some() || args.exclude.is_some() {
                format!(" (filtered from {} total)", response.data.len())
            } else {
                String::new()
            };
            output.push_str(&format!(
                "{} ({} artifact{}{})\n\n",
                "Build Artifacts".bold(),
                filtered_artifacts.len(),
                if filtered_artifacts.len() == 1 { "" } else { "s" },
                filter_note
            ));

            for artifact in &filtered_artifacts {
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
        OutputFormat::Json => {
            let artifacts_data: Vec<_> = filtered_artifacts.to_vec();
            Ok(serde_json::to_string_pretty(&artifacts_data)?)
        }
    }
}
