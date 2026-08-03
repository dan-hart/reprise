pub mod json;
pub mod pretty;

use chrono::Duration;
use crate::bitrise::{App, Artifact, Build, Pipeline};
use crate::cli::OutputFormat;
use crate::error::Result;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BuildTiming {
    pub elapsed: Option<Duration>,
    pub average: Option<Duration>,
    pub progress_percent: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TimingOptions {
    pub elapsed: bool,
    pub average: bool,
    pub progress: bool,
}

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

/// Format builds with requested timing metrics.
pub(crate) fn format_builds_with_timing(
    builds: &[Build],
    timings: &[BuildTiming],
    options: TimingOptions,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_builds_with_timing(builds, timings, options)),
        OutputFormat::Json => json::format_builds_with_timing(builds, timings, options),
    }
}

/// Format a single build based on output format
pub fn format_build(build: &Build, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_build(build)),
        OutputFormat::Json => json::format_build(build),
    }
}

/// Format a list of pipelines based on output format
pub fn format_pipelines(pipelines: &[Pipeline], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_pipelines(pipelines)),
        OutputFormat::Json => json::format_pipelines(pipelines),
    }
}

/// Format a single pipeline based on output format
pub fn format_pipeline(pipeline: &Pipeline, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_pipeline(pipeline)),
        OutputFormat::Json => json::format_pipeline(pipeline),
    }
}

/// Format a list of artifacts based on output format
pub fn format_artifacts(artifacts: &[Artifact], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Pretty => Ok(pretty::format_artifacts(artifacts)),
        OutputFormat::Json => json::format_artifacts(artifacts),
    }
}
