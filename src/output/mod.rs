pub mod json;
pub mod pretty;

use crate::bitrise::{App, Build};
use crate::cli::OutputFormat;
use crate::error::Result;

/// Format a list of apps based on output format
pub fn format_apps(apps: &[App], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_apps(apps)),
        OutputFormat::Json => json::format_apps(apps),
    }
}

/// Format a single app based on output format
pub fn format_app(app: &App, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_app(app)),
        OutputFormat::Json => json::format_app(app),
    }
}

/// Format a list of builds based on output format
pub fn format_builds(builds: &[Build], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_builds(builds)),
        OutputFormat::Json => json::format_builds(builds),
    }
}

/// Format a single build based on output format
pub fn format_build(build: &Build, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_build(build)),
        OutputFormat::Json => json::format_build(build),
    }
}
