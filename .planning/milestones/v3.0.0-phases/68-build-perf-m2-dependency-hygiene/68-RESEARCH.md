# Phase 68: Build-perf M2 — dependency hygiene — Research

**Researched:** 2026-06-15
**Domain:** Cargo feature flags, Rust logging ecosystem, library hygiene
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 — LogLevel / with_logs() fate:** Remove `LogLevel` enum, `with_logs()` builder method, and `log_level: LogLevel` field from `Configuration` entirely. They only existed to configure `env_logger::Builder.filter_level()`. With the auto-init removed they are dead code. v3.0.0 is the correct moment to drop them. MIGRATION.md documents the removal. Only 1 example (`examples/memetic_rastrigin.rs`) is affected.

**D-02 — log feature flags:** Keep `log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"] }` feature list unchanged. The `kv_unstable` structured key-value syntax is used throughout `LogObserver` and is stable enough in practice. No reformatting of LogObserver call sites needed.

**D-03 — LogObserver gating strategy:** Gate `LogObserver` (and its `pub use` re-export in `src/observe/observer/mod.rs:552`) behind `#[cfg(feature = "logging")]`. When users build with `default-features = false`, `LogObserver` is unavailable — which is correct since they have no log subscriber anyway. The no-op struct approach is rejected.

**D-04 — Internal macro vs. inline #[cfg]:** Use a tiny internal `crate::log_info!()` / `crate::log_debug!()` / `crate::log_trace!()` / `crate::log_warn!()` / `crate::log_error!()` macro family that expands to the real `log::` call when `logging` is on, and to `()` (no-op) when off. This avoids 109+ inline `#[cfg]` gates across 63 files. The macro family lives in `src/lib.rs` or a new `src/macros.rs` module.

### Claude's Discretion

None specified — all decisions are locked.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

Phase 68 eliminates the `env_logger` anti-pattern from the `genetic_algorithms` library and gates the `log` crate behind a new default-on `logging` feature. This is a pure dependency-hygiene change: the library continues to emit `log!()` events as today, but the application owns logger installation.

The work is split into two sequential plans. Plan 68-01 removes the auto-init call and `LogLevel` plumbing, moves `env_logger` to dev-deps, and updates every example. Plan 68-02 adds the `logging` feature gate, introduces a five-macro internal family to route 109+ call sites, gates `LogObserver` behind the feature, and extends the feature-matrix CI.

**Primary recommendation:** Follow the two-plan wave sequence strictly. Plan 68-01 has zero risk (removes code). Plan 68-02 carries the only complexity (macro family + 63-file sweep). Do not merge them in a single commit — reversibility requires atomic commits with `Revert plan:` bodies.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Logger auto-install removal | Library internals (`src/engines/ga.rs`) | — | The anti-pattern is a single call in `ga.rs:1588`; removing it is a library-level change |
| `env_logger` placement | `Cargo.toml` `[dev-dependencies]` | Examples (`examples/*.rs`) | Crate is still needed for examples/tests; only the library's `[dependencies]` changes |
| `LogLevel` / `with_logs()` removal | Configuration layer (`src/configuration.rs`, `src/traits/configuration.rs`, `src/configuration/builders.rs`) | `src/engines/ga.rs` (import + mapping block) | All four files must be cleaned together for zero dead-code warnings |
| `logging` feature gate | `Cargo.toml` + `src/lib.rs` macros | 63 `src/**/*.rs` call sites | Feature is declared in Cargo.toml; call sites are converted via the macro family |
| `LogObserver` gating | `src/observe/observer/mod.rs:551-552` + `src/lib.rs:357` | — | Two `cfg` attributes: one on the `mod log;` declaration, one on the `pub use` |
| Feature-matrix CI extension | `.github/workflows/feature-matrix.yml` | — | Two new matrix rows needed: `no-default-features` and `logging` feature explicit |
| Documentation | `MIGRATION.md`, `CHANGELOG.md`, `README.md`, `docs/getting-started.md`, `src/lib.rs`, `.planning/intel/` | — | Six audience-facing documents + two AI-facing intel files |

---

## Standard Stack

### Core (no new packages — dependency removal phase)

| Library | Current Role | Change |
|---------|-------------|--------|
| `env_logger = "0.11.6"` | `[dependencies]` (pulls 12 transitive crates) | Move to `[dev-dependencies]` |
| `log = "0.4.29"` (with `std`, `serde`, `kv_unstable`) | `[dependencies]` unconditional | Becomes `optional = true`, gated by new `logging` feature |

**No new crates are introduced in this phase.** [VERIFIED: Cargo.toml inspection]

### Verified current dep counts
- Total transitive deps (default features): **94** [VERIFIED: `cargo tree --prefix none | sort -u | wc -l` → 94]
- Baseline recorded in `.planning/baselines/v3.0.0-baseline.json`: **97** (includes some dev-dep counting variation)
- `env_logger` transitive crates to be shed: `aho-corasick`, `anstream`, `anstyle`, `anstyle-parse`, `anstyle-query`, `colorchoice`, `env_filter`, `env_logger`, `humantime`, `is_terminal_polyfill`, `memchr`, `regex`, `regex-automata`, `regex-syntax` — **14 unique entries** in the tree [VERIFIED: `cargo tree` inspection]

Note: `memchr` and `regex-automata` may be shared with `rand` or other deps; actual net reduction may be 10-12 unique crates rather than 14. The baseline gate will confirm.

### Package Legitimacy Audit

No new packages are installed. This phase removes packages. Audit not required.

---

## Architecture Patterns

### System Architecture Diagram

```
[Plan 68-01: Logger removal]

  Cargo.toml
    env_logger -> [dependencies]          REMOVE
    env_logger -> [dev-dependencies]      ADD

  src/engines/ga.rs (lines 1580-1590)
    LogLevel mapping block                REMOVE
    env_logger::Builder::try_init()       REMOVE
    `use configuration::LogLevel`         REMOVE

  src/configuration.rs
    LogLevel enum                         REMOVE
    log_level: LogLevel field             REMOVE
    log() accessor                        REMOVE
    LogLevel::Off default                 REMOVE

  src/traits/configuration.rs
    with_logs(self, log_level: LogLevel)  REMOVE (from trait definition)

  src/configuration/builders.rs
    with_logs(mut self, ...) impl         REMOVE (from ConfigurationT impl)

  examples/memetic_rastrigin.rs
    .with_logs(LogLevel::Warn)            REMOVE
    env_logger::init()                    ADD (in main())

  examples/*.rs (23 other examples)
    env_logger::init() in main()          ADD (restores log output)

  tests/test_no_logger_installed.rs       CREATE (new test)
  .planning/intel/logger-history.md       CREATE (rationale doc)
  MIGRATION.md, CHANGELOG.md, README.md,
  docs/getting-started.md                 UPDATE

[Plan 68-02: logging feature gate]

  Cargo.toml
    log = { ..., optional = true }        CHANGE
    default = ["logging"]                 ADD
    logging = ["dep:log"]                 ADD

  src/lib.rs (or src/macros.rs)
    macro_rules! log_info! { ... }        ADD
    macro_rules! log_debug! { ... }       ADD
    macro_rules! log_trace! { ... }       ADD
    macro_rules! log_warn! { ... }        ADD
    macro_rules! log_error! { ... }       ADD

  src/**/*.rs (63 files, 109+ call sites)
    log::info!(...) -> crate::log_info!() CHANGE
    `use log::{...}` imports              REMOVE (macros replace them)

  src/observe/observer/mod.rs:551-552
    mod log;                              ADD #[cfg(feature = "logging")]
    pub use log::LogObserver;             ADD #[cfg(feature = "logging")]

  src/lib.rs:357
    pub use observer::LogObserver;        ADD #[cfg(feature = "logging")]

  .github/workflows/feature-matrix.yml   UPDATE (add 2 new matrix rows)
  .planning/intel/feature-flags.md       CREATE (AI rationale doc)
  src/lib.rs Feature Flags table          UPDATE
  README.md Features table               UPDATE
  CHANGELOG.md                           UPDATE
```

### Recommended Project Structure (unchanged)

No structural changes to `src/`. New files:
```
src/
└── macros.rs           (optional — may go in lib.rs directly)
tests/
└── test_no_logger_installed.rs    (new test)
.planning/intel/
├── build-profile.md    (exists from Phase 67)
├── logger-history.md   (new)
└── feature-flags.md    (new)
```

### Pattern 1: Internal log macro family (D-04)

**What:** Five `macro_rules!` macros that delegate to `log::*` when the `logging` feature is active, and expand to `()` when it is not.

**When to use:** Every call site in `src/**` that currently calls `log::info!`, `log::debug!`, `log::trace!`, `log::warn!`, `log::error!`.

**Example:**
```rust
// Defined once in src/lib.rs (or src/macros.rs, then #[macro_use] mod macros;)
// Source: D-04 in 68-CONTEXT.md

#[cfg(feature = "logging")]
macro_rules! log_info {
    ($($arg:tt)*) => { log::info!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_info {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "logging")]
macro_rules! log_debug {
    ($($arg:tt)*) => { log::debug!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_debug {
    ($($arg:tt)*) => {};
}

// ... similarly for log_trace!, log_warn!, log_error!
```

Call sites become:
```rust
// Before:
log::info!(target="ga_events", method="run"; "Generation: {}", n);

// After:
crate::log_info!(target="ga_events", method="run"; "Generation: {}", n);
```

**Critical detail about `use log::{...}` imports:** Files that currently do `use log::{debug, trace};` and then call `debug!(...)` directly must be converted to `crate::log_debug!(...)` with the `use` import removed. There is no way to make the bare `debug!` macro work with feature gating without also touching the imports. [ASSUMED — based on how Rust macro scoping works]

### Pattern 2: `dep:` optional dependency in Cargo.toml

**What:** The Cargo `dep:` prefix makes a crate optional without creating an implicit feature of the same name.

**Example:**
```toml
# Source: Cargo.toml inspection + established pattern in this codebase
[features]
default = ["logging"]
logging = ["dep:log"]

[dependencies]
log = { version = "0.4.29", features = ["std", "serde", "kv_unstable"], optional = true }
```

This matches the existing pattern used by `serde = ["dep:serde", "dep:serde_json"]` in this codebase. [VERIFIED: Cargo.toml inspection]

### Pattern 3: `#[cfg(feature = "logging")]` gate on a module

**What:** Gate an entire module and its re-export with matching `cfg` attributes.

**Example:**
```rust
// src/observe/observer/mod.rs
// Source: established pattern from observer-tracing and observer-metrics in this file

#[cfg(feature = "logging")]
mod log;
#[cfg(feature = "logging")]
pub use log::LogObserver;

// (existing pattern already used for observer-tracing and observer-metrics at lines 554-568)
```

And the crate-level re-export in `src/lib.rs:357`:
```rust
#[cfg(feature = "logging")]
pub use observer::LogObserver;
```

### Pattern 4: test_no_logger_installed.rs

**What:** A test that verifies no logger is installed during GA execution.

**Approach (from CONTEXT.md §Specific Ideas):**
```rust
// tests/test_no_logger_installed.rs
// Register a custom Log impl that panics if called.
// Run the GA without any subscriber.
// If the test passes, the auto-init is gone.

struct PanicLogger;
impl log::Log for PanicLogger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }
    fn log(&self, record: &log::Record) {
        panic!("GA installed a logger and emitted: {}", record.args());
    }
    fn flush(&self) {}
}

#[test]
fn ga_does_not_install_logger() {
    // set_logger fails if already set — use try_set_logger or similar
    // The test verifies PanicLogger is never triggered.
    let _ = log::set_logger(&PanicLogger).map(|()| log::set_max_level(log::LevelFilter::Trace));
    // ... build and run a minimal GA ...
    // If we reach here without panicking, the auto-init is gone.
}
```

**Warning:** `log::set_logger` is a once-per-process call. Tests run in parallel by default. This test must run in isolation or use `log::try_set_logger`. [ASSUMED — based on log crate semantics; confirm with `cargo test -- --test-per-thread` or `serial_test` crate if needed. The simpler approach may be checking the source code directly rather than a runtime test.]

**Simpler alternative approach:** Instead of a panic logger, assert at compile time that `env_logger` is not in `[dependencies]` by checking `cargo tree --depth 1 | grep env_logger`. However, the CONTEXT.md specifies a runtime test, so the panic-logger approach is the specified design.

### Anti-Patterns to Avoid

- **Bare `#[cfg]` at every call site:** 109 inline `#[cfg(feature = "logging")]` gates across 63 files — rejected in favor of the macro family (D-04).
- **No-op `LogObserver` struct when feature is off:** Rejected in D-03 as misleading and complex.
- **`use log` imports left in files after call-site conversion:** If a file does `use log::{debug, trace}` and then calls `debug!()`, converting the call to `crate::log_debug!()` but leaving the `use` will produce dead-import warnings (or errors with `#[deny(unused_imports)]`). Every converted file needs its `use log::*` imports removed.
- **Forgetting the `src/lib.rs:357` re-export gate:** The `observer/mod.rs` gate alone is not sufficient; `src/lib.rs` also re-exports `LogObserver` at line 357 and needs its own `#[cfg(feature = "logging")]`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Optional log routing | Per-site `#[cfg]` gates | Internal `crate::log_*!` macro family | 5 macros cover all 109+ sites; macros are zero-cost (expand to `()` when feature is off) |
| Feature-optional crate | `Option<Box<dyn Log>>` at runtime | Cargo `dep:log` optional dependency | Cargo handles the compile-time erasure; no runtime overhead |
| Logger non-installation test | Process-level log subscriber inspection | `log::set_logger` + panic impl | The `log` crate's global logger slot is designed for exactly this pattern |

**Key insight:** Cargo optional dependencies + compile-time `cfg` gates are the idiomatic Rust solution for shedding dep weight. Runtime feature detection adds unnecessary complexity and overhead.

---

## Common Pitfalls

### Pitfall 1: Partial call-site sweep (Plan 68-02)

**What goes wrong:** Some files' `log::*` calls are converted but others are missed, leaving unconditional `log::` references that fail to compile when `logging` is off.

**Why it happens:** 63 files across `src/operations/`, `src/engines/`, `src/observe/`, and `src/types/`. A manual sweep misses files.

**How to avoid:** After the sweep, run `cargo check --no-default-features` locally before committing. CI feature-matrix (`--no-default-features`) will catch this, but catching it locally is cheaper.

**Warning signs:** Compile error mentioning `log::info` or `use log::` in any `src/` file when building without the `logging` feature.

### Pitfall 2: `use log::{debug, trace}` bare imports not removed

**What goes wrong:** A file converts `debug!(...)` to `crate::log_debug!(...)` but still has `use log::{debug, trace}` at the top. When `logging` is off, `log` is not compiled in, so the `use` statement is a dangling import — compile error.

**Why it happens:** The sweep focuses on call sites and misses import statements.

**How to avoid:** For each of the 63 files, grep for `use log::` and remove or gate those lines alongside the call-site conversion.

### Pitfall 3: `kv_unstable` syntax in LogObserver not available when gated wrong

**What goes wrong:** `src/observe/observer/log.rs` uses `log::info!(target="ga_events", key=value; "msg")` KV syntax. If `log` is present but `kv_unstable` feature is not on, these calls fail to compile.

**Why it happens:** Gating `log` as optional without preserving the feature list.

**How to avoid:** The `optional = true` change in Cargo.toml must preserve `features = ["std", "serde", "kv_unstable"]` — which is exactly what D-02 specifies. The planner must not simplify this to `log = { version = "...", optional = true }` (bare).

### Pitfall 4: `log::set_logger` is a once-per-process call in the test

**What goes wrong:** `tests/test_no_logger_installed.rs` calls `log::set_logger` but another test in the same process already called it (e.g., a test that uses `env_logger::try_init()`), causing `set_logger` to return `Err(SetLoggerError)`.

**Why it happens:** Rust's `cargo test` runs tests in parallel threads within the same process by default. The `log` global logger is set once.

**How to avoid:** Use `log::set_logger` inside the test with `log::try_set_logger` (or `set_logger` + ignore `Err`), or run the test in isolation (`#[test] -- test_name --exact`). The test should be in its own integration test binary (`tests/test_no_logger_installed.rs`) which is a separate process, so it gets a clean logger slot. Integration test files are separate processes in Rust. [VERIFIED: Rust test model — `tests/` integration tests are separate binaries]

### Pitfall 5: `LogLevel` still in `src/lib.rs` public re-export path

**What goes wrong:** `LogLevel` is re-exported from `src/configuration.rs` → visible as `genetic_algorithms::configuration::LogLevel`. Removing it without checking the public API surface will break any downstream user who imports it.

**Why it happens:** v3.0.0 is a breaking-change release, so removal is intentional — but the `cargo public-api` diff must confirm the removal is accounted for in MIGRATION.md and CHANGELOG.md.

**How to avoid:** After removal, run `cargo doc --no-deps` and verify no rustdoc warnings. Check `cargo public-api` diff shows only the intentional removal.

### Pitfall 6: `obs/log` module file naming collision

**What goes wrong:** When the `logging` feature is off, `mod log;` is behind `#[cfg(feature = "logging")]`. But the file `src/observe/observer/log.rs` still exists on disk. Rust's module system simply doesn't compile the file — no error. However, if a developer removes the cfg gate by mistake, they get a name clash with the `log` crate.

**Why it happens:** The file is named `log.rs`, same as the `log` crate. Rust resolves module paths through the `mod` declaration, not the crate name, so there is no actual conflict in practice.

**How to avoid:** This is not a real problem; document in the code comment above `mod log;` that the name matches the crate intentionally.

---

## Code Examples

### Cargo.toml: Making log optional

```toml
# Source: established codebase pattern (see serde = ["dep:serde", ...]) + D-02/D-04

[features]
default = ["logging"]
logging = ["dep:log"]

[dependencies]
log = { version = "0.4.29", features = ["std", "serde", "kv_unstable"], optional = true }
env_logger = "0.11.6"   # REMOVED from here, moves to [dev-dependencies]

[dev-dependencies]
env_logger = "0.11.6"   # MOVED HERE
```

### src/lib.rs: Internal macro family

```rust
// Source: D-04 in 68-CONTEXT.md + Rust macro_rules! documentation

// Placed near the top of src/lib.rs, before other modules, or in a
// dedicated `mod macros { ... }` block with `#[macro_use]`.

#[cfg(feature = "logging")]
macro_rules! log_info {
    ($($arg:tt)*) => { ::log::info!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_info {
    ($($arg:tt)*) => { () };
}

#[cfg(feature = "logging")]
macro_rules! log_debug {
    ($($arg:tt)*) => { ::log::debug!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_debug {
    ($($arg:tt)*) => { () };
}

#[cfg(feature = "logging")]
macro_rules! log_trace {
    ($($arg:tt)*) => { ::log::trace!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_trace {
    ($($arg:tt)*) => { () };
}

#[cfg(feature = "logging")]
macro_rules! log_warn {
    ($($arg:tt)*) => { ::log::warn!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_warn {
    ($($arg:tt)*) => { () };
}

#[cfg(feature = "logging")]
macro_rules! log_error {
    ($($arg:tt)*) => { ::log::error!($($arg)*) };
}
#[cfg(not(feature = "logging"))]
macro_rules! log_error {
    ($($arg:tt)*) => { () };
}
```

**Note on `crate::` prefix:** `macro_rules!` macros defined at crate root are accessible throughout the crate without a path prefix in the same way `#[macro_export]` works. For intra-crate use without `#[macro_export]`, macros defined in `lib.rs` are visible to all modules as `crate::log_info!()` via the 2018+ edition's macro namespacing. [ASSUMED — verify with a quick `cargo check` after defining the macros]

### examples/memetic_rastrigin.rs: Remove with_logs

```rust
// Before (line 82):
.with_logs(genetic_algorithms::configuration::LogLevel::Warn)

// After: remove that line entirely.
// Add to fn main():
env_logger::init();   // or env_logger::try_init().ok();
```

### feature-matrix.yml: Two new matrix rows

```yaml
# Source: .github/workflows/feature-matrix.yml inspection

# Add after existing "all-features" entry:
- name: "no-default-features"
  features: "--no-default-features"
  cmd: "cargo test --quiet --no-default-features"
- name: "logging-explicit"
  features: "--no-default-features --features logging"
  cmd: "cargo test --quiet --no-default-features --features logging"
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Library installs `env_logger` auto-logger | Library only emits `log!()` events; app installs subscriber | Phase 68 (v3.0.0) | Removes 12-14 transitive crates from user builds |
| `log` unconditional dep | `log` gated behind `logging` feature (default-on) | Phase 68 (v3.0.0) | `--no-default-features` shed the log crate entirely |
| `LogLevel` enum in public API | Removed | Phase 68 (v3.0.0) | Users filter log output via their own subscriber config |

**Deprecated/outdated:**
- `LogLevel` enum: removed in v3.0.0; users who imported it must remove the import.
- `.with_logs(LogLevel::X)` builder method: removed in v3.0.0; document in MIGRATION.md.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Bare `debug!(...)` calls (via `use log::{debug}`) require both the import removal AND the call-site conversion in every file | Pitfall 2 | Missing imports cause compile errors on `--no-default-features` |
| A2 | `crate::log_info!()` syntax works from within modules to access macros defined in `lib.rs` (2018+ edition intra-crate macro scoping) | Code Examples — macro family | If wrong, `#[macro_use]` or `pub(crate)` annotation needed on each macro |
| A3 | `tests/test_no_logger_installed.rs` as an integration test file runs in a separate process, giving it a clean `log` global slot | Pitfall 4 | If shared-process, `set_logger` may fail silently; test may not exercise what it claims |
| A4 | `memchr`, `regex-automata` etc. are unique to `env_logger` and will be shed after the move to dev-deps | Standard Stack — dep counts | If shared with another dep, actual crate reduction will be less than 12 |

---

## Open Questions

1. **`crate::log_*!` macro accessibility within modules**
   - What we know: Rust 2018 edition allows `use crate::my_macro` for `macro_rules!` macros defined at crate root (they are in scope implicitly via `$crate`)
   - What's unclear: Whether the macros need `#[macro_export]` for use outside `lib.rs` or can rely on implicit crate scoping
   - Recommendation: Define macros in `src/lib.rs` and verify with `cargo check` immediately after adding the first macro + converting one call site. No `#[macro_export]` is needed for intra-crate use in Rust 2018+.

2. **Which examples use log output implicitly (via the auto-init)**
   - What we know: Only `memetic_rastrigin.rs` uses `.with_logs(LogLevel::Warn)` explicitly. No other example calls `env_logger::init()`.
   - What's unclear: Whether any other example's stdout output (checked in golden tests or examples-smoke CI) includes log lines that would disappear after the change
   - Recommendation: The golden tests check for specific output lines (`Finished. Best fitness:`, `Best fitness:`) not log lines. The examples-smoke CI runs examples and checks exit code. Adding `env_logger::try_init().ok()` to all 24 examples is safe — it restores the pre-change log behaviour for developers running examples manually. Plan 68-01 should add this to all examples.

3. **Is `colorchoice` / `is_terminal_polyfill` also pulled in by `anstream` from a non-env_logger dep?**
   - What we know: `cargo tree` shows both under `env_logger -> anstream -> *`. No other dependency in the current tree appears to depend on `anstream`.
   - What's unclear: Whether `rand`, `rayon`, or any other dep has a transitive path to `anstream`
   - Recommendation: After moving `env_logger` to dev-deps, run `cargo tree --prefix none | sort -u | wc -l` and compare to current 94. The baseline gate in CI will catch any discrepancy.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build + test | ✓ | (current stable) | — |
| `wasm32-unknown-unknown` target | WASM check gate | ✓ | (CI installs) | — |
| `cargo-public-api` | API surface verification | [ASSUMED] unknown | — | Manual rustdoc diff |
| Python 3 | `build-perf-gate.yml` regression script | ✓ | (CI Ubuntu has it) | — |

**Missing dependencies with no fallback:** None that block the phase.

**Missing dependencies with fallback:**
- `cargo-public-api`: If unavailable, manually check `cargo doc --no-deps` output for any unintended removals. The MIGRATION.md documents all intentional API removals.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | `Cargo.toml` (`[profile.test]`) |
| Quick run command | `cargo test --quiet` |
| Full suite command | `cargo test --quiet && cargo test --quiet --no-default-features && cargo test --quiet --all-features` |

### Phase Requirements to Test Map

| Behavior | Test Type | Automated Command | File Exists? |
|----------|-----------|-------------------|-------------|
| GA does not call `env_logger::Builder::try_init()` | Integration (runtime) | `cargo test --test test_no_logger_installed` | ❌ Wave 0 (Plan 68-01) |
| `--no-default-features` compiles clean (logging off) | Compile | `cargo check --no-default-features` | ✓ (CI) |
| `--features logging` compiles clean | Compile | `cargo check --features logging` | ✓ after CI update |
| All feature combinations green | Integration | Feature-matrix CI | ✓ (needs 2 new rows) |
| CC-3 golden tests byte-identical | Regression | `cargo test --test golden_tests --release` | ✓ (tests/golden_tests.rs) |
| WASM check still passes | Compile | `cargo check --target wasm32-unknown-unknown --lib` | ✓ (CI) |
| Zero rustdoc warnings | Doc | `cargo doc --no-deps 2>&1 \| grep warning` | ✓ (part of CI) |

### Sampling Rate

- **Per task commit:** `cargo check --no-default-features && cargo check`
- **Per wave merge:** `cargo test --quiet && cargo test --quiet --no-default-features`
- **Phase gate:** Feature-matrix CI green + CC-3 golden tests byte-identical + `cargo doc --no-deps` zero warnings

### Wave 0 Gaps

- [ ] `tests/test_no_logger_installed.rs` — verify GA does not call `env_logger::init()` (new file, Plan 68-01)
- [ ] `.github/workflows/feature-matrix.yml` — add `no-default-features` and `logging-explicit` rows (Plan 68-02)

---

## Security Domain

Security domain is not applicable to this phase. This phase removes a dependency and adds a feature gate. It introduces no authentication, session management, cryptography, input parsing, or access-control logic. No ASVS categories apply.

---

## Project Constraints (from CLAUDE.md)

The following CLAUDE.md directives are directly relevant to this phase:

1. **WASM Compatibility (mandatory):** Every change must compile for `wasm32-unknown-unknown`. After Plan 68-01 and Plan 68-02, run `cargo check --target wasm32-unknown-unknown`. The `env_logger` removal helps WASM (env_logger doesn't compile for wasm32). The `logging` feature gate must be verified on WASM: with `default-features` (which includes `logging`), the log macros must expand to real calls; the `log` crate itself is wasm-compatible. [VERIFIED: log crate supports wasm32 via `std` feature which is OS-independent]

2. **No Breaking Changes (default policy) — exception applies:** v3.0.0 is the declared breaking-change release. `LogLevel` removal and `with_logs()` removal are intentional v3.0.0 breaks, documented in MIGRATION.md. The `env_logger` auto-install removal is a behavioural change also documented. This is the correct moment for these breaks.

3. **Observability initiative:** `LogObserver` must survive this phase — it is only gated, not removed. When `logging` feature is on (the default), `LogObserver` is available as before. No observability hook is removed.

4. **Signed commits mandatory:** Every commit must be GPG-signed. The `Revert plan:` commit body line is also required per BUILD-PERF.md non-negotiable #5.

5. **Tests in tests/ folder:** `tests/test_no_logger_installed.rs` correctly goes in `tests/` (not inline). ✓

6. **Branching:** Work goes on `feat/68-*` branches targeting the milestone branch, never directly to milestone or main.

---

## Sources

### Primary (HIGH confidence)

- `68-CONTEXT.md` — All locked decisions (D-01 through D-04), file lists, line numbers — read directly
- `.planning/v3.0.0-BUILD-PERF.md` — Action #1 and #2 exact change specs, non-negotiable guarantees, revert plans — read directly
- `.planning/ROADMAP.md §Phase 68` — Success criteria (8 items), 2-plan wave structure — read directly
- `Cargo.toml` — Current dep list, `env_logger = "0.11.6"` in `[dependencies]`, existing feature patterns — read directly
- `src/observe/observer/mod.rs:551-552` — `mod log;` + `pub use log::LogObserver;` exact lines — verified by inspection
- `src/lib.rs:357` — `pub use observer::LogObserver;` — verified by inspection
- `cargo tree` output — 14 `env_logger` transitive crates identified, 94 current unique deps — run directly
- `.planning/baselines/v3.0.0-baseline.json` — dep_count: 97 (baseline), dev_build_s: 3.658 s — read directly

### Secondary (MEDIUM confidence)

- `src/configuration.rs:84-110` — `LogLevel` enum definition, `log_level` field, default `Off` — read directly
- `src/configuration/builders.rs:221` — `with_logs` impl — verified by grep
- `src/engines/ga.rs:1580-1590` — Auto-init block location — verified by grep
- `examples/memetic_rastrigin.rs:82` — Only example using `.with_logs()` — verified by grep

### Tertiary (LOW confidence)

- `crate::log_*!` macro accessibility within modules (2018 edition intra-crate scoping) — based on Rust language knowledge, not verified with a `cargo check` in this session [ASSUMED — A2 in Assumptions Log]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — current Cargo.toml verified directly; no new crates
- Architecture: HIGH — all file locations verified by grep; exact line numbers from inspection
- Pitfalls: HIGH (structural) / MEDIUM (macro scoping detail) — confirmed by codebase inspection
- Patterns: HIGH — based on existing codebase conventions confirmed by grep

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (stable domain; Rust/Cargo feature semantics do not change rapidly)
