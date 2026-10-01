use is_terminal::IsTerminal;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::redirect::Policy;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct NetworkOptions {
    pub timeout: u64,
    pub retries: u32,
    pub download_timeout: u64,
    pub search_limit: u32,
    pub quiet: bool,
}
impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            timeout: 30,
            retries: 2,
            download_timeout: 600,
            search_limit: 500,
            quiet: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchMetadata {
    pub scanned: u32,
    pub capped: bool,
}
#[derive(Debug)]
pub struct BuildSearch {
    pub data: Vec<Build>,
    pub metadata: SearchMetadata,
}

struct ChunkReader {
    chunks: std::vec::IntoIter<LogChunk>,
    current: std::io::Cursor<String>,
}
impl Read for ChunkReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        loop {
            let count = self.current.read(buffer)?;
            if count > 0 {
                return Ok(count);
            }
            match self.chunks.next() {
                Some(chunk) => self.current = std::io::Cursor::new(chunk.chunk),
                None => return Ok(0),
            }
        }
    }
}

fn retry_delay(header: Option<&reqwest::header::HeaderValue>, attempt: u32) -> Option<Duration> {
    let seconds = header.and_then(|v| v.to_str().ok()).and_then(|v| {
        v.parse::<u64>().ok().or_else(|| {
            chrono::DateTime::parse_from_rfc2822(v).ok().map(|date| {
                (date.with_timezone(&chrono::Utc) - chrono::Utc::now())
                    .num_seconds()
                    .max(0) as u64
            })
        })
    });
    if header.is_some() && seconds.is_none() {
        return None;
    }
    let delay = seconds
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_millis(100 * (1 << attempt)));
    (delay <= Duration::from_secs(30)).then_some(delay)
}
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
    pub options: NetworkOptions,
    me_cache: Mutex<Option<(Instant, UserResponse)>>,
    history_cache: Mutex<HashMap<String, (Instant, Vec<Build>)>>,
}

impl BitriseClient {
    pub fn new(config: &Config) -> Result<Self> {
        Self::with_options(config.require_token()?, config.network.clone())
    }

    pub fn with_token(token: impl Into<String>) -> Result<Self> {
        Self::with_options(token, NetworkOptions::default())
    }

    pub fn with_options(token: impl Into<String>, options: NetworkOptions) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(options.timeout))
            .redirect(Policy::none())
            .build()?;
        Ok(Self {
            client,
            token: token.into(),
            base_url: DEFAULT_BASE_URL.into(),
            options,
            me_cache: Mutex::new(None),
            history_cache: Mutex::new(HashMap::new()),
        })
    }

    #[cfg(test)]
    pub fn with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let mut client = Self::with_token(token)?;
        client.base_url = base_url.into();
        Ok(client)
    }

    fn send_get(&self, url: &str, authenticated: bool, timeout: u64) -> Result<Response> {
        let mut destination = url.to_string();
        for redirect in 0..=5 {
            let mut response = None;
            for attempt in 0..=self.options.retries.min(5) {
                let mut request = self
                    .client
                    .get(&destination)
                    .timeout(Duration::from_secs(timeout));
                if authenticated {
                    request = request.header("Authorization", &self.token);
                }
                match request.send() {
                    Ok(result) => {
                        let status = result.status();
                        if (status.as_u16() == 429 || status.is_server_error())
                            && attempt < self.options.retries.min(5)
                        {
                            let header = result.headers().get(reqwest::header::RETRY_AFTER);
                            let delay = retry_delay(header, attempt);
                            if let Some(delay) = delay {
                                std::thread::sleep(delay);
                                continue;
                            }
                            let hint = header
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("invalid value")
                                .to_string();
                            return Err(RepriseError::api(status.as_u16(), format!("Retry-After {hint:?} exceeds the 30-second retry wait limit or is invalid; wait before retrying. {}", result.text().unwrap_or_default())));
                        }
                        response = Some(result);
                        break;
                    }
                    Err(error)
                        if (error.is_connect() || error.is_timeout())
                            && attempt < self.options.retries.min(5) =>
                    {
                        std::thread::sleep(Duration::from_millis(100 * (1 << attempt)));
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            let response = response.ok_or_else(|| {
                RepriseError::InvalidArgument("Request did not produce a response".into())
            })?;
            if !authenticated && response.status().is_redirection() {
                if redirect == 5 {
                    return Err(RepriseError::InvalidArgument(
                        "Too many download redirects".into(),
                    ));
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        RepriseError::InvalidArgument("Missing redirect location".into())
                    })?;
                destination = Url::parse(&destination)
                    .and_then(|base| base.join(location))
                    .map_err(|e| RepriseError::InvalidArgument(e.to_string()))?
                    .to_string();
                self.validate_external_url(&destination, "Redirect")?;
                continue;
            }
            if !response.status().is_success() {
                let status = response.status().as_u16();
                return Err(RepriseError::api(
                    status,
                    response.text().unwrap_or_default(),
                ));
            }
            return Ok(response);
        }
        unreachable!()
    }

    /// Make a GET request to the Bitrise API
    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_with_timeout(path, self.options.timeout)
    }

    fn get_with_timeout<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        timeout: u64,
    ) -> Result<T> {
        let url = format!("{}{path}", self.base_url);
        let response = self.send_get(&url, true, timeout)?;

        let body = response.text()?;
        serde_json::from_str(&body).map_err(RepriseError::Json)
    }

    /// Make a GET request and return raw text
    fn get_text(&self, path: &str) -> Result<String> {
        let url = format!("{}{path}", self.base_url);
        let response = self.send_get(&url, true, self.options.timeout)?;
        Ok(response.text()?)
    }

    /// Fetch raw content from a URL (for log files)
    fn get_raw(&self, url: &str) -> Result<String> {
        let response = self.send_get(url, false, self.options.timeout)?;
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
        let mut cache = self
            .me_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((at, value)) = cache.as_ref() {
            if at.elapsed() < Duration::from_secs(60) {
                return Ok(value.clone());
            }
        }
        let value: UserResponse = self.get("/me")?;
        *cache = Some((Instant::now(), value.clone()));
        Ok(value)
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

    /// Get the app bitrise.yml contents
    pub fn get_bitrise_yml(&self, app_slug: &str) -> Result<String> {
        self.get_text(&format!("/apps/{app_slug}/bitrise.yml"))
    }

    /// Upload a new app bitrise.yml contents
    pub fn update_bitrise_yml(
        &self,
        app_slug: &str,
        yml_content: &str,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "app_config_datastore_yaml": yml_content,
        });

        self.post(&format!("/apps/{app_slug}/bitrise.yml"), &body)
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

    /// Walk only the documented next cursor; never follow an API-supplied URL.
    pub fn search_builds<F>(
        &self,
        app: &str,
        status: Option<i32>,
        branch: Option<&str>,
        workflow: Option<&str>,
        wanted: u32,
        matches: F,
    ) -> Result<BuildSearch>
    where
        F: Fn(&Build) -> bool,
    {
        let mut data = Vec::new();
        let mut scanned = 0;
        let mut next: Option<String> = None;
        let mut seen = HashSet::new();
        let mut capped = false;
        while scanned < self.options.search_limit && data.len() < wanted as usize {
            let remaining = self.options.search_limit - scanned;
            let limit = if next.is_none() && wanted <= 25 {
                25.min(remaining)
            } else {
                50.min(remaining)
            };
            let mut params = vec![("limit", limit.to_string())];
            if let Some(value) = status {
                params.push(("status", value.to_string()));
            }
            if let Some(value) = branch {
                params.push(("branch", value.to_string()));
            }
            if let Some(value) = workflow {
                params.push(("workflow", value.to_string()));
            }
            if let Some(value) = next.as_ref() {
                params.push(("next", value.clone()));
            }
            let query = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(params)
                .finish();
            let response: BuildListResponse = self.get(&format!("/apps/{app}/builds?{query}"))?;
            let page_count = response.data.len();
            for build in response.data.into_iter().take(remaining as usize) {
                scanned += 1;
                if matches(&build) {
                    data.push(build);
                    if data.len() >= wanted as usize {
                        break;
                    }
                }
            }
            next = response.paging.next.filter(|value| !value.is_empty());
            if data.len() >= wanted as usize {
                break;
            }
            if scanned >= self.options.search_limit
                && (next.is_some() || page_count > remaining as usize)
            {
                capped = true;
            }
            if page_count == 0 || next.is_none() {
                break;
            }
            if next
                .as_ref()
                .is_some_and(|cursor| !seen.insert(cursor.clone()))
            {
                return Err(RepriseError::InvalidArgument(
                    "Build pagination repeated a cursor; search stopped".into(),
                ));
            }
            if scanned >= self.options.search_limit {
                capped = true;
            }
        }
        Ok(BuildSearch {
            data,
            metadata: SearchMetadata { scanned, capped },
        })
    }

    /// Per-client timing history, refreshed after sixty seconds.
    pub fn timing_history(&self, app: &str) -> Result<Vec<Build>> {
        let mut cache = self
            .history_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((at, builds)) = cache.get(app) {
            if at.elapsed() < Duration::from_secs(60) {
                return Ok(builds.clone());
            }
        }
        let builds = self.list_builds(app, None, None, None, 50)?.data;
        cache.insert(app.to_string(), (Instant::now(), builds.clone()));
        Ok(builds)
    }

    pub fn refresh_caches(&self) {
        *self
            .me_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        self.history_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
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

        if parsed_url.scheme() != "https"
            || !parsed_url.username().is_empty()
            || parsed_url.password().is_some()
        {
            return Err(RepriseError::InvalidArgument(format!(
                "{purpose} URL must use HTTPS without user information"
            )));
        }

        let host = parsed_url
            .host_str()
            .filter(|h| !h.is_empty())
            .ok_or_else(|| {
                RepriseError::InvalidArgument(format!("{} URL has no valid host: {}", purpose, url))
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

        self.download_to_path(url, path)
    }

    fn download_to_path(&self, url: &str, path: &std::path::Path) -> Result<()> {
        let response = self.send_get(url, false, self.options.download_timeout)?;
        self.persist_stream(response, path, true)
    }

    fn persist_stream<R: Read>(
        &self,
        source: R,
        path: &std::path::Path,
        progress: bool,
    ) -> Result<()> {
        self.persist_stream_checked(source, path, progress, false)
    }

    fn persist_stream_checked<R: Read>(
        &self,
        mut source: R,
        path: &std::path::Path,
        progress: bool,
        require_log_content: bool,
    ) -> Result<()> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        let mut buffer = [0u8; 64 * 1024];
        let mut transferred = 0u64;
        let mut last_progress = Instant::now();
        loop {
            let count = source.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            temporary.write_all(&buffer[..count])?;
            transferred += count as u64;
            if progress
                && !self.options.quiet
                && std::io::stderr().is_terminal()
                && last_progress.elapsed() >= Duration::from_secs(1)
            {
                eprint!("\rDownloaded {} bytes", transferred);
                last_progress = Instant::now();
            }
        }
        if require_log_content && transferred == 0 {
            return Err(RepriseError::LogNotAvailable(
                "Log content is empty or not yet available.".into(),
            ));
        }
        temporary.as_file_mut().sync_all()?;
        temporary
            .persist(path)
            .map_err(|e| RepriseError::Io(e.error))?;
        if progress && !self.options.quiet && std::io::stderr().is_terminal() {
            eprintln!("\rDownloaded {} bytes", transferred);
        }
        Ok(())
    }

    fn save_raw_log(&self, url: &str, path: &std::path::Path) -> Result<()> {
        let raw = self.send_get(url, false, self.options.download_timeout)?;
        self.persist_stream_checked(raw, path, false, true)
    }

    /// Atomically save complete log content with the configured transfer timeout.
    pub fn save_log(&self, app: &str, build: &str, path: &std::path::Path) -> Result<()> {
        let response: LogResponse = self.get_with_timeout(
            &format!("/apps/{app}/builds/{build}/log"),
            self.options.download_timeout,
        )?;
        if let Some(url) = response.expiring_raw_log_url {
            self.validate_external_url(&url, "Log")?;
            self.save_raw_log(&url, path)
        } else {
            let source = ChunkReader {
                chunks: response.log_chunks.into_iter(),
                current: std::io::Cursor::new(String::new()),
            };
            self.persist_stream_checked(source, path, false, true)
        }
    }

    /// Save complete log content, then retain only the requested tail.
    /// Chunk-only API responses remain supported without guessing incremental endpoints.
    pub fn save_log_tail(
        &self,
        app: &str,
        build: &str,
        path: &std::path::Path,
        tail: usize,
    ) -> Result<String> {
        self.save_log(app, build, path)?;
        let file = std::fs::File::open(path)?;
        let mut lines = std::collections::VecDeque::new();
        use std::io::BufRead;
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if tail > 0 {
                if lines.len() == tail {
                    lines.pop_front();
                }
                lines.push_back(line);
            }
        }
        Ok(lines.into_iter().collect::<Vec<_>>().join("\n"))
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
        let raw: serde_json::Value =
            self.get(&format!("/apps/{app_slug}/pipelines/{pipeline_id}"))?;

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

        // Pipelines are triggered through the standard build-trigger endpoint;
        // there is no dedicated POST /pipelines endpoint. The returned
        // build_slug is the pipeline ID when a pipeline is triggered.
        let response: PipelineTriggerResponse =
            self.post(&format!("/apps/{app_slug}/builds"), &body)?;

        // Get the pipeline details to return full Pipeline object
        if let Some(id) = response.pipeline_id() {
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
            .with_body(format!(
                r#"{{"data": {}}}"#,
                make_app_json("my-app", "My App")
            ))
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

    #[test]
    fn test_get_bitrise_yml_success() {
        let mut server = Server::new();
        let yml = "format_version: \"13\"\\nworkflows:\\n  primary:\\n    steps: []\\n";

        let mock = server
            .mock("GET", "/apps/test-app/bitrise.yml")
            .match_header("Authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(yml)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.get_bitrise_yml("test-app");

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), yml);
    }

    #[test]
    fn test_update_bitrise_yml_success() {
        let mut server = Server::new();
        let yml = "format_version: \"13\"\\nworkflows:\\n  primary:\\n    steps: []\\n";

        let mock = server
            .mock("POST", "/apps/test-app/bitrise.yml")
            .match_header("Authorization", "test-token")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "app_config_datastore_yaml": yml
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"warnings":null}"#)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.update_bitrise_yml("test-app", yml);

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!({ "warnings": null }));
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
            .with_body(format!(
                r#"{{"data": {}}}"#,
                make_build_json("build-slug", 42, 1)
            ))
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
            .with_body(format!(
                r#"{{"data": {}}}"#,
                make_pipeline_json("pipeline-id", 1)
            ))
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

    #[test]
    fn test_trigger_pipeline_uses_build_endpoint() {
        let mut server = Server::new();

        // Pipelines are triggered via the standard build-trigger endpoint;
        // the response's build_slug is the new pipeline's ID
        let trigger_mock = server
            .mock("POST", "/apps/test-app/builds")
            .match_body(Matcher::PartialJson(serde_json::json!({
                "hook_info": { "type": "bitrise" },
                "build_params": {
                    "pipeline_id": "build-and-test",
                    "branch": "main"
                }
            })))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "status": "ok",
                    "message": "webhook processed",
                    "slug": "test-app",
                    "service": "bitrise",
                    "build_slug": "new-pipeline-id",
                    "build_number": 42,
                    "build_url": "https://app.bitrise.io/app/test-app/pipelines/new-pipeline-id",
                    "triggered_pipeline": "build-and-test"
                }"#,
            )
            .create();

        let get_mock = server
            .mock("GET", "/apps/test-app/pipelines/new-pipeline-id")
            .with_status(200)
            .with_body(make_pipeline_json("new-pipeline-id", 0))
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let params = PipelineTriggerParams {
            pipeline_id: "build-and-test".to_string(),
            branch: Some("main".to_string()),
            environments: vec![],
        };
        let result = client.trigger_pipeline("test-app", params);

        trigger_mock.assert();
        get_mock.assert();
        let pipeline = result.unwrap();
        assert_eq!(pipeline.id, "new-pipeline-id");
        assert!(pipeline.is_running());
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
            .expect(3)
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
            .expect(3)
            .create();

        let client = BitriseClient::with_base_url("test-token", server.url()).unwrap();
        let result = client.list_apps(10);

        mock.assert();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use mockito::{Matcher, Server};

    fn build(slug: &str, pr: i64) -> serde_json::Value {
        serde_json::json!({"slug":slug,"triggered_at":"2026-01-01T00:00:00Z","status":1,"status_text":"success","branch":"main","build_number":1,"triggered_workflow":"primary","pull_request_id":pr})
    }
    fn page(builds: Vec<serde_json::Value>, next: Option<&str>) -> String {
        serde_json::json!({"data":builds,"paging":{"total_item_count":100,"page_item_limit":50,"next":next}}).to_string()
    }
    #[test]
    fn filtered_search_walks_second_page_and_encodes_cursor() {
        let mut server = Server::new();
        let first = server
            .mock("GET", "/apps/a/builds?limit=25")
            .with_body(page(vec![build("old", 1)], Some("cursor +/&")))
            .expect(1)
            .create();
        let second = server
            .mock("GET", "/apps/a/builds?limit=50&next=cursor+%2B%2F%26")
            .with_body(page(vec![build("match", 42)], None))
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let result = client
            .search_builds("a", None, None, None, 1, |b| b.pull_request_id == Some(42))
            .unwrap();
        assert_eq!(result.data[0].slug, "match");
        assert_eq!(result.metadata.scanned, 2);
        assert!(!result.metadata.capped);
        first.assert();
        second.assert();
    }
    #[test]
    fn capped_latest_search_reports_incomplete_not_found() {
        let mut server = Server::new();
        let request = server
            .mock("GET", "/apps/a/builds?limit=2")
            .with_body(page(vec![build("one", 1), build("two", 1)], Some("next")))
            .expect(1)
            .create();
        let mut client = BitriseClient::with_base_url("token", server.url()).unwrap();
        client.options.search_limit = 2;
        let error = crate::cli::commands::common::resolve_latest_build(
            &client,
            "a",
            None,
            None,
            None,
            Some(42),
            false,
        )
        .unwrap_err();
        assert!(error.to_string().contains("capped"));
        request.assert();
    }
    #[test]
    fn oversized_last_page_still_marks_search_capped() {
        let mut server = Server::new();
        let request = server
            .mock("GET", "/apps/a/builds?limit=1")
            .with_body(page(vec![build("one", 1), build("two", 42)], None))
            .expect(1)
            .create();
        let mut client = BitriseClient::with_base_url("token", server.url()).unwrap();
        client.options.search_limit = 1;
        let result = client
            .search_builds("a", None, None, None, 1, |b| b.pull_request_id == Some(42))
            .unwrap();
        assert!(result.data.is_empty());
        assert!(result.metadata.capped);
        assert_eq!(result.metadata.scanned, 1);
        request.assert();
    }

    #[test]
    fn pagination_cycle_stops_after_one_repeated_cursor() {
        let mut server = Server::new();
        let first = server
            .mock("GET", "/apps/a/builds?limit=25")
            .with_body(page(vec![build("one", 1)], Some("same")))
            .expect(1)
            .create();
        let second = server
            .mock("GET", "/apps/a/builds?limit=50&next=same")
            .with_body(page(vec![build("two", 1)], Some("same")))
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        assert!(client
            .search_builds("a", None, None, None, 1, |_| false)
            .unwrap_err()
            .to_string()
            .contains("repeated"));
        first.assert();
        second.assert();
    }
    #[test]
    fn performance_cached_identity_and_history_request_counts() {
        let mut server = Server::new();
        let identity = server
            .mock("GET", "/me")
            .with_body(r#"{"data":{"username":"tester","email":"a@b.c","slug":"user"}}"#)
            .expect(1)
            .create();
        let history = server
            .mock("GET", "/apps/a/builds?limit=50")
            .with_body(page(vec![build("one", 1)], None))
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let start = Instant::now();
        for _ in 0..100 {
            client.get_me().unwrap();
            client.timing_history("a").unwrap();
        }
        eprintln!(
            "performance_cached_identity_and_history: 100 refreshes, 2 HTTP requests, {:?}",
            start.elapsed()
        );
        identity.assert();
        history.assert();
    }
    #[test]
    fn caches_expire_and_explicit_refresh_refetches() {
        let mut server = Server::new();
        let identity = server
            .mock("GET", "/me")
            .with_body(r#"{"data":{"username":"tester","email":"a@b.c","slug":"user"}}"#)
            .expect(3)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        client.get_me().unwrap();
        client.me_cache.lock().unwrap().as_mut().unwrap().0 =
            Instant::now() - Duration::from_secs(61);
        client.get_me().unwrap();
        client.refresh_caches();
        client.get_me().unwrap();
        identity.assert();
    }
    #[test]
    fn get_retries_500_but_never_401_or_post() {
        let mut server = Server::new();
        let transient = server
            .mock("GET", "/apps?limit=1")
            .with_status(500)
            .expect(3)
            .create();
        let unauthorized = server
            .mock("GET", "/apps?limit=2")
            .with_status(401)
            .expect(1)
            .create();
        let mutation = server
            .mock("POST", "/apps/a/builds/b/abort")
            .with_status(500)
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        assert!(client.list_apps(1).is_err());
        assert!(client.list_apps(2).is_err());
        assert!(client.abort_build("a", "b", None).is_err());
        transient.assert();
        unauthorized.assert();
        mutation.assert();
    }
    #[test]
    fn retry_after_zero_retries_and_excessive_wait_returns_error() {
        let mut server = Server::new();
        let retry = server
            .mock("GET", "/apps?limit=1")
            .with_status(429)
            .with_header("retry-after", "0")
            .expect(3)
            .create();
        let excessive = server
            .mock("GET", "/apps?limit=2")
            .with_status(429)
            .with_header("retry-after", "3600")
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        assert!(client.list_apps(1).is_err());
        assert!(client.list_apps(2).is_err());
        retry.assert();
        excessive.assert();
        let date = reqwest::header::HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT");
        assert_eq!(retry_delay(Some(&date), 0), Some(Duration::ZERO));
    }
    #[test]
    fn failed_stream_preserves_destination_and_removes_temp_file() {
        struct Broken(bool);
        impl Read for Broken {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                if self.0 {
                    Err(std::io::Error::other("transfer interrupted"))
                } else {
                    self.0 = true;
                    bytes[..3].copy_from_slice(b"new");
                    Ok(3)
                }
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("artifact");
        std::fs::write(&destination, "existing").unwrap();
        let client = BitriseClient::with_token("token").unwrap();
        assert!(client
            .persist_stream(Broken(false), &destination, false)
            .is_err());
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "existing");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        client
            .persist_stream(std::io::Cursor::new(b"complete"), &destination, false)
            .unwrap();
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "complete");
    }
    #[test]
    fn saved_chunk_log_retains_full_log_with_tail_output() {
        let mut server = Server::new();
        let request = server.mock("GET", "/apps/a/builds/b/log").match_query(Matcher::Any).with_body(r#"{"expiring_raw_log_url":null,"is_archived":false,"log_chunks":[{"chunk":"one\ntwo\n","position":0},{"chunk":"three\n","position":8}]}"#).create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("log");
        assert_eq!(
            client.save_log_tail("a", "b", &path, 2).unwrap(),
            "two\nthree"
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), "one\ntwo\nthree\n");
        request.assert();
    }
}

#[cfg(test)]
mod transfer_tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn truncated_http_body_preserves_destination_and_cleans_temp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            socket.read(&mut request).unwrap();
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\npartial",
                )
                .unwrap();
        });
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("artifact");
        std::fs::write(&path, "previous").unwrap();
        let client = BitriseClient::with_token("token").unwrap();
        assert!(client
            .download_to_path(&format!("http://{address}/artifact"), &path)
            .is_err());
        worker.join().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "previous");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }
    #[test]
    fn get_attempt_timeout_is_bounded_and_retried() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let mut sockets = Vec::new();
            for _ in 0..2 {
                sockets.push(listener.accept().unwrap().0);
            }
            std::thread::sleep(Duration::from_millis(1200));
        });
        let mut client =
            BitriseClient::with_base_url("token", format!("http://{address}")).unwrap();
        client.options.timeout = 1;
        client.options.retries = 1;
        let start = Instant::now();
        let error = client.list_apps(1).unwrap_err();
        assert!(matches!(error, RepriseError::Http(ref error) if error.is_timeout()));
        assert!(start.elapsed() >= Duration::from_secs(2));
        assert!(start.elapsed() < Duration::from_secs(5));
        worker.join().unwrap();
    }
    #[test]
    fn authenticated_redirect_is_never_followed() {
        let mut source = mockito::Server::new();
        let mut target = mockito::Server::new();
        let redirect = source
            .mock("GET", "/me")
            .with_status(302)
            .with_header("location", &format!("{}/me", target.url()))
            .expect(1)
            .create();
        let target_request = target.mock("GET", "/me").expect(0).create();
        let client = BitriseClient::with_base_url("secret", source.url()).unwrap();
        assert!(client.get_me().is_err());
        redirect.assert();
        target_request.assert();
    }
    #[test]
    fn external_redirect_to_untrusted_host_is_rejected() {
        let mut source = mockito::Server::new();
        let mut target = mockito::Server::new();
        let redirect = source
            .mock("GET", "/file")
            .with_status(302)
            .with_header("location", &format!("{}/file", target.url()))
            .expect(1)
            .create();
        let target_request = target.mock("GET", "/file").expect(0).create();
        let client = BitriseClient::with_base_url("secret", source.url()).unwrap();
        let directory = tempfile::tempdir().unwrap();
        assert!(client
            .download_to_path(
                &format!("{}/file", source.url()),
                &directory.path().join("file")
            )
            .is_err());
        redirect.assert();
        target_request.assert();
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::*;
    #[test]
    fn performance_many_chunks_are_streamed_iteratively() {
        let chunks: Vec<LogChunk> = (0..100_000)
            .map(|position| LogChunk {
                chunk: "chunk\n".into(),
                position,
            })
            .collect();
        let mut source = ChunkReader {
            chunks: chunks.into_iter(),
            current: std::io::Cursor::new(String::new()),
        };
        let start = Instant::now();
        let bytes = std::io::copy(&mut source, &mut std::io::sink()).unwrap();
        assert_eq!(bytes, 600_000);
        eprintln!(
            "performance_many_chunks: 100000 chunks, 600000 streamed bytes, {:?}",
            start.elapsed()
        );
    }
    #[test]
    fn timing_history_cache_is_scoped_by_app_and_expires() {
        let mut server = mockito::Server::new();
        let body =
            r#"{"data":[],"paging":{"total_item_count":0,"page_item_limit":50,"next":null}}"#;
        let a = server
            .mock("GET", "/apps/a/builds?limit=50")
            .with_body(body)
            .expect(2)
            .create();
        let b = server
            .mock("GET", "/apps/b/builds?limit=50")
            .with_body(body)
            .expect(1)
            .create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        client.timing_history("a").unwrap();
        client.timing_history("b").unwrap();
        client.timing_history("a").unwrap();
        client.history_cache.lock().unwrap().get_mut("a").unwrap().0 =
            Instant::now() - Duration::from_secs(61);
        client.timing_history("a").unwrap();
        client.timing_history("b").unwrap();
        a.assert();
        b.assert();
    }
}

#[cfg(test)]
mod saved_log_tests {
    use super::*;
    #[test]
    fn empty_chunk_save_errors_preserving_existing_file_even_tail_zero() {
        let mut server = mockito::Server::new();
        let request = server.mock("GET", "/apps/a/builds/b/log")
            .with_body(r#"{"expiring_raw_log_url":null,"is_archived":false,"log_chunks":[{"chunk":"","position":0}]}"#).expect(1).create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("log");
        std::fs::write(&path, "previous log").unwrap();
        assert!(matches!(
            client.save_log_tail("a", "b", &path, 0),
            Err(RepriseError::LogNotAvailable(_))
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "previous log");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        request.assert();
    }
    #[test]
    fn empty_raw_save_errors_preserving_existing_file() {
        let mut server = mockito::Server::new();
        let request = server.mock("GET", "/raw").with_body("").expect(1).create();
        let client = BitriseClient::with_base_url("token", server.url()).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("log");
        std::fs::write(&path, "previous log").unwrap();
        assert!(matches!(
            client.save_raw_log(&format!("{}/raw", server.url()), &path),
            Err(RepriseError::LogNotAvailable(_))
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "previous log");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        request.assert();
    }
    #[test]
    fn saved_raw_transfer_uses_download_timeout() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            socket.read(&mut request).unwrap();
            std::thread::sleep(Duration::from_millis(1200));
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nlog\n",
                )
                .unwrap();
        });
        let mut client = BitriseClient::with_token("token").unwrap();
        client.options.timeout = 1;
        client.options.download_timeout = 3;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("log");
        client
            .save_raw_log(&format!("http://{address}/raw"), &path)
            .unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "log\n");
        worker.join().unwrap();
    }
}
