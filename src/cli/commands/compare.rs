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
    let left = client.get_build(app_slug, &args.left)?.data;
    let right = client.get_build(app_slug, &args.right)?.data;
    let left_artifacts = client
        .list_artifacts(app_slug, &args.left)
        .map(|response| response.data)
        .unwrap_or_default();
    let right_artifacts = client
        .list_artifacts(app_slug, &args.right)
        .map(|response| response.data)
        .unwrap_or_default();

    match format {
        OutputFormat::Pretty => Ok(format_pretty(
            &left,
            &right,
            &left_artifacts,
            &right_artifacts,
        )),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "left": left,
                "right": right,
                "artifact_delta": artifact_delta(&left_artifacts, &right_artifacts),
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
