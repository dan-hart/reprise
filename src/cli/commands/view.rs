use crate::bitrise::BitriseClient;
use crate::cli::args::{
    BuildStatusFilter, BuildsArgs, OutputFormat, PipelinesArgs, ViewArgs, ViewCommands, ViewKindArg,
};
use crate::config::{Config, SavedView, SavedViewKind};
use crate::error::{RepriseError, Result};

pub fn view(
    config: &mut Config,
    inline_token: Option<&str>,
    args: &ViewArgs,
    format: OutputFormat,
) -> Result<String> {
    match &args.command {
        ViewCommands::List => list_views(config, format),
        ViewCommands::Show { name } => show_view(config, name, format),
        ViewCommands::Remove { name } => remove_view(config, name, format),
        ViewCommands::Save {
            name,
            kind,
            app,
            status,
            branch,
            workflow,
            triggered_by,
            me,
            since,
            pr,
            limit,
        } => save_view(
            config,
            name,
            *kind,
            app.clone(),
            status.map(status_to_string),
            branch.clone(),
            workflow.clone(),
            triggered_by.clone(),
            *me,
            since.clone(),
            *pr,
            *limit,
            format,
        ),
        ViewCommands::Run { name } => run_view(config, inline_token, name, format),
    }
}

fn list_views(config: &Config, format: OutputFormat) -> Result<String> {
    let views = config.list_views();

    match format {
        OutputFormat::Pretty => {
            if views.is_empty() {
                return Ok("No saved views configured.".to_string());
            }

            let mut output = String::from("Saved views\n──────────\n");
            for (name, view) in views {
                output.push_str(&format!("{} ({})\n", name, view_kind_label(view.kind)));
            }
            Ok(output)
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&config.views)?),
    }
}

fn show_view(config: &Config, name: &str, format: OutputFormat) -> Result<String> {
    let view = config
        .get_view(name)
        .ok_or_else(|| RepriseError::Config(format!("View '{}' not found", name)))?;

    match format {
        OutputFormat::Pretty => {
            let mut output = String::new();
            output.push_str(&format!("View {}\n", name));
            output.push_str("──────────\n");
            output.push_str(&format!("kind = {}\n", view_kind_label(view.kind)));
            output.push_str(&format!(
                "app = {}\n",
                view.app.as_deref().unwrap_or("(default)")
            ));
            output.push_str(&format!(
                "status = {}\n",
                view.status.as_deref().unwrap_or("(any)")
            ));
            output.push_str(&format!(
                "branch = {}\n",
                view.branch.as_deref().unwrap_or("(any)")
            ));
            if let Some(workflow) = view.workflow.as_deref() {
                output.push_str(&format!("workflow = {}\n", workflow));
            }
            if let Some(triggered_by) = view.triggered_by.as_deref() {
                output.push_str(&format!("triggered_by = {}\n", triggered_by));
            }
            if let Some(since) = view.since.as_deref() {
                output.push_str(&format!("since = {}\n", since));
            }
            if let Some(pr) = view.pr {
                output.push_str(&format!("pr = {}\n", pr));
            }
            if let Some(limit) = view.limit {
                output.push_str(&format!("limit = {}\n", limit));
            }
            Ok(output)
        }
        OutputFormat::Json => Ok(serde_json::to_string_pretty(view)?),
    }
}

#[allow(clippy::too_many_arguments)]
fn save_view(
    config: &mut Config,
    name: &str,
    kind: ViewKindArg,
    app: Option<String>,
    status: Option<String>,
    branch: Option<String>,
    workflow: Option<String>,
    triggered_by: Option<String>,
    me: bool,
    since: Option<String>,
    pr: Option<i64>,
    limit: Option<u32>,
    format: OutputFormat,
) -> Result<String> {
    let saved = SavedView {
        kind: match kind {
            ViewKindArg::Builds => SavedViewKind::Builds,
            ViewKindArg::Pipelines => SavedViewKind::Pipelines,
        },
        app,
        status,
        branch,
        workflow,
        triggered_by,
        me,
        since,
        pr,
        limit,
    };

    config.set_view(name.to_string(), saved);
    config.save()?;

    match format {
        OutputFormat::Pretty => Ok(format!("Saved view '{}'", name)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "saved": true,
            "name": name
        }))?),
    }
}

fn remove_view(config: &mut Config, name: &str, format: OutputFormat) -> Result<String> {
    config
        .remove_view(name)
        .ok_or_else(|| RepriseError::Config(format!("View '{}' not found", name)))?;
    config.save()?;

    match format {
        OutputFormat::Pretty => Ok(format!("Removed view '{}'", name)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "removed": true,
            "name": name
        }))?),
    }
}

fn run_view(
    config: &Config,
    inline_token: Option<&str>,
    name: &str,
    format: OutputFormat,
) -> Result<String> {
    let view = config
        .get_view(name)
        .cloned()
        .ok_or_else(|| RepriseError::Config(format!("View '{}' not found", name)))?;

    let client = match inline_token {
        Some(token) => BitriseClient::with_token(token)?,
        None => BitriseClient::new(config)?,
    };

    match view.kind {
        SavedViewKind::Builds => super::builds::builds(
            &client,
            config,
            &BuildsArgs {
                app: view.app,
                status: parse_status(view.status.as_deref())?,
                branch: view.branch,
                current_branch: false,
                workflow: view.workflow,
                workflow_contains: None,
                triggered_by: view.triggered_by,
                me: view.me,
                since: view.since,
                pr: view.pr,
                limit: view.limit.unwrap_or(25),
                elapsed: false,
                average: false,
                progress: false,
                watch: false,
                interval: 10,
            },
            format,
        ),
        SavedViewKind::Pipelines => super::pipelines::pipelines(
            &client,
            config,
            &PipelinesArgs {
                app: view.app,
                status: parse_status(view.status.as_deref())?,
                branch: view.branch,
                current_branch: false,
                triggered_by: view.triggered_by,
                me: view.me,
                since: view.since,
                limit: view.limit.unwrap_or(25),
            },
            format,
        ),
    }
}

fn parse_status(value: Option<&str>) -> Result<Option<BuildStatusFilter>> {
    match value {
        None => Ok(None),
        Some("running") => Ok(Some(BuildStatusFilter::Running)),
        Some("success") => Ok(Some(BuildStatusFilter::Success)),
        Some("failed") => Ok(Some(BuildStatusFilter::Failed)),
        Some("aborted") => Ok(Some(BuildStatusFilter::Aborted)),
        Some(other) => Err(RepriseError::InvalidArgument(format!(
            "Unknown saved view status '{}'",
            other
        ))),
    }
}

fn status_to_string(status: BuildStatusFilter) -> String {
    match status {
        BuildStatusFilter::Running => "running".to_string(),
        BuildStatusFilter::Success => "success".to_string(),
        BuildStatusFilter::Failed => "failed".to_string(),
        BuildStatusFilter::Aborted => "aborted".to_string(),
    }
}

fn view_kind_label(kind: SavedViewKind) -> &'static str {
    match kind {
        SavedViewKind::Builds => "builds",
        SavedViewKind::Pipelines => "pipelines",
    }
}
