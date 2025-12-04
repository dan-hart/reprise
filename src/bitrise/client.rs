use reqwest::blocking::Client;
use std::time::Duration;

use super::types::*;
use crate::config::Config;
use crate::error::{RepriseError, Result};

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
        let mut params = vec![format!("limit={limit}")];

        if let Some(s) = status {
            params.push(format!("status={s}"));
        }
        if let Some(b) = branch {
            params.push(format!("branch={b}"));
        }
        if let Some(w) = workflow {
            params.push(format!("workflow={w}"));
        }

        let query = params.join("&");
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

    /// Fetch the full raw log content
    pub fn fetch_raw_log(&self, log_url: &str) -> Result<String> {
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
}
