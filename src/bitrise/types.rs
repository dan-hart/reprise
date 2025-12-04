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
