use crate::bitrise::BitriseClient;
use crate::cli::args::{AppsArgs, OutputFormat};
use crate::error::Result;
use crate::output;

/// Handle the apps command
pub fn apps(client: &BitriseClient, args: &AppsArgs, format: OutputFormat) -> Result<String> {
    let response = client.list_apps(args.limit)?;

    // Apply filter if provided
    let apps: Vec<_> = if let Some(ref filter) = args.filter {
        let filter_lower = filter.to_lowercase();
        response
            .data
            .into_iter()
            .filter(|app| app.title.to_lowercase().contains(&filter_lower))
            .collect()
    } else {
        // Apply limit
        response.data.into_iter().take(args.limit as usize).collect()
    };

    output::format_apps(&apps, format)
}
