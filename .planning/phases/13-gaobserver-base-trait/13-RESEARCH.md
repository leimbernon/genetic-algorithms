# Phase 13: GaObserver Base Trait - Research

**Researched:** 2026-03-25
**Domain:** Rust trait design for lifecycle observability in a concurrent genetic algorithm library
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Hook surface — 11 hooks:**

Lifecycle:
- `on_run_start()` — no payload
- `on_generation_start(generation: usize)` — fires before operators run
- `on_generation_end(stats: &GenerationStats)` — fires after stats collected
- `on_run_end(cause: TerminationCause, all_stats: &[GenerationStats])` — mirrors Reporter's on_finish

Special events:
- `on_new_best(generation: usize, best: U)` — owned clone of chromosome
- `on_stagnation(generation: usize, stagnation_count: usize)` — fires when stagnation counter increments
- `on_extension_triggered(event: ExtensionEvent)` — typed struct payload

Operator hooks:
- `on_selection_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_crossover_complete(generation: usize, duration: Duration, offspring_count: usize)`
- `on_mutation_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_survivor_selection_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_fitness_evaluation_complete(generation: usize, duration: Duration, population_size: usize)`

All hooks have default no-op bodies.
All hooks are zero-cost when no observer is attached.

**ExtensionEvent struct:**
```rust
pub struct ExtensionEvent {
    pub generation: usize,
    pub diversity: f64,
    pub extension_type: &'static str,  // e.g. "MassExtinction", "MassGenesis"
}
```

**Operator timing measurement scope:** `Instant::now()` immediately before operator call, `.elapsed()` immediately after. Fitness re-evaluation after crossover measured separately by `on_fitness_evaluation_complete`.

**Storage and thread safety:**
- `observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>` field on `Ga<U>`
- Builder: `pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self`
- All hooks are `&self` (not `&mut self`)
- `GaObserver<U>: Send + Sync` supertraits are mandatory from day one

**Reporter deprecation (soft, v2.2.0):**
- `Reporter` trait gets `#[deprecated(since = "2.2.0", note = "use GaObserver<U> instead...")]`
- `with_reporter()` builder method gets the same `#[deprecated]` attribute
- Both `reporter` and `observer` fields coexist on `Ga<U>` in v2.2.0; zero breakage

### Claude's Discretion

- Module structure: `src/observer/mod.rs` mirroring `src/reporter/` layout
- Exact naming of the `notify_*` helper (inline or method on `Ga<U>`)
- Whether `NoopObserver` is a public struct or just documented as "implement with all defaults"
- `Instant` measurement: skip it when `observer.is_none()` (should be)
- Re-export from `src/lib.rs` prelude

### Deferred Ideas (OUT OF SCOPE)

- Per-operator timing hooks with individual chromosome counts (e.g., per-offspring fitness time) — EXT-01, deferred to v2.3+
- `on_checkpoint_saved` hook — EXT-02, low priority, deferred to v2.3+
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| OBS-01 | User can attach a `GaObserver<U>` to `Ga<U>` via `with_observer()` and receive notifications for run start/end, each generation, new best, and special events (stagnation, extension triggered) | `src/reporter/mod.rs` pattern is the direct template; `Ga<U>` struct has a well-understood field location (line 129) and builder pattern (line 523); all 11 notification points are identified in the run loop |
| OBS-02 | `GaObserver<U>` has default no-op implementations for all hooks — users implement only the hooks they care about | Default method bodies `{}` compile away completely; same pattern as existing `Reporter<U>` (verified in `src/reporter/mod.rs`) |
| OBS-03 | No overhead when no observer is attached (`Option::None` branch eliminates all vtable dispatch and measurement) | `if let Some(ref obs) = self.observer` pattern eliminates dispatch; `observer.is_none()` pre-check before `Instant::now()` eliminates measurement cost; criterion bench `benches/ga_run.rs` exists to verify |
| OBS-04 | `GaObserver<U>` is safely shareable across rayon threads (`Arc<dyn GaObserver<U> + Send + Sync>`) | `Send + Sync` as supertraits enforces this at `with_observer()` call site; `Arc` is `Clone + Sync` by construction; contrast with `Reporter<U>` which uses `Box` + `&mut self` |
</phase_requirements>

---

## Summary

Phase 13 adds the `GaObserver<U>` trait and integrates it into `Ga<U>`. The trait is structurally identical to the existing `Reporter<U>` (same file layout, same default method pattern) but with three key differences: it uses `Arc` instead of `Box` for thread-safe sharing, all hooks take `&self` instead of `&mut self` to permit rayon-compatible shared access, and it carries 11 hooks versus Reporter's 4.

The implementation is entirely additive. The existing `reporter` field on `Ga<U>` stays untouched; a new `observer` field is added alongside it. `Reporter<U>` receives a soft deprecation attribute but continues to compile. The notification points in `ga.rs` are well-understood: 4 existing Reporter call sites are at lines 711, 878, 1048, 1095 — the GaObserver adds 7 more call sites around the 5 operator phases.

The single most critical design constraint is that `GaObserver<U>: Send + Sync` must be locked in as supertraits in this phase. Adding them after the fact is a breaking change that cannot be patched.

**Primary recommendation:** Create `src/observer/mod.rs` mirroring `src/reporter/mod.rs`, add the `observer` field to `Ga<U>`, add 11 notification call sites with `Instant`-gated measurements, add `#[deprecated]` to `Reporter`/`with_reporter()`, and export from `src/lib.rs`.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::sync::Arc` | stdlib | Observer storage and thread-safe sharing | Required for rayon compatibility; avoids Box's exclusive-ownership constraint |
| `std::time::{Duration, Instant}` | stdlib | Per-operator timing measurements | Zero-dependency; `Instant::now()` + `.elapsed()` is the canonical Rust timing idiom |
| `std::sync::{Mutex, AtomicU64}` | stdlib | Interior mutability in custom observer impls | Enables `&self` hook methods while allowing state updates in user observers |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rayon` | existing dep | Already used in `parent_crossover`; no new dep needed | Not added in this phase; only relevant for confirming `Arc + Send + Sync` is sufficient |
| `criterion` | existing dev dep | Benchmark verification of zero-overhead claim | Run `benches/ga_run.rs` with/without observer to verify OBS-03 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Arc<dyn GaObserver<U> + Send + Sync>` | `Box<dyn GaObserver<U> + Send>` | Box is cheaper (no refcount) but cannot be shared across rayon threads — prohibited by island GA requirements (Phase 16) |
| `&self` hooks | `&mut self` hooks | `&mut self` matches Reporter but is incompatible with `Arc`-sharing without a `Mutex` per call — defeats zero-overhead goal |
| `Option<Arc<dyn ...>>` | Hardcoded `NoopObserver` as default type | Type erasure with `Option::None` is cleaner; no generic parameter pollution on `Ga<U>` |

**Installation:** No new dependencies. Uses only `std`.

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── observer/
│   └── mod.rs           # GaObserver<U> trait, ExtensionEvent, NoopObserver, re-exports
├── ga.rs                # Modified: +observer field, +with_observer(), +notify() helper,
│                        #   +Instant measurements, +11 notification call sites,
│                        #   +#[deprecated] on Reporter/with_reporter
└── lib.rs               # Modified: pub mod observer; re-export GaObserver, ExtensionEvent
```

`src/observer/mod.rs` is a single file for this phase. Sub-files (`island.rs`, `nsga2.rs`, `log_observer.rs`) are created in later phases.

### Pattern 1: `GaObserver<U>` Trait with Default No-Op Bodies

**What:** A generic trait over `ChromosomeT` with all 11 hook methods defined with empty default bodies. Supertraits `Send + Sync` are declared on the trait itself.

**When to use:** Always — this is the entire trait definition.

**Example:**
```rust
// src/observer/mod.rs
use std::time::Duration;
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    fn on_run_start(&self) {}
    fn on_generation_start(&self, _generation: usize) {}
    fn on_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_crossover_complete(&self, _generation: usize, _duration: Duration, _offspring_count: usize) {}
    fn on_mutation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_fitness_evaluation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_survivor_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_new_best(&self, _generation: usize, _best: U) {}
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {}
    fn on_extension_triggered(&self, _event: ExtensionEvent) {}
    fn on_generation_end(&self, _stats: &GenerationStats) {}
    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}
```

Note: This is 12 methods (the 11 in CONTEXT.md plus `on_generation_start`). `on_generation_start` is a locked decision.

### Pattern 2: `Option<Arc<dyn ...>>` Field with `notify()` Helper

**What:** Store the observer as an `Option<Arc<dyn GaObserver<U> + Send + Sync>>`. Use a private `notify()` method on `Ga<U>` to dispatch calls without repeating the `if let Some` check at every call site.

**When to use:** At every notification point in the run loop — avoids 11 copies of the same Option-check idiom.

**Example:**
```rust
// In Ga<U> struct
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// Private helper on Ga<U>
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}

// Usage in run loop
self.notify(|obs| obs.on_run_start());
```

### Pattern 3: `Instant`-Gated Timing

**What:** Check `observer.is_none()` before calling `Instant::now()`. When no observer is attached, skip the timing call entirely — not just the notification.

**When to use:** At each of the 5 operator measurement points.

**Example:**
```rust
// In ga.rs run loop, before selection:
let t_selection = if self.observer.is_some() { Some(Instant::now()) } else { None };
let parents = selection::factory(...)?;
if let Some(t) = t_selection {
    self.notify(|obs| obs.on_selection_complete(i, t.elapsed(), parents.len()));
}
```

Alternative (more concise): Always call `Instant::now()` and gate only the notify call. The Instant overhead is ~20ns and acceptable if preferred for code clarity. The locked decision says skip it — use the `is_none()` guard.

### Pattern 4: `ExtensionEvent` as Stack-Allocated Struct

**What:** A `Copy`-able struct with `&'static str` for the type name — zero heap allocation.

**When to use:** When firing `on_extension_triggered` in the extension block of the run loop.

**Example:**
```rust
// src/observer/mod.rs
#[derive(Debug, Clone, Copy)]
pub struct ExtensionEvent {
    pub generation: usize,
    pub diversity: f64,
    pub extension_type: &'static str,
}

// In ga.rs, inside the extension block:
self.notify(|obs| obs.on_extension_triggered(ExtensionEvent {
    generation: i,
    diversity: gen_stats.diversity,
    extension_type: "MassExtinction",  // derived from ext_config.method
}));
```

The `extension_type` string must be derived from the `Extension` enum variant at the call site. Use `match ext_config.method { Extension::MassExtinction => "MassExtinction", ... }` or implement `fn as_static_str(&self) -> &'static str` on `Extension`.

### Pattern 5: `#[deprecated]` on `Reporter<U>` and `with_reporter()`

**What:** Add `#[deprecated(since = "2.2.0", note = "...")]` to the `Reporter<U>` trait definition and the `with_reporter()` builder method. Do not change any behavior.

**When to use:** In the same PR/commit as the GaObserver addition.

**Example:**
```rust
// src/reporter/mod.rs
#[deprecated(since = "2.2.0", note = "use GaObserver<U> instead. Reporter will be removed in v3.0.0.")]
pub trait Reporter<U: ChromosomeT>: Send { ... }

// src/ga.rs
#[deprecated(since = "2.2.0", note = "use with_observer() instead. Reporter will be removed in v3.0.0.")]
pub fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self { ... }
```

`#[allow(deprecated)]` must be added to every existing test that uses `with_reporter()` or implements `Reporter<U>` to avoid CI failures. There are currently 8 tests in `tests/test_reporter.rs` and 1 usage in reporter tests in `src/reporter/mod.rs`.

### Anti-Patterns to Avoid

- **`Box` instead of `Arc` for observer storage:** `Box<dyn GaObserver<U> + Send>` is not thread-safe for `par_iter_mut()` in island GA. Always use `Arc`.
- **`&mut self` on hook methods:** Incompatible with `Arc` sharing. All hooks must be `&self`.
- **`Arc::clone` inside a parallel closure:** Clone the `Arc` once before entering the parallel region; pass a shared borrow (`observer.as_deref()`) into the closure.
- **Removing `Reporter<U>`:** It is a public trait in a published crate. Add `#[deprecated]`, do not remove.
- **Calling `Instant::now()` unconditionally:** When no observer is attached, timing calls are pure overhead. Gate them behind `observer.is_some()`.
- **`on_stagnation` firing only at termination:** The hook fires every time `stagnation_count` increments (line 1052 in ga.rs), not only when `StagnationReached` is the termination cause.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thread-safe shared reference | Custom smart pointer | `Arc<dyn GaObserver<U> + Send + Sync>` | Arc is stdlib, correct refcount semantics, clone-cheap |
| Wall-clock timing | Manual `SystemTime` comparisons | `std::time::Instant` | Monotonic, nanosecond precision, no leap-second skew |
| Object-safe trait dispatch | Enum dispatch for observers | `dyn GaObserver<U>` trait objects | Users must be able to supply custom types; enum dispatch prevents that |
| Interior mutability in observers | `unsafe` raw pointer mutation | `std::sync::Mutex<T>` or `AtomicU64` | Safe, well-understood, idiomatic; `Arc<Mutex<T>>` is the established Rust pattern |

**Key insight:** Every mechanism needed for this phase is already in `std`. The design challenge is using the right combination of existing primitives, not building new ones.

---

## Common Pitfalls

### Pitfall 1: Missing `Send + Sync` Supertraits

**What goes wrong:** `GaObserver<U>` is defined without `Send + Sync` supertraits. Everything compiles for `Ga<U>` (single-threaded outer loop). When Phase 16 adds `IslandGa` support, the `par_iter_mut()` closure cannot hold the observer reference — compile error after the trait is already public.

**Why it happens:** `Ga<U>` does not use rayon in its outer loop; the observer feels single-threaded. The rayon usage is inside `parent_crossover` (not the outer for-loop) and in the island model which is Phase 16.

**How to avoid:** Declare `pub trait GaObserver<U: ChromosomeT>: Send + Sync` from day one. This phase is the only chance to add these supertraits without a semver break.

**Warning signs:** Any existing custom `impl GaObserver<U>` for a type that is not `Send` or not `Sync` would fail to compile — this is correct and intentional.

### Pitfall 2: `Arc::clone` Inside the Parallel Crossover Region

**What goes wrong:** If `Arc::clone(&self.observer)` is placed inside the `par_iter()` closure in `parent_crossover`, every parent pair clones the same Arc. With 1000+ parents on 8 threads, atomic refcount cache-line bouncing measurably degrades throughput.

**Why it happens:** The existing `fitness_fn` pattern (`Arc::clone(&fitness_fn)`) in island/mod.rs is done in a sequential loop and looks identical. Developers copy it into `par_iter()`.

**How to avoid:** Do not place observer calls inside `parent_crossover`'s parallel region at all. The operator hooks (`on_selection_complete`, `on_crossover_complete`, etc.) fire in the sequential outer loop — after the parallel work returns.

**Warning signs:** Any `Arc::clone` or observer method call inside a `par_iter` or `par_iter_mut` closure.

### Pitfall 3: `on_stagnation` Firing at Wrong Point

**What goes wrong:** The hook fires only when `TerminationCause::StagnationReached` is set (the break condition), missing all intermediate stagnation increments.

**Why it happens:** The `stagnation_count >= max_stagnation` termination check is the most visible use of `stagnation_count`. The increment happens 4 lines earlier in the `else` branch (line 1052).

**How to avoid:** Place `on_stagnation` call in the `else` branch at line 1052 (where `stagnation_count += 1`), not in the termination-break block.

### Pitfall 4: `#[deprecated]` on `Reporter` Breaks Existing Tests Without `#[allow(deprecated)]`

**What goes wrong:** Adding `#[deprecated]` to `Reporter<U>` causes `cargo test` to emit warnings that may become errors under `#[deny(warnings)]`. The 8 existing tests in `tests/test_reporter.rs` and the 4 tests inside `src/reporter/mod.rs` all use `Reporter` directly.

**Why it happens:** Rust's `#[deprecated]` attribute generates a warning at every use site — including `impl Reporter<U>` blocks in tests.

**How to avoid:** When adding the `#[deprecated]` attribute, also add `#[allow(deprecated)]` at the top of `tests/test_reporter.rs` and wrap the existing `src/reporter/mod.rs` test module with `#[allow(deprecated)]`.

### Pitfall 5: `on_generation_end` vs `on_generation_start` Ordering

**What goes wrong:** Both hooks are added but the call order relative to stats collection is wrong. `on_generation_start` must fire before operators run; `on_generation_end` must fire after `GenerationStats` is computed and pushed.

**Why it happens:** The run loop is long (~400 lines in ga.rs). It is easy to misplace a notification call.

**How to avoid:** `on_generation_start` fires immediately after `info!(target="ga_events", ...)` at line 717 (before any operator call). `on_generation_end` fires immediately after `self.stats.push(gen_stats.clone())` at line 876 — before the existing `reporter.on_generation_complete()` call at line 878.

---

## Code Examples

### `GaObserver<U>` Trait Definition

```rust
// src/observer/mod.rs
// Source: src/reporter/mod.rs (direct structural template) + CONTEXT.md decisions

use std::time::Duration;
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

/// Structured lifecycle observer for `Ga<U>`.
///
/// All methods have default no-op bodies; implement only the hooks you need.
/// The `Send + Sync` supertraits are required for use with the island model
/// and any multi-threaded context.
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    fn on_run_start(&self) {}
    fn on_generation_start(&self, _generation: usize) {}
    fn on_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_crossover_complete(&self, _generation: usize, _duration: Duration, _offspring_count: usize) {}
    fn on_mutation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_fitness_evaluation_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_survivor_selection_complete(&self, _generation: usize, _duration: Duration, _population_size: usize) {}
    fn on_new_best(&self, _generation: usize, _best: U) {}
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {}
    fn on_extension_triggered(&self, _event: ExtensionEvent) {}
    fn on_generation_end(&self, _stats: &GenerationStats) {}
    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}

/// Zero-sized no-op observer. Useful as a compile-check type.
pub struct NoopObserver;

impl<U: ChromosomeT> GaObserver<U> for NoopObserver {}
```

### Adding `observer` Field and `with_observer()` to `Ga<U>`

```rust
// src/ga.rs — field addition (alongside existing reporter field at line 129)
// Source: existing reporter field pattern

observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// In Default impl:
observer: None,

// Builder method (alongside with_reporter() at line 520-526):
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}

// Private notify helper on Ga<U>:
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Notification Call Sites in the Run Loop

```rust
// Source: existing reporter call sites in src/ga.rs, pattern extended to 11 hooks

// Before the for-loop (after reporter.on_start()):
self.notify(|obs| obs.on_run_start());

// Inside for-loop, after line 717 info! macro:
self.notify(|obs| obs.on_generation_start(i));

// Before selection (line 721):
let t_selection = if self.observer.is_some() { Some(Instant::now()) } else { None };
let parents = selection::factory(...)?;
if let (Some(t), ref obs) = (t_selection, &self.observer) {
    let _ = obs; // suppress unused; actual call:
    self.notify(|obs| obs.on_selection_complete(i, t.elapsed(), parents.len()));
}

// After parent_crossover() (line 742):
// on_crossover_complete, on_mutation_complete, on_fitness_evaluation_complete
// (all share the same elapsed() from t_crossover — see timing scope decision)

// After survivor::factory() (line 765):
// on_survivor_selection_complete

// In the improved==true branch (line 1048):
// on_new_best

// In the stagnation_count += 1 branch (line 1052):
self.notify(|obs| obs.on_stagnation(i, stagnation_count));

// After gen_stats push and reporter.on_generation_complete (line 878-880):
self.notify(|obs| obs.on_generation_end(&gen_stats));

// After extension block (line ~987):
// on_extension_triggered (only when extension fires)

// After the for-loop (after reporter.on_finish, line 1095-1097):
self.notify(|obs| obs.on_run_end(self.termination_cause, &self.stats));
```

### `#[deprecated]` on `Reporter<U>`

```rust
// src/reporter/mod.rs
#[deprecated(
    since = "2.2.0",
    note = "use GaObserver<U> instead. Reporter will be removed in v3.0.0."
)]
pub trait Reporter<U: ChromosomeT>: Send { ... }

// src/ga.rs
#[deprecated(
    since = "2.2.0",
    note = "use with_observer() instead. Reporter will be removed in v3.0.0."
)]
pub fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self { ... }
```

### Compile-Time Object-Safety Verification

```rust
// In tests/test_observer.rs (new file)
use std::sync::Arc;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;

// Compile test: GaObserver is object-safe
#[test]
fn gaobserver_is_object_safe() {
    let obs: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> =
        Arc::new(genetic_algorithms::observer::NoopObserver);
    drop(obs);
}

// Compile test: partial implementation compiles
#[test]
fn partial_implementation_compiles() {
    struct CountingObserver(std::sync::atomic::AtomicU64);
    impl GaObserver<BinaryChromosome> for CountingObserver {
        fn on_generation_end(&self, _stats: &genetic_algorithms::stats::GenerationStats) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    // remaining 11 hooks use defaults — must compile
    let obs: Arc<dyn GaObserver<BinaryChromosome>> = Arc::new(CountingObserver(0.into()));
    drop(obs);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Reporter<U>` with `Box` + `&mut self` (4 hooks) | `GaObserver<U>` with `Arc` + `&self` (11+ hooks) | v2.2.0 (this phase) | Thread-safe, granular, forward-compatible for island model |
| Hardcoded `log!()` calls in ga.rs | Observer hooks (Phase 14) | v2.2.0 Phase 14 | Not in scope for Phase 13; Reporter call sites stay as-is |

**Deprecated in this phase:**
- `Reporter<U>` trait: gets `#[deprecated(since = "2.2.0")]`; continues to compile, unchanged semantics
- `with_reporter()` method: same deprecation attribute

---

## Open Questions

1. **`on_crossover_complete` vs `on_mutation_complete` timing scope**
   - What we know: `parent_crossover()` performs crossover, mutation, and fitness evaluation in a single function call (lines 734-742 in ga.rs). Separating per-operator timings requires wrapping individual operator calls inside `parent_crossover`.
   - What's unclear: The CONTEXT.md says "each operator's Duration is measured immediately before/after the operator call" and "fitness re-evaluation measured separately." However, `parent_crossover` is an opaque function from ga.rs's perspective — it does not currently return per-operator durations.
   - Recommendation: For Phase 13, measure `parent_crossover` as a single timed block and report its total time in `on_crossover_complete`. Fire `on_mutation_complete` and `on_fitness_evaluation_complete` with `Duration::ZERO` or skip them until Phase 13's plan clarifies whether to instrument `parent_crossover` internally. Confirm with the planner.

2. **`extension_type` string derivation from `Extension` enum**
   - What we know: `Extension` enum variants are in `src/extension/` or `src/operations/`. The `&'static str` decision avoids allocation.
   - What's unclear: Whether `Extension` enum already has a method returning `&'static str`, or whether a `match` must be added at the call site in ga.rs.
   - Recommendation: Add `impl Extension { pub fn as_str(&self) -> &'static str { ... } }` or use a match expression at the call site. Either works; a method on `Extension` is cleaner.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — uses Cargo.toml test configuration |
| Quick run command | `cargo test test_observer` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OBS-01 | `on_run_start` fires once, `on_generation_end` fires N times, `on_new_best` fires on improvement, `on_run_end` fires once with correct cause | integration | `cargo test test_observer_hook_fire_counts` | ❌ Wave 0 |
| OBS-01 | `on_stagnation` fires when stagnation counter increments | integration | `cargo test test_observer_stagnation_fires` | ❌ Wave 0 |
| OBS-01 | `on_extension_triggered` fires when extension condition is met | integration | `cargo test test_observer_extension_triggered` | ❌ Wave 0 |
| OBS-02 | A type implementing only one hook compiles without error | compile test | `cargo test test_observer_partial_impl_compiles` | ❌ Wave 0 |
| OBS-03 | Running without observer produces statistically indistinguishable timing vs baseline | bench | `cargo bench ga_run` (compare with/without observer field) | ✅ (benches/ga_run.rs exists) |
| OBS-04 | `Arc<dyn GaObserver<U> + Send + Sync>` assignment compiles; non-`Send` type is rejected | compile test | `cargo test test_observer_is_object_safe` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test test_observer`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_observer.rs` — covers OBS-01 (hook fire counts, stagnation, extension), OBS-02 (partial impl compile), OBS-04 (object safety + Send+Sync enforcement)
- [ ] `#[allow(deprecated)]` added to `tests/test_reporter.rs` and `src/reporter/mod.rs` test module — required once `#[deprecated]` is added to `Reporter<U>`

*(The existing `benches/ga_run.rs` covers OBS-03 without modification.)*

---

## Sources

### Primary (HIGH confidence)

- `src/reporter/mod.rs` — direct structural template for `GaObserver<U>` (trait definition, module layout, test patterns)
- `src/ga.rs` lines 127-130, 520-526, 711, 878, 1048, 1052, 1095 — existing field, builder, and all reporter call sites; stagnation_count increment location
- `src/stats.rs` — `GenerationStats` struct fields available to `on_generation_end` hook
- `src/ga.rs` lines 79-87 — `TerminationCause` enum; available to `on_run_end` hook
- `.planning/research/ARCHITECTURE.md` — integration point inventory, notification flow diagram, anti-patterns
- `.planning/research/PITFALLS.md` — critical pitfalls with phase mapping
- `.planning/phases/13-gaobserver-base-trait/13-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)

- `tests/test_reporter.rs` — 8 existing integration tests; pattern for new `tests/test_observer.rs`
- `benches/ga_run.rs` — existing criterion benchmark; usable for OBS-03 zero-overhead verification without modification

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — uses only `std`; no external dependencies
- Architecture: HIGH — direct inspection of existing `reporter/` module + all ga.rs call sites; patterns are proven
- Pitfalls: HIGH — derived from codebase analysis + prior milestone research (PITFALLS.md)

**Research date:** 2026-03-25
**Valid until:** 2026-05-25 (stable Rust stdlib patterns; no external dependency churn risk)
