use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::time::Duration;
use url::Url;

use super::types::*;
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Allowed hosts for external URL fetching (SSRF protection)
const ALLOWED_HOSTS: &[&str] = &[
    "bitrise.io",
    "app.bitrise.io",
    // Log archive hosts (S3 buckets)
    "bitrise-build-log-archives.s3.amazonaws.com",
    "bitrise-build-log-archives-eu-west-1.s3.eu-west-1.amazonaws.com",
    // Artifact download hosts (S3 buckets)
    "bitrise-prod-build-storage.s3.amazonaws.com",
    "bitrise-prod-build-storage.s3.us-west-2.amazonaws.com",
    // Google Cloud Storage (used by Bitrise for logs)
    "storage.googleapis.com",
];

const DEFAULT_BASE_URL: &str = "https://api.bitrise.io/v0.1";
const USER_AGENT: &str = concat!("reprise/", env!("CARGO_PKG_VERSION"));

/// Bitrise API client
pub struct BitriseClient {
    client: Client,
    token: String,
    base_url: String,
}

impl BitriseClient {
    /// Create a new client from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let token = config.require_token()?.to_string();

        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(Policy::limited(5))
            .build()?;

        Ok(Self {
            client,
            token,
            base_url: DEFAULT_BASE_URL.to_string(),
        })
    }

    /// Create a new client with an explicit token
    pub fn with_token(token: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(Policy::limited(5))
            .build()?;

        Ok(Self {
            client,
            token: token.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        })
    }

    /// Create a new client with custom base URL (for testing)
    #[cfg(test)]
    pub fn with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(Policy::limited(5))
            .build()?;

        Ok(Self {
            client,
            token: token.into(),
            base_url: base_url.into(),
        })
    }

    /// Make a GET request to the Bitrise API
    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
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
        let url = format!("{}{path}", self.base_url);
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
    // User Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the current authenticated user
    pub fn get_me(&self) -> Result<UserResponse> {
        self.get("/me")
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

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// List pipelines for an app with optional filters
    pub fn list_pipelines(
        &self,
        app_slug: &str,
        status: Option<i32>,
        branch: Option<&str>,
        limit: u32,
    ) -> Result<PipelineListResponse> {
        let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];

        if let Some(s) = status {
            params.push(("status", s.to_string()));
        }
        if let Some(b) = branch {
            params.push(("branch", b.to_string()));
        }

        let query: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        self.get(&format!("/apps/{app_slug}/pipelines?{query}"))
    }

    /// Get a specific pipeline
    pub fn get_pipeline(&self, app_slug: &str, pipeline_id: &str) -> Result<PipelineResponse> {
        // Get raw response to handle different API formats
        let raw: serde_json::Value = self.get(&format!("/apps/{app_slug}/pipelines/{pipeline_id}"))?;

        // Try to parse as wrapped format first, then as direct Pipeline
        if raw.get("data").is_some() {
            serde_json::from_value(raw).map_err(RepriseError::Json)
        } else {
            // Direct pipeline object - wrap it
            let pipeline: Pipeline = serde_json::from_value(raw).map_err(RepriseError::Json)?;
            Ok(PipelineResponse::Unwrapped(pipeline))
        }
    }

    /// Trigger a new pipeline
    pub fn trigger_pipeline(
        &self,
        app_slug: &str,
        params: PipelineTriggerParams,
    ) -> Result<Pipeline> {
        let mut build_params = serde_json::json!({
            "pipeline_id": params.pipeline_id,
        });

        if let Some(ref branch) = params.branch {
            build_params["branch"] = serde_json::json!(branch);
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

        let response: PipelineTriggerResponse =
            self.post(&format!("/apps/{app_slug}/pipelines"), &body)?;

        // Get the pipeline details to return full Pipeline object
        if let Some(ref id) = response.id {
            let pipeline_response = self.get_pipeline(app_slug, id)?;
            Ok(pipeline_response.into_pipeline())
        } else {
            Err(RepriseError::Api {
                status: 500,
                message: format!(
                    "Pipeline triggered but no ID returned: {}",
                    response.message
                ),
            })
        }
    }

    /// Abort a running pipeline
    pub fn abort_pipeline(
        &self,
        app_slug: &str,
        pipeline_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let body = serde_json::json!({
            "abort_reason": reason.unwrap_or("Aborted via reprise CLI"),
            "abort_with_success": false,
            "skip_notifications": false,
        });

        let _: serde_json::Value = self.post(
            &format!("/apps/{app_slug}/pipelines/{pipeline_id}/abort"),
            &body,
        )?;

        Ok(())
    }

    /// Rebuild a pipeline
    pub fn rebuild_pipeline(
        &self,
        app_slug: &str,
        pipeline_id: &str,
        partial: bool,
    ) -> Result<Pipeline> {
        let body = serde_json::json!({
            "partial": partial,
        });

        let response: PipelineTriggerResponse = self.post(
            &format!("/apps/{app_slug}/pipelines/{pipeline_id}/rebuild"),
            &body,
        )?;

        // Get the pipeline details to return full Pipeline object
        if let Some(ref id) = response.id {
            let pipeline_response = self.get_pipeline(app_slug, id)?;
            Ok(pipeline_response.into_pipeline())
        } else {
            // If no new ID, fetch the original pipeline
            let pipeline_response = self.get_pipeline(app_slug, pipeline_id)?;
            Ok(pipeline_response.into_pipeline())
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    // ─────────────────────────────────────────────────────────────────────────
    // Test Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_app_json(slug: &str, title: &str) -> String {
        format!(
            r#"{{
                "slug": "{}",
                "title": "{}",
                "project_type": "ios",
                "provider": "github",
                "is_disabled": false,
                "status": 1,
                "isPublic": false,
                "owner": {{
                    "account_type": "user",
                    "name": "Test User",
                    "slug": "user-slug"
                }}
            }}"#,
            slug, title
        )
    }

    fn make_build_json(slug: &str, build_number: i64, status: i32) -> String {
        format!(
            r#"{{
                "slug": "{}",
                "build_number": {},
                "status": {},
                "status_text": "success",
                "triggered_at": "2024-01-01T12:00:00Z",
                "branch": "main",
                "triggered_workflow": "primary"
            }}"#,
            slug, build_number, status
        )
    }

    fn make_pipeline_json(id: &str, status: i32) -> String {
        format!(
            r#"{{
                "id": "{}",
                "app_slug": "test-app",
                "status": {},
                "status_text": "success",
                "triggered_at": "2024-01-01T12:00:00Z",
                "branch": "main",
                "pipeline_id": "build-and-test",
                "workflows": []
            }}"#,
            id, status
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // User Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_me_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/me")
            .match_header("Authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": {"username": "testuser", "slug": "user123", "email": "test@example.com"}}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_me();

        mock.assert();
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.data.username, "testuser");
        assert_eq!(user.data.slug, "user123");
    }

    #[test]
    fn test_get_me_unauthorized() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/me")
            .with_status(401)
            .with_body(r#"{"message": "Unauthorized"}"#)
            .create();

        let client = BitriseClient::with_base_url("bad-token", server.url()).unwrap();
        let result = client.get_me();

        mock.assert();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 77); // EX_NOPERM
    }

    // ─────────────────────────────────────────────────────────────────────────
    // App Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_apps_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps?limit=10")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"data": [{}], "paging": {{"total_item_count": 1, "page_item_limit": 10, "next": null}}}}"#,
                make_app_json("app-slug", "Test App")
            ))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_apps(10);

        mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].slug, "app-slug");
        assert_eq!(response.data[0].title, "Test App");
    }

    #[test]
    fn test_list_apps_empty() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps?limit=10")
            .with_status(200)
            .with_body(r#"{"data": [], "paging": {"total_item_count": 0, "page_item_limit": 10, "next": null}}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_apps(10);

        mock.assert();
        assert!(result.is_ok());
        assert!(result.unwrap().data.is_empty());
    }

    #[test]
    fn test_get_app_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/my-app")
            .with_status(200)
            .with_body(format!(r#"{{"data": {}}}"#, make_app_json("my-app", "My App")))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_app("my-app");

        mock.assert();
        assert!(result.is_ok());
        let app = result.unwrap();
        assert_eq!(app.data.slug, "my-app");
        assert_eq!(app.data.title, "My App");
    }

    #[test]
    fn test_get_app_not_found() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/nonexistent")
            .with_status(404)
            .with_body(r#"{"message": "Not found"}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_app("nonexistent");

        mock.assert();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 66); // EX_NOINPUT
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Build Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_builds_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/builds?limit=10")
            .with_status(200)
            .with_body(format!(
                r#"{{"data": [{}], "paging": {{"total_item_count": 1, "page_item_limit": 10, "next": null}}}}"#,
                make_build_json("build-123", 1, 1)
            ))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_builds("test-app", None, None, None, 10);

        mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].slug, "build-123");
    }

    #[test]
    fn test_list_builds_with_filters() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", Matcher::Regex(r"/apps/test-app/builds\?.*status=1.*".to_string()))
            .with_status(200)
            .with_body(r#"{"data": [], "paging": {"total_item_count": 0, "page_item_limit": 10, "next": null}}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_builds("test-app", Some(1), Some("main"), None, 10);

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_build_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/builds/build-slug")
            .with_status(200)
            .with_body(format!(r#"{{"data": {}}}"#, make_build_json("build-slug", 42, 1)))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_build("test-app", "build-slug");

        mock.assert();
        assert!(result.is_ok());
        let build = result.unwrap();
        assert_eq!(build.data.slug, "build-slug");
        assert_eq!(build.data.build_number, 42);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Log Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_build_log_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/builds/build-slug/log")
            .with_status(200)
            .with_body(r#"{"log_chunks": [{"chunk": "Hello", "position": 0}], "expiring_raw_log_url": null, "is_archived": false}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_build_log("test-app", "build-slug");

        mock.assert();
        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.log_chunks.len(), 1);
        assert_eq!(log.log_chunks[0].chunk, "Hello");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_pipelines_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/pipelines?limit=10")
            .with_status(200)
            .with_body(format!(
                r#"{{"data": [{}], "paging": {{"total_item_count": 1, "page_item_limit": 10, "next": null}}}}"#,
                make_pipeline_json("pipeline-uuid", 1)
            ))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_pipelines("test-app", None, None, 10);

        mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, "pipeline-uuid");
    }

    #[test]
    fn test_get_pipeline_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/pipelines/pipeline-id")
            .with_status(200)
            .with_body(format!(r#"{{"data": {}}}"#, make_pipeline_json("pipeline-id", 1)))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_pipeline("test-app", "pipeline-id");

        mock.assert();
        assert!(result.is_ok());
        let pipeline = result.unwrap();
        assert_eq!(pipeline.into_pipeline().id, "pipeline-id");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Artifact Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_artifacts_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/builds/build-slug/artifacts")
            .with_status(200)
            .with_body(r#"{"data": [{"title": "app.ipa", "slug": "art-slug", "artifact_type": "file", "file_size_bytes": 1024, "is_public_page_enabled": false}], "paging": {"total_item_count": 1, "page_item_limit": 25, "next": null}}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_artifacts("test-app", "build-slug");

        mock.assert();
        assert!(result.is_ok());
        let artifacts = result.unwrap();
        assert_eq!(artifacts.data.len(), 1);
        assert_eq!(artifacts.data[0].title, "app.ipa");
    }

    #[test]
    fn test_get_artifact_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps/test-app/builds/build-slug/artifacts/art-slug")
            .with_status(200)
            .with_body(r#"{"data": {"title": "app.ipa", "slug": "art-slug", "artifact_type": "file", "file_size_bytes": 2048, "is_public_page_enabled": true, "expiring_download_url": "https://example.com/download"}}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_artifact("test-app", "build-slug", "art-slug");

        mock.assert();
        assert!(result.is_ok());
        let artifact = result.unwrap();
        assert_eq!(artifact.data.slug, "art-slug");
        assert!(artifact.data.is_public_page_enabled);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Abort Operations Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_abort_build_success() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/apps/test-app/builds/build-slug/abort")
            .with_status(200)
            .with_body(r#"{"status": "ok"}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.abort_build("test-app", "build-slug", Some("Test abort"));

        mock.assert();
        assert!(result.is_ok());
    }

    #[test]
    fn test_abort_pipeline_success() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/apps/test-app/pipelines/pipeline-id/abort")
            .with_status(200)
            .with_body(r#"{"status": "ok"}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.abort_pipeline("test-app", "pipeline-id", None);

        mock.assert();
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // URL Validation Tests (SSRF Protection)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_validate_external_url_allowed_bitrise() {
        let client = BitriseClient::with_base_url("token", "http://localhost").unwrap();
        assert!(client
            .validate_external_url("https://app.bitrise.io/log/123", "Log")
            .is_ok());
    }

    #[test]
    fn test_validate_external_url_allowed_s3() {
        let client = BitriseClient::with_base_url("token", "http://localhost").unwrap();
        assert!(client
            .validate_external_url(
                "https://bitrise-build-log-archives.s3.amazonaws.com/log.txt",
                "Log"
            )
            .is_ok());
    }

    #[test]
    fn test_validate_external_url_blocked_untrusted() {
        let client = BitriseClient::with_base_url("token", "http://localhost").unwrap();
        let result = client.validate_external_url("https://evil.com/malicious", "Log");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("untrusted host"));
    }

    #[test]
    fn test_validate_external_url_invalid_url() {
        let client = BitriseClient::with_base_url("token", "http://localhost").unwrap();
        let result = client.validate_external_url("not-a-valid-url", "Log");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Error Handling Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_server_error_returns_api_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/me")
            .with_status(500)
            .with_body(r#"{"message": "Internal server error"}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_me();

        mock.assert();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 69); // EX_UNAVAILABLE
    }

    #[test]
    fn test_rate_limit_error() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/apps?limit=10")
            .with_status(429)
            .with_body(r#"{"message": "Rate limit exceeded"}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_apps(10);

        mock.assert();
        assert!(result.is_err());
    }
}
