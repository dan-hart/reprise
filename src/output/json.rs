use serde::Serialize;

use crate::bitrise::{App, Artifact, Build, Pipeline};
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

/// Format pipelines as JSON
pub fn format_pipelines(pipelines: &[Pipeline]) -> Result<String> {
    Ok(serde_json::to_string_pretty(pipelines)?)
}

/// Format a single pipeline as JSON
pub fn format_pipeline(pipeline: &Pipeline) -> Result<String> {
    Ok(serde_json::to_string_pretty(pipeline)?)
}

/// Format any serializable value as JSON
pub fn format_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

/// Format artifacts as JSON
pub fn format_artifacts(artifacts: &[Artifact]) -> Result<String> {
    Ok(serde_json::to_string_pretty(artifacts)?)
}
