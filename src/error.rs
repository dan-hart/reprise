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
}
