# Reprise CLI Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the 10 approved CLI improvements in a phased rollout while keeping the existing command surface stable and backward compatible.

**Architecture:** Add shared resolution helpers for builds, artifacts, git context, profiles, and views, then layer new commands and flags on top of the existing `clap` command tree and output formatters. Favor incremental command extensions over large rewrites so each phase remains testable.

**Tech Stack:** Rust, clap, serde/toml, reqwest blocking client, chrono, colored

---

### Task 1: Persist design docs and roadmap tests

**Files:**
- Modify: `tests/cli_tests.rs`

- [ ] **Step 1: Write the failing tests for new top-level help surfaces**

Add integration tests that assert help output for the upcoming `doctor`, `diagnose`, `compare`, and `view` commands plus new flags like `--latest` / `--current-branch`.

- [ ] **Step 2: Run the focused CLI tests and verify they fail**

Run: `cargo test --quiet test_doctor_help test_diagnose_help test_compare_help test_view_help`
Expected: FAIL because the commands do not exist yet

- [ ] **Step 3: Implement the minimal clap command additions**

Touch:
- `src/cli/args.rs`
- `src/main.rs`
- `src/cli/commands/mod.rs`

- [ ] **Step 4: Re-run the focused CLI tests and verify they pass**

Run: `cargo test --quiet test_doctor_help test_diagnose_help test_compare_help test_view_help`
Expected: PASS

### Task 2: Phase 1 shared build targeting

**Files:**
- Modify: `src/cli/args.rs`
- Modify: `src/cli/commands/common.rs`
- Modify: `src/cli/commands/build.rs`
- Modify: `src/cli/commands/log.rs`
- Modify: `src/cli/commands/artifacts.rs`

- [ ] **Step 1: Write failing unit tests for latest and current-branch resolution**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement shared build-target resolution helpers**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 3: Phase 1 interactive picker

**Files:**
- Modify: `src/cli/args.rs`
- Modify: `src/cli/commands/common.rs`
- Modify: `src/cli/commands/app.rs`
- Modify: `src/cli/commands/build.rs`
- Modify: `src/cli/commands/log.rs`
- Modify: `src/cli/commands/artifacts.rs`
- Modify: `src/cli/commands/pipeline.rs`

- [ ] **Step 1: Write failing tests for optional targets and picker validation helpers**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement minimal interactive selection**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 4: Phase 1 smarter artifacts

**Files:**
- Modify: `src/cli/args.rs`
- Modify: `src/cli/commands/artifacts.rs`
- Modify: `src/cli/commands/common.rs`

- [ ] **Step 1: Write failing tests for artifact action selection**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement `--open`, `--copy-url`, and latest-aware artifact resolution**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 5: Phase 2 profile-aware config

**Files:**
- Modify: `src/config/settings.rs`
- Modify: `src/cli/args.rs`
- Modify: `src/cli/commands/config.rs`

- [ ] **Step 1: Write failing config round-trip tests for profiles**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement profile storage and active-profile-aware accessors**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 6: Phase 2 saved views

**Files:**
- Modify: `src/config/settings.rs`
- Modify: `src/cli/args.rs`
- Create: `src/cli/commands/view.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing tests for view persistence and command help**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement view CRUD and execution**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 7: Phase 2 doctor and dashboard

**Files:**
- Modify: `src/cli/args.rs`
- Create: `src/cli/commands/doctor.rs`
- Modify: `src/cli/commands/builds.rs`
- Modify: `src/cli/commands/pipeline.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing tests for doctor output and dashboard summaries**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement doctor checks and richer watch headers**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 8: Phase 3 diagnose

**Files:**
- Modify: `src/cli/args.rs`
- Create: `src/cli/commands/diagnose.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing tests for diagnosis heuristics**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement build diagnosis**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 9: Phase 3 compare

**Files:**
- Modify: `src/cli/args.rs`
- Create: `src/cli/commands/compare.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing tests for build diff generation**
- [ ] **Step 2: Run the focused tests and verify they fail**
- [ ] **Step 3: Implement build comparison**
- [ ] **Step 4: Re-run focused tests and verify they pass**

### Task 10: Full verification

**Files:**
- Modify: `README.md`
- Modify: any touched command help docs as needed

- [ ] **Step 1: Update user-facing docs to reflect new commands and flags**
- [ ] **Step 2: Run `cargo test --quiet`**
- [ ] **Step 3: Run `cargo fmt --check`**
- [ ] **Step 4: Run targeted `cargo clippy` checks on touched modules where practical**
- [ ] **Step 5: Summarize residual risks**
