---
phase: 68-build-perf-m2-dependency-hygiene
plan: "01"
subsystem: build-perf
tags:
  - dependency-hygiene
  - env-logger
  - log-level
  - breaking-change
dependency_graph:
  requires:
    - 67-04-PLAN (build-perf-m1 complete)
  provides:
    - env_logger removed from [dependencies]
    - LogLevel/with_logs/log_level plumbing removed
    - tests/test_no_logger_installed.rs integration test
    - MIGRATION.md Logger setup recipe
    - .planning/intel/logger-history.md rationale doc
  affects:
    - Cargo.toml
    - src/engines/ga.rs
    - src/configuration.rs
    - src/configuration/builders.rs
    - src/traits/configuration.rs
    - examples/ (24 files)
    - tests/test_no_logger_installed.rs
    - MIGRATION.md
    - CHANGELOG.md
    - README.md
    - docs/getting-started.md
tech_stack:
  added: []
  patterns:
    - env_logger moved to [dev-dependencies] — library consumers no longer pay the cost
    - All examples call env_logger::init() or env_logger::try_init() explicitly in main()
    - PanicLogger integration test pattern: verify logger slot still free after Ga::run()
key_files:
  created:
    - tests/test_no_logger_installed.rs
    - .planning/intel/logger-history.md
  modified:
    - Cargo.toml
    - src/engines/ga.rs
    - src/configuration.rs
    - src/configuration/builders.rs
    - src/traits/configuration.rs
    - examples/memetic_rastrigin.rs (removed .with_logs call, added env_logger::init())
    - examples/*.rs (23 files — added env_logger::try_init() to main())
    - MIGRATION.md
    - CHANGELOG.md
    - README.md
    - docs/getting-started.md
decisions:
  - "env_logger moved to [dev-dependencies] — library consumers get ~12 fewer transitive deps"
  - "PanicLogger test installs logger AFTER Ga::run() to verify slot still free (not before, since library emits log!() events normally)"
  - "LogLevel/with_logs/log_level removed entirely — dead code with no auto-installer"
metrics:
  duration: "~25 minutes"
  completed: "2026-06-15"
  tasks_completed: 3
  tasks_total: 3
  files_created: 2
  files_modified: 33
---

# Phase 68 Plan 01: env_logger Removal and Dependency Hygiene Summary

**One-liner:** Removed env_logger auto-install anti-pattern from Ga::run(), deleted LogLevel/with_logs/log_level dead code, moved env_logger to dev-dependencies, added explicit logger calls to all 24 examples, and added PanicLogger integration test.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove env_logger auto-install and LogLevel/with_logs plumbing | f5f00b7 | Cargo.toml, src/engines/ga.rs, src/configuration.rs, src/configuration/builders.rs, src/traits/configuration.rs |
| 2 | Update examples + add test_no_logger_installed integration test | c9cfd22 | examples/*.rs (24 files), tests/test_no_logger_installed.rs |
| 3 | Write MIGRATION.md / CHANGELOG.md / README.md / docs / intel updates | 37f9d0f | MIGRATION.md, CHANGELOG.md, README.md, docs/getting-started.md |

## What Was Done

### Task 1 — Library cleanup

- `src/engines/ga.rs`: Removed the 12-line block that mapped `config.log_level` to a `log::LevelFilter` and called `env_logger::Builder::from_default_env().filter_level(log_level).try_init()`. Also removed the `with_logs(mut self, log_level: LogLevel)` impl and the `LogLevel` import from the `use crate::{configuration::...}` block.
- `src/configuration.rs`: Deleted the entire `LogLevel` enum (26 lines with doc comments and serde derives), the `pub(crate) log_level: LogLevel` field from `GaConfiguration`, the `log_level: LogLevel::Off` initializer from `Default`, the `pub fn log(&self) -> LogLevel` accessor method, and a stale doc link `/// [`ConfigurationT::with_logs`]`.
- `src/traits/configuration.rs`: Deleted the `fn with_logs(self, log_level: LogLevel) -> Self;` trait method from `ConfigurationT` and removed the `LogLevel` from the `use crate::configuration::{LogLevel, ProblemSolving}` import.
- `src/configuration/builders.rs`: Deleted the `fn with_logs(mut self, log_level: LogLevel) -> Self` impl and removed `LogLevel` from the use import.
- `Cargo.toml`: Removed `env_logger = "0.11.5"` from `[dependencies]`; added `env_logger = "0.11.5"` to `[dev-dependencies]`.

### Task 2 — Examples and integration test

- `examples/memetic_rastrigin.rs`: Removed `.with_logs(genetic_algorithms::configuration::LogLevel::Warn)` from the builder chain; added `env_logger::init();` as the first statement of `main()`.
- All 23 other examples: Added `let _ = env_logger::try_init();` as the first statement of `main()`. Used `try_init` rather than `init` to avoid panics when multiple examples run in the same test context.
- `tests/test_no_logger_installed.rs`: New integration test that runs `Ga::run()` with no logger installed, then calls `log::set_logger(&PANIC_LOGGER)` and expects it to succeed — proving the library did not occupy the global logger slot during its run. If the library had called `env_logger::try_init()`, `set_logger` would return `Err` and the test would fail.

### Task 3 — Documentation

- `MIGRATION.md`: Added `## Logger setup (v2 auto-init → v3 explicit)` section with before/after code snippets and rationale, plus `### Removed: LogLevel enum and with_logs() builder method` subsection.
- `CHANGELOG.md`: Added breaking-change bullet ("Library no longer auto-installs env_logger") and Removed bullet ("configuration::LogLevel enum and ConfigurationT::with_logs() builder method") under v3.0.0.
- `README.md`: Removed stale `log_level` from the configuration list; added `### Logging` subsection explaining the explicit logger installation pattern.
- `docs/getting-started.md`: Added explanatory note about library not installing a logger, added `env_logger::init()` as the first statement of the First Run `main()` snippet.
- `.planning/intel/logger-history.md`: Created dated rationale doc (2026-06-15) covering why the library must never install env_logger, what must not be reintroduced, the canonical pattern for emitting log events, and how to verify.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build` | green |
| `cargo build --no-default-features` | green |
| `cargo check --target wasm32-unknown-unknown` | green |
| `cargo doc --no-deps` warnings | 0 |
| `cargo test --test test_no_logger_installed` | PASSED (1/1) |
| `cargo build --examples` | green |
| `grep -rn "env_logger" src/` | 0 matches |
| `grep -rn "LogLevel" src/` | 0 matches |
| `grep -rn "with_logs" src/` | 0 matches |
| `grep -rn "log_level" src/` | 0 matches |
| `env_logger` in `[dev-dependencies]` only | confirmed (line 59) |
| All commits GPG-signed | confirmed (Good signature × 3) |
| All commits have `Revert plan:` line | confirmed × 3 |

## Dependency Delta

Runtime dependency count (no-dev, via `cargo tree --edges no-dev`):

- Before: ~30 crates (included env_logger + humantime + is-terminal + termcolor + winapi-* etc.)
- After: 19 crates (env_logger and all its transitive deps removed from runtime graph)

The `cargo tree | sort -u | wc -l` total (including dev-deps) stays at 97 because env_logger was already counted in the dev tree; the saving is purely in what library consumers must compile.

## Deviations from Plan

### Auto-fixed: Test design adjustment

**Found during:** Task 2

**Issue:** The plan spec for `test_no_logger_installed.rs` said to install PanicLogger FIRST with `LevelFilter::Trace` then run the GA, with the expectation that "Reaching the end of the function asserts the auto-init is gone." However, the library legitimately emits `log::debug!` events during `Ga::run()` (e.g., "Started the population fitness calculation" from `population.rs:132`). Installing PanicLogger first and setting LevelFilter::Trace meant those events fired through PanicLogger and caused the test to panic unconditionally — even after removing the auto-install.

**Fix:** Reversed the order: run GA first (no subscriber installed), then call `log::set_logger(&PANIC_LOGGER).expect(...)` afterwards. The `set_logger` call returns `Ok` only if the global slot is still free — which is true now that the library no longer installs env_logger. This correctly tests the "library does not install a logger" invariant without interfering with the library's legitimate log event emissions.

**CONTEXT.md §Specific Ideas** says "run the GA without setting up any subscriber" — the revised test correctly implements this: no subscriber before the run, then verify the slot is free after.

## Known Stubs

None — all changes are complete and wired.

## Threat Flags

None — this plan removes dependencies and dead code; it introduces no new attack surface.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| tests/test_no_logger_installed.rs exists | FOUND |
| .planning/intel/logger-history.md exists | FOUND |
| 68-01-SUMMARY.md exists | FOUND |
| Commit f5f00b7 exists | FOUND |
| Commit c9cfd22 exists | FOUND |
| Commit 37f9d0f exists | FOUND |
