//! Common utilities shared across CLI commands
//!
//! This module contains helper functions that are used by multiple commands
//! to avoid code duplication.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;

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
///     "webhook-github/dan-hart",
///     "bitrise-user",
///     Some("dan-hart"),
/// );
/// assert!(matches);
/// ```
pub fn matches_user(triggered_by: &str, bitrise_username: &str, github_username: Option<&str>) -> bool {
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
/// default app.
///
/// # Arguments
/// * `app_arg` - Optional app slug from command line argument
/// * `config` - Application configuration
///
/// # Returns
/// - The app slug from args if provided
/// - The default app slug from config if args is None
/// - An error if neither is available
///
/// # Example
/// ```ignore
/// let app_slug = resolve_app_slug(args.app.as_deref(), config)?;
/// ```
pub fn resolve_app_slug<'a>(app_arg: Option<&'a str>, config: &'a Config) -> Result<&'a str> {
    app_arg
        .map(Ok)
        .unwrap_or_else(|| config.require_default_app())
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
            "webhook-github/dan-hart",
            "bitrise-user",
            Some("dan-hart")
        ));
    }

    #[test]
    fn test_matches_user_webhook_github_case_insensitive() {
        assert!(matches_user(
            "webhook-github/Dan-Hart",
            "bitrise-user",
            Some("dan-hart")
        ));
        assert!(matches_user(
            "webhook-github/dan-hart",
            "bitrise-user",
            Some("Dan-Hart")
        ));
    }

    #[test]
    fn test_matches_user_webhook_github_no_match() {
        assert!(!matches_user(
            "webhook-github/other-user",
            "bitrise-user",
            Some("dan-hart")
        ));
    }

    #[test]
    fn test_matches_user_no_github_username() {
        // Should fall back to Bitrise username matching
        assert!(!matches_user("webhook-github/dan-hart", "bitrise-user", None));
    }

    #[test]
    fn test_matches_user_neither_match() {
        assert!(!matches_user(
            "webhook-github/other-user",
            "bitrise-user",
            Some("dan-hart")
        ));
        assert!(!matches_user(
            "manual-other-user",
            "bitrise-user",
            Some("dan-hart")
        ));
    }

    #[test]
    fn test_matches_user_empty_triggered_by() {
        assert!(!matches_user("", "bitrise-user", Some("dan-hart")));
    }

    #[test]
    fn test_matches_user_bitrise_match_takes_precedence() {
        // If Bitrise username matches, we don't need GitHub
        assert!(matches_user(
            "manual-bitrise-user",
            "bitrise-user",
            None
        ));
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
