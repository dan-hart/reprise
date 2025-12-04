//! URL parsing for Bitrise URLs
//!
//! Supports parsing various Bitrise URL formats:
//! - App URLs: `https://app.bitrise.io/app/{app-slug}`
//! - Build URLs: `https://app.bitrise.io/build/{build-slug}`
//! - Pipeline URLs: `https://app.bitrise.io/app/{app-slug}/pipelines/{pipeline-id}`

use url::Url;

use crate::error::{RepriseError, Result};

/// Represents a parsed Bitrise URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitriseUrl {
    /// An app URL: https://app.bitrise.io/app/{slug}
    App { slug: String },
    /// A build URL: https://app.bitrise.io/build/{slug}
    Build { slug: String },
    /// A pipeline URL: https://app.bitrise.io/app/{app_slug}/pipelines/{pipeline_id}
    Pipeline {
        app_slug: String,
        pipeline_id: String,
    },
}

impl BitriseUrl {
    /// Get the app slug if available
    pub fn app_slug(&self) -> Option<&str> {
        match self {
            BitriseUrl::App { slug } => Some(slug),
            BitriseUrl::Pipeline { app_slug, .. } => Some(app_slug),
            BitriseUrl::Build { .. } => None,
        }
    }

    /// Get the build slug if this is a build URL
    pub fn build_slug(&self) -> Option<&str> {
        match self {
            BitriseUrl::Build { slug } => Some(slug),
            _ => None,
        }
    }

    /// Get the pipeline ID if this is a pipeline URL
    pub fn pipeline_id(&self) -> Option<&str> {
        match self {
            BitriseUrl::Pipeline { pipeline_id, .. } => Some(pipeline_id),
            _ => None,
        }
    }

    /// Get a human-readable description of this URL type
    pub fn description(&self) -> &'static str {
        match self {
            BitriseUrl::App { .. } => "app",
            BitriseUrl::Build { .. } => "build",
            BitriseUrl::Pipeline { .. } => "pipeline",
        }
    }

    /// Reconstruct the URL
    pub fn to_url(&self) -> String {
        match self {
            BitriseUrl::App { slug } => format!("https://app.bitrise.io/app/{}", slug),
            BitriseUrl::Build { slug } => format!("https://app.bitrise.io/build/{}", slug),
            BitriseUrl::Pipeline {
                app_slug,
                pipeline_id,
            } => format!(
                "https://app.bitrise.io/app/{}/pipelines/{}",
                app_slug, pipeline_id
            ),
        }
    }
}

/// Parse a Bitrise URL into its components
///
/// Supports the following URL patterns:
/// - `https://app.bitrise.io/app/{app-slug}`
/// - `https://app.bitrise.io/build/{build-slug}`
/// - `https://app.bitrise.io/app/{app-slug}/pipelines/{pipeline-id}`
pub fn parse_bitrise_url(input: &str) -> Result<BitriseUrl> {
    let url = Url::parse(input).map_err(|_| {
        RepriseError::InvalidArgument(format!("Invalid URL: {}", input))
    })?;

    // Validate host
    let host = url.host_str().ok_or_else(|| {
        RepriseError::InvalidArgument(format!("URL has no host: {}", input))
    })?;

    if host != "app.bitrise.io" {
        return Err(RepriseError::InvalidArgument(format!(
            "Not a Bitrise URL (expected app.bitrise.io, got {}): {}",
            host, input
        )));
    }

    // Parse path segments
    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.collect())
        .unwrap_or_default();

    match segments.as_slice() {
        // /app/{slug}
        ["app", slug] if !slug.is_empty() => Ok(BitriseUrl::App {
            slug: slug.to_string(),
        }),
        // /app/{slug}/pipelines/{id}
        ["app", app_slug, "pipelines", pipeline_id] if !app_slug.is_empty() && !pipeline_id.is_empty() => {
            Ok(BitriseUrl::Pipeline {
                app_slug: app_slug.to_string(),
                pipeline_id: pipeline_id.to_string(),
            })
        }
        // /build/{slug}
        ["build", slug] if !slug.is_empty() => Ok(BitriseUrl::Build {
            slug: slug.to_string(),
        }),
        _ => Err(RepriseError::InvalidArgument(format!(
            "Unrecognized Bitrise URL pattern: {}. Expected /app/{{slug}}, /build/{{slug}}, or /app/{{slug}}/pipelines/{{id}}",
            input
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_app_url() {
        let url = parse_bitrise_url("https://app.bitrise.io/app/abc123").unwrap();
        assert_eq!(url, BitriseUrl::App { slug: "abc123".to_string() });
        assert_eq!(url.app_slug(), Some("abc123"));
        assert_eq!(url.build_slug(), None);
        assert_eq!(url.description(), "app");
    }

    #[test]
    fn test_parse_build_url() {
        let url = parse_bitrise_url("https://app.bitrise.io/build/xyz789").unwrap();
        assert_eq!(url, BitriseUrl::Build { slug: "xyz789".to_string() });
        assert_eq!(url.app_slug(), None);
        assert_eq!(url.build_slug(), Some("xyz789"));
        assert_eq!(url.description(), "build");
    }

    #[test]
    fn test_parse_pipeline_url() {
        let url = parse_bitrise_url("https://app.bitrise.io/app/abc123/pipelines/def456").unwrap();
        assert_eq!(url, BitriseUrl::Pipeline {
            app_slug: "abc123".to_string(),
            pipeline_id: "def456".to_string(),
        });
        assert_eq!(url.app_slug(), Some("abc123"));
        assert_eq!(url.pipeline_id(), Some("def456"));
        assert_eq!(url.description(), "pipeline");
    }

    #[test]
    fn test_parse_uuid_style_urls() {
        // Test with UUID-style slugs (common in Bitrise)
        let url = parse_bitrise_url(
            "https://app.bitrise.io/app/36f58731-9f78-4142-9479-866acc94e15a/pipelines/d7790456-f02a-4267-bcd5-09f394e2cd29"
        ).unwrap();
        assert_eq!(url, BitriseUrl::Pipeline {
            app_slug: "36f58731-9f78-4142-9479-866acc94e15a".to_string(),
            pipeline_id: "d7790456-f02a-4267-bcd5-09f394e2cd29".to_string(),
        });
    }

    #[test]
    fn test_invalid_host() {
        let result = parse_bitrise_url("https://example.com/app/abc123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a Bitrise URL"));
    }

    #[test]
    fn test_invalid_url() {
        let result = parse_bitrise_url("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_path() {
        let result = parse_bitrise_url("https://app.bitrise.io/unknown/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unrecognized"));
    }

    #[test]
    fn test_empty_slug() {
        let result = parse_bitrise_url("https://app.bitrise.io/app/");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_url() {
        let app = BitriseUrl::App { slug: "abc".to_string() };
        assert_eq!(app.to_url(), "https://app.bitrise.io/app/abc");

        let build = BitriseUrl::Build { slug: "xyz".to_string() };
        assert_eq!(build.to_url(), "https://app.bitrise.io/build/xyz");

        let pipeline = BitriseUrl::Pipeline {
            app_slug: "abc".to_string(),
            pipeline_id: "123".to_string(),
        };
        assert_eq!(pipeline.to_url(), "https://app.bitrise.io/app/abc/pipelines/123");
    }
}
