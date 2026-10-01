//! Conservative, evidence-based log classification. A match is a signal, not proof of root cause.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ContextLine {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct LogSummary {
    pub category: String,
    pub confidence: &'static str,
    pub first_error: Option<String>,
    pub last_error: Option<String>,
    pub first_error_line: Option<usize>,
    pub failed_step: Option<String>,
    pub context: Vec<ContextLine>,
    pub suggested_next_step: String,
}

pub(super) fn summarize(log: &str) -> LogSummary {
    let lines: Vec<_> = log.lines().collect();
    let mut signals = Vec::new();
    let mut generic = Vec::new();
    let mut step = None;
    let mut failed_step = None;
    for (index, line) in lines.iter().enumerate() {
        let cleaned = strip_ansi(line);
        let lower = cleaned.to_lowercase();
        if let Some(name) = cleaned.strip_prefix("Running step:") {
            step = Some(name.trim().to_string());
        }
        if let Some(name) = cleaned
            .strip_prefix("Failed step:")
            .or_else(|| cleaned.strip_prefix("Step failed:"))
        {
            failed_step = Some(name.trim().to_string());
        }
        if let Some(category) = classify(&lower) {
            signals.push((index, category));
            if failed_step.is_none() {
                failed_step = step.clone();
            }
        } else if is_failure(&lower) {
            generic.push(index);
        }
    }
    let category = signals
        .first()
        .map(|(_, category)| *category)
        .unwrap_or("unknown");
    let first = signals
        .first()
        .map(|(index, _)| *index)
        .or_else(|| generic.first().copied());
    let last = signals
        .last()
        .map(|(index, _)| *index)
        .or_else(|| generic.last().copied());
    let context = first
        .map(|index| {
            let start = index.saturating_sub(2);
            let end = (index + 3).min(lines.len());
            (start..end)
                .map(|i| ContextLine {
                    line: i + 1,
                    text: strip_ansi(lines[i]),
                })
                .collect()
        })
        .unwrap_or_default();
    LogSummary {
        category: category.into(),
        confidence: if !signals.is_empty() { "high" } else if first.is_some() { "low" } else { "none" },
        first_error: first.map(|i| strip_ansi(lines[i]).trim().to_string()),
        last_error: last.map(|i| strip_ansi(lines[i]).trim().to_string()),
        first_error_line: first.map(|i| i + 1),
        failed_step,
        context,
        suggested_next_step: match category {
            "tests" => "Open the full log and inspect the first failing test and its assertion.",
            "dependencies" => "Check dependency resolution, repository access, and missing file configuration.",
            "compile" => "Inspect compiler output and source location near the first error line.",
            "signing" => "Verify signing credentials, certificates, and provisioning settings.",
            _ => "No specific cause established. Inspect the full log and compare against a successful build.",
        }.into(),
    }
}

fn failure_counts(line: &str) -> (bool, bool) {
    let words: Vec<_> = line
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    let mut zero = false;
    let mut positive = false;
    for (i, word) in words.iter().enumerate() {
        if let Ok(count) = word.parse::<u64>() {
            let next = words.get(i + 1).copied().unwrap_or("");
            let after = words.get(i + 2).copied().unwrap_or("");
            if matches!(next, "error" | "errors" | "failure" | "failures" | "failed")
                || matches!(next, "test" | "tests") && after == "failed"
            {
                zero |= count == 0;
                positive |= count > 0;
            }
        }
    }
    (zero, positive)
}

fn explicit_failure(line: &str) -> bool {
    line.contains("build failed")
        || line.contains("compilation failed")
        || line.contains("test case") && line.contains("failed")
        || line.contains("test result: failed")
        || line.contains("assertionerror")
        || line.contains("assertion failed")
        || line.contains("error:")
        || line.contains("fatal:")
}

fn is_zero_summary(line: &str) -> bool {
    let (zero, positive) = failure_counts(line);
    zero && !positive && !explicit_failure(line)
}

fn classify(line: &str) -> Option<&'static str> {
    if is_zero_summary(line) || line.contains("no errors") && !explicit_failure(line) {
        return None;
    }
    if line.contains("no profiles for")
        || line.contains("requires a provisioning profile")
        || line.contains("no signing certificate")
        || line.contains("provisioning profile") && is_failure(line)
        || line.contains("codesign") && is_failure(line)
    {
        Some("signing")
    } else if line.contains("test case") && line.contains("failed")
        || line.contains("test result: failed")
        || line.contains("test failed")
        || line.contains("tests failed")
        || line.contains("there were failing tests")
        || line.contains("assertionerror")
        || line.contains("assertion failed")
        || line.contains("error: assertion")
    {
        Some("tests")
    } else if line.starts_with("error[e")
        || line.contains(".swift:") && line.contains("error:")
        || line.contains(".kt:") && (line.contains("error:") || line.starts_with("e:"))
        || line.contains(".java:") && line.contains("error:")
        || line.contains("rustc") && line.contains("error")
        || line.contains("cannot find") && line.contains("in scope")
        || line.contains("unresolved reference:")
        || line.contains("compilation failed")
        || line.contains("error:")
            && line.contains("unable to read input file")
            && [".swift", ".kt", ".java", ".rs"]
                .iter()
                .any(|ext| line.contains(ext))
    {
        Some("compile")
    } else if line.contains("could not resolve")
        && [
            "all files",
            "dependency",
            "dependencies",
            "artifact",
            "configuration",
            "module",
        ]
        .iter()
        .any(|signal| line.contains(signal))
        || line.contains("could not find") && line.contains("artifact")
        || line.contains("no such file or directory")
            && ["node_modules", "pods/", "package", "dependency"]
                .iter()
                .any(|signal| line.contains(signal))
        || line.contains("failed to download")
        || line.contains("unable to resolve dependency")
    {
        Some("dependencies")
    } else {
        None
    }
}

fn is_failure(line: &str) -> bool {
    if is_zero_summary(line)
        || line.contains("no errors") && !explicit_failure(line)
        || line.contains("error handling")
    {
        return false;
    }
    line.contains("error:")
        || line.contains("fatal:")
        || line.contains("failed")
        || line.contains("exception:")
        || line.contains("panicked at")
        || line.contains("could not resolve host")
}

fn strip_ansi(line: &str) -> String {
    let mut text = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            text.push(c);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concrete_rust_signal_includes_source_context_and_step() {
        let result = summarize("Running step: Rust tests\nCompiling checkout\n\u{1b}[31merror[E0425]: cannot find value `price`\u{1b}[0m\n --> src/cart.rs:12\nFailed step: Rust tests");
        assert_eq!(result.category, "compile");
        assert_eq!(result.confidence, "high");
        assert_eq!(result.first_error_line, Some(3));
        assert_eq!(result.failed_step.as_deref(), Some("Rust tests"));
        assert!(result
            .context
            .iter()
            .any(|line| line.text.contains("cart.rs:12")));
        assert!(!result.first_error.unwrap().contains('\u{1b}'));
    }
    #[test]
    fn generic_build_failure_is_uncertain_and_does_not_guess_signing() {
        let result = summarize("Signing resources\n** BUILD FAILED **");
        assert_eq!(result.category, "unknown");
        assert_eq!(result.confidence, "low");
        assert!(result.failed_step.is_none());
    }
}

#[cfg(test)]
mod review_regressions {
    use super::*;
    #[test]
    fn zero_counts_are_not_signals() {
        for log in ["0 tests failed", "rustc: 0 errors", "0 errors, 0 failures"] {
            let result = summarize(log);
            assert_eq!(result.confidence, "none", "{log}");
            assert!(result.first_error.is_none(), "{log}");
        }
    }
    #[test]
    fn positive_counts_survive_zero_count_substrings() {
        for log in [
            "fatal: build failed with 10 errors",
            "Build failed: 1 failure, 0 errors",
        ] {
            assert!(summarize(log).first_error.is_some(), "{log}");
        }
    }
    #[test]
    fn missing_source_and_dns_are_not_dependency_guesses() {
        assert_eq!(
            summarize(
                "error: unable to read input file 'Checkout.swift': No such file or directory"
            )
            .category,
            "compile"
        );
        let dns = summarize("curl: (6) Could not resolve host: api.example.com");
        assert_eq!(dns.category, "unknown");
        assert_ne!(dns.confidence, "high");
    }
}

#[cfg(test)]
mod mixed_failure_tests {
    use super::*;
    #[test]
    fn independent_failure_markers_survive_zero_error_counts() {
        for log in [
            "** BUILD FAILED ** (0 errors reported)",
            "Test case 'Example.test()' failed (0 errors)",
        ] {
            assert!(summarize(log).first_error.is_some(), "{log}");
        }
    }
}
