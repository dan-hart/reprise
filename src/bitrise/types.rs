use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response wrapper for app list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppListResponse {
    pub data: Vec<App>,
    pub paging: Paging,
}

/// Response wrapper for single app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppResponse {
    pub data: App,
}

/// Bitrise application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub slug: String,
    pub title: String,
    pub project_type: Option<String>,
    pub provider: Option<String>,
    pub repo_owner: Option<String>,
    pub repo_slug: Option<String>,
    pub repo_url: Option<String>,
    pub is_disabled: bool,
    pub status: i32,
    #[serde(rename = "isPublic", default)]
    pub is_public: bool,
    pub owner: Owner,
}

/// App owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub account_type: String,
    pub name: String,
    pub slug: String,
}

/// Response wrapper for build list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildListResponse {
    pub data: Vec<Build>,
    pub paging: Paging,
}

/// Response wrapper for single build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResponse {
    pub data: Build,
}

/// Bitrise build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub slug: String,
    pub triggered_at: DateTime<Utc>,
    pub started_on_worker_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: i32,
    pub status_text: String,
    pub abort_reason: Option<String>,
    pub branch: String,
    pub build_number: i64,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub tag: Option<String>,
    pub triggered_workflow: String,
    pub triggered_by: Option<String>,
    pub stack_identifier: Option<String>,
    pub machine_type_id: Option<String>,
    pub pull_request_id: Option<i64>,
    pub pull_request_target_branch: Option<String>,
    pub credit_cost: Option<i32>,
}

impl Build {
    /// Get a human-readable status string
    pub fn status_display(&self) -> &str {
        match self.status {
            0 => "running",
            1 => "success",
            2 => "failed",
            3 => "aborted",
            4 => "aborted-success",
            _ => "unknown",
        }
    }

    /// Calculate build duration if available
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_on_worker_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Format duration as human-readable string
    pub fn duration_display(&self) -> String {
        match self.duration() {
            Some(d) => {
                let secs = d.num_seconds();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                }
            }
            None => "-".to_string(),
        }
    }

    /// Check if build is still running
    pub fn is_running(&self) -> bool {
        self.status == 0
    }

    /// Check if build failed
    pub fn is_failed(&self) -> bool {
        self.status == 2
    }
}

/// Build log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResponse {
    pub log_chunks: Vec<LogChunk>,
    pub expiring_raw_log_url: Option<String>,
    pub is_archived: bool,
}

/// Individual log chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogChunk {
    pub chunk: String,
    pub position: i64,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub total_item_count: i64,
    pub page_item_limit: i64,
    pub next: Option<String>,
}

/// Parameters for triggering a build
#[derive(Debug, Clone, Default)]
pub struct TriggerParams {
    pub branch: Option<String>,
    pub workflow_id: String,
    pub commit_message: Option<String>,
    pub environments: Vec<(String, String)>,
}

/// Response from triggering a build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResponse {
    pub status: String,
    pub message: String,
    pub slug: Option<String>,
    pub build_slug: Option<String>,
    pub build_number: Option<i64>,
    pub build_url: Option<String>,
    pub triggered_workflow: Option<String>,
}

/// Response wrapper for artifact list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactListResponse {
    pub data: Vec<Artifact>,
    pub paging: Paging,
}

/// Response wrapper for single artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactResponse {
    pub data: Artifact,
}

/// Build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub title: String,
    pub slug: String,
    pub artifact_type: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub is_public_page_enabled: bool,
    pub expiring_download_url: Option<String>,
    pub public_install_page_url: Option<String>,
}

impl Artifact {
    /// Get human-readable file size
    pub fn size_display(&self) -> String {
        match self.file_size_bytes {
            Some(bytes) if bytes < 1024 => format!("{} B", bytes),
            Some(bytes) if bytes < 1024 * 1024 => format!("{:.1} KB", bytes as f64 / 1024.0),
            Some(bytes) => format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)),
            None => "-".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline Types
// ─────────────────────────────────────────────────────────────────────────────

/// Response wrapper for pipeline list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineListResponse {
    pub data: Vec<Pipeline>,
    pub paging: Paging,
}

/// Response wrapper for single pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResponse {
    pub data: Pipeline,
}

/// Bitrise pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(alias = "uuid", default)]
    pub id: String,
    #[serde(default)]
    pub app_slug: String,
    pub status: i32,
    #[serde(default)]
    pub status_text: Option<String>,
    pub triggered_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub branch: String,
    /// The pipeline definition name/ID
    #[serde(default)]
    pub pipeline_id: String,
    pub triggered_by: Option<String>,
    pub abort_reason: Option<String>,
    #[serde(default)]
    pub workflows: Vec<PipelineWorkflow>,
}

impl Pipeline {
    /// Get a human-readable status string
    pub fn status_display(&self) -> &str {
        match self.status {
            0 => "running",
            1 => "success",
            2 => "failed",
            3 => "aborted",
            4 => "aborted-success",
            _ => "unknown",
        }
    }

    /// Calculate pipeline duration if available
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Format duration as human-readable string
    pub fn duration_display(&self) -> String {
        match self.duration() {
            Some(d) => {
                let secs = d.num_seconds();
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                }
            }
            None => "-".to_string(),
        }
    }

    /// Check if pipeline is still running
    pub fn is_running(&self) -> bool {
        self.status == 0
    }

    /// Check if pipeline failed
    pub fn is_failed(&self) -> bool {
        self.status == 2
    }
}

/// Workflow within a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineWorkflow {
    #[serde(alias = "uuid", default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub status: i32,
    #[serde(default)]
    pub status_text: Option<String>,
}

impl PipelineWorkflow {
    /// Get a human-readable status string
    pub fn status_display(&self) -> &str {
        match self.status {
            0 => "running",
            1 => "success",
            2 => "failed",
            3 => "aborted",
            _ => "unknown",
        }
    }
}

/// Parameters for triggering a pipeline
#[derive(Debug, Clone, Default)]
pub struct PipelineTriggerParams {
    pub pipeline_id: String,
    pub branch: Option<String>,
    pub environments: Vec<(String, String)>,
}

/// Response from triggering a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTriggerResponse {
    pub status: String,
    pub message: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub pipeline_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// User Types
// ─────────────────────────────────────────────────────────────────────────────

/// Response wrapper for current user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub data: User,
}

/// Current authenticated user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub slug: String,
    pub email: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ─────────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_build(status: i32, started: Option<DateTime<Utc>>, finished: Option<DateTime<Utc>>) -> Build {
        Build {
            slug: "test-slug".to_string(),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_on_worker_at: started,
            finished_at: finished,
            status,
            status_text: "test".to_string(),
            abort_reason: None,
            branch: "main".to_string(),
            build_number: 1,
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

    fn make_pipeline(status: i32, started: Option<DateTime<Utc>>, finished: Option<DateTime<Utc>>) -> Pipeline {
        Pipeline {
            id: "test-id".to_string(),
            app_slug: "test-app".to_string(),
            status,
            status_text: Some("test".to_string()),
            triggered_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            started_at: started,
            finished_at: finished,
            branch: "main".to_string(),
            pipeline_id: "test-pipeline".to_string(),
            triggered_by: None,
            abort_reason: None,
            workflows: vec![],
        }
    }

    fn make_artifact(size: Option<i64>) -> Artifact {
        Artifact {
            title: "test.ipa".to_string(),
            slug: "artifact-slug".to_string(),
            artifact_type: Some("file".to_string()),
            file_size_bytes: size,
            is_public_page_enabled: false,
            expiring_download_url: None,
            public_install_page_url: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Build Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_status_display_running() {
        let build = make_build(0, None, None);
        assert_eq!(build.status_display(), "running");
    }

    #[test]
    fn test_build_status_display_success() {
        let build = make_build(1, None, None);
        assert_eq!(build.status_display(), "success");
    }

    #[test]
    fn test_build_status_display_failed() {
        let build = make_build(2, None, None);
        assert_eq!(build.status_display(), "failed");
    }

    #[test]
    fn test_build_status_display_aborted() {
        let build = make_build(3, None, None);
        assert_eq!(build.status_display(), "aborted");
    }

    #[test]
    fn test_build_status_display_aborted_success() {
        let build = make_build(4, None, None);
        assert_eq!(build.status_display(), "aborted-success");
    }

    #[test]
    fn test_build_status_display_unknown() {
        let build = make_build(99, None, None);
        assert_eq!(build.status_display(), "unknown");
    }

    #[test]
    fn test_build_duration_with_timestamps() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 5, 30).unwrap();
        let build = make_build(1, Some(start), Some(end));

        let duration = build.duration().unwrap();
        assert_eq!(duration.num_seconds(), 330); // 5m 30s
    }

    #[test]
    fn test_build_duration_without_start() {
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 5, 30).unwrap();
        let build = make_build(1, None, Some(end));
        assert!(build.duration().is_none());
    }

    #[test]
    fn test_build_duration_without_end() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let build = make_build(0, Some(start), None);
        assert!(build.duration().is_none());
    }

    #[test]
    fn test_build_duration_display_seconds() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 45).unwrap();
        let build = make_build(1, Some(start), Some(end));
        assert_eq!(build.duration_display(), "45s");
    }

    #[test]
    fn test_build_duration_display_minutes() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 5, 30).unwrap();
        let build = make_build(1, Some(start), Some(end));
        assert_eq!(build.duration_display(), "5m 30s");
    }

    #[test]
    fn test_build_duration_display_hours() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 14, 30, 0).unwrap();
        let build = make_build(1, Some(start), Some(end));
        assert_eq!(build.duration_display(), "2h 30m");
    }

    #[test]
    fn test_build_duration_display_no_duration() {
        let build = make_build(0, None, None);
        assert_eq!(build.duration_display(), "-");
    }

    #[test]
    fn test_build_is_running_true() {
        let build = make_build(0, None, None);
        assert!(build.is_running());
    }

    #[test]
    fn test_build_is_running_false() {
        let build = make_build(1, None, None);
        assert!(!build.is_running());
    }

    #[test]
    fn test_build_is_failed_true() {
        let build = make_build(2, None, None);
        assert!(build.is_failed());
    }

    #[test]
    fn test_build_is_failed_false() {
        let build = make_build(1, None, None);
        assert!(!build.is_failed());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_pipeline_status_display_running() {
        let pipeline = make_pipeline(0, None, None);
        assert_eq!(pipeline.status_display(), "running");
    }

    #[test]
    fn test_pipeline_status_display_success() {
        let pipeline = make_pipeline(1, None, None);
        assert_eq!(pipeline.status_display(), "success");
    }

    #[test]
    fn test_pipeline_status_display_failed() {
        let pipeline = make_pipeline(2, None, None);
        assert_eq!(pipeline.status_display(), "failed");
    }

    #[test]
    fn test_pipeline_status_display_aborted() {
        let pipeline = make_pipeline(3, None, None);
        assert_eq!(pipeline.status_display(), "aborted");
    }

    #[test]
    fn test_pipeline_status_display_aborted_success() {
        let pipeline = make_pipeline(4, None, None);
        assert_eq!(pipeline.status_display(), "aborted-success");
    }

    #[test]
    fn test_pipeline_status_display_unknown() {
        let pipeline = make_pipeline(99, None, None);
        assert_eq!(pipeline.status_display(), "unknown");
    }

    #[test]
    fn test_pipeline_duration_with_timestamps() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 10, 0).unwrap();
        let pipeline = make_pipeline(1, Some(start), Some(end));

        let duration = pipeline.duration().unwrap();
        assert_eq!(duration.num_seconds(), 600); // 10 minutes
    }

    #[test]
    fn test_pipeline_duration_without_timestamps() {
        let pipeline = make_pipeline(0, None, None);
        assert!(pipeline.duration().is_none());
    }

    #[test]
    fn test_pipeline_duration_display_seconds() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 30).unwrap();
        let pipeline = make_pipeline(1, Some(start), Some(end));
        assert_eq!(pipeline.duration_display(), "30s");
    }

    #[test]
    fn test_pipeline_duration_display_minutes() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 15, 45).unwrap();
        let pipeline = make_pipeline(1, Some(start), Some(end));
        assert_eq!(pipeline.duration_display(), "15m 45s");
    }

    #[test]
    fn test_pipeline_duration_display_hours() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 15, 20, 0).unwrap();
        let pipeline = make_pipeline(1, Some(start), Some(end));
        assert_eq!(pipeline.duration_display(), "3h 20m");
    }

    #[test]
    fn test_pipeline_duration_display_no_duration() {
        let pipeline = make_pipeline(0, None, None);
        assert_eq!(pipeline.duration_display(), "-");
    }

    #[test]
    fn test_pipeline_is_running_true() {
        let pipeline = make_pipeline(0, None, None);
        assert!(pipeline.is_running());
    }

    #[test]
    fn test_pipeline_is_running_false() {
        let pipeline = make_pipeline(1, None, None);
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_pipeline_is_failed_true() {
        let pipeline = make_pipeline(2, None, None);
        assert!(pipeline.is_failed());
    }

    #[test]
    fn test_pipeline_is_failed_false() {
        let pipeline = make_pipeline(1, None, None);
        assert!(!pipeline.is_failed());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PipelineWorkflow Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_pipeline_workflow_status_display_running() {
        let wf = PipelineWorkflow {
            id: "wf-id".to_string(),
            name: "build".to_string(),
            status: 0,
            status_text: Some("running".to_string()),
        };
        assert_eq!(wf.status_display(), "running");
    }

    #[test]
    fn test_pipeline_workflow_status_display_success() {
        let wf = PipelineWorkflow {
            id: "wf-id".to_string(),
            name: "build".to_string(),
            status: 1,
            status_text: Some("success".to_string()),
        };
        assert_eq!(wf.status_display(), "success");
    }

    #[test]
    fn test_pipeline_workflow_status_display_failed() {
        let wf = PipelineWorkflow {
            id: "wf-id".to_string(),
            name: "build".to_string(),
            status: 2,
            status_text: Some("failed".to_string()),
        };
        assert_eq!(wf.status_display(), "failed");
    }

    #[test]
    fn test_pipeline_workflow_status_display_aborted() {
        let wf = PipelineWorkflow {
            id: "wf-id".to_string(),
            name: "build".to_string(),
            status: 3,
            status_text: Some("aborted".to_string()),
        };
        assert_eq!(wf.status_display(), "aborted");
    }

    #[test]
    fn test_pipeline_workflow_status_display_unknown() {
        let wf = PipelineWorkflow {
            id: "wf-id".to_string(),
            name: "build".to_string(),
            status: 99,
            status_text: Some("unknown".to_string()),
        };
        assert_eq!(wf.status_display(), "unknown");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Artifact Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_artifact_size_display_bytes() {
        let artifact = make_artifact(Some(500));
        assert_eq!(artifact.size_display(), "500 B");
    }

    #[test]
    fn test_artifact_size_display_kilobytes() {
        let artifact = make_artifact(Some(2048));
        assert_eq!(artifact.size_display(), "2.0 KB");
    }

    #[test]
    fn test_artifact_size_display_megabytes() {
        let artifact = make_artifact(Some(5 * 1024 * 1024));
        assert_eq!(artifact.size_display(), "5.0 MB");
    }

    #[test]
    fn test_artifact_size_display_fractional_kb() {
        let artifact = make_artifact(Some(1536)); // 1.5 KB
        assert_eq!(artifact.size_display(), "1.5 KB");
    }

    #[test]
    fn test_artifact_size_display_fractional_mb() {
        let artifact = make_artifact(Some(3 * 1024 * 1024 + 512 * 1024)); // 3.5 MB
        assert_eq!(artifact.size_display(), "3.5 MB");
    }

    #[test]
    fn test_artifact_size_display_none() {
        let artifact = make_artifact(None);
        assert_eq!(artifact.size_display(), "-");
    }

    #[test]
    fn test_artifact_size_display_zero() {
        let artifact = make_artifact(Some(0));
        assert_eq!(artifact.size_display(), "0 B");
    }

    #[test]
    fn test_artifact_size_display_boundary_1kb() {
        let artifact = make_artifact(Some(1023));
        assert_eq!(artifact.size_display(), "1023 B");

        let artifact = make_artifact(Some(1024));
        assert_eq!(artifact.size_display(), "1.0 KB");
    }

    #[test]
    fn test_artifact_size_display_boundary_1mb() {
        let artifact = make_artifact(Some(1024 * 1024 - 1));
        assert!(artifact.size_display().contains("KB"));

        let artifact = make_artifact(Some(1024 * 1024));
        assert_eq!(artifact.size_display(), "1.0 MB");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Serialization Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_app_deserialize_with_is_public() {
        let json = r#"{
            "slug": "abc123",
            "title": "My App",
            "is_disabled": false,
            "status": 1,
            "isPublic": true,
            "owner": {
                "account_type": "user",
                "name": "testuser",
                "slug": "user123"
            }
        }"#;

        let app: App = serde_json::from_str(json).unwrap();
        assert_eq!(app.slug, "abc123");
        assert!(app.is_public);
    }

    #[test]
    fn test_app_deserialize_without_is_public() {
        let json = r#"{
            "slug": "abc123",
            "title": "My App",
            "is_disabled": false,
            "status": 1,
            "owner": {
                "account_type": "user",
                "name": "testuser",
                "slug": "user123"
            }
        }"#;

        let app: App = serde_json::from_str(json).unwrap();
        assert!(!app.is_public); // defaults to false
    }

    #[test]
    fn test_pipeline_deserialize_with_uuid_alias() {
        let json = r#"{
            "uuid": "pipeline-uuid-123",
            "app_slug": "app123",
            "status": 1,
            "status_text": "success",
            "triggered_at": "2024-01-01T12:00:00Z",
            "branch": "main",
            "pipeline_id": "test-pipeline"
        }"#;

        let pipeline: Pipeline = serde_json::from_str(json).unwrap();
        assert_eq!(pipeline.id, "pipeline-uuid-123");
    }

    #[test]
    fn test_build_serialize_roundtrip() {
        let build = make_build(1, None, None);
        let json = serde_json::to_string(&build).unwrap();
        let deserialized: Build = serde_json::from_str(&json).unwrap();

        assert_eq!(build.slug, deserialized.slug);
        assert_eq!(build.status, deserialized.status);
        assert_eq!(build.branch, deserialized.branch);
    }
}
