use crate::bitrise::BitriseClient;
use crate::cli::args::{DoctorArgs, OutputFormat};
use crate::config::{Config, Paths};
use crate::error::{RepriseError, Result};
use std::collections::BTreeMap;

pub struct DoctorReport {
    pub output: String,
    pub failure: Option<RepriseError>,
}

#[derive(serde::Serialize)]
struct Check {
    status: &'static str,
    detail: String,
    hint: Option<String>,
}

/// Library formatter; CLI callers use doctor_report to preserve its exit status.
pub fn doctor(
    config: &Config,
    inline_token: Option<&str>,
    args: &DoctorArgs,
    format: OutputFormat,
) -> Result<String> {
    Ok(doctor_report(config, inline_token, args, format)?.output)
}

pub fn doctor_report(
    config: &Config,
    inline_token: Option<&str>,
    _args: &DoctorArgs,
    format: OutputFormat,
) -> Result<DoctorReport> {
    let paths = Paths::new()?;
    let has_config_file = paths.config_file.exists();
    let has_token = inline_token.is_some_and(|s| !s.is_empty()) || config.require_token().is_ok();
    let default_app = config.require_default_app().ok().map(str::to_string);
    let active_profile = config
        .selected_profile
        .as_ref()
        .or(config.active_profile.as_ref());
    let github_user = super::common::get_github_username();
    let git_branch = super::common::current_git_branch().ok();
    let mut checks = BTreeMap::new();
    checks.insert(
        "token",
        Check {
            status: if has_token { "ok" } else { "failed" },
            detail: if has_token {
                "Configured"
            } else {
                "No API token configured"
            }
            .into(),
            hint: (!has_token).then(|| "Run reprise config init or set BITRISE_TOKEN".into()),
        },
    );
    checks.insert(
        "default_app",
        Check {
            status: if default_app.is_some() {
                "ok"
            } else {
                "failed"
            },
            detail: default_app
                .clone()
                .unwrap_or_else(|| "No default app configured".into()),
            hint: default_app
                .is_none()
                .then(|| "Run reprise apps, then reprise app set APP".into()),
        },
    );
    let api = if has_token {
        let client = match inline_token {
            Some(token) => BitriseClient::with_options(token, config.network.clone()),
            None => BitriseClient::new(config),
        };
        Some(client.and_then(|client| client.get_me()))
    } else {
        None
    };
    let api_user = api
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|response| response.data.username.clone());
    checks.insert(
        "api",
        match api.as_ref() {
            Some(Ok(response)) => Check {
                status: "ok",
                detail: format!("Connected as {}", response.data.username),
                hint: None,
            },
            Some(Err(error)) => Check {
                status: "failed",
                detail: error.to_string(),
                hint: Some(recovery_hint(error).into()),
            },
            None => Check {
                status: "skipped",
                detail: "Missing credentials; connectivity was not checked".into(),
                hint: None,
            },
        },
    );
    let failure = match api {
        Some(Err(error)) => Some(error),
        _ if !has_token => Some(RepriseError::config_missing(
            "Doctor found missing credentials",
        )),
        _ if default_app.is_none() => Some(RepriseError::NoDefaultApp),
        _ => None,
    };
    let healthy = failure.is_none();
    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "healthy": healthy, "checks": checks,
            "config_path": paths.config_file, "config_exists": has_config_file,
            "has_token": has_token, "active_profile": active_profile,
            "default_app": default_app, "git_branch": git_branch,
            "github_user": github_user, "api_user": api_user,
        }))?,
        OutputFormat::Pretty => {
            let mut output = format!("Reprise diagnostics\n──────────────────\nConfig file: {} ({})\nProfile: {}\nGit branch: {}\n", paths.config_file.display(), if has_config_file { "present" } else { "optional; absent" }, active_profile.map(String::as_str).unwrap_or("(default)"), git_branch.as_deref().unwrap_or("(unavailable)"));
            for (name, check) in checks {
                output.push_str(&format!("{name}: {} — {}\n", check.status, check.detail));
                if let Some(hint) = check.hint {
                    output.push_str(&format!("  Next: {hint}\n"));
                }
            }
            output
        }
    };
    Ok(DoctorReport { output, failure })
}

fn recovery_hint(error: &RepriseError) -> &'static str {
    match error {
        RepriseError::Api { status: 401, .. } => {
            "Refresh your token in Bitrise security settings, then run reprise config init"
        }
        RepriseError::Api { status: 403, .. } => {
            "Check token scopes and your access to the selected Bitrise account"
        }
        RepriseError::Api { status: 429, .. } => {
            "Wait for the rate limit to reset; increase polling intervals"
        }
        RepriseError::Http(error) if error.is_timeout() => {
            "Check connectivity or increase --timeout"
        }
        RepriseError::Http(_) => {
            "Check network connectivity, proxy settings, and api.bitrise.io availability"
        }
        _ => "Inspect the reported API error; retry later if Bitrise is unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_credentials_are_reported_as_actionable_failed_checks() {
        let report = doctor(&Config::default(), None, &DoctorArgs {}, OutputFormat::Json).unwrap();
        let value: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(value["healthy"], false);
        assert_eq!(value["checks"]["token"]["status"], "failed");
        assert!(value["checks"]["token"]["hint"]
            .as_str()
            .unwrap()
            .contains("config init"));
        assert_eq!(value["checks"]["api"]["status"], "skipped");
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn authentication_permission_and_rate_limit_have_distinct_recovery() {
        assert!(recovery_hint(&RepriseError::api(401, "Unauthorized")).contains("Refresh"));
        assert!(recovery_hint(&RepriseError::api(403, "Forbidden")).contains("scopes"));
        assert!(recovery_hint(&RepriseError::api(429, "Rate limit")).contains("polling"));
    }
}
