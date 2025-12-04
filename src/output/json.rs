use serde::Serialize;

use crate::bitrise::{App, Artifact, Build, Pipeline};
use crate::error::Result;

/// Format apps as JSON
pub fn format_apps(apps: &[App]) -> Result<String> {
    Ok(serde_json::to_string_pretty(apps)?)
}

/// Format a single app as JSON
pub fn format_app(app: &App) -> Result<String> {
    Ok(serde_json::to_string_pretty(app)?)
}

/// Format builds as JSON
pub fn format_builds(builds: &[Build]) -> Result<String> {
    Ok(serde_json::to_string_pretty(builds)?)
}

/// Format a single build as JSON
pub fn format_build(build: &Build) -> Result<String> {
    Ok(serde_json::to_string_pretty(build)?)
}

/// Format pipelines as JSON
pub fn format_pipelines(pipelines: &[Pipeline]) -> Result<String> {
    Ok(serde_json::to_string_pretty(pipelines)?)
}

/// Format a single pipeline as JSON
pub fn format_pipeline(pipeline: &Pipeline) -> Result<String> {
    Ok(serde_json::to_string_pretty(pipeline)?)
}

/// Format any serializable value as JSON
pub fn format_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

/// Format artifacts as JSON
pub fn format_artifacts(artifacts: &[Artifact]) -> Result<String> {
    Ok(serde_json::to_string_pretty(artifacts)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitrise::Owner;
    use chrono::{TimeZone, Utc};

    // ─────────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_test_app(slug: &str, title: &str) -> App {
        App {
            slug: slug.to_string(),
            title: title.to_string(),
            project_type: Some("ios".to_string()),
            provider: Some("github".to_string()),
            repo_owner: Some("testowner".to_string()),
            repo_slug: Some("testrepo".to_string()),
            repo_url: Some("https://github.com/test/repo".to_string()),
            is_disabled: false,
            status: 1,
            is_public: false,
            owner: Owner {
                account_type: "user".to_string(),
                name: "Test User".to_string(),
                slug: "user-slug".to_string(),
            },
        }
    }

    fn make_test_build(slug: &str, build_number: i64) -> Build {
        Build {
            slug: slug.to_string(),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_on_worker_at: None,
            finished_at: None,
            status: 1,
            status_text: "success".to_string(),
            abort_reason: None,
            branch: "main".to_string(),
            build_number,
            commit_hash: None,
            commit_message: None,
            tag: None,
            triggered_workflow: "primary".to_string(),
            triggered_by: None,
            stack_identifier: None,
            machine_type_id: None,
            pull_request_id: None,
            pull_request_target_branch: None,
            credit_cost: None,
        }
    }

    fn make_test_pipeline(id: &str) -> Pipeline {
        Pipeline {
            id: id.to_string(),
            app_slug: "test-app".to_string(),
            status: 1,
            status_text: Some("success".to_string()),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_at: None,
            finished_at: None,
            branch: "main".to_string(),
            pipeline_id: "build-and-test".to_string(),
            triggered_by: None,
            abort_reason: None,
            workflows: vec![],
        }
    }

    fn make_test_artifact(slug: &str, title: &str) -> Artifact {
        Artifact {
            title: title.to_string(),
            slug: slug.to_string(),
            artifact_type: Some("file".to_string()),
            file_size_bytes: Some(1024),
            is_public_page_enabled: false,
            expiring_download_url: None,
            public_install_page_url: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_apps Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_apps_empty() {
        let result = format_apps(&[]).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_format_apps_valid_json() {
        let apps = vec![make_test_app("slug1", "My App")];
        let result = format_apps(&apps).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_format_apps_contains_fields() {
        let apps = vec![make_test_app("test-slug", "Test App")];
        let result = format_apps(&apps).unwrap();
        assert!(result.contains("\"slug\": \"test-slug\""));
        assert!(result.contains("\"title\": \"Test App\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_app Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_app_valid_json() {
        let app = make_test_app("slug1", "My App");
        let result = format_app(&app).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["slug"], "slug1");
    }

    #[test]
    fn test_format_app_contains_all_fields() {
        let app = make_test_app("test-slug", "Test App");
        let result = format_app(&app).unwrap();
        assert!(result.contains("\"slug\""));
        assert!(result.contains("\"title\""));
        assert!(result.contains("\"owner\""));
        assert!(result.contains("\"is_disabled\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_builds Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_builds_empty() {
        let result = format_builds(&[]).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_format_builds_valid_json() {
        let builds = vec![make_test_build("slug1", 123)];
        let result = format_builds(&builds).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_format_builds_contains_fields() {
        let builds = vec![make_test_build("build-slug", 456)];
        let result = format_builds(&builds).unwrap();
        assert!(result.contains("\"slug\": \"build-slug\""));
        assert!(result.contains("\"build_number\": 456"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_build Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_build_valid_json() {
        let build = make_test_build("slug1", 789);
        let result = format_build(&build).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["build_number"], 789);
    }

    #[test]
    fn test_format_build_contains_all_fields() {
        let build = make_test_build("test-slug", 1);
        let result = format_build(&build).unwrap();
        assert!(result.contains("\"slug\""));
        assert!(result.contains("\"branch\""));
        assert!(result.contains("\"triggered_workflow\""));
        assert!(result.contains("\"status\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_pipelines Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_pipelines_empty() {
        let result = format_pipelines(&[]).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_format_pipelines_valid_json() {
        let pipelines = vec![make_test_pipeline("pipeline-id")];
        let result = format_pipelines(&pipelines).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_pipeline Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_pipeline_valid_json() {
        let pipeline = make_test_pipeline("test-id");
        let result = format_pipeline(&pipeline).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["id"], "test-id");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_artifacts Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_artifacts_empty() {
        let result = format_artifacts(&[]).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_format_artifacts_valid_json() {
        let artifacts = vec![make_test_artifact("art-slug", "my-app.ipa")];
        let result = format_artifacts(&artifacts).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_format_artifacts_contains_fields() {
        let artifacts = vec![make_test_artifact("artifact-123", "test.ipa")];
        let result = format_artifacts(&artifacts).unwrap();
        assert!(result.contains("\"slug\": \"artifact-123\""));
        assert!(result.contains("\"title\": \"test.ipa\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // format_json Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_json_simple_struct() {
        #[derive(Serialize)]
        struct Simple {
            name: String,
            count: i32,
        }

        let value = Simple {
            name: "test".to_string(),
            count: 42,
        };
        let result = format_json(&value).unwrap();
        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"count\": 42"));
    }

    #[test]
    fn test_format_json_vec() {
        let values = vec![1, 2, 3];
        let result = format_json(&values).unwrap();
        let parsed: Vec<i32> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, vec![1, 2, 3]);
    }

    #[test]
    fn test_format_json_hashmap() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("key", "value");
        let result = format_json(&map).unwrap();
        assert!(result.contains("\"key\": \"value\""));
    }
}
