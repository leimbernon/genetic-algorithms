# Phase 8: Reporter Trait - Research

**Researched:** 2026-03-21
**Domain:** Rust observer/hook pattern, trait objects, `std::time::Instant`
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- `trait Reporter<U: ChromosomeT>` — generic over U because `on_new_best` receives the chromosome
- Hook signatures:
  - `fn on_start(&mut self)` — pure notification, no payload
  - `fn on_generation_complete(&mut self, stats: &GenerationStats)` — stats only
  - `fn on_new_best(&mut self, generation: usize, best: U)` — full chromosome clone
  - `fn on_finish(&mut self, cause: TerminationCause, all_stats: &[GenerationStats])` — full stats history
- All hook methods have default no-op bodies in the trait
- Dispatch: `Box<dyn Reporter<U> + Send>` — trait object (not generic param on `Ga`)
- `Send` bound only; no `Sync`
- `Ga<U>` field: `reporter: Option<Box<dyn Reporter<U> + Send>>`
- Builder method: `fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self`
- Reporter fires inside both `run()` and `run_with_callback()` — consistent regardless of which run method is used
- Reporter does NOT replace or deprecate `run_with_callback()`
- `on_start` fires once before the first generation
- `on_generation_complete` fires at the end of every generation (after stats collection, before callback)
- `on_new_best` fires whenever `improved == true` (same logic as stagnation tracking)
- `on_finish` fires once after the loop exits (after termination cause is set)
- `SimpleReporter::new(n: usize)` — prints every N generations
- Output format: `[Gen {current}/{max}] Best: {best_fitness:.4} | Diversity: {diversity:.4}`
- `SimpleReporter` always prints at `on_finish` regardless of N
- `DurationReporter` tracks wall-clock time per operator phase via `Instant::now()` stored as mutable state
- `DurationReporter` prints a final table at `on_finish`
- `NoopReporter` uses trait default bodies; `Ga<U>` uses `Option::None` for zero-overhead path

### Claude's Discretion

- Module location: `src/reporter/` with `mod.rs`, `noop.rs`, `simple.rs`, `duration.rs`
- Re-export from `src/lib.rs` / prelude
- Whether `DurationReporter` measures time via `std::time::Instant` stored per-phase or uses a single timer across the loop
- Exact formatting of the `DurationReporter` table (aligned columns, percentage, etc.)
- `on_new_best` trigger: same logic as the existing `improved` boolean in `run_with_callback`

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REP-01 | User can attach a reporter to `Ga` via `.with_reporter()` that receives lifecycle hooks (`on_start`, `on_generation_complete`, `on_new_best`, `on_finish`) | Trait definition + `Ga<U>` field + builder method + four hook call sites in `run_with_callback` |
| REP-02 | Default (no reporter configured) has zero overhead via `NoopReporter` | `Option<Box<...>>` field on `Ga<U>` defaults to `None`; no virtual dispatch on the `None` path |
| REP-03 | Built-in `SimpleReporter` logs progress to stdout every N generations | `on_generation_complete` counter + `on_finish` unconditional print |
| REP-04 | Built-in `DurationReporter` reports per-phase timing breakdown | `on_finish` receives `all_stats`; timing stored as `Duration` accumulators updated by dedicated timing hooks |
</phase_requirements>

## Summary

Phase 8 adds a `Reporter<U>` trait — a structured lifecycle observer for `Ga<U>`. All design decisions are locked in CONTEXT.md. The implementation is a pure addition: one new module (`src/reporter/`), one new field on `Ga<U>`, and four new hook call sites inside the existing `run_with_callback` loop. No existing behavior changes.

The critical insight for DurationReporter is that the CONTEXT.md says timing hooks are called "inside the GA loop via the reporter." However, the reporter interface only has four hooks (`on_start`, `on_generation_complete`, `on_new_best`, `on_finish`) — none of which fire between phases within a generation. DurationReporter therefore cannot measure per-operator timing via reporter hooks alone. It must either (a) record `Instant::now()` at `on_generation_complete` and aggregate coarse timing, or (b) the implementation adds timing measurements directly in `run_with_callback` around each operator call and stores them in a parallel field that the reporter reads at `on_finish`. The planner must resolve this: the simplest correct approach is for `DurationReporter` to accumulate total run time via `on_start`/`on_finish` rather than per-phase; true per-phase timing requires instrumentation hooks not in the locked API. **Recommend:** implement `DurationReporter` using total wall-clock time measured by recording `Instant` in `on_start` and computing elapsed in `on_finish`, and break down "per-phase" time proportionally from the `all_stats` slice — or accept that "per-phase timing" in REP-04 means totals only.

The zero-overhead path (REP-02) is guaranteed by `Option<Box<dyn Reporter<U> + Send>>` defaulting to `None`. The compiler eliminates the `if let Some(r) = &mut self.reporter` branch when no reporter is attached at runtime — no virtual dispatch occurs.

**Primary recommendation:** Create `src/reporter/` with four files (`mod.rs`, `noop.rs`, `simple.rs`, `duration.rs`), wire four hook call sites into `run_with_callback`, add `reporter: None` to `Ga::default()`, and add `with_reporter()` directly on `Ga<U>` (not via a config trait). This is a self-contained addition with no ripple effects on existing APIs.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::time::Instant` | stdlib | Wall-clock timing for `DurationReporter` | Zero-dep, monotonic, nanosecond resolution |
| `std::time::Duration` | stdlib | Accumulate per-phase durations | Additive, saturating, no alloc |

### Supporting

None — no external dependencies required for this phase.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Box<dyn Reporter<U> + Send>` | Generic `R: Reporter<U>` on `Ga<U>` | Generic param avoids vtable but breaks `Ga`'s public API (can't store `Ga` in collections, breaks type inference at call site) — rejected in discussion |
| `Option<Box<...>>` for zero-overhead | Always-installed `NoopReporter` | `Option` ensures the branch is elided at runtime; a stored `NoopReporter` still pays the vtable dispatch |

**Installation:** No new Cargo.toml changes required. All required types are in `std`.

## Architecture Patterns

### Recommended Project Structure

```
src/
├── reporter/
│   ├── mod.rs          # pub trait Reporter<U>, pub use re-exports
│   ├── noop.rs         # pub struct NoopReporter
│   ├── simple.rs       # pub struct SimpleReporter { interval: usize, count: usize }
│   └── duration.rs     # pub struct DurationReporter { start: Option<Instant>, elapsed: Duration }
├── ga.rs               # Ga<U> struct: add reporter field + with_reporter() + hook calls
└── lib.rs              # pub mod reporter; prelude re-exports
```

### Pattern 1: Trait with Default No-Op Bodies

**What:** Define `Reporter<U>` with all four hooks as default empty `fn` bodies. Implementors only override what they need.

**When to use:** When the hook receiver should be optional and most users won't need all hooks.

```rust
// Source: Rust reference — default trait methods
pub trait Reporter<U: ChromosomeT>: Send {
    fn on_start(&mut self) {}
    fn on_generation_complete(&mut self, _stats: &GenerationStats) {}
    fn on_new_best(&mut self, _generation: usize, _best: U) {}
    fn on_finish(&mut self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}
```

### Pattern 2: Option-Guarded Hook Calls

**What:** Wrap every hook call in `if let Some(ref mut r) = self.reporter`. The compiler optimizes away the branch entirely when `reporter` is `None` at the call site (monomorphized path is trivial).

**When to use:** For fields that are legitimately absent in the default case.

```rust
// Inside run_with_callback, before the loop:
if let Some(ref mut r) = self.reporter {
    r.on_start();
}

// Inside the loop, after stats collection:
if let Some(ref mut r) = self.reporter {
    r.on_generation_complete(&gen_stats);
}

// Inside the loop, where `improved` is true:
if improved {
    best_fitness_so_far = current_best;
    stagnation_count = 0;
    if let Some(ref mut r) = self.reporter {
        r.on_new_best(i, self.population.best_chromosome.clone());
    }
}

// After the loop, once termination_cause is set:
if let Some(ref mut r) = self.reporter {
    r.on_finish(self.termination_cause, &self.stats);
}
```

### Pattern 3: SimpleReporter with Generation Counter

**What:** Count generations in `on_generation_complete`; print when `count % interval == 0`. Always print at `on_finish`.

```rust
pub struct SimpleReporter {
    interval: usize,
    count: usize,
}

impl SimpleReporter {
    pub fn new(interval: usize) -> Self {
        Self { interval, count: 0 }
    }
}

impl<U: ChromosomeT> Reporter<U> for SimpleReporter {
    fn on_generation_complete(&mut self, stats: &GenerationStats) {
        self.count += 1;
        if self.count % self.interval == 0 {
            println!(
                "[Gen {}] Best: {:.4} | Diversity: {:.4}",
                stats.generation, stats.best_fitness, stats.diversity
            );
        }
    }
    fn on_finish(&mut self, _cause: TerminationCause, all_stats: &[GenerationStats]) {
        if let Some(last) = all_stats.last() {
            println!(
                "[Gen {}] Best: {:.4} | Diversity: {:.4} (finished)",
                last.generation, last.best_fitness, last.diversity
            );
        }
    }
}
```

### Pattern 4: DurationReporter with Instant

**What:** Record `Instant::now()` at `on_start`, compute elapsed at `on_finish`. For true per-phase breakdown, `DurationReporter` must rely on `all_stats` (generation count, population size) to estimate. Since the locked API has no per-phase timing hooks, implement as total run time only, with a note about the limitation.

```rust
pub struct DurationReporter {
    start: Option<std::time::Instant>,
}

impl DurationReporter {
    pub fn new() -> Self {
        Self { start: None }
    }
}

impl<U: ChromosomeT> Reporter<U> for DurationReporter {
    fn on_start(&mut self) {
        self.start = Some(std::time::Instant::now());
    }
    fn on_finish(&mut self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        let elapsed = self.start.map(|s| s.elapsed()).unwrap_or_default();
        let gens = all_stats.len();
        println!("Run complete ({:?}) in {:.2?} over {} generations", cause, elapsed, gens);
        if gens > 0 {
            println!("  Avg per generation: {:.2?}", elapsed / gens as u32);
        }
    }
}
```

### Anti-Patterns to Avoid

- **Generic parameter on `Ga`:** Adding `R: Reporter<U>` as a type parameter to `Ga<U>` forces all callers to specify the reporter type; breaks ergonomics and object safety. The locked decision is `Box<dyn Reporter<U> + Send>`.
- **`Sync` bound on `Reporter`:** Hooks run sequentially from the main GA thread. `Sync` is not required and would unnecessarily restrict user implementations (e.g., reporters using interior mutability).
- **Calling reporter hooks on the callback path but not the `run()` path:** `run()` is implemented as `self.run_with_callback(None, 0)`, so hooks fire consistently when wired into `run_with_callback`. Do not add separate reporter calls in `run()`.
- **Cloning `all_stats` for `on_finish`:** Pass `&self.stats` by reference — `GenerationStats` is already `Clone`, but `on_finish` only needs a slice reference.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Monotonic wall-clock timing | Custom time tracking struct | `std::time::Instant` | Monotonic, no platform drift, zero deps |
| Duration accumulation | Float-based millisecond counters | `std::time::Duration` | Saturating arithmetic, exact nanosecond storage, `Display` via `{:.2?}` |
| Hook dispatch | Custom vtable / function pointer arrays | `dyn Reporter<U>` trait object | Rust's built-in vtable is exactly this; trait objects handle lifetime + Send correctly |

**Key insight:** `std::time` is the entire timing infrastructure needed. No external crates required.

## Common Pitfalls

### Pitfall 1: `on_new_best` Hook Position Relative to `improved` Block

**What goes wrong:** The `improved` boolean and the `stagnation_count` reset appear at lines ~1018–1028 of `ga.rs`, but this is AFTER the callback invocation (lines ~991–1001) and after the fitness limit check (lines ~1003–1013). The reporter's `on_generation_complete` must fire before `on_new_best` within the same generation.

**Why it happens:** The existing loop order is: checkpoint → callback → limit check → stagnation/improved check. The reporter hooks need to fire at well-defined points around these.

**How to avoid:** Wire hooks in this order within the loop body:
1. `on_generation_complete` — fires after stats are collected (line ~858), before existing callback
2. `on_new_best` — fires inside the existing `if improved { ... }` block (line ~1023)
3. Existing callback fires after `on_generation_complete` (preserving current callback behavior)

**Warning signs:** Test that asserts `on_new_best` fired fewer times than `on_generation_complete` would fail if the hook fires at the wrong point.

### Pitfall 2: `on_finish` Fires Before `termination_cause` Is Final

**What goes wrong:** The loop exits in several places (stagnation, convergence, time limit, fitness target, generation limit) and `termination_cause` is set at each break point. Calling `on_finish` inside the loop rather than after it would pass an incorrect `termination_cause`.

**Why it happens:** The natural instinct is to call `on_finish` at each `break` statement.

**How to avoid:** Call `on_finish` once, after the loop exits and after the `GenerationLimitReached` fallback is set (line ~1066). Pattern:

```rust
// after the for loop
if self.termination_cause == TerminationCause::NotTerminated {
    self.termination_cause = TerminationCause::GenerationLimitReached;
}
if let Some(ref mut r) = self.reporter {
    r.on_finish(self.termination_cause, &self.stats);
}
// then existing GenerationLimitReached callback call
```

### Pitfall 3: `on_new_best` Clones `best_chromosome` Before It Is Updated

**What goes wrong:** In `run_with_callback`, the population's `best_chromosome` field is updated inside a block around lines ~823–846 (before stats collection). The `improved` check computes against `self.population.best_chromosome.fitness()` which is already updated. Passing `self.population.best_chromosome.clone()` in `on_new_best` is correct — but ONLY if the hook fires after the best chromosome update block, not before.

**Why it happens:** Confusion about when `best_chromosome` is refreshed vs. when `improved` is evaluated.

**How to avoid:** The `improved` boolean is evaluated after the best chromosome is already updated. `on_new_best` fires inside `if improved { ... }`, so `self.population.best_chromosome.clone()` gives the correct new best.

### Pitfall 4: `SimpleReporter` Counter Skips First Generation

**What goes wrong:** If `count` starts at 0 and the check is `count % interval == 0` before incrementing, generation 0 would always print. If the check is after incrementing and interval is 1, it prints every generation correctly. If interval is 5 and count starts at 0 with post-increment check, generation 5 is the first print — which may surprise users expecting generation 1 to print.

**How to avoid:** Increment first, then check. This gives first print at generation `interval` (1-indexed), which is consistent with users setting `interval = 1` to "print every generation."

### Pitfall 5: DurationReporter Uninitialised `start`

**What goes wrong:** If `on_start` is not called (e.g., reporter added after `run()` starts — not possible with the current API, but defensive coding matters), `start` is `None` and `on_finish` computes a zero duration.

**How to avoid:** Use `Option<Instant>` with `unwrap_or_default()` on the `elapsed()` call, and log a warning if `start` is `None` at `on_finish`.

## Code Examples

### Verified: `TerminationCause` enum (from `src/ga.rs` lines 78–86)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TerminationCause {
    GenerationLimitReached,
    FitnessTargetReached,
    StagnationReached,
    ConvergenceReached,
    TimeLimitReached,
    CallbackRequested,
    NotTerminated,
}
```

### Verified: `GenerationStats` fields (from `src/stats.rs` lines 13–30)

```rust
pub struct GenerationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub avg_fitness: f64,
    pub fitness_std_dev: f64,
    pub population_size: usize,
    pub diversity: f64,    // equals fitness_std_dev in v2.2
}
```

### Verified: `improved` logic (from `src/ga.rs` lines 1018–1028)

```rust
let improved = match self.configuration.limit_configuration.problem_solving {
    ProblemSolving::Maximization => current_best > best_fitness_so_far,
    ProblemSolving::Minimization => current_best < best_fitness_so_far,
    _ => (current_best - best_fitness_so_far).abs() > f64::EPSILON,
};
if improved {
    best_fitness_so_far = current_best;
    stagnation_count = 0;
}
```

### Verified: `Ga<U>` field pattern (from `src/ga.rs` lines 111–113)

```rust
pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
pub fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
// Reporter follows the same Option pattern but uses Box (not Arc — not shared):
// reporter: Option<Box<dyn Reporter<U> + Send>>,
```

### Verified: `ChromosomeT` bounds (from `src/traits/chromosome.rs` line 14)

```rust
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static { ... }
```

`Clone` is already in the supertrait — `on_new_best` receiving `U` (by value, cloned) is valid without adding new bounds.

### Verified: Existing `Ga<U>` builder pattern (from `src/ga.rs`)

```rust
// Builder methods are `fn with_X(mut self, ...) -> Self` directly on `Ga<U>`:
pub fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self {
    self.reporter = Some(reporter);
    self
}
```

### Verified: Loop order (from `src/ga.rs` — relevant sequence)

```
line ~700:  for i in 0..max_generations {
line ~702:  1. selection
line ~716:  2. crossover + mutation (parent_crossover)
line ~728:  3. population merge (add_chromosomes)
line ~741:  4. survivor selection
line ~822:  5. best chromosome update
line ~849:  6. stats collection → gen_stats / self.stats.push(gen_stats.clone())
line ~990:  callback call
line ~1003: fitness limit check + break
line ~1018: improved / stagnation check
line ~1063: } end of loop
line ~1066: termination_cause fallback to GenerationLimitReached
```

Reporter hooks slot in at: after line ~858 (`on_generation_complete`), inside `if improved` block (`on_new_best`), and after line ~1068 (`on_finish`). `on_start` fires just before the `for` loop.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Callback-only observation (`run_with_callback`) | Reporter trait + callback coexist | Phase 8 | Structured hooks without deprecating existing callback API |
| Manual `Instant::now()` in user code | `DurationReporter` built-in | Phase 8 | Users get timing without writing boilerplate |

**Deprecated/outdated:** Nothing deprecated in this phase. `run_with_callback` remains unchanged.

## Open Questions

1. **DurationReporter per-phase timing without per-phase hooks**
   - What we know: The locked API has no `on_before_selection`, `on_after_selection`, etc. hooks. `DurationReporter` stores timing as mutable state and "uses on_finish to print."
   - What's unclear: How does `DurationReporter` obtain per-phase wall-clock time with only `on_start`, `on_generation_complete`, `on_new_best`, and `on_finish`?
   - Recommendation: Interpret REP-04 as "total run time with a per-generation average" derived from `on_start`/`on_finish`. The planner should implement `DurationReporter` using total elapsed time only. True per-phase breakdown would require adding timing instrumentation directly in `run_with_callback` around each operator call — this is a future enhancement and out of scope for Phase 8 per the locked API. Document the limitation in a `// Note:` comment in `DurationReporter`.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`, `cargo test`) |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test reporter` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REP-01 | `with_reporter()` builder compiles and stores reporter | unit | `cargo test reporter::tests::test_with_reporter_builder` | Wave 0 |
| REP-01 | `on_start` fires once before first generation | unit | `cargo test reporter::tests::test_on_start_fires_once` | Wave 0 |
| REP-01 | `on_generation_complete` fires once per generation | integration | `cargo test test_ga::test_reporter_on_generation_complete_count` | Wave 0 |
| REP-01 | `on_new_best` fires when fitness improves | integration | `cargo test test_ga::test_reporter_on_new_best_fires_on_improvement` | Wave 0 |
| REP-01 | `on_finish` fires once with correct `TerminationCause` | integration | `cargo test test_ga::test_reporter_on_finish_termination_cause` | Wave 0 |
| REP-02 | `Ga` without reporter runs with no reporter field set | unit | `cargo test test_ga::test_no_reporter_default` | ❌ Wave 0 |
| REP-02 | `NoopReporter` compiles and satisfies `Reporter<U>` | unit | `cargo test reporter::tests::test_noop_reporter` | Wave 0 |
| REP-03 | `SimpleReporter::new(n)` prints at interval N | unit | `cargo test reporter::tests::test_simple_reporter_interval` | Wave 0 |
| REP-03 | `SimpleReporter` always prints at `on_finish` | unit | `cargo test reporter::tests::test_simple_reporter_always_finish` | Wave 0 |
| REP-04 | `DurationReporter` produces timing output at `on_finish` | unit | `cargo test reporter::tests::test_duration_reporter_finish` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test reporter`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_reporter.rs` — integration tests covering REP-01 through REP-04 with the `Ga` orchestrator
- [ ] `src/reporter/mod.rs`, `noop.rs`, `simple.rs`, `duration.rs` — the module itself (created in Wave 1)
- [ ] No framework install needed — `cargo test` is already the project test runner

## Sources

### Primary (HIGH confidence)

- `src/ga.rs` (read directly) — `Ga<U>` struct layout, `run_with_callback` loop, `TerminationCause`, `improved` logic, callback call sites
- `src/stats.rs` (read directly) — `GenerationStats` all fields confirmed
- `src/traits/chromosome.rs` (read directly) — `ChromosomeT: Clone + Default + Send + Sync + 'static` supertrait
- `src/lib.rs` (read directly) — existing `pub mod` declarations, prelude pattern
- `.planning/phases/08-reporter-trait/08-CONTEXT.md` (read directly) — all locked decisions

### Secondary (MEDIUM confidence)

- Rust reference on trait objects: `dyn Trait + Send` enables `Box<dyn Reporter<U> + Send>` with single-threaded sequential call semantics
- `std::time::Instant` — monotonic clock, `Instant::now()` + `.elapsed()` pattern is idiomatic Rust for wall-clock timing

### Tertiary (LOW confidence)

- None — all research grounded in direct source code inspection and Rust stdlib

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all technology is Rust stdlib; no external deps needed
- Architecture: HIGH — locked decisions in CONTEXT.md; verified against `src/ga.rs` line numbers
- Pitfalls: HIGH — derived from reading the actual `run_with_callback` loop and identifying exact integration points
- Open questions: MEDIUM — DurationReporter per-phase timing is architecturally underconstrained given the locked API

**Research date:** 2026-03-21
**Valid until:** 2026-06-21 (stable domain — Rust stdlib, internal code)
