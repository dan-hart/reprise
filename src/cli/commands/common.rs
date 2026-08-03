//! Common utilities shared across CLI commands
//!
//! This module contains helper functions that are used by multiple commands
//! to avoid code duplication.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, io::Write};

use is_terminal::IsTerminal;

use crate::bitrise::{App, BitriseClient, Build};
use crate::cli::args::{BuildStatusFilter, OutputFormat};
use crate::config::Config;
use crate::error::{RepriseError, Result};

/// Get GitHub username from git config, if available.
///
/// This function retrieves the user's GitHub username by running
/// `git config --global github.user`. This is used by the `--me` flag
/// to match webhook-triggered builds that use the pattern
/// `webhook-github/<username>`.
///
/// # Returns
/// - `Some(username)` if the git config value exists and is non-empty
/// - `None` if the config is not set, empty, or if git command fails
///
/// # Example
/// ```ignore
/// if let Some(gh_user) = get_github_username() {
///     println!("GitHub user: {}", gh_user);
/// }
/// ```
pub fn get_github_username() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "--global", "github.user"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
}

/// Check if a `triggered_by` value matches the user.
///
/// This function handles both direct triggers (manual builds) and webhook
/// triggers from GitHub. It performs case-insensitive matching.
///
/// # Arguments
/// * `triggered_by` - The trigger source string from the build/pipeline
/// * `bitrise_username` - The user's Bitrise username
/// * `github_username` - The user's GitHub username (if available)
///
/// # Matching Logic
/// - For manual triggers: partial match on Bitrise username (e.g., "manual-username")
/// - For webhook triggers: exact match on `webhook-github/<github-username>`
///
/// # Example
/// ```ignore
/// let matches = matches_user(
///     "webhook-github/octocat",
///     "bitrise-user",
///     Some("octocat"),
/// );
/// assert!(matches);
/// ```
pub fn matches_user(
    triggered_by: &str,
    bitrise_username: &str,
    github_username: Option<&str>,
) -> bool {
    let t_lower = triggered_by.to_lowercase();
    let bitrise_lower = bitrise_username.to_lowercase();

    // Match Bitrise username (partial match for manual triggers)
    if t_lower.contains(&bitrise_lower) {
        return true;
    }

    // Match webhook pattern with GitHub username
    if let Some(gh) = github_username {
        let webhook_pattern = format!("webhook-github/{}", gh.to_lowercase());
        if t_lower == webhook_pattern {
            return true;
        }
    }

    false
}

/// Resolve the app slug from command args or config default.
///
/// This is a common pattern used across many commands where the app
/// can be specified via `--app` flag or falls back to the configured
/// default app. If the provided value matches a configured alias,
/// the alias is resolved to its corresponding app slug.
///
/// # Arguments
/// * `app_arg` - Optional app slug or alias from command line argument
/// * `config` - Application configuration
///
/// # Returns
/// - The resolved app slug from args (after alias lookup) if provided
/// - The default app slug from config if args is None
/// - An error if neither is available
///
/// # Example
/// ```ignore
/// // With alias "ignite-ios" -> "abc123def456" configured:
/// let app_slug = resolve_app_slug(Some("ignite-ios"), config)?;
/// // Returns "abc123def456"
///
/// // Without alias:
/// let app_slug = resolve_app_slug(Some("xyz789"), config)?;
/// // Returns "xyz789" (passed through unchanged)
/// ```
pub fn resolve_app_slug<'a>(app_arg: Option<&'a str>, config: &'a Config) -> Result<&'a str> {
    match app_arg {
        Some(input) => Ok(config.resolve_alias(input)),
        None => config
            .require_default_app()
            .or_else(|_| resolve_repo_alias(config).ok_or(RepriseError::NoDefaultApp)),
    }
}

/// Get the current git branch from the repository in the current working directory.
pub fn current_git_branch() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(RepriseError::InvalidArgument(
            "Unable to determine the current git branch.".to_string(),
        ));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(RepriseError::InvalidArgument(
            "Current git branch is empty.".to_string(),
        ));
    }

    Ok(branch)
}

/// Try to resolve the current repository name as an app alias.
pub fn resolve_repo_alias(config: &Config) -> Option<&str> {
    let cwd = std::env::current_dir().ok()?;
    let repo_name = cwd.file_name()?.to_str()?;
    config.get_alias(repo_name)
}

/// Resolve the latest build matching the given filters.
pub fn resolve_latest_build(
    client: &BitriseClient,
    app_slug: &str,
    branch: Option<&str>,
    workflow: Option<&str>,
    status: Option<BuildStatusFilter>,
    pr: Option<i64>,
    current_branch: bool,
) -> Result<Build> {
    let branch = if current_branch {
        Some(current_git_branch()?)
    } else {
        branch.map(str::to_string)
    };

    let response = client.list_builds(
        app_slug,
        status.map(BuildStatusFilter::to_api_code),
        branch.as_deref(),
        workflow,
        latest_build_page_size(pr),
    )?;

    response
        .data
        .into_iter()
        .find(|build| pr.is_none_or(|pr_num| build.pull_request_id == Some(pr_num)))
        .ok_or_else(|| {
            RepriseError::BuildNotFound("No build matched the latest filters".to_string())
        })
}

fn latest_build_page_size(pr: Option<i64>) -> u32 {
    if pr.is_some() {
        50
    } else {
        25
    }
}

fn ensure_interactive_selection_allowed(format: OutputFormat, resource: &str) -> Result<()> {
    if format == OutputFormat::Json {
        return Err(RepriseError::InvalidArgument(format!(
            "Interactive {} selection is not supported with --output json. Provide an explicit target instead.",
            resource
        )));
    }

    Ok(())
}

/// Whether the current process can safely prompt the user for interactive input.
pub fn can_prompt() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Prompt the user to pick one option from a numbered list.
pub fn prompt_select(title: &str, options: &[String]) -> Result<usize> {
    if !can_prompt() {
        return Err(RepriseError::InvalidArgument(
            "Interactive selection requires a terminal.".to_string(),
        ));
    }

    if options.is_empty() {
        return Err(RepriseError::InvalidArgument(format!(
            "No {} available to choose from.",
            title
        )));
    }

    eprintln!("{}", title);
    for (index, option) in options.iter().enumerate() {
        eprintln!("  {}. {}", index + 1, option);
    }

    eprint!("Select an option [1-{}]: ", options.len());
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input
        .trim()
        .parse()
        .map_err(|_| RepriseError::InvalidArgument("Invalid selection.".to_string()))?;

    if selection == 0 || selection > options.len() {
        return Err(RepriseError::InvalidArgument(
            "Selection out of range.".to_string(),
        ));
    }

    Ok(selection - 1)
}

/// Resolve a build slug from an explicit slug, latest filters, or interactive selection.
#[allow(clippy::too_many_arguments)]
pub fn resolve_build_slug(
    client: &BitriseClient,
    app_slug: &str,
    slug: Option<&str>,
    latest: bool,
    branch: Option<&str>,
    workflow: Option<&str>,
    status: Option<BuildStatusFilter>,
    pr: Option<i64>,
    current_branch: bool,
    format: OutputFormat,
) -> Result<String> {
    if let Some(slug) = slug {
        return Ok(slug.to_string());
    }

    if latest {
        return Ok(resolve_latest_build(
            client,
            app_slug,
            branch,
            workflow,
            status,
            pr,
            current_branch,
        )?
        .slug);
    }

    ensure_interactive_selection_allowed(format, "build")?;

    let response = client.list_builds(app_slug, None, None, None, 10)?;
    let options: Vec<String> = response
        .data
        .iter()
        .map(|build| {
            format!(
                "#{} {} [{}] {}",
                build.build_number,
                build.slug,
                build.status_display(),
                build.branch
            )
        })
        .collect();
    let selected = prompt_select("Select a build:", &options)?;
    Ok(response.data[selected].slug.clone())
}

/// Resolve an app from a provided identifier or interactive selection.
pub fn resolve_app_from_identifier(
    client: &BitriseClient,
    identifier: Option<&str>,
    format: OutputFormat,
) -> Result<App> {
    if let Some(identifier) = identifier {
        return match client.get_app(identifier) {
            Ok(response) => Ok(response.data),
            Err(_) => client
                .find_app_by_name(identifier)?
                .ok_or_else(|| RepriseError::AppNotFound(identifier.to_string())),
        };
    }

    ensure_interactive_selection_allowed(format, "app")?;

    let response = client.list_apps(25)?;
    let options: Vec<String> = response
        .data
        .iter()
        .map(|app| format!("{} ({})", app.title, app.slug))
        .collect();
    let selected = prompt_select("Select an app:", &options)?;
    Ok(response.data[selected].clone())
}

/// Resolve a pipeline ID from an explicit ID or interactive selection.
pub fn resolve_pipeline_id(
    client: &BitriseClient,
    app_slug: &str,
    pipeline_id: Option<&str>,
    branch: Option<&str>,
    current_branch: bool,
    format: OutputFormat,
) -> Result<String> {
    if let Some(pipeline_id) = pipeline_id {
        return Ok(pipeline_id.to_string());
    }

    let branch = if current_branch {
        Some(current_git_branch()?)
    } else {
        branch.map(str::to_string)
    };

    ensure_interactive_selection_allowed(format, "pipeline")?;

    let response = client.list_pipelines(app_slug, None, branch.as_deref(), 10)?;
    let options: Vec<String> = response
        .data
        .iter()
        .map(|pipeline| {
            format!(
                "{} [{}] {}",
                pipeline.id,
                pipeline.status_display(),
                pipeline.get_branch()
            )
        })
        .collect();
    let selected = prompt_select("Select a pipeline:", &options)?;
    Ok(response.data[selected].id.clone())
}

/// Open a URL in the default browser.
pub fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(windows_start_args(url))
            .spawn()?;
    }

    Ok(())
}

#[cfg(any(target_os = "windows", test))]
fn windows_start_args(url: &str) -> Vec<String> {
    vec![
        "/C".to_string(),
        "start".to_string(),
        String::new(),
        format!("\"{url}\""),
    ]
}

#[cfg(any(target_os = "linux", test))]
fn linux_clipboard_candidates() -> [&'static [&'static str]; 2] {
    [&["xclip", "-selection", "clipboard"], &["wl-copy"]]
}

/// Copy text to the system clipboard using common platform tools.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        for candidate in linux_clipboard_candidates() {
            let mut command = std::process::Command::new(candidate[0]);
            if candidate.len() > 1 {
                command.args(&candidate[1..]);
            }

            if let Ok(mut child) = command.stdin(std::process::Stdio::piped()).spawn() {
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()?;
                return Ok(());
            }
        }

        Err(RepriseError::InvalidArgument(
            "No clipboard command is available on this system.".to_string(),
        ))
    }

    #[cfg(target_os = "windows")]
    {
        let mut child = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
}

/// Set up a Ctrl+C interrupt handler for graceful cancellation.
///
/// Creates an atomic boolean that will be set to `true` when the user
/// presses Ctrl+C. This allows long-running operations like log following
/// or build waiting to exit gracefully.
///
/// # Returns
/// An `Arc<AtomicBool>` that should be checked periodically. When the
/// value is `true`, the operation should terminate.
///
/// # Note
/// If a handler is already set (e.g., from a previous call), the new
/// handler registration will silently fail but the returned atomic
/// will still work for the current operation.
///
/// # Example
/// ```ignore
/// let interrupted = setup_interrupt_handler();
///
/// loop {
///     if interrupted.load(Ordering::SeqCst) {
///         eprintln!("Interrupted by user");
///         break;
///     }
///     // ... do work ...
/// }
/// ```
pub fn setup_interrupt_handler() -> Arc<AtomicBool> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .ok(); // Ignore error if handler already set

    interrupted
}

/// Check if the interrupt flag has been set.
///
/// Convenience function for checking the interrupt status.
///
/// # Arguments
/// * `interrupted` - The atomic boolean from `setup_interrupt_handler()`
///
/// # Returns
/// `true` if Ctrl+C was pressed, `false` otherwise
#[inline]
pub fn is_interrupted(interrupted: &AtomicBool) -> bool {
    interrupted.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // matches_user Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_matches_user_bitrise_username_exact() {
        assert!(matches_user("manual-testuser", "testuser", None));
    }

    #[test]
    fn test_matches_user_bitrise_username_partial() {
        assert!(matches_user("manual-TestUser-trigger", "testuser", None));
    }

    #[test]
    fn test_matches_user_bitrise_username_case_insensitive() {
        assert!(matches_user("manual-TESTUSER", "testuser", None));
        assert!(matches_user("manual-testuser", "TESTUSER", None));
    }

    #[test]
    fn test_matches_user_webhook_github_exact() {
        assert!(matches_user(
            "webhook-github/octocat",
            "bitrise-user",
            Some("octocat")
        ));
    }

    #[test]
    fn test_matches_user_webhook_github_case_insensitive() {
        assert!(matches_user(
            "webhook-github/Octocat",
            "bitrise-user",
            Some("octocat")
        ));
        assert!(matches_user(
            "webhook-github/octocat",
            "bitrise-user",
            Some("Octocat")
        ));
    }

    #[test]
    fn test_matches_user_webhook_github_no_match() {
        assert!(!matches_user(
            "webhook-github/other-user",
            "bitrise-user",
            Some("octocat")
        ));
    }

    #[test]
    fn test_matches_user_no_github_username() {
        // Should fall back to Bitrise username matching
        assert!(!matches_user(
            "webhook-github/octocat",
            "bitrise-user",
            None
        ));
    }

    #[test]
    fn test_matches_user_neither_match() {
        assert!(!matches_user(
            "webhook-github/other-user",
            "bitrise-user",
            Some("octocat")
        ));
        assert!(!matches_user(
            "manual-other-user",
            "bitrise-user",
            Some("octocat")
        ));
    }

    #[test]
    fn test_matches_user_empty_triggered_by() {
        assert!(!matches_user("", "bitrise-user", Some("octocat")));
    }

    #[test]
    fn test_matches_user_bitrise_match_takes_precedence() {
        // If Bitrise username matches, we don't need GitHub
        assert!(matches_user("manual-bitrise-user", "bitrise-user", None));
    }

    #[test]
    fn test_latest_build_page_size_defaults_to_recent_window() {
        assert_eq!(latest_build_page_size(None), 25);
    }

    #[test]
    fn test_latest_build_page_size_expands_for_pr_filter() {
        assert_eq!(latest_build_page_size(Some(1234)), 50);
    }

    #[test]
    fn test_ensure_interactive_selection_allowed_rejects_json_output() {
        let err =
            ensure_interactive_selection_allowed(crate::cli::args::OutputFormat::Json, "build")
                .unwrap_err();

        assert!(matches!(err, RepriseError::InvalidArgument(_)));
        assert!(err.to_string().contains("--output json"));
    }

    #[test]
    fn test_windows_start_args_quote_the_url_and_set_empty_title() {
        let args = windows_start_args("https://example.com/?a=1&b=2");

        assert_eq!(
            args,
            vec!["/C", "start", "", "\"https://example.com/?a=1&b=2\""]
        );
    }

    #[test]
    fn test_linux_clipboard_candidates_support_multiple_argument_lengths() {
        let candidates = linux_clipboard_candidates();

        assert_eq!(candidates[0], &["xclip", "-selection", "clipboard"]);
        assert_eq!(candidates[1], &["wl-copy"]);
    }

    #[test]
    fn test_resolve_repo_alias_miss() {
        let config = Config::default();
        assert!(resolve_repo_alias(&config).is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // setup_interrupt_handler Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_setup_interrupt_handler_returns_false_initially() {
        let interrupted = setup_interrupt_handler();
        assert!(!interrupted.load(Ordering::SeqCst));
    }

    #[test]
    fn test_is_interrupted_helper() {
        let interrupted = Arc::new(AtomicBool::new(false));
        assert!(!is_interrupted(&interrupted));

        interrupted.store(true, Ordering::SeqCst);
        assert!(is_interrupted(&interrupted));
    }

    // Note: We can't easily test the actual Ctrl+C handling in unit tests
    // since it requires signal handling, but we can verify the atomic works

    #[test]
    fn test_interrupt_flag_can_be_set() {
        let interrupted = setup_interrupt_handler();
        assert!(!is_interrupted(&interrupted));

        // Simulate what the handler does
        interrupted.store(true, Ordering::SeqCst);
        assert!(is_interrupted(&interrupted));
    }
}
