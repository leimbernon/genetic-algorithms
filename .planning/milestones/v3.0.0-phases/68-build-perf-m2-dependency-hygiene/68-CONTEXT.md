# Phase 68: Build-perf M2 — dependency hygiene - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Eliminate the `env_logger` anti-pattern from the library (auto-install in `src/engines/ga.rs:1588`) and gate `log` behind a default-on `logging` feature. Two plans run sequentially (Wave 0 → Wave 1 in ROADMAP). No behavioural change for users who build with defaults.

**What this phase delivers:**
- `env_logger` removed from `[dependencies]`, moved to `[dev-dependencies]`
- `LogLevel` enum, `with_logs()` builder method, and `log_level` config field removed entirely
- Every example that relied on the auto-installed logger calls `env_logger::init()` explicitly
- New `logging` feature flag (default-on) gates the `log` crate dependency
- `LogObserver` gated behind `#[cfg(feature = "logging")]`
- Test `tests/test_no_logger_installed.rs` asserts GA does not install a logger
- Documentation updated across MIGRATION.md, CHANGELOG.md, README.md, docs/

</domain>

<decisions>
## Implementation Decisions

### LogLevel / with_logs() fate
- **D-01:** Remove `LogLevel` enum, `with_logs()` builder method, and `log_level: LogLevel` field from `Configuration` entirely. They only existed to configure `env_logger::Builder.filter_level()`. With the auto-init removed they are dead code. v3.0.0 is the correct moment to drop them. MIGRATION.md documents the removal. Only 1 example (`examples/memetic_rastrigin.rs`) is affected.

### log feature flags
- **D-02:** Keep `log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"] }` feature list unchanged. The `kv_unstable` structured key-value syntax is used throughout `LogObserver` (`src/observe/observer/log.rs`) and is stable enough in practice. No reformatting of LogObserver call sites needed.

### LogObserver gating strategy
- **D-03:** Gate `LogObserver` (and its `pub use` re-export in `src/observe/observer/mod.rs:552`) behind `#[cfg(feature = "logging")]`. When users build with `default-features = false`, `LogObserver` is unavailable — which is correct since they have no log subscriber anyway. The no-op struct approach is rejected as misleading and complex.

### Internal macro vs. inline #[cfg] for log call sites
- **D-04:** Follow BUILD-PERF.md §Action #2 preference: use a tiny internal `crate::log_info!()` / `crate::log_debug!()` etc. macro family that expands to the real `log::` call when `logging` is on, and to `()` (no-op) when off. This avoids 109+ inline `#[cfg]` gates across 63 files. The macro lives in `src/lib.rs` or a new `src/macros.rs` module.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary specification
- `.planning/v3.0.0-BUILD-PERF.md` §Wave M2 — Exact change list, verification criteria, doc deliverables, and revert plans for both plans. MUST READ before writing any plan.
- `.planning/ROADMAP.md` §Phase 68 — Success criteria (7 items), plan list (2 sequential plans), dependency on Phase 67.

### Files to change (Plan 68-01)
- `src/engines/ga.rs` — Remove `env_logger::Builder::from_default_env().filter_level(log_level).try_init()` block (lines 1580–1590); remove `LogLevel` import and mapping block
- `src/configuration.rs` — Remove `LogLevel` enum, `log_level: LogLevel` field from `GaConfiguration`, `log()` accessor, default assignment
- `src/traits/configuration.rs` — Remove `with_logs(self, log_level: LogLevel) -> Self` from `ConfigurationT` trait
- `Cargo.toml` — Move `env_logger` from `[dependencies]` to `[dev-dependencies]`
- `examples/memetic_rastrigin.rs` — Remove `.with_logs(LogLevel::Warn)` call; add `env_logger::init()` in `main()`
- All other `examples/*.rs` — Add `env_logger::init()` (or `env_logger::try_init().ok()`) to `main()` to restore log output they implicitly relied on

### Files to change (Plan 68-02)
- `Cargo.toml` — Add `logging` feature gating `dep:log`; `default = ["logging"]`
- `src/lib.rs` — Add internal log macro family (`log_info!`, `log_debug!`, `log_trace!`, `log_warn!`, `log_error!`) with no-op expansions when `logging` off
- `src/**` — 63 files, 109+ `log::*` call sites converted to internal macros
- `src/observe/observer/log.rs` — Gate entire file behind `#[cfg(feature = "logging")]`
- `src/observe/observer/mod.rs:552` — Gate `pub use log::LogObserver` behind `#[cfg(feature = "logging")]`
- `.github/workflows/` — Feature-matrix CI must add `--no-default-features` and `--features logging` combinations

### Documentation deliverables
- `MIGRATION.md` — New recipe "Logger setup (v2 auto-init → v3 explicit)" + "LogLevel removed" entry
- `CHANGELOG.md` — v3.0.0 Changed/breaking: env_logger removal, LogLevel removal, logging feature added
- `README.md` — "Logging" subsection updated; Features table: add `logging` row
- `docs/getting-started.md` — Quick Start snippet adds `env_logger::init()`
- `src/lib.rs` — Feature Flags table: add `logging` row
- `.planning/intel/logger-history.md` — Rationale record so future AI agents don't reintroduce auto-init
- `.planning/intel/feature-flags.md` — AI-readable note on feature-flag philosophy

### Non-negotiable guarantees
- `.planning/v3.0.0-BUILD-PERF.md` §Non-negotiable guarantees — All 6 guarantees: zero behavioural regression, zero public-API regression (beyond intentional breaks), all feature combos green, zero rustdoc warnings, reversibility (every commit body has `Revert plan:`), measurement before and after.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/observe/observer/log.rs` — `LogObserver` implementation: 20+ `log::` calls using `kv_unstable` key-value syntax. Entire file gets gated behind `#[cfg(feature = "logging")]` in Plan 68-02.
- `src/observe/observer/mod.rs:552` — `pub use log::LogObserver` re-export. Needs `#[cfg(feature = "logging")]` guard added.

### Established Patterns
- Feature flag gating: existing flags (`serde`, `observer-tracing`, `observer-metrics`, `visualization`, `benchmarks`) use `#[cfg(feature = "...")]` at the module and `use` statement level — same pattern applies to `logging`.
- WASM cfg gate: `#[cfg(not(target_arch = "wasm32"))]` is the established pattern. The new `#[cfg(feature = "logging")]` gates are additive and follow the same style.
- `dep:` prefix in Cargo.toml features (e.g., `serde = ["dep:serde", "dep:serde_json"]`) — use same pattern for `logging = ["dep:log"]`.

### Integration Points
- `src/engines/ga.rs:1580–1590` — Only location where `env_logger` is called. The `LogLevel` mapping block (lines 1580–1586) and the `try_init()` call (1588–1590) are both removed in Plan 68-01.
- `Cargo.toml:43` — `log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"] }` becomes `log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"], optional = true }`.
- `Cargo.toml:46` — `env_logger = "0.11.5"` moves from `[dependencies]` to `[dev-dependencies]`.

### Build Verification
- `cargo check --target wasm32-unknown-unknown` — must pass after each plan.
- `cargo test --all-features` and `cargo test --no-default-features` — both must pass after Plan 68-02.
- CC-3 golden tests (`tests/golden/`) — must be byte-identical after both plans.

</code_context>

<specifics>
## Specific Ideas

- The new test `tests/test_no_logger_installed.rs` verifies the GA does not install a logger. Approach: register a custom `log::Log` implementation that panics if called, then run the GA without setting up any subscriber. If the test passes, the auto-init is gone.
- Commit bodies for both plans MUST include a `Revert plan:` line per BUILD-PERF.md non-negotiable guarantee #5.
- `.planning/intel/logger-history.md` is a human+AI-readable rationale file. Date the rationale entry: 2026-06-15.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 68-build-perf-m2-dependency-hygiene*
*Context gathered: 2026-06-15*
