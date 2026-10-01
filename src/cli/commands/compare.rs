use crate::bitrise::{Artifact, BitriseClient, Build};
use crate::cli::args::{CompareArgs, OutputFormat};
use crate::config::Config;
use crate::error::Result;

pub fn compare(
    client: &BitriseClient,
    config: &Config,
    args: &CompareArgs,
    format: OutputFormat,
) -> Result<String> {
    let app_slug = super::common::resolve_app_slug(args.app.as_deref(), config)?;
    // Four independent requests, bounded to four scoped workers.
    let (left, right, left_artifacts, right_artifacts) = std::thread::scope(|scope| {
        let left = scope.spawn(|| client.get_build(app_slug, &args.left));
        let right = scope.spawn(|| client.get_build(app_slug, &args.right));
        let left_artifacts = scope.spawn(|| client.list_artifacts(app_slug, &args.left));
        let right_artifacts = scope.spawn(|| client.list_artifacts(app_slug, &args.right));
        (
            left.join().unwrap_or_else(|_| {
                Err(crate::error::RepriseError::InvalidArgument(
                    "metadata worker failed".into(),
                ))
            }),
            right.join().unwrap_or_else(|_| {
                Err(crate::error::RepriseError::InvalidArgument(
                    "metadata worker failed".into(),
                ))
            }),
            left_artifacts.join().unwrap_or_else(|_| {
                Err(crate::error::RepriseError::InvalidArgument(
                    "artifact worker failed".into(),
                ))
            }),
            right_artifacts.join().unwrap_or_else(|_| {
                Err(crate::error::RepriseError::InvalidArgument(
                    "artifact worker failed".into(),
                ))
            }),
        )
    });
    let left = left?.data;
    let right = right?.data;
    let artifact_errors = serde_json::json!({
        "left": left_artifacts.as_ref().err().map(ToString::to_string),
        "right": right_artifacts.as_ref().err().map(ToString::to_string),
    });
    let artifacts_available = left_artifacts.is_ok() && right_artifacts.is_ok();
    let left_artifacts = left_artifacts.ok().map(|response| response.data);
    let right_artifacts = right_artifacts.ok().map(|response| response.data);

    match format {
        OutputFormat::Pretty => {
            if artifacts_available {
                Ok(format_pretty(
                    &left,
                    &right,
                    left_artifacts.as_deref().unwrap_or_default(),
                    right_artifacts.as_deref().unwrap_or_default(),
                ))
            } else {
                let mut output = format_pretty(&left, &right, &[], &[]);
                output =
                    output.replace("Artifacts: 0 -> 0\n", "Artifacts: comparison unavailable\n");
                for side in ["left", "right"] {
                    if let Some(error) = artifact_errors[side].as_str() {
                        output.push_str(&format!("{side} artifacts: {error}\n"));
                    }
                }
                Ok(output)
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "left": left,
                "right": right,
                "artifact_delta": if artifacts_available { Some(artifact_delta(left_artifacts.as_deref().unwrap_or_default(), right_artifacts.as_deref().unwrap_or_default())) } else { None },
                "artifact_errors": artifact_errors,
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

fn format_pretty(
    left: &Build,
    right: &Build,
    left_artifacts: &[Artifact],
    right_artifacts: &[Artifact],
) -> String {
    let mut output = String::new();
    output.push_str("Build comparison\n");
    output.push_str("────────────────\n");
    output.push_str(&format!("Left:  #{} {}\n", left.build_number, left.slug));
    output.push_str(&format!(
        "Right: #{} {}\n\n",
        right.build_number, right.slug
    ));
    output.push_str(&format!(
        "Status:   {} -> {}\n",
        left.status_display(),
        right.status_display()
    ));
    output.push_str(&format!("Branch:   {} -> {}\n", left.branch, right.branch));
    output.push_str(&format!(
        "Workflow: {} -> {}\n",
        left.triggered_workflow, right.triggered_workflow
    ));
    output.push_str(&format!(
        "Duration: {} -> {}\n",
        left.duration_display(),
        right.duration_display()
    ));
    output.push_str(&format!(
        "Commit:   {} -> {}\n",
        left.commit_hash.as_deref().unwrap_or("-"),
        right.commit_hash.as_deref().unwrap_or("-")
    ));
    output.push_str(&format!(
        "Artifacts: {} -> {}\n",
        left_artifacts.len(),
        right_artifacts.len()
    ));

    let delta = artifact_delta(left_artifacts, right_artifacts);
    if !delta.added.is_empty() {
        output.push_str(&format!("Added artifacts: {}\n", delta.added.join(", ")));
    }
    if !delta.removed.is_empty() {
        output.push_str(&format!(
            "Removed artifacts: {}\n",
            delta.removed.join(", ")
        ));
    }

    output
}

#[derive(Debug, Clone, serde::Serialize)]
struct ArtifactDelta {
    added: Vec<String>,
    removed: Vec<String>,
}

fn artifact_delta(left: &[Artifact], right: &[Artifact]) -> ArtifactDelta {
    let left_titles: std::collections::BTreeSet<_> =
        left.iter().map(|item| item.title.clone()).collect();
    let right_titles: std::collections::BTreeSet<_> =
        right.iter().map(|item| item.title.clone()).collect();

    ArtifactDelta {
        added: right_titles.difference(&left_titles).cloned().collect(),
        removed: left_titles.difference(&right_titles).cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(title: &str) -> Artifact {
        Artifact {
            title: title.to_string(),
            slug: format!("slug-{title}"),
            artifact_type: None,
            file_size_bytes: None,
            is_public_page_enabled: false,
            expiring_download_url: None,
            public_install_page_url: None,
        }
    }

    #[test]
    fn test_artifact_delta_reports_added_and_removed() {
        let delta = artifact_delta(
            &[artifact("old.ipa"), artifact("shared.txt")],
            &[artifact("shared.txt"), artifact("new.ipa")],
        );

        assert_eq!(delta.added, vec!["new.ipa".to_string()]);
        assert_eq!(delta.removed, vec!["old.ipa".to_string()]);
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    #[test]
    fn performance_compare_four_requests_and_unavailable_artifact_visible() {
        let mut server = mockito::Server::new();
        let build = serde_json::json!({"data":{"slug":"build","triggered_at":"2026-01-01T00:00:00Z","status":1,"status_text":"success","branch":"main","build_number":1,"triggered_workflow":"primary"}}).to_string();
        let left = server
            .mock("GET", "/apps/a/builds/left")
            .with_body(&build)
            .expect(2)
            .create();
        let right = server
            .mock("GET", "/apps/a/builds/right")
            .with_body(&build)
            .expect(2)
            .create();
        let unavailable = server
            .mock("GET", "/apps/a/builds/left/artifacts")
            .with_status(403)
            .with_body("forbidden")
            .expect(2)
            .create();
        let available = server
            .mock("GET", "/apps/a/builds/right/artifacts")
            .with_body(
                r#"{"data":[],"paging":{"total_item_count":0,"page_item_limit":50,"next":null}}"#,
            )
            .expect(2)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let args = CompareArgs {
            app: Some("a".into()),
            left: "left".into(),
            right: "right".into(),
        };
        let start = std::time::Instant::now();
        let json: serde_json::Value = serde_json::from_str(
            &compare(&client, &Config::default(), &args, OutputFormat::Json).unwrap(),
        )
        .unwrap();
        assert!(json["artifact_delta"].is_null());
        assert!(json["artifact_errors"]["left"]
            .as_str()
            .unwrap()
            .contains("403"));
        let pretty = compare(&client, &Config::default(), &args, OutputFormat::Pretty).unwrap();
        assert!(pretty.contains("comparison unavailable"));
        assert!(!pretty.contains("Artifacts: 0 -> 0"));
        eprintln!(
            "performance_compare: two comparisons, 8 HTTP requests, {:?}",
            start.elapsed()
        );
        left.assert();
        right.assert();
        unavailable.assert();
        available.assert();
    }
}
