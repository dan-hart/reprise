use crate::bitrise::BitriseClient;
use crate::cli::args::{BuildArgs, OutputFormat};
use crate::config::Config;
use crate::error::Result;
use crate::output;

/// Handle the build command (show details)
pub fn build(
    client: &BitriseClient,
    config: &Config,
    args: &BuildArgs,
    format: OutputFormat,
) -> Result<String> {
    // Resolve app slug from args or config default
    let app_slug = args
        .app
        .as_deref()
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())?;

    let response = client.get_build(app_slug, &args.slug)?;

    output::format_build(&response.data, format)
}
