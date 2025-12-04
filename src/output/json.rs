use serde::Serialize;

use crate::bitrise::{App, Build};
use crate::error::Result;

/// Format apps as JSON
pub fn format_apps(apps: &[App]) -> Result<String> {
    Ok(serde_json::to_string_pretty(apps)?)
}

/// Format a single app as JSON
pub fn format_app(app: &App) -> Result<String> {
    Ok(serde_json::to_string_pretty(app)?)
}

/// Format builds as JSON
pub fn format_builds(builds: &[Build]) -> Result<String> {
    Ok(serde_json::to_string_pretty(builds)?)
}

/// Format a single build as JSON
pub fn format_build(build: &Build) -> Result<String> {
    Ok(serde_json::to_string_pretty(build)?)
}

/// Format any serializable value as JSON
pub fn format_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}
