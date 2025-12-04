use colored::Colorize;

use crate::bitrise::{App, Artifact, Build, Pipeline};

/// Safely truncate a string to n characters, appending "..." if truncated.
/// Works correctly with multi-byte UTF-8 characters.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        let truncated: String = chars.iter().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Safely get first n characters of a string.
/// Works correctly with multi-byte UTF-8 characters.
fn first_n_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Format a list of apps for pretty output
pub fn format_apps(apps: &[App]) -> String {
    if apps.is_empty() {
        return "No apps found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Apps".bold()));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    for app in apps {
        let status = if app.is_disabled {
            "disabled".red()
        } else {
            "active".green()
        };

        let slug_display = format!("({})", app.slug);
        let status_display = format!("[{}]", status);
        output.push_str(&format!(
            "{} {} {}\n",
            app.title.bold(),
            slug_display.dimmed(),
            status_display
        ));

        if let Some(ref project_type) = app.project_type {
            output.push_str(&format!("  Type: {}\n", project_type));
        }
        if let Some(ref repo_url) = app.repo_url {
            output.push_str(&format!("  Repo: {}\n", repo_url.dimmed()));
        }
        output.push('\n');
    }

    output
}

/// Format a single app for pretty output
pub fn format_app(app: &App) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}\n", app.title.bold()));
    output.push_str(&"─".repeat(40));
    output.push('\n');

    output.push_str(&format!("Slug:    {}\n", app.slug));
    output.push_str(&format!("Owner:   {} ({})\n", app.owner.name, app.owner.account_type));

    if let Some(ref project_type) = app.project_type {
        output.push_str(&format!("Type:    {}\n", project_type));
    }
    if let Some(ref provider) = app.provider {
        output.push_str(&format!("Provider: {}\n", provider));
    }
    if let Some(ref repo_url) = app.repo_url {
        output.push_str(&format!("Repo:    {}\n", repo_url));
    }

    let status = if app.is_disabled { "disabled" } else { "active" };
    output.push_str(&format!("Status:  {}\n", status));

    output
}

/// Format a list of builds for pretty output
pub fn format_builds(builds: &[Build]) -> String {
    if builds.is_empty() {
        return "No builds found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Builds".bold()));
    output.push_str(&"─".repeat(80));
    output.push('\n');

    for build in builds {
        let status_colored = match build.status {
            0 => "running".yellow().bold(),
            1 => "success".green(),
            2 => "failed".red().bold(),
            3 => "aborted".red(),
            _ => "unknown".dimmed(),
        };

        let branch_display = truncate_str(&build.branch, 20);

        output.push_str(&format!(
            "#{:<6} {:12} {:20} {:15} {}\n",
            build.build_number.to_string().bold(),
            status_colored,
            branch_display,
            build.triggered_workflow,
            build.duration_display().dimmed()
        ));

        // Show commit message preview for failed builds
        if build.is_failed() {
            if let Some(ref msg) = build.commit_message {
                let preview: String = msg.lines().next().unwrap_or("").chars().take(60).collect();
                output.push_str(&format!("        {}\n", preview.dimmed()));
            }
            if let Some(ref reason) = build.abort_reason {
                output.push_str(&format!("        Reason: {}\n", reason.red()));
            }
        }
    }

    output
}

/// Format a single build for pretty output
pub fn format_build(build: &Build) -> String {
    let mut output = String::new();

    let status_colored = match build.status {
        0 => format!("{}", "RUNNING".yellow().bold()),
        1 => format!("{}", "SUCCESS".green().bold()),
        2 => format!("{}", "FAILED".red().bold()),
        3 => format!("{}", "ABORTED".red()),
        _ => format!("{}", "UNKNOWN".dimmed()),
    };

    output.push_str(&format!("Build #{} {}\n", build.build_number.to_string().bold(), status_colored));
    output.push_str(&"─".repeat(50));
    output.push('\n');

    output.push_str(&format!("Slug:     {}\n", build.slug));
    output.push_str(&format!("Branch:   {}\n", build.branch));
    output.push_str(&format!("Workflow: {}\n", build.triggered_workflow));
    output.push_str(&format!("Duration: {}\n", build.duration_display()));

    if let Some(ref commit) = build.commit_hash {
        output.push_str(&format!("Commit:   {}\n", first_n_chars(commit, 7)));
    }
    if let Some(ref msg) = build.commit_message {
        let preview: String = msg.lines().next().unwrap_or("").chars().take(60).collect();
        output.push_str(&format!("Message:  {}\n", preview));
    }

    output.push_str(&format!("\nTriggered: {}\n", build.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")));

    if let Some(ref started) = build.started_on_worker_at {
        output.push_str(&format!("Started:   {}\n", started.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(ref finished) = build.finished_at {
        output.push_str(&format!("Finished:  {}\n", finished.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    if let Some(ref by) = build.triggered_by {
        output.push_str(&format!("\nTriggered by: {}\n", by));
    }

    if let Some(ref stack) = build.stack_identifier {
        output.push_str(&format!("Stack:    {}\n", stack));
    }
    if let Some(ref machine) = build.machine_type_id {
        output.push_str(&format!("Machine:  {}\n", machine));
    }

    if let Some(pr_id) = build.pull_request_id {
        output.push_str(&format!("\nPull Request: #{}", pr_id));
        if let Some(ref target) = build.pull_request_target_branch {
            output.push_str(&format!(" → {}", target));
        }
        output.push('\n');
    }

    if let Some(ref reason) = build.abort_reason {
        output.push_str(&format!("\n{}: {}\n", "Abort Reason".red().bold(), reason));
    }

    output
}

/// Format a list of pipelines for pretty output
pub fn format_pipelines(pipelines: &[Pipeline]) -> String {
    if pipelines.is_empty() {
        return "No pipelines found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Pipelines".bold()));
    output.push_str(&"─".repeat(80));
    output.push('\n');

    for pipeline in pipelines {
        let status_colored = match pipeline.status {
            0 => "running".yellow().bold(),
            1 => "success".green(),
            2 => "failed".red().bold(),
            3 => "aborted".red(),
            _ => "unknown".dimmed(),
        };

        let branch_display = truncate_str(&pipeline.branch, 20);
        let pipeline_name = truncate_str(&pipeline.pipeline_id, 20);

        // Use first 8 chars of ID for display
        let id_display = first_n_chars(&pipeline.id, 8);

        output.push_str(&format!(
            "{:<10} {:12} {:20} {:20} {}\n",
            id_display.bold(),
            status_colored,
            branch_display,
            pipeline_name,
            pipeline.duration_display().dimmed()
        ));

        // Show workflow statuses for running/failed pipelines
        if pipeline.is_running() || pipeline.is_failed() {
            for wf in &pipeline.workflows {
                let wf_status = match wf.status {
                    0 => "●".yellow(),
                    1 => "✓".green(),
                    2 => "✗".red(),
                    3 => "○".dimmed(),
                    _ => "?".dimmed(),
                };
                output.push_str(&format!("           {} {}\n", wf_status, wf.name.dimmed()));
            }
        }
    }

    output
}

/// Format a single pipeline for pretty output
pub fn format_pipeline(pipeline: &Pipeline) -> String {
    let mut output = String::new();

    let status_colored = match pipeline.status {
        0 => format!("{}", "RUNNING".yellow().bold()),
        1 => format!("{}", "SUCCESS".green().bold()),
        2 => format!("{}", "FAILED".red().bold()),
        3 => format!("{}", "ABORTED".red()),
        _ => format!("{}", "UNKNOWN".dimmed()),
    };

    output.push_str(&format!("Pipeline {} {}\n", pipeline.id.bold(), status_colored));
    output.push_str(&"─".repeat(50));
    output.push('\n');

    output.push_str(&format!("ID:       {}\n", pipeline.id));
    if !pipeline.pipeline_id.is_empty() {
        output.push_str(&format!("Pipeline: {}\n", pipeline.pipeline_id));
    }
    output.push_str(&format!("Branch:   {}\n", pipeline.branch));
    output.push_str(&format!("Duration: {}\n", pipeline.duration_display()));

    output.push_str(&format!("\nTriggered: {}\n", pipeline.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")));

    if let Some(ref started) = pipeline.started_at {
        output.push_str(&format!("Started:   {}\n", started.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(ref finished) = pipeline.finished_at {
        output.push_str(&format!("Finished:  {}\n", finished.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    if let Some(ref by) = pipeline.triggered_by {
        output.push_str(&format!("\nTriggered by: {}\n", by));
    }

    // Show workflow statuses
    if !pipeline.workflows.is_empty() {
        output.push_str(&format!("\n{}\n", "Workflows".bold()));
        output.push_str(&"─".repeat(30));
        output.push('\n');

        for wf in &pipeline.workflows {
            let wf_status_colored = match wf.status {
                0 => "running".yellow().bold(),
                1 => "success".green(),
                2 => "failed".red().bold(),
                3 => "aborted".red(),
                _ => "unknown".dimmed(),
            };
            output.push_str(&format!("  {} {:12}\n", wf.name, wf_status_colored));
        }
    }

    if let Some(ref reason) = pipeline.abort_reason {
        output.push_str(&format!("\n{}: {}\n", "Abort Reason".red().bold(), reason));
    }

    output
}

/// Format a list of artifacts for pretty output
pub fn format_artifacts(artifacts: &[Artifact]) -> String {
    if artifacts.is_empty() {
        return "No artifacts found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{} ({} artifact{})\n\n",
        "Build Artifacts".bold(),
        artifacts.len(),
        if artifacts.len() == 1 { "" } else { "s" }
    ));

    for artifact in artifacts {
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

    output.trim_end().to_string()
}
