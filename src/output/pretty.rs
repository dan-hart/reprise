use colored::Colorize;

use crate::bitrise::{App, Build};

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

        output.push_str(&format!(
            "{} {} {}\n",
            app.title.bold(),
            format!("({})", app.slug).dimmed(),
            format!("[{}]", status)
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

        let branch_display = if build.branch.len() > 20 {
            format!("{}...", &build.branch[..17])
        } else {
            build.branch.clone()
        };

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
        output.push_str(&format!("Commit:   {}\n", &commit[..7.min(commit.len())]));
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
