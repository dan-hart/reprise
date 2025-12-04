use reqwest::blocking::Client;
use std::time::Duration;
use url::Url;

use super::types::*;
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Allowed hosts for external URL fetching (SSRF protection)
const ALLOWED_HOSTS: &[&str] = &[
    "bitrise.io",
    "app.bitrise.io",
    "bitrise-build-log-archives.s3.amazonaws.com",
    "bitrise-build-log-archives-eu-west-1.s3.eu-west-1.amazonaws.com",
    // Artifact download hosts (S3 buckets)
    "bitrise-prod-build-storage.s3.amazonaws.com",
    "bitrise-prod-build-storage.s3.us-west-2.amazonaws.com",
    "amazonaws.com",
];

const BASE_URL: &str = "https://api.bitrise.io/v0.1";
const USER_AGENT: &str = concat!("reprise/", env!("CARGO_PKG_VERSION"));

/// Bitrise API client
pub struct BitriseClient {
    client: Client,
    token: String,
}

impl BitriseClient {
    /// Create a new client from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let token = config.require_token()?.to_string();

        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client, token })
    }

    /// Create a new client with an explicit token
    pub fn with_token(token: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            token: token.into(),
        })
    }

    /// Make a GET request to the Bitrise API
    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{BASE_URL}{path}");
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.token)
            .send()?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().unwrap_or_default();
            return Err(RepriseError::api(status.as_u16(), message));
        }

        let body = response.text()?;
        serde_json::from_str(&body).map_err(|e| {
            RepriseError::Json(e)
        })
    }

    /// Fetch raw content from a URL (for log files)
    fn get_raw(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send()?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().unwrap_or_default();
            return Err(RepriseError::api(status.as_u16(), message));
        }

        Ok(response.text()?)
    }

    /// Make a POST request to the Bitrise API
    fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{BASE_URL}{path}");
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.token)
            .json(body)
            .send()?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().unwrap_or_default();
            return Err(RepriseError::api(status.as_u16(), message));
        }

        let body = response.text()?;
        serde_json::from_str(&body).map_err(RepriseError::Json)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // App Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List all accessible apps
    pub fn list_apps(&self, limit: u32) -> Result<AppListResponse> {
        self.get(&format!("/apps?limit={limit}"))
    }

    /// Get a specific app by slug
    pub fn get_app(&self, slug: &str) -> Result<AppResponse> {
        self.get(&format!("/apps/{slug}"))
    }

    /// Find an app by name (partial match)
    pub fn find_app_by_name(&self, name: &str) -> Result<Option<App>> {
        let response = self.list_apps(100)?;
        let name_lower = name.to_lowercase();

        Ok(response
            .data
            .into_iter()
            .find(|app| app.title.to_lowercase().contains(&name_lower)))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Build Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List builds for an app with optional filters
    pub fn list_builds(
        &self,
        app_slug: &str,
        status: Option<i32>,
        branch: Option<&str>,
        workflow: Option<&str>,
        limit: u32,
    ) -> Result<BuildListResponse> {
        // Use proper URL encoding for query parameters
        let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];

        if let Some(s) = status {
            params.push(("status", s.to_string()));
        }
        if let Some(b) = branch {
            params.push(("branch", b.to_string()));
        }
        if let Some(w) = workflow {
            params.push(("workflow", w.to_string()));
        }

        let query: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.get(&format!("/apps/{app_slug}/builds?{query}"))
    }

    /// Get a specific build
    pub fn get_build(&self, app_slug: &str, build_slug: &str) -> Result<BuildResponse> {
        self.get(&format!("/apps/{app_slug}/builds/{build_slug}"))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Log Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Get build log metadata (including expiring URL)
    pub fn get_build_log(&self, app_slug: &str, build_slug: &str) -> Result<LogResponse> {
        self.get(&format!("/apps/{app_slug}/builds/{build_slug}/log"))
    }

    /// Validate a URL is from an allowed host (SSRF protection)
    fn validate_external_url(&self, url: &str, purpose: &str) -> Result<()> {
        let parsed_url = Url::parse(url).map_err(|_| {
            RepriseError::InvalidArgument(format!("Invalid {} URL: {}", purpose, url))
        })?;

        let host = parsed_url
            .host_str()
            .filter(|h| !h.is_empty())
            .ok_or_else(|| {
                RepriseError::InvalidArgument(format!(
                    "{} URL has no valid host: {}",
                    purpose, url
                ))
            })?;

        let is_allowed = ALLOWED_HOSTS
            .iter()
            .any(|allowed| host == *allowed || host.ends_with(&format!(".{}", allowed)));

        if !is_allowed {
            return Err(RepriseError::InvalidArgument(format!(
                "{} URL from untrusted host: {}",
                purpose, host
            )));
        }

        Ok(())
    }

    /// Fetch the full raw log content
    ///
    /// Validates that the URL is from an allowed Bitrise domain to prevent SSRF.
    pub fn fetch_raw_log(&self, log_url: &str) -> Result<String> {
        self.validate_external_url(log_url, "Log")?;
        self.get_raw(log_url)
    }

    /// Get the full log for a build
    pub fn get_full_log(&self, app_slug: &str, build_slug: &str) -> Result<String> {
        let log_response = self.get_build_log(app_slug, build_slug)?;

        match log_response.expiring_raw_log_url {
            Some(url) => self.fetch_raw_log(&url),
            None => {
                // Fall back to log chunks if no raw URL available
                let log = log_response
                    .log_chunks
                    .iter()
                    .map(|c| c.chunk.as_str())
                    .collect::<Vec<_>>()
                    .join("");
                Ok(log)
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Build Trigger Operations
    // ─────────────────────────────────────────────────────────────────────────

    // ─────────────────────────────────────────────────────────────────────────
    // Artifact Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List artifacts for a build
    pub fn list_artifacts(&self, app_slug: &str, build_slug: &str) -> Result<ArtifactListResponse> {
        self.get(&format!("/apps/{app_slug}/builds/{build_slug}/artifacts"))
    }

    /// Get a specific artifact with download URL
    pub fn get_artifact(
        &self,
        app_slug: &str,
        build_slug: &str,
        artifact_slug: &str,
    ) -> Result<ArtifactResponse> {
        self.get(&format!(
            "/apps/{app_slug}/builds/{build_slug}/artifacts/{artifact_slug}"
        ))
    }

    /// Download an artifact to a file
    ///
    /// Validates that the URL is from an allowed host to prevent SSRF attacks.
    pub fn download_artifact(&self, url: &str, path: &std::path::Path) -> Result<()> {
        // Validate URL is from allowed hosts (SSRF protection)
        self.validate_external_url(url, "Artifact")?;

        let response = self.client.get(url).send()?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().unwrap_or_default();
            return Err(RepriseError::api(status.as_u16(), message));
        }

        let bytes = response.bytes()?;
        std::fs::write(path, &bytes)?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Build Trigger Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Abort a running build
    pub fn abort_build(
        &self,
        app_slug: &str,
        build_slug: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let body = serde_json::json!({
            "abort_reason": reason.unwrap_or("Aborted via reprise CLI"),
            "abort_with_success": false,
            "skip_notifications": false,
        });

        let _: serde_json::Value = self.post(
            &format!("/apps/{app_slug}/builds/{build_slug}/abort"),
            &body,
        )?;

        Ok(())
    }

    /// Trigger a new build
    pub fn trigger_build(&self, app_slug: &str, params: TriggerParams) -> Result<Build> {
        // Build the request body according to Bitrise API spec
        let mut build_params = serde_json::json!({
            "workflow_id": params.workflow_id,
        });

        if let Some(ref branch) = params.branch {
            build_params["branch"] = serde_json::json!(branch);
        }

        if let Some(ref msg) = params.commit_message {
            build_params["commit_message"] = serde_json::json!(msg);
        }

        if !params.environments.is_empty() {
            let envs: Vec<_> = params
                .environments
                .iter()
                .map(|(k, v)| {
                    serde_json::json!({
                        "mapped_to": k,
                        "value": v,
                        "is_expand": true,
                    })
                })
                .collect();
            build_params["environments"] = serde_json::json!(envs);
        }

        let body = serde_json::json!({
            "hook_info": {
                "type": "bitrise",
            },
            "build_params": build_params,
        });

        let response: TriggerResponse = self.post(&format!("/apps/{app_slug}/builds"), &body)?;

        // Get the build details to return full Build object
        if let Some(ref build_slug) = response.build_slug {
            let build_response = self.get_build(app_slug, build_slug)?;
            Ok(build_response.data)
        } else {
            Err(RepriseError::Api {
                status: 500,
                message: format!("Build triggered but no slug returned: {}", response.message),
            })
        }
    }
}
