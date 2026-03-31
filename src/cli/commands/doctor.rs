use std::path::Path;

use crate::bitrise::BitriseClient;
use crate::cli::args::{DoctorArgs, OutputFormat};
use crate::config::{Config, Paths};
use crate::error::Result;

pub fn doctor(
    config: &Config,
    inline_token: Option<&str>,
    _args: &DoctorArgs,
    format: OutputFormat,
) -> Result<String> {
    let paths = Paths::new()?;
    let config_path = paths.config_file;
    let has_config_file = config_path.exists();
    let has_token = inline_token.is_some() || config.require_token().is_ok();
    let default_app = config.require_default_app().ok().map(str::to_string);
    let active_profile = config.active_profile.clone();
    let github_user = super::common::get_github_username();
    let git_branch = super::common::current_git_branch().ok();
    let api_user = if has_token {
        let client = match inline_token {
            Some(token) => BitriseClient::with_token(token),
            None => BitriseClient::new(config),
        };

        match client.and_then(|client| client.get_me()) {
            Ok(user) => Some(user.data.username),
            Err(_) => None,
        }
    } else {
        None
    };

    match format {
        OutputFormat::Pretty => Ok(format_doctor_pretty(
            &config_path,
            has_config_file,
            has_token,
            active_profile.as_deref(),
            default_app.as_deref(),
            git_branch.as_deref(),
            github_user.as_deref(),
            api_user.as_deref(),
        )),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "config_path": config_path,
                "config_exists": has_config_file,
                "has_token": has_token,
                "active_profile": active_profile,
                "default_app": default_app,
                "git_branch": git_branch,
                "github_user": github_user,
                "api_user": api_user,
            });
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn format_doctor_pretty(
    config_path: &Path,
    has_config_file: bool,
    has_token: bool,
    active_profile: Option<&str>,
    default_app: Option<&str>,
    git_branch: Option<&str>,
    github_user: Option<&str>,
    api_user: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str("Reprise diagnostics\n");
    output.push_str("──────────────────\n");
    output.push_str(&format!("Config file: {}\n", config_path.display()));
    output.push_str(&format!(
        "Config status: {}\n",
        if has_config_file {
            "present"
        } else {
            "missing"
        }
    ));
    output.push_str(&format!(
        "Token: {}\n",
        if has_token { "configured" } else { "missing" }
    ));
    output.push_str(&format!(
        "Profile: {}\n",
        active_profile.unwrap_or("(default)")
    ));
    output.push_str(&format!(
        "Default app: {}\n",
        default_app.unwrap_or("(not set)")
    ));
    output.push_str(&format!(
        "Git branch: {}\n",
        git_branch.unwrap_or("(unavailable)")
    ));
    output.push_str(&format!(
        "GitHub user: {}\n",
        github_user.unwrap_or("(not set)")
    ));
    output.push_str(&format!(
        "API user: {}\n",
        api_user.unwrap_or("(not checked)")
    ));
    output
}
