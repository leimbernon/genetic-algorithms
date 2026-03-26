# Phase 15: TracingObserver - Research

**Researched:** 2026-03-26
**Domain:** Rust `tracing` crate, feature-gated optional dependencies, observer pattern with Send+Sync constraints
**Confidence:** HIGH

## Summary

Phase 15 implements `TracingObserver` — a `GaObserver<U>` that emits structured tracing spans and events — behind the `observer-tracing` feature flag. The phase is narrow in scope: one new file (`src/observer/tracing.rs`), two lines in `Cargo.toml`, and three lines in `src/observer/mod.rs` and `src/lib.rs`. All observer infrastructure is already in place from Phase 13 and 14.

The critical design discovery concerns `EnteredSpan: !Send`. The CONTEXT.md decision to store `Mutex<Option<tracing::span::EnteredSpan>>` will **not compile** because `EnteredSpan` explicitly implements `!Send`, making `Mutex<Option<EnteredSpan>>` also `!Send`, which fails the `GaObserver<U>: Send + Sync` requirement. The correct storage type is `Mutex<Option<tracing::span::Span>>` — `Span` itself is `Send + Sync`. Each hook re-enters the stored span for its duration via a local `enter()` guard.

A secondary finding: the CONTEXT.md says `GenerationStats` fields include `mean_fitness`, but the actual struct field is `avg_fitness`. The planner must use the real field name when mapping stats to tracing event fields.

**Primary recommendation:** Store `Mutex<Option<Span>>` (not `EnteredSpan`). Call `span.enter()` locally within each hook to nest events under the generation span. This is correct, compiles, and keeps the observer `Send + Sync`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Span architecture:**
- Two-level span hierarchy: root span `"ga_run"` (INFO) for the entire run + child span `"ga_generation"` (DEBUG) per generation
- `on_run_start` enters the `ga_run` span; `on_run_end` drops it
- `on_generation_start` enters the `ga_generation` span; `on_generation_end` exits it (and emits a final event with stats fields before dropping)
- All operator events and special events appear nested under the active `ga_generation` span
- Interior mutability: `Mutex<Option<...>>` for both `run_span` and `gen_span` fields (see research for the corrected type)

**Hook coverage and levels — all 12 hooks emit:**
| Hook | Level | Notes |
|------|-------|-------|
| `on_run_start` | INFO | fires once |
| `on_run_end` | INFO | includes `cause` and `total_generations` |
| `on_generation_start` | DEBUG | span entry (no event emitted separately) |
| `on_generation_end` | DEBUG | event with full `GenerationStats` fields before dropping span |
| `on_new_best` | INFO | includes `generation` and `fitness` |
| `on_stagnation` | WARN | includes `generation` and `stagnation_count` |
| `on_extension_triggered` | INFO | includes all `ExtensionEvent` fields |
| `on_selection_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_crossover_complete` | TRACE | includes `generation`, `duration_ms`, `offspring_count` |
| `on_mutation_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_fitness_evaluation_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_survivor_selection_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |

**Field naming:**
- Duration: `duration_ms = dur.as_secs_f64() * 1000.0` (f64)
- GenerationStats fields: match struct field names directly — `generation`, `best_fitness`, `mean_fitness`, `diversity` — no translation layer (see research: actual field is `avg_fitness`, not `mean_fitness`)
- Span names: `"ga_run"` and `"ga_generation"`
- ExtensionEvent fields: `generation`, `diversity`, `extension_type`, `threshold`

**Struct design:**
```rust
pub struct TracingObserver {
    run_span: Mutex<Option<tracing::span::EnteredSpan>>,  // see research: must be Span, not EnteredSpan
    gen_span: Mutex<Option<tracing::span::EnteredSpan>>,  // same correction
}

impl TracingObserver {
    pub fn new() -> Self { ... }
}

impl Default for TracingObserver { ... }
```
Users attach with: `ga.with_observer(Arc::new(TracingObserver::new()))`

**Feature flag:** `observer-tracing`
- Adds `tracing` as optional dependency
- Entire `src/observer/tracing.rs` is `#[cfg(feature = "observer-tracing")]`

**Critical constraint: no log::* calls**
`TracingObserver` must emit exclusively via `tracing::event!()` and `tracing::span!()` — never `log::*`.

### Claude's Discretion
- Whether to use `tracing::span!()` macro or `tracing::Span::new()` API for span creation
- Exact handling of Mutex poison (can `unwrap()`)
- Whether `TracingObserver` implements `Default` in addition to `new()`
- Test strategy for TRAC-03 (LogTracer integration test)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TRAC-01 | User can attach `TracingObserver` (behind `observer-tracing` feature flag) to emit structured tracing spans and events per generation | Tracing crate API verified; span + event macros documented; Send+Sync-safe storage pattern established |
| TRAC-02 | `TracingObserver` compiles only when `--features observer-tracing` is enabled; default builds are entirely unaffected | Feature flag pattern verified from `serde` and `visualization` precedents in Cargo.toml |
| TRAC-03 | `TracingObserver` is safe to use alongside `LogTracer` — emits exclusively via `tracing::event!()`, no infinite recursion possible | LogTracer bridge mechanism verified; no-log constraint enforced in CONTEXT.md; test pattern documented |
</phase_requirements>

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tracing` | 0.1.44 | Span creation (`info_span!`, `debug_span!`), event emission (`info!`, `debug!`, `trace!`, `warn!`) | Tokio-ecosystem standard; OTel/Jaeger/Tempo all consume tracing spans |

### Dev-Dependencies (for tests only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing-subscriber` | 0.3.23 | Subscriber for integration tests; captures events for assertion | TRAC-03 test — must be dev-dependency only, never in `[dependencies]` |
| `tracing-log` | 0.2.0 | `LogTracer` for TRAC-03 bridge test | dev-dependency only; verifies no infinite recursion when both active |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tracing::info_span!` | `tracing::span!(Level::INFO, ...)` | Both work; `info_span!` is more concise and idiomatic |
| `Mutex<Option<Span>>` + re-enter | `thread_local!` span storage | `Mutex` is `Send+Sync`, thread_local would break island parallelism |

**Installation (Cargo.toml additions):**
```toml
[dependencies]
tracing = { version = "0.1", optional = true }

[features]
observer-tracing = ["dep:tracing"]

[dev-dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-log = "0.2"
```

**Version verification:**
- `tracing`: 0.1.44 (verified 2026-03-26, crates.io)
- `tracing-subscriber`: 0.3.23 (verified 2026-03-26, crates.io)
- `tracing-log`: 0.2.0 (verified 2026-03-26, crates.io)

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── observer/
│   ├── mod.rs           # GaObserver trait, ExtensionEvent, NoopObserver, re-exports
│   ├── log.rs           # LogObserver (already exists - structural template)
│   └── tracing.rs       # TracingObserver (new, behind #[cfg(feature = "observer-tracing")])
```

### Pattern 1: Feature-Gated Observer Module

This is the exact pattern used by `src/checkpoint.rs` (`serde`) and `src/visualization/` (`visualization`).

**What:** The module file exists only when the feature is enabled. The parent module conditionally declares it.
**When to use:** Any optional dependency that should add zero cost to default builds.

In `src/observer/mod.rs`:
```rust
#[cfg(feature = "observer-tracing")]
mod tracing_observer;
#[cfg(feature = "observer-tracing")]
pub use tracing_observer::TracingObserver;
```

Note: the module file cannot be named `tracing.rs` because `tracing` is a crate name. Use `tracing_observer.rs` to avoid the name collision.

In `src/lib.rs`:
```rust
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
```

### Pattern 2: Send+Sync-Safe Span Storage

**What:** Store `Span` (which is `Send + Sync`) in a `Mutex<Option<Span>>`. Do NOT store `EnteredSpan` (which is `!Send`).
**When to use:** Any observer that needs to maintain span lifetime across multiple `&self` method calls.

```rust
// Source: tracing 0.1.44 docs — Span implements Send + Sync, EnteredSpan implements !Send
use std::sync::Mutex;
use tracing::Span;

pub struct TracingObserver {
    run_span: Mutex<Option<Span>>,
    gen_span: Mutex<Option<Span>>,
}
```

Each hook that needs to nest events under the generation span re-enters it locally:
```rust
fn on_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
    let guard = self.gen_span.lock().unwrap();
    let _enter = guard.as_ref().map(|s| s.enter());
    let duration_ms = duration.as_secs_f64() * 1000.0;
    tracing::trace!(generation, duration_ms, population_size, "selection_complete");
}
```

### Pattern 3: Two-Level Span Lifecycle

**What:** `ga_run` span covers the entire run; `ga_generation` span covers one generation.
**When to use:** Structured profiling where per-generation operator timing must roll up to a run-level span.

```rust
// on_run_start: create and store run span
fn on_run_start(&self) {
    let span = tracing::info_span!("ga_run");
    *self.run_span.lock().unwrap() = Some(span);
    tracing::info!("ga run started");
}

// on_generation_start: create and store generation span as child of run span
fn on_generation_start(&self, generation: usize) {
    let run_guard = self.run_span.lock().unwrap();
    let _run_enter = run_guard.as_ref().map(|s| s.enter());
    let span = tracing::debug_span!("ga_generation", generation);
    *self.gen_span.lock().unwrap() = Some(span);
}

// on_generation_end: emit stats event, then drop the generation span
fn on_generation_end(&self, stats: &GenerationStats) {
    let guard = self.gen_span.lock().unwrap();
    let _enter = guard.as_ref().map(|s| s.enter());
    tracing::debug!(
        generation = stats.generation,
        best_fitness = stats.best_fitness,
        avg_fitness = stats.avg_fitness,
        diversity = stats.diversity,
        "generation_end"
    );
    drop(guard);
    *self.gen_span.lock().unwrap() = None;  // drops the span
}

// on_run_end: emit run-end event, then drop the run span
fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
    let guard = self.run_span.lock().unwrap();
    let _enter = guard.as_ref().map(|s| s.enter());
    let total_generations = all_stats.len();
    tracing::info!(cause = ?cause, total_generations, "ga run ended");
    drop(guard);
    *self.run_span.lock().unwrap() = None;
}
```

### Pattern 4: Span Child Relationship

The `ga_generation` child span is created inside the `ga_run` span's context. This requires entering `run_span` when calling `tracing::debug_span!("ga_generation", ...)` so the subscriber registers it as a child. This happens naturally in `on_generation_start` if `run_span` is entered before the `debug_span!` call.

### Anti-Patterns to Avoid

- **Storing `EnteredSpan` in a struct field:** `EnteredSpan: !Send` breaks `Send + Sync` requirement. The observer will not compile if stored as `Arc<dyn GaObserver + Send + Sync>`.
- **Using `log::*` in TracingObserver:** When `LogTracer` is active, any `log::*` call inside a tracing hook gets converted back to a tracing event, creating infinite recursion. Use only `tracing::event!()` variants.
- **Naming the module `tracing.rs`:** Conflicts with the `tracing` crate name in `use` statements within the file. Use `tracing_observer.rs` instead.
- **Calling `span.entered()` instead of `span.enter()`:** `entered()` consumes the span and returns `EnteredSpan: !Send`. Use `enter()` which borrows and returns `Entered<'_>`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Span hierarchy and context propagation | Custom parent-tracking via IDs | `tracing` crate span nesting (entering `run_span` when creating `gen_span`) | Subscriber handles parent-child recording; manual ID passing breaks with non-OTel backends |
| Event filtering by verbosity | Custom level fields | `tracing` built-in level system + subscriber `EnvFilter` | Users can filter with `RUST_LOG=genetic_algorithms=trace` without touching library code |
| Log→tracing bridge | Custom adapter | `tracing-log` `LogTracer` | Standard crate; handles all `log` crate features and targets |

**Key insight:** The `tracing` subscriber ecosystem handles all routing, filtering, and export. The library's only job is to emit events at correct levels with correct fields. Never implement filtering, routing, or backend logic inside `TracingObserver`.

---

## Common Pitfalls

### Pitfall 1: EnteredSpan in Struct Fields

**What goes wrong:** `TracingObserver` fails to compile with "the trait `Send` is not implemented for `EnteredSpan`" when the compiler tries to satisfy `Arc<dyn GaObserver<U> + Send + Sync>`.
**Why it happens:** `EnteredSpan` is explicitly `!Send` because dropping it in a different thread from where it was entered produces incorrect subscriber state.
**How to avoid:** Store `Mutex<Option<Span>>`. Call `span.enter()` locally within each hook to get a temporary `Entered<'_>` guard.
**Warning signs:** Compiler error mentioning `EnteredSpan` and `Send`.

### Pitfall 2: LogTracer Infinite Recursion

**What goes wrong:** Stack overflow with nested `log` → `tracing` → `log` calls.
**Why it happens:** `LogTracer::init()` redirects all `log::*` calls to the tracing subscriber. If `TracingObserver` calls `log::*` inside a hook, it creates `log → tracing → observer hook → log → ...`.
**How to avoid:** Use only `tracing::info!`, `tracing::debug!`, `tracing::trace!`, `tracing::warn!`, `tracing::event!` inside `TracingObserver`. Zero `log::` references.
**Warning signs:** Stack overflow in TRAC-03 integration test; test success_criteria 3.

### Pitfall 3: Module Name Collision with `tracing` Crate

**What goes wrong:** `mod tracing;` in `observer/mod.rs` creates a local module named `tracing`, shadowing the `tracing` crate in that scope, causing "use of undeclared crate or module `tracing`" errors inside `tracing.rs`.
**Why it happens:** Rust resolves `tracing::` relative to the current module path first.
**How to avoid:** Name the file `src/observer/tracing_observer.rs` and declare it as `mod tracing_observer;` in `observer/mod.rs`.
**Warning signs:** Confusing compiler errors about `tracing::span` or `tracing::info_span` not being found in a module that imports `tracing`.

### Pitfall 4: GenerationStats Field Name Mismatch

**What goes wrong:** Tracing event field `mean_fitness` emits but the value is always 0.0 or wrong because the struct field is actually `avg_fitness`.
**Why it happens:** CONTEXT.md mentions `mean_fitness` but `src/stats.rs` defines `avg_fitness: f64`.
**How to avoid:** Use `avg_fitness = stats.avg_fitness` in the `on_generation_end` event. The field name in the tracing event can be `avg_fitness` (matches struct) or aliased as `mean_fitness = stats.avg_fitness` — use actual struct field to avoid runtime errors.
**Warning signs:** Tracing event field with a hard-coded 0.0 default; compiler does not catch this since field values are evaluated at runtime.

### Pitfall 5: Generation Span Not Registered as Child of Run Span

**What goes wrong:** Tracing backends (Jaeger, Tempo) show `ga_generation` spans as root spans unconnected to `ga_run`, breaking the hierarchy.
**Why it happens:** A span's parent is determined at creation time from the current span context. If `run_span` is not entered when `debug_span!("ga_generation", ...)` is called, the generation span has no parent.
**How to avoid:** In `on_generation_start`, enter `run_span` before creating `gen_span`. The subscriber will record the parent-child relationship automatically.

---

## Code Examples

Verified patterns from official sources:

### Span Creation and Entry (tracing 0.1.44)
```rust
// Source: https://docs.rs/tracing/0.1.44/tracing/struct.Span.html
use tracing::Span;

// Create a named span at INFO level
let span = tracing::info_span!("ga_run");

// Enter it locally (guard exits when dropped)
let _enter = span.enter();

// Create a DEBUG-level child (while run span is entered, this becomes its child)
let gen_span = tracing::debug_span!("ga_generation", generation = 42usize);
```

### Event Emission with Structured Fields
```rust
// Source: https://docs.rs/tracing/0.1.44/tracing/index.html
// Key-value fields before the message string
tracing::info!(cause = ?cause, total_generations = 10usize, "ga run ended");
tracing::trace!(generation = 1usize, duration_ms = 12.4f64, population_size = 100usize, "selection_complete");
tracing::warn!(generation = 5usize, stagnation_count = 3usize, "stagnation detected");
```

### Send+Sync-Safe Observer Struct
```rust
// Pattern verified: Span: Send + Sync (tracing 0.1.44 docs)
// Mutex<Option<Span>>: Send + Sync because Span: Send + Sync
use std::sync::Mutex;
use tracing::Span;

pub struct TracingObserver {
    run_span: Mutex<Option<Span>>,
    gen_span: Mutex<Option<Span>>,
}

impl TracingObserver {
    pub fn new() -> Self {
        Self {
            run_span: Mutex::new(None),
            gen_span: Mutex::new(None),
        }
    }
}

impl Default for TracingObserver {
    fn default() -> Self { Self::new() }
}
```

### Re-entering a Stored Span in a Hook
```rust
// Enter the generation span for the duration of this hook call
fn on_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
    let guard = self.gen_span.lock().unwrap();
    let _enter = guard.as_ref().map(|s| s.enter());
    let duration_ms = duration.as_secs_f64() * 1000.0;
    tracing::trace!(generation, duration_ms, population_size, "selection_complete");
}
```

### TRAC-03 Integration Test Pattern
```rust
// dev-dependency: tracing-subscriber + tracing-log
// Tests that LogTracer + TracingObserver coexist without stack overflow
#[test]
fn test_tracing_observer_with_logtracer_no_recursion() {
    use tracing_subscriber::fmt;
    use tracing_log::LogTracer;

    // Init LogTracer: redirects all log::* to tracing
    let _ = LogTracer::init();  // ignore error (may already be init in test suite)

    // Set up a simple subscriber
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Run 10 generations — no stack overflow means TRAC-03 passes
    use genetic_algorithms::observer::TracingObserver;
    use std::sync::Arc;
    let obs = Arc::new(TracingObserver::new());
    let mut ga = build_test_ga(10, obs);
    ga.run().expect("GA with TracingObserver + LogTracer must not stack overflow");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `span.entered()` returning `EnteredSpan` stored in fields | `Mutex<Option<Span>>` + re-enter per hook via `span.enter()` | tracing design invariant (always) | Avoids `!Send` compile failure |
| Nested `log!()` calls inside observers | Exclusively `tracing::event!()` variants | Phase 15 constraint | Prevents `LogTracer` infinite recursion |

**Deprecated/outdated:**
- `tracing::Span::new()` with raw metadata: Internal API; use `info_span!`/`debug_span!` macros for stable public API.

---

## Open Questions

1. **`mean_fitness` vs `avg_fitness` field name in tracing events**
   - What we know: CONTEXT.md says to use `mean_fitness` as the event field name; `stats.rs` struct field is `avg_fitness`
   - What's unclear: Whether the event field name should match the struct field (`avg_fitness`) or the human-readable name (`mean_fitness`)
   - Recommendation: Use `avg_fitness = stats.avg_fitness` (matches struct field, avoids potential future confusion). A named field `mean_fitness = stats.avg_fitness` is also acceptable as a tracing display name — planner decides.

2. **Guard lifetime when re-entering stored Span**
   - What we know: `guard.as_ref().map(|s| s.enter())` returns `Option<Entered<'_>>` where the lifetime is tied to `guard`. Since `guard` is a `MutexGuard<'_, Option<Span>>` and the `Entered<'_>` borrows from it, the guard and the entered span must live in the same scope.
   - What's unclear: Whether binding `let _enter = guard.as_ref().map(|s| s.enter());` correctly chains lifetimes in all compiler versions.
   - Recommendation: Pattern is correct and idiomatic — `_enter` binds the guard keeping the span entered until end of scope. No issue expected. Verified by Rust lifetime rules.

3. **`on_generation_end` span drop ordering**
   - What we know: `on_generation_end` should emit the stats event inside the span, then drop the span. Rust drops in reverse declaration order.
   - What's unclear: Whether emitting after `drop(guard)` would break nesting.
   - Recommendation: Emit the event while the span guard is still held (before setting `gen_span` to `None`). The drop of the span (setting to `None`) should come last.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None — standard Rust test runner |
| Quick run command | `cargo test --features observer-tracing test_tracing` |
| Full suite command | `cargo test --features observer-tracing && cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRAC-01 | `TracingObserver` attaches to `Ga<U>` and emits events when GA runs | integration | `cargo test --features observer-tracing test_tracing_observer_attaches_and_runs` | ❌ Wave 0 |
| TRAC-01 | `TracingObserver` is `Send + Sync` | unit (compile check) | `cargo test --features observer-tracing test_tracing_observer_is_send_sync` | ❌ Wave 0 |
| TRAC-01 | `TracingObserver` re-exported from crate root | unit (compile check) | `cargo test --features observer-tracing test_tracing_observer_crate_reexport` | ❌ Wave 0 |
| TRAC-02 | Default build compiles without pulling in `tracing` | build check | `cargo build` (no features) — must succeed and not reference tracing | ❌ Wave 0 |
| TRAC-02 | `tracing` absent from default dependency tree | unit | `cargo test test_tracing_feature_gated` (source-level cfg guard) | ❌ Wave 0 |
| TRAC-03 | `LogTracer` + `TracingObserver` completes 10 generations without stack overflow | integration | `cargo test --features observer-tracing test_tracing_observer_with_logtracer_no_recursion` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --features observer-tracing`
- **Per wave merge:** `cargo test --features observer-tracing && cargo test && cargo clippy --features observer-tracing`
- **Phase gate:** Full suite green (both with and without `observer-tracing` feature) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_tracing_observer.rs` — covers TRAC-01, TRAC-02, TRAC-03
- [ ] Add to `[dev-dependencies]` in `Cargo.toml`: `tracing-subscriber = "0.3"`, `tracing-log = "0.2"`

---

## Sources

### Primary (HIGH confidence)

- `tracing` 0.1.44 — https://docs.rs/tracing/0.1.44/tracing/span/struct.Span.html — `Span: Send + Sync` confirmed; `enter()` vs `entered()` semantics verified
- `tracing` 0.1.44 — https://docs.rs/tracing/0.1.44/tracing/span/struct.EnteredSpan.html — `EnteredSpan: !Send` confirmed explicitly
- `tracing` 0.1.44 — https://docs.rs/tracing/0.1.44/tracing/index.html — macro API (`info_span!`, `debug_span!`, `trace!`, `info!`, `warn!`) verified
- `tracing-log` 0.2.0 — https://docs.rs/tracing-log/0.2.0/tracing_log/struct.LogTracer.html — `LogTracer::init()` and bridge mechanism verified
- `src/observer/log.rs` — structural template for `TracingObserver` implementation pattern
- `src/observer/mod.rs` — `GaObserver<U>` trait: all 12 hook signatures confirmed
- `src/stats.rs` — `GenerationStats` struct: field names confirmed (`avg_fitness`, not `mean_fitness`)
- `Cargo.toml` — existing optional dependency and feature flag patterns (`serde`, `visualization`)

### Secondary (MEDIUM confidence)

- `tracing-subscriber` 0.3.23 — https://crates.io/crates/tracing-subscriber — version verified; used for dev-dependency in TRAC-03 test
- `tracing-log` 0.2.0 — https://crates.io/crates/tracing-log — version verified for dev-dependency

### Tertiary (LOW confidence)

- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified directly from crates.io API
- Architecture: HIGH — `Span: Send + Sync` and `EnteredSpan: !Send` verified from official docs; feature flag pattern verified from existing Cargo.toml
- Pitfalls: HIGH — `EnteredSpan !Send` issue is compiler-enforced and documented; LogTracer recursion is mechanically guaranteed by "no log:: calls" rule
- Field name discrepancy: HIGH — directly read from `src/stats.rs`

**Research date:** 2026-03-26
**Valid until:** 2026-06-26 (tracing 0.1.x is stable; API has been stable for years)
