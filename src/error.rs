use thiserror::Error;

/// Result type alias for Reprise operations
pub type Result<T> = std::result::Result<T, RepriseError>;

/// Errors that can occur during Reprise operations
#[derive(Error, Debug)]
pub enum RepriseError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Missing required configuration
    #[error("{0}")]
    ConfigMissing(String),

    /// API error with HTTP status
    #[error("Bitrise API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("Failed to parse response: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("Failed to parse config file: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("Failed to write config file: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// No default app configured
    #[error("No default app configured. Run 'reprise app set <slug>' first.")]
    NoDefaultApp,

    /// App not found
    #[error("App not found: {0}")]
    AppNotFound(String),

    /// Build not found
    #[error("Build not found: {0}")]
    BuildNotFound(String),

    /// Build log not available
    #[error("Build log not available: {0}")]
    LogNotAvailable(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Environment variable error
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),
}

impl RepriseError {
    /// Create an API error from HTTP status and message
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create a config missing error with helpful message
    pub fn config_missing(message: impl Into<String>) -> Self {
        Self::ConfigMissing(message.into())
    }

    /// Get the appropriate exit code for this error type.
    ///
    /// Uses standard exit codes where applicable:
    /// - 1: General errors (network, parsing)
    /// - 2: Usage/argument errors
    /// - 78: Configuration errors (EX_CONFIG from sysexits.h)
    /// - 69: Service unavailable (EX_UNAVAILABLE) for API errors
    /// - 66: Not found errors (EX_NOINPUT)
    pub fn exit_code(&self) -> i32 {
        match self {
            // Configuration errors
            Self::Config(_) | Self::ConfigMissing(_) | Self::NoDefaultApp => 78,

            // Usage/argument errors
            Self::InvalidArgument(_) => 2,

            // Not found errors
            Self::AppNotFound(_) | Self::BuildNotFound(_) | Self::LogNotAvailable(_) => 66,

            // API/service unavailable errors
            Self::Api { status, .. } => {
                match *status {
                    401 | 403 => 77, // EX_NOPERM - permission denied
                    404 => 66,       // EX_NOINPUT - not found
                    _ => 69,         // EX_UNAVAILABLE - service unavailable
                }
            }

            // Network errors
            Self::Http(_) => 69,

            // IO errors
            Self::Io(_) | Self::Env(_) => 74, // EX_IOERR

            // Parsing errors
            Self::Json(_) | Self::Toml(_) | Self::TomlSerialize(_) => 65, // EX_DATAERR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Helper Function Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_api_helper_creates_error() {
        let err = RepriseError::api(404, "Not found");
        match err {
            RepriseError::Api { status, message } => {
                assert_eq!(status, 404);
                assert_eq!(message, "Not found");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_api_helper_with_string() {
        let err = RepriseError::api(500, String::from("Server error"));
        match err {
            RepriseError::Api { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Server error");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_config_missing_helper() {
        let err = RepriseError::config_missing("API token not set");
        match err {
            RepriseError::ConfigMissing(msg) => {
                assert_eq!(msg, "API token not set");
            }
            _ => panic!("Expected ConfigMissing error"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - Configuration Errors (78)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_config() {
        let err = RepriseError::Config("invalid config".to_string());
        assert_eq!(err.exit_code(), 78);
    }

    #[test]
    fn test_exit_code_config_missing() {
        let err = RepriseError::ConfigMissing("missing token".to_string());
        assert_eq!(err.exit_code(), 78);
    }

    #[test]
    fn test_exit_code_no_default_app() {
        let err = RepriseError::NoDefaultApp;
        assert_eq!(err.exit_code(), 78);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - Usage Errors (2)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_invalid_argument() {
        let err = RepriseError::InvalidArgument("bad arg".to_string());
        assert_eq!(err.exit_code(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - Not Found Errors (66)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_app_not_found() {
        let err = RepriseError::AppNotFound("my-app".to_string());
        assert_eq!(err.exit_code(), 66);
    }

    #[test]
    fn test_exit_code_build_not_found() {
        let err = RepriseError::BuildNotFound("build-123".to_string());
        assert_eq!(err.exit_code(), 66);
    }

    #[test]
    fn test_exit_code_log_not_available() {
        let err = RepriseError::LogNotAvailable("build in progress".to_string());
        assert_eq!(err.exit_code(), 66);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - API Errors (varies by status)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_api_401_unauthorized() {
        let err = RepriseError::api(401, "Unauthorized");
        assert_eq!(err.exit_code(), 77); // EX_NOPERM
    }

    #[test]
    fn test_exit_code_api_403_forbidden() {
        let err = RepriseError::api(403, "Forbidden");
        assert_eq!(err.exit_code(), 77); // EX_NOPERM
    }

    #[test]
    fn test_exit_code_api_404_not_found() {
        let err = RepriseError::api(404, "Not Found");
        assert_eq!(err.exit_code(), 66); // EX_NOINPUT
    }

    #[test]
    fn test_exit_code_api_500_server_error() {
        let err = RepriseError::api(500, "Internal Server Error");
        assert_eq!(err.exit_code(), 69); // EX_UNAVAILABLE
    }

    #[test]
    fn test_exit_code_api_502_bad_gateway() {
        let err = RepriseError::api(502, "Bad Gateway");
        assert_eq!(err.exit_code(), 69); // EX_UNAVAILABLE
    }

    #[test]
    fn test_exit_code_api_503_service_unavailable() {
        let err = RepriseError::api(503, "Service Unavailable");
        assert_eq!(err.exit_code(), 69); // EX_UNAVAILABLE
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - IO Errors (74)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = RepriseError::Io(io_err);
        assert_eq!(err.exit_code(), 74); // EX_IOERR
    }

    #[test]
    fn test_exit_code_env_error() {
        let env_err = std::env::VarError::NotPresent;
        let err = RepriseError::Env(env_err);
        assert_eq!(err.exit_code(), 74); // EX_IOERR
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Exit Code Tests - Parsing Errors (65)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exit_code_json_error() {
        let json_err: serde_json::Error = serde_json::from_str::<String>("invalid").unwrap_err();
        let err = RepriseError::Json(json_err);
        assert_eq!(err.exit_code(), 65); // EX_DATAERR
    }

    #[test]
    fn test_exit_code_toml_error() {
        let toml_err: toml::de::Error = toml::from_str::<toml::Value>("invalid = ").unwrap_err();
        let err = RepriseError::Toml(toml_err);
        assert_eq!(err.exit_code(), 65); // EX_DATAERR
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Error Display Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_error_display_config() {
        let err = RepriseError::Config("invalid format".to_string());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("invalid format"));
    }

    #[test]
    fn test_error_display_api() {
        let err = RepriseError::api(404, "Not found");
        assert!(err.to_string().contains("HTTP 404"));
        assert!(err.to_string().contains("Not found"));
    }

    #[test]
    fn test_error_display_no_default_app() {
        let err = RepriseError::NoDefaultApp;
        assert!(err.to_string().contains("No default app"));
        assert!(err.to_string().contains("reprise app set"));
    }

    #[test]
    fn test_error_display_app_not_found() {
        let err = RepriseError::AppNotFound("my-app".to_string());
        assert!(err.to_string().contains("App not found"));
        assert!(err.to_string().contains("my-app"));
    }

    #[test]
    fn test_error_display_build_not_found() {
        let err = RepriseError::BuildNotFound("build-slug".to_string());
        assert!(err.to_string().contains("Build not found"));
        assert!(err.to_string().contains("build-slug"));
    }

    #[test]
    fn test_error_display_invalid_argument() {
        let err = RepriseError::InvalidArgument("bad value".to_string());
        assert!(err.to_string().contains("Invalid argument"));
        assert!(err.to_string().contains("bad value"));
    }
}
