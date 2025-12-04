use crate::bitrise::BitriseClient;
use crate::cli::args::{BuildsArgs, OutputFormat};
use crate::config::Config;
use crate::error::Result;
use crate::output;

/// Handle the builds command
pub fn builds(
    client: &BitriseClient,
    config: &Config,
    args: &BuildsArgs,
    format: OutputFormat,
) -> Result<String> {
    // Resolve app slug from args or config default
    let app_slug = args
        .app
        .as_deref()
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    // Convert status filter to API code
    let status = args.status.map(|s| s.to_api_code());

    let response = client.list_builds(
        app_slug,
        status,
        args.branch.as_deref(),
        args.workflow.as_deref(),
        args.limit,
    )?;

    output::format_builds(&response.data, format)
}
