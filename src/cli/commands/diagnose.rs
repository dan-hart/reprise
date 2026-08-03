use crate::bitrise::{Artifact, BitriseClient, Build};
use crate::cli::args::{DiagnoseArgs, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

pub fn diagnose(
    client: &BitriseClient,
    config: &Config,
    args: &DiagnoseArgs,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = super::common::resolve_app_slug(args.app.as_deref(), config)?;
    let build = if let Some(slug) = &args.slug {
        client.get_build(app_slug, slug)?.data
    } else if args.latest {
        super::common::resolve_latest_build(
            client,
            app_slug,
            args.branch.as_deref(),
            args.workflow.as_deref(),
            args.status,
            args.pr,
            args.current_branch,
        )?
    } else {
        return Err(RepriseError::InvalidArgument(
            "Provide a build slug or use --latest".to_string(),
        ));
    };

    let log = client.get_full_log(app_slug, &build.slug).ok();
    let artifacts = client.list_artifacts(app_slug, &build.slug).ok();
    let summary = summarize_log(log.as_deref());

    match format {
        OutputFormat::Pretty => Ok(format_pretty(
            &build,
            summary.as_ref(),
            artifacts.as_ref().map(|r| r.data.as_slice()),
        )),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "build": build,
                "summary": summary,
                "artifact_count": artifacts.as_ref().map(|r| r.data.len()),
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct LogSummary {
    category: String,
    first_error: Option<String>,
    last_error: Option<String>,
    suggested_next_step: String,
}

fn summarize_log(log: Option<&str>) -> Option<LogSummary> {
    let log = log?;
    let mut error_lines = Vec::new();

    for line in log.lines() {
        let lower = line.to_lowercase();
        if lower.contains("error")
            || lower.contains("failed")
            || lower.contains("fatal")
            || lower.contains("exception")
            || lower.contains("panic")
        {
            error_lines.push(line.trim().to_string());
        }
    }

    let category = if log.contains("test") && log.contains("failed") {
        "tests"
    } else if log.contains("Could not resolve") || log.contains("No such file or directory") {
        "dependencies"
    } else if log.contains("compile") || log.contains("rustc") {
        "compile"
    } else if log.contains("sign") || log.contains("provision") {
        "signing"
    } else {
        "generic"
    };

    Some(LogSummary {
        category: category.to_string(),
        first_error: error_lines.first().cloned(),
        last_error: error_lines.last().cloned(),
        suggested_next_step: match category {
            "tests" => "Open the full log and inspect the first failing test.".to_string(),
            "dependencies" => {
                "Check dependency installation and missing file configuration.".to_string()
            }
            "compile" => "Inspect compiler output near the first error line.".to_string(),
            "signing" => {
                "Verify signing credentials, certificates, and provisioning settings.".to_string()
            }
            _ => "Inspect the full log and compare against the last successful build.".to_string(),
        },
    })
}

fn format_pretty(
    build: &Build,
    summary: Option<&LogSummary>,
    artifacts: Option<&[Artifact]>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("Build diagnosis for #{}\n", build.build_number));
    output.push_str("────────────────────────\n");
    output.push_str(&format!("Slug: {}\n", build.slug));
    output.push_str(&format!("Status: {}\n", build.status_display()));
    output.push_str(&format!("Branch: {}\n", build.branch));
    output.push_str(&format!("Workflow: {}\n", build.triggered_workflow));
    output.push_str(&format!("Duration: {}\n", build.duration_display()));

    if let Some(summary) = summary {
        output.push_str(&format!("\nLikely category: {}\n", summary.category));
        if let Some(first) = &summary.first_error {
            output.push_str(&format!("First error: {}\n", first));
        }
        if let Some(last) = &summary.last_error {
            output.push_str(&format!("Last error: {}\n", last));
        }
        output.push_str(&format!("Next step: {}\n", summary.suggested_next_step));
    }

    if let Some(artifacts) = artifacts {
        output.push_str(&format!("\nArtifacts: {}\n", artifacts.len()));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_log_detects_test_failures() {
        let summary = summarize_log(Some("running tests\n1 test failed\nerror: assertion"));
        let summary = summary.expect("summary");
        assert_eq!(summary.category, "tests");
        assert!(summary.suggested_next_step.contains("failing test"));
    }

    #[test]
    fn test_summarize_log_detects_compile_failures() {
        let summary = summarize_log(Some("rustc compile error\nfatal: build failed"));
        let summary = summary.expect("summary");
        assert_eq!(summary.category, "compile");
        assert!(summary.first_error.is_some());
        assert!(summary.last_error.is_some());
    }
}
