//! Desktop notification support for build completion
//!
//! Provides cross-platform notifications for macOS and Linux.

use notify_rust::Notification;

use crate::bitrise::Build;

/// Send a notification for build completion
pub fn build_completed(build: &Build, app_name: Option<&str>) {
    let (title, icon) = match build.status {
        1 => ("Build Succeeded", "dialog-positive"),
        2 => ("Build Failed", "dialog-error"),
        3 => ("Build Aborted", "dialog-warning"),
        _ => ("Build Completed", "dialog-information"),
    };

    let app_display = app_name.unwrap_or("Bitrise");
    let summary = format!("{} - #{}", app_display, build.build_number);

    let body = format!(
        "Workflow: {}\nBranch: {}\nDuration: {}",
        build.triggered_workflow,
        build.branch,
        build.duration_display()
    );

    let _ = Notification::new()
        .summary(&format!("{}: {}", title, summary))
        .body(&body)
        .icon(icon)
        .appname("reprise")
        .timeout(5000) // 5 seconds
        .show();
}

/// Send a notification for build triggered
pub fn build_triggered(build: &Build, app_name: Option<&str>) {
    let app_display = app_name.unwrap_or("Bitrise");

    let _ = Notification::new()
        .summary(&format!("Build Triggered - {}", app_display))
        .body(&format!(
            "Build #{}\nWorkflow: {}\nBranch: {}",
            build.build_number, build.triggered_workflow, build.branch
        ))
        .icon("media-playback-start")
        .appname("reprise")
        .timeout(3000) // 3 seconds
        .show();
}
