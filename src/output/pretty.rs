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
    output.push_str(&"─".repeat(70));
    output.push('\n');

    for app in apps {
        let status = if app.is_disabled {
            "disabled".red()
        } else {
            "active".green()
        };

        // Show slug prominently for easy copy-paste
        output.push_str(&format!(
            "{} [{}]\n",
            app.title.bold(),
            status
        ));
        output.push_str(&format!(
            "  {} {}\n",
            "Slug:".cyan(),
            app.slug
        ));
        output.push_str(&format!(
            "  {} {}\n",
            "Owner:".cyan(),
            app.owner.name
        ));

        if let Some(ref project_type) = app.project_type {
            output.push_str(&format!("  {} {}\n", "Type:".cyan(), project_type));
        }
        if let Some(ref repo_url) = app.repo_url {
            output.push_str(&format!("  {} {}\n", "Repo:".cyan(), repo_url.dimmed()));
        }
        output.push('\n');
    }

    output
}

/// Format a single app for pretty output
pub fn format_app(app: &App) -> String {
    let mut output = String::new();

    let status_colored = if app.is_disabled {
        "disabled".red()
    } else {
        "active".green()
    };

    output.push_str(&format!("{} [{}]\n", app.title.bold(), status_colored));
    output.push_str(&"─".repeat(50));
    output.push('\n');

    // Show slug prominently for easy copy-paste
    output.push_str(&format!("{} {}\n", "Slug:".cyan(), app.slug));
    output.push_str(&format!("{} {} ({})\n", "Owner:".cyan(), app.owner.name, app.owner.account_type));

    if let Some(ref project_type) = app.project_type {
        output.push_str(&format!("{} {}\n", "Type:".cyan(), project_type));
    }
    if let Some(ref provider) = app.provider {
        output.push_str(&format!("{} {}\n", "Provider:".cyan(), provider));
    }
    if let Some(ref repo_url) = app.repo_url {
        output.push_str(&format!("{} {}\n", "Repo:".cyan(), repo_url));
    }

    // Show visibility
    let visibility = if app.is_public { "public" } else { "private" };
    output.push_str(&format!("{} {}\n", "Visibility:".cyan(), visibility));

    // Bitrise URL
    output.push_str(&format!(
        "\n{} https://app.bitrise.io/app/{}\n",
        "URL:".cyan(),
        app.slug
    ));

    output
}

/// Format a list of builds for pretty output
pub fn format_builds(builds: &[Build]) -> String {
    if builds.is_empty() {
        return "No builds found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Builds".bold()));
    output.push_str(&"─".repeat(90));
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
        let workflow_display = truncate_str(&build.triggered_workflow, 15);

        // Main build line with build number, status, branch, workflow, duration
        output.push_str(&format!(
            "#{:<6} {:12} {:20} {:15} {}\n",
            build.build_number.to_string().bold(),
            status_colored,
            branch_display,
            workflow_display,
            build.duration_display().dimmed()
        ));

        // Show slug prominently for easy copy-paste
        output.push_str(&format!("        {} {}", "Slug:".cyan(), build.slug));

        // Show PR indicator if present
        if let Some(pr_id) = build.pull_request_id {
            output.push_str(&format!("  {}#{}", "PR".magenta(), pr_id));
        }

        // Show tag if present
        if let Some(ref tag) = build.tag {
            output.push_str(&format!("  {}{}", "Tag:".cyan(), tag));
        }

        output.push('\n');

        // Show triggered by
        if let Some(ref by) = build.triggered_by {
            output.push_str(&format!("        {} {}\n", "By:".cyan(), by.dimmed()));
        }

        // Show commit message preview for failed builds
        if build.is_failed() {
            if let Some(ref msg) = build.commit_message {
                let preview: String = msg.lines().next().unwrap_or("").chars().take(60).collect();
                output.push_str(&format!("        {}\n", preview.dimmed()));
            }
            if let Some(ref reason) = build.abort_reason {
                output.push_str(&format!("        {} {}\n", "Reason:".red(), reason.red()));
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
    output.push_str(&"─".repeat(60));
    output.push('\n');

    // Show slug prominently for easy copy-paste
    output.push_str(&format!("{} {}\n", "Slug:".cyan(), build.slug));
    output.push_str(&format!("{} {}\n", "Branch:".cyan(), build.branch));
    output.push_str(&format!("{} {}\n", "Workflow:".cyan(), build.triggered_workflow));
    output.push_str(&format!("{} {}\n", "Duration:".cyan(), build.duration_display()));

    // Show tag if present
    if let Some(ref tag) = build.tag {
        output.push_str(&format!("{} {}\n", "Tag:".cyan(), tag));
    }

    if let Some(ref commit) = build.commit_hash {
        output.push_str(&format!("{} {}\n", "Commit:".cyan(), first_n_chars(commit, 7)));
    }
    if let Some(ref msg) = build.commit_message {
        let preview: String = msg.lines().next().unwrap_or("").chars().take(60).collect();
        output.push_str(&format!("{} {}\n", "Message:".cyan(), preview));
    }

    // Pull request info
    if let Some(pr_id) = build.pull_request_id {
        output.push_str(&format!("{} #{}", "PR:".magenta(), pr_id));
        if let Some(ref target) = build.pull_request_target_branch {
            output.push_str(&format!(" → {}", target));
        }
        output.push('\n');
    }

    // Timestamps section
    output.push_str(&format!("\n{} {}\n", "Triggered:".cyan(), build.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")));

    if let Some(ref started) = build.started_on_worker_at {
        output.push_str(&format!("{} {}\n", "Started:".cyan(), started.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(ref finished) = build.finished_at {
        output.push_str(&format!("{} {}\n", "Finished:".cyan(), finished.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    if let Some(ref by) = build.triggered_by {
        output.push_str(&format!("{} {}\n", "Triggered by:".cyan(), by));
    }

    // Infrastructure info
    if let Some(ref stack) = build.stack_identifier {
        output.push_str(&format!("{} {}\n", "Stack:".cyan(), stack));
    }
    if let Some(ref machine) = build.machine_type_id {
        output.push_str(&format!("{} {}\n", "Machine:".cyan(), machine));
    }

    // Credit cost
    if let Some(cost) = build.credit_cost {
        output.push_str(&format!("{} {}\n", "Credits:".cyan(), cost));
    }

    if let Some(ref reason) = build.abort_reason {
        output.push_str(&format!("\n{} {}\n", "Abort Reason:".red().bold(), reason));
    }

    // Bitrise URL
    output.push_str(&format!(
        "\n{} https://app.bitrise.io/build/{}\n",
        "URL:".cyan(),
        build.slug
    ));

    output
}

/// Format a list of pipelines for pretty output
pub fn format_pipelines(pipelines: &[Pipeline]) -> String {
    if pipelines.is_empty() {
        return "No pipelines found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{}\n", "Pipelines".bold()));
    output.push_str(&"─".repeat(90));
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

        // Use first 8 chars of ID for display in header
        let id_display = first_n_chars(&pipeline.id, 8);

        output.push_str(&format!(
            "{:<10} {:12} {:20} {:20} {}\n",
            id_display.bold(),
            status_colored,
            branch_display,
            pipeline_name,
            pipeline.duration_display().dimmed()
        ));

        // Show full ID prominently for easy copy-paste
        output.push_str(&format!("           {} {}", "ID:".cyan(), pipeline.id));

        // Show triggered by
        if let Some(ref by) = pipeline.triggered_by {
            output.push_str(&format!("  {} {}", "By:".cyan(), by.dimmed()));
        }
        output.push('\n');

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
                output.push_str(&format!("           {} {}\n", wf_status, wf.name));
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

    // Use short ID in header
    let short_id = first_n_chars(&pipeline.id, 8);
    output.push_str(&format!("Pipeline {} {}\n", short_id.bold(), status_colored));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    // Show full ID prominently for easy copy-paste
    output.push_str(&format!("{} {}\n", "ID:".cyan(), pipeline.id));
    if !pipeline.pipeline_id.is_empty() {
        output.push_str(&format!("{} {}\n", "Pipeline:".cyan(), pipeline.pipeline_id));
    }
    output.push_str(&format!("{} {}\n", "Branch:".cyan(), pipeline.branch));
    output.push_str(&format!("{} {}\n", "Duration:".cyan(), pipeline.duration_display()));

    // Show app slug if available
    if !pipeline.app_slug.is_empty() {
        output.push_str(&format!("{} {}\n", "App:".cyan(), pipeline.app_slug));
    }

    // Timestamps section
    output.push_str(&format!("\n{} {}\n", "Triggered:".cyan(), pipeline.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")));

    if let Some(ref started) = pipeline.started_at {
        output.push_str(&format!("{} {}\n", "Started:".cyan(), started.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(ref finished) = pipeline.finished_at {
        output.push_str(&format!("{} {}\n", "Finished:".cyan(), finished.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    if let Some(ref by) = pipeline.triggered_by {
        output.push_str(&format!("{} {}\n", "Triggered by:".cyan(), by));
    }

    // Show workflow statuses
    if !pipeline.workflows.is_empty() {
        output.push_str(&format!("\n{}\n", "Workflows".bold()));
        output.push_str(&"─".repeat(40));
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
        output.push_str(&format!("\n{} {}\n", "Abort Reason:".red().bold(), reason));
    }

    // Bitrise URL (if we have app_slug)
    if !pipeline.app_slug.is_empty() {
        output.push_str(&format!(
            "\n{} https://app.bitrise.io/app/{}/pipelines/{}\n",
            "URL:".cyan(),
            pipeline.app_slug,
            pipeline.id
        ));
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
        "{} ({} artifact{})\n",
        "Build Artifacts".bold(),
        artifacts.len(),
        if artifacts.len() == 1 { "" } else { "s" }
    ));
    output.push_str(&"─".repeat(60));
    output.push_str("\n\n");

    for artifact in artifacts {
        output.push_str(&format!(
            "  {} {}\n",
            "•".cyan(),
            artifact.title.bold()
        ));
        // Show slug prominently for easy copy-paste
        output.push_str(&format!(
            "    {} {}\n",
            "Slug:".cyan(),
            artifact.slug
        ));
        output.push_str(&format!(
            "    {} {}\n",
            "Size:".cyan(),
            artifact.size_display()
        ));
        if let Some(ref artifact_type) = artifact.artifact_type {
            output.push_str(&format!("    {} {}\n", "Type:".cyan(), artifact_type));
        }

        // Show public page indicator and URL
        if artifact.is_public_page_enabled {
            output.push_str(&format!("    {} {}\n", "Public:".cyan(), "yes".green()));
            if let Some(ref url) = artifact.public_install_page_url {
                output.push_str(&format!("    {} {}\n", "Install URL:".cyan(), url));
            }
        }

        output.push('\n');
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitrise::{Owner, PipelineWorkflow};
    use chrono::{TimeZone, Utc};

    // ─────────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_test_app(slug: &str, title: &str, disabled: bool) -> App {
        App {
            slug: slug.to_string(),
            title: title.to_string(),
            project_type: Some("ios".to_string()),
            provider: Some("github".to_string()),
            repo_owner: Some("testowner".to_string()),
            repo_slug: Some("testrepo".to_string()),
            repo_url: Some("https://github.com/test/repo".to_string()),
            is_disabled: disabled,
            status: 1,
            is_public: false,
            owner: Owner {
                account_type: "user".to_string(),
                name: "Test User".to_string(),
                slug: "user-slug".to_string(),
            },
        }
    }

    fn make_test_build(slug: &str, build_number: i64, status: i32) -> Build {
        Build {
            slug: slug.to_string(),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_on_worker_at: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 1, 0).unwrap()),
            finished_at: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 6, 30).unwrap()),
            status,
            status_text: "test".to_string(),
            abort_reason: None,
            branch: "main".to_string(),
            build_number,
            commit_hash: Some("abc1234567890".to_string()),
            commit_message: Some("Test commit message".to_string()),
            tag: None,
            triggered_workflow: "primary".to_string(),
            triggered_by: Some("manual".to_string()),
            stack_identifier: Some("osx-xcode-14.3".to_string()),
            machine_type_id: Some("g2-m1.4core".to_string()),
            pull_request_id: None,
            pull_request_target_branch: None,
            credit_cost: Some(10),
        }
    }

    fn make_test_pipeline(id: &str, status: i32) -> Pipeline {
        Pipeline {
            id: id.to_string(),
            app_slug: "test-app".to_string(),
            status,
            status_text: Some("test".to_string()),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_at: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 1, 0).unwrap()),
            finished_at: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 10, 0).unwrap()),
            branch: "main".to_string(),
            pipeline_id: "build-and-test".to_string(),
            triggered_by: Some("webhook".to_string()),
            abort_reason: None,
            workflows: vec![],
        }
    }

    fn make_test_artifact(slug: &str, title: &str, size: Option<i64>) -> Artifact {
        Artifact {
            title: title.to_string(),
            slug: slug.to_string(),
            artifact_type: Some("file".to_string()),
            file_size_bytes: size,
            is_public_page_enabled: false,
            expiring_download_url: None,
            public_install_page_url: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // truncate_str Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_str_unicode() {
        assert_eq!(truncate_str("héllo wörld", 8), "héllo...");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // first_n_chars Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_first_n_chars_short() {
        assert_eq!(first_n_chars("abc", 5), "abc");
    }

    #[test]
    fn test_first_n_chars_exact() {
        assert_eq!(first_n_chars("abc", 3), "abc");
    }

    #[test]
    fn test_first_n_chars_long() {
        assert_eq!(first_n_chars("abcdef", 3), "abc");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_apps Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_apps_empty() {
        let result = format_apps(&[]);
        assert_eq!(result, "No apps found.");
    }

    #[test]
    fn test_format_apps_contains_title() {
        let apps = vec![make_test_app("slug1", "My App", false)];
        let result = format_apps(&apps);
        assert!(result.contains("My App"));
    }

    #[test]
    fn test_format_apps_contains_slug() {
        let apps = vec![make_test_app("slug1", "My App", false)];
        let result = format_apps(&apps);
        assert!(result.contains("slug1"));
    }

    #[test]
    fn test_format_apps_contains_owner() {
        let apps = vec![make_test_app("slug1", "My App", false)];
        let result = format_apps(&apps);
        assert!(result.contains("Test User"));
    }

    #[test]
    fn test_format_apps_multiple() {
        let apps = vec![
            make_test_app("app1", "First App", false),
            make_test_app("app2", "Second App", true),
        ];
        let result = format_apps(&apps);
        assert!(result.contains("First App"));
        assert!(result.contains("Second App"));
        assert!(result.contains("app1"));
        assert!(result.contains("app2"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_app Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_app_contains_title() {
        let app = make_test_app("test-slug", "Test App", false);
        let result = format_app(&app);
        assert!(result.contains("Test App"));
    }

    #[test]
    fn test_format_app_contains_slug() {
        let app = make_test_app("test-slug", "Test App", false);
        let result = format_app(&app);
        assert!(result.contains("test-slug"));
    }

    #[test]
    fn test_format_app_contains_url() {
        let app = make_test_app("test-slug", "Test App", false);
        let result = format_app(&app);
        assert!(result.contains("https://app.bitrise.io/app/test-slug"));
    }

    #[test]
    fn test_format_app_contains_owner() {
        let app = make_test_app("test-slug", "Test App", false);
        let result = format_app(&app);
        assert!(result.contains("Test User"));
        assert!(result.contains("user"));
    }

    #[test]
    fn test_format_app_contains_visibility() {
        let mut app = make_test_app("test-slug", "Test App", false);
        app.is_public = true;
        let result = format_app(&app);
        assert!(result.contains("public"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_builds Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_builds_empty() {
        let result = format_builds(&[]);
        assert_eq!(result, "No builds found.");
    }

    #[test]
    fn test_format_builds_contains_build_number() {
        let builds = vec![make_test_build("slug1", 123, 1)];
        let result = format_builds(&builds);
        assert!(result.contains("123"));
    }

    #[test]
    fn test_format_builds_contains_slug() {
        let builds = vec![make_test_build("build-slug-123", 1, 1)];
        let result = format_builds(&builds);
        assert!(result.contains("build-slug-123"));
    }

    #[test]
    fn test_format_builds_contains_branch() {
        let builds = vec![make_test_build("slug1", 1, 1)];
        let result = format_builds(&builds);
        assert!(result.contains("main"));
    }

    #[test]
    fn test_format_builds_contains_triggered_by() {
        let builds = vec![make_test_build("slug1", 1, 1)];
        let result = format_builds(&builds);
        assert!(result.contains("manual"));
    }

    #[test]
    fn test_format_builds_shows_pr() {
        let mut build = make_test_build("slug1", 1, 1);
        build.pull_request_id = Some(42);
        let result = format_builds(&[build]);
        assert!(result.contains("PR"));
        assert!(result.contains("42"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_build Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_build_contains_slug() {
        let build = make_test_build("build-abc123", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("build-abc123"));
    }

    #[test]
    fn test_format_build_contains_url() {
        let build = make_test_build("build-abc123", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("https://app.bitrise.io/build/build-abc123"));
    }

    #[test]
    fn test_format_build_contains_workflow() {
        let build = make_test_build("slug1", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("primary"));
    }

    #[test]
    fn test_format_build_contains_commit() {
        let build = make_test_build("slug1", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("abc1234")); // First 7 chars
    }

    #[test]
    fn test_format_build_contains_stack() {
        let build = make_test_build("slug1", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("osx-xcode-14.3"));
    }

    #[test]
    fn test_format_build_contains_credits() {
        let build = make_test_build("slug1", 1, 1);
        let result = format_build(&build);
        assert!(result.contains("10"));
    }

    #[test]
    fn test_format_build_shows_pr_info() {
        let mut build = make_test_build("slug1", 1, 1);
        build.pull_request_id = Some(99);
        build.pull_request_target_branch = Some("develop".to_string());
        let result = format_build(&build);
        assert!(result.contains("99"));
        assert!(result.contains("develop"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_pipelines Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_pipelines_empty() {
        let result = format_pipelines(&[]);
        assert_eq!(result, "No pipelines found.");
    }

    #[test]
    fn test_format_pipelines_contains_id() {
        let pipelines = vec![make_test_pipeline("pipeline-uuid-123", 1)];
        let result = format_pipelines(&pipelines);
        assert!(result.contains("pipeline-uuid-123"));
    }

    #[test]
    fn test_format_pipelines_contains_branch() {
        let pipelines = vec![make_test_pipeline("id1", 1)];
        let result = format_pipelines(&pipelines);
        assert!(result.contains("main"));
    }

    #[test]
    fn test_format_pipelines_contains_triggered_by() {
        let pipelines = vec![make_test_pipeline("id1", 1)];
        let result = format_pipelines(&pipelines);
        assert!(result.contains("webhook"));
    }

    #[test]
    fn test_format_pipelines_shows_workflows_when_running() {
        let mut pipeline = make_test_pipeline("id1", 0); // Running
        pipeline.workflows = vec![
            PipelineWorkflow {
                id: "wf1".to_string(),
                name: "build".to_string(),
                status: 1,
                status_text: Some("success".to_string()),
            },
            PipelineWorkflow {
                id: "wf2".to_string(),
                name: "test".to_string(),
                status: 0,
                status_text: Some("running".to_string()),
            },
        ];
        let result = format_pipelines(&[pipeline]);
        assert!(result.contains("build"));
        assert!(result.contains("test"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_pipeline Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_pipeline_contains_id() {
        let pipeline = make_test_pipeline("full-pipeline-uuid", 1);
        let result = format_pipeline(&pipeline);
        assert!(result.contains("full-pipeline-uuid"));
    }

    #[test]
    fn test_format_pipeline_contains_url() {
        let pipeline = make_test_pipeline("pipeline-id", 1);
        let result = format_pipeline(&pipeline);
        assert!(result.contains("https://app.bitrise.io/app/test-app/pipelines/pipeline-id"));
    }

    #[test]
    fn test_format_pipeline_contains_app_slug() {
        let pipeline = make_test_pipeline("id1", 1);
        let result = format_pipeline(&pipeline);
        assert!(result.contains("test-app"));
    }

    #[test]
    fn test_format_pipeline_shows_workflows() {
        let mut pipeline = make_test_pipeline("id1", 1);
        pipeline.workflows = vec![
            PipelineWorkflow {
                id: "wf1".to_string(),
                name: "build-workflow".to_string(),
                status: 1,
                status_text: Some("success".to_string()),
            },
        ];
        let result = format_pipeline(&pipeline);
        assert!(result.contains("build-workflow"));
        assert!(result.contains("Workflows"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_artifacts Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_artifacts_empty() {
        let result = format_artifacts(&[]);
        assert_eq!(result, "No artifacts found.");
    }

    #[test]
    fn test_format_artifacts_contains_title() {
        let artifacts = vec![make_test_artifact("art1", "my-app.ipa", Some(1024))];
        let result = format_artifacts(&artifacts);
        assert!(result.contains("my-app.ipa"));
    }

    #[test]
    fn test_format_artifacts_contains_slug() {
        let artifacts = vec![make_test_artifact("artifact-slug-123", "app.ipa", Some(1024))];
        let result = format_artifacts(&artifacts);
        assert!(result.contains("artifact-slug-123"));
    }

    #[test]
    fn test_format_artifacts_contains_size() {
        let artifacts = vec![make_test_artifact("art1", "app.ipa", Some(2048))];
        let result = format_artifacts(&artifacts);
        assert!(result.contains("2.0 KB"));
    }

    #[test]
    fn test_format_artifacts_shows_public_indicator() {
        let mut artifact = make_test_artifact("art1", "app.ipa", Some(1024));
        artifact.is_public_page_enabled = true;
        artifact.public_install_page_url = Some("https://install.example.com".to_string());
        let result = format_artifacts(&[artifact]);
        assert!(result.contains("yes"));
        assert!(result.contains("https://install.example.com"));
    }

    #[test]
    fn test_format_artifacts_count_singular() {
        let artifacts = vec![make_test_artifact("art1", "app.ipa", Some(1024))];
        let result = format_artifacts(&artifacts);
        assert!(result.contains("1 artifact)"));
    }

    #[test]
    fn test_format_artifacts_count_plural() {
        let artifacts = vec![
            make_test_artifact("art1", "app.ipa", Some(1024)),
            make_test_artifact("art2", "app.apk", Some(2048)),
        ];
        let result = format_artifacts(&artifacts);
        assert!(result.contains("2 artifacts)"));
    }
}
