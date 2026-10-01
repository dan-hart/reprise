use super::log_diagnosis::{self, LogSummary};
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

    let log_result = client.get_full_log(app_slug, &build.slug);
    let log_error = log_result.as_ref().err().map(ToString::to_string);
    let log = log_result.ok().filter(|log| !log.trim().is_empty());
    let log_error = log_error.or_else(|| {
        log.is_none()
            .then(|| "Log is empty or not yet available".into())
    });
    let artifacts_result = client.list_artifacts(app_slug, &build.slug);
    let artifact_error = artifacts_result.as_ref().err().map(ToString::to_string);
    let artifacts = artifacts_result.ok();
    let summary = summarize_log(log.as_deref());

    match format {
        OutputFormat::Pretty => Ok(format_pretty(
            &build,
            summary.as_ref(),
            artifacts.as_ref().map(|r| r.data.as_slice()),
            log_error.as_deref(),
            artifact_error.as_deref(),
        )),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "build": build,
                "summary": summary,
                "log_error": log_error,
                "artifact_error": artifact_error,
                "artifact_count": artifacts.as_ref().map(|r| r.data.len()),
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

fn summarize_log(log: Option<&str>) -> Option<LogSummary> {
    log.map(log_diagnosis::summarize)
}

fn format_pretty(
    build: &Build,
    summary: Option<&LogSummary>,
    artifacts: Option<&[Artifact]>,
    log_error: Option<&str>,
    artifact_error: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("Build diagnosis for #{}\n", build.build_number));
    output.push_str("────────────────────────\n");
    output.push_str(&format!("Slug: {}\n", build.slug));
    output.push_str(&format!("Status: {}\n", build.status_display()));
    output.push_str(&format!("Branch: {}\n", build.branch));
    output.push_str(&format!("Workflow: {}\n", build.triggered_workflow));
    output.push_str(&format!("Duration: {}\n", build.duration_display()));

    if let Some(error) = log_error {
        output.push_str(&format!("\nLog unavailable: {error}\n"));
    }
    if let Some(error) = artifact_error {
        output.push_str(&format!("Artifacts unavailable: {error}\n"));
    }
    if let Some(summary) = summary {
        output.push_str(&format!(
            "\nLikely category: {} (signal confidence: {})\n",
            summary.category, summary.confidence
        ));
        if let Some(step) = &summary.failed_step {
            output.push_str(&format!("Step at failure: {step}\n"));
        }
        if let Some(first) = &summary.first_error {
            output.push_str(&format!("First error: {}\n", first));
        }
        if let Some(last) = &summary.last_error {
            output.push_str(&format!("Last error: {}\n", last));
        }
        if !summary.context.is_empty() {
            output.push_str("Context (log line numbers):\n");
            for line in &summary.context {
                output.push_str(&format!("  {}: {}\n", line.line, line.text));
            }
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

#[cfg(test)]
mod regression_tests {
    use super::*;
    #[test]
    fn successful_summary_and_incidental_design_are_not_failure_evidence() {
        let result = summarize_log(Some(
            "Design resources copied\n0 errors, 0 failures\nAll tests passed\nBUILD SUCCEEDED",
        ))
        .unwrap();
        assert_eq!(result.category, "unknown");
        assert!(result.first_error.is_none());
    }
    #[test]
    fn xcode_signing_error_has_precise_category() {
        let result = summarize_log(Some("Testing build\nerror: No profiles for 'com.example.app' were found\n** BUILD FAILED **")).unwrap();
        assert_eq!(result.category, "signing");
    }
    #[test]
    fn swift_compiler_error_does_not_require_compile_word() {
        let result = summarize_log(Some(
            "Checkout.swift:42:9: error: cannot find 'price' in scope\n** BUILD FAILED **",
        ))
        .unwrap();
        assert_eq!(result.category, "compile");
    }
    #[test]
    fn gradle_dependency_resolution_is_recognized() {
        let result = summarize_log(Some("FAILURE: Build failed with an exception.\nCould not resolve all files for configuration ':app:debugRuntimeClasspath'.")).unwrap();
        assert_eq!(result.category, "dependencies");
    }
}

#[cfg(test)]
mod availability_tests {
    use super::*;
    use crate::cli::args::{Cli, Commands};
    use clap::Parser;
    #[test]
    fn unavailable_evidence_keeps_error_reasons_and_null_counts() {
        let mut server = mockito::Server::new();
        let build = server.mock("GET", "/apps/app/builds/abc").with_status(200).with_body(r#"{"data":{"slug":"abc","triggered_at":"2026-10-01T00:00:00Z","status":2,"status_text":"failed","branch":"main","build_number":1,"triggered_workflow":"primary"}}"#).create();
        let log = server
            .mock("GET", "/apps/app/builds/abc/log")
            .with_status(401)
            .with_body("Unauthorized")
            .create();
        let artifacts = server
            .mock("GET", "/apps/app/builds/abc/artifacts")
            .with_status(403)
            .with_body("Forbidden")
            .create();
        let client = BitriseClient::with_base_url("fixture-token", server.url()).unwrap();
        let mut config = Config::default();
        config.defaults.app_slug = Some("app".into());
        let cli = Cli::try_parse_from(["reprise", "diagnose", "abc"]).unwrap();
        let Commands::Diagnose(args) = cli.command else {
            panic!("diagnose command");
        };
        let output = diagnose(&client, &config, &args, OutputFormat::Json).unwrap();
        let result: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(result["summary"].is_null());
        assert!(result["artifact_count"].is_null());
        assert!(result["log_error"].as_str().unwrap().contains("401"));
        assert!(result["artifact_error"].as_str().unwrap().contains("403"));
        build.assert();
        log.assert();
        artifacts.assert();
    }
}
