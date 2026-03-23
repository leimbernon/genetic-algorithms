# Domain Pitfalls

**Domain:** Adding observability (observer trait, tracing, metrics) to a high-performance Rust GA library
**Researched:** 2026-03-23
**Project:** genetic_algorithms v2.1.0

---

## Critical Pitfalls

Mistakes that cause rewrites, race conditions, or silent performance regressions.

---

### Pitfall 1: Arc Clone Inside the Rayon Parallel Closure

**What goes wrong:** The observer is proposed as `Option<Arc<dyn GaObserver<U>>>`. If an `Arc::clone` is performed inside the `par_iter()` closure in `parent_crossover` (ga.rs:958), every parent pair processed in parallel will perform an atomic reference-count increment/decrement on the same cache line. With large populations (e.g., 1000 parent pairs), this becomes a bottleneck that defeats the purpose of rayon parallelism.

**Why it happens:** The natural instinct is to clone the `Arc` inside the closure so each rayon task "owns" its reference. This is what happens with `fitness_fn` in island/mod.rs:457 (`let ff = Arc::clone(&fitness_fn)`), but that pattern works because fitness_fn is cloned inside a sequential for-loop, not inside a `par_iter()` over hundreds of items simultaneously.

**Consequences:**
- Atomic contention across all rayon worker threads on a single memory location
- Hot cache-line bouncing between CPU cores
- Performance regression that scales with population size and thread count — worst at the exact use case this library is designed for (large populations, many threads)

**Prevention:**
- Clone the `Arc<dyn GaObserver<U>>` **once before** the parallel region, not inside it
- The observer reference passed into the rayon closure should be a shared borrow `&Arc<dyn GaObserver<U>>` (rayon closures can capture shared references as long as the Arc lives longer than the parallel region)
- Pattern: `let obs = observer.as_deref();` then move `obs` into the closure as a `Option<&dyn GaObserver<U>>`
- If mutation inside the closure is needed, prefer per-observer-call read locks or design observer methods as `&self` (immutable)

**Detection:** Benchmark `ga_run` with and without a no-op observer attached. Any overhead above ~1% per generation indicates contention.

**Phase:** Phase 1 (GaObserver trait definition). The access pattern must be decided during trait design, not retrofitted later.

---

### Pitfall 2: tracing `Span::enter()` Guard in Rayon Closures

**What goes wrong:** `tracing::span.enter()` returns an `Entered<'_>` guard that must be dropped on the same thread it was created on. Rayon closures can be stolen by arbitrary worker threads. Using `let _guard = span.enter()` inside a rayon parallel iterator will cause the guard to be dropped on a different thread than it was entered on, violating `tracing`'s threading contract.

**Why it happens:** The existing `debug!()` calls inside `parent_crossover`'s `par_iter()` (line 1011) work because `log::debug!()` is thread-safe. TracingObserver developers may copy this pattern and assume tracing macros behave identically. They do not — `tracing::instrument` and `Span::enter()` are fundamentally different from `log::debug!()`.

**Consequences:**
- `tracing` in strict mode (or with a subscriber that validates thread-local state) will panic
- Spans may appear unclosed in Jaeger/OpenTelemetry traces, creating orphaned spans
- If guards are dropped silently, the subscriber's parent-child span relationship is corrupted, producing unreadable traces
- The bug only manifests under load (with actual rayon workers stealing tasks), making it intermittent

**Prevention:**
- Never use `span.enter()` or `#[tracing::instrument]` inside rayon closures
- Use `tracing::Span::in_scope(|| ...)` instead of `enter()` guards — `in_scope` enters and exits on the same call stack frame safely
- Alternatively, use `span.follows_from(&parent_span)` to link async/parallel spans without nesting
- TracingObserver hooks that are called from inside parallel regions must document "do not enter spans here; record events only"
- Observer hooks inside `parent_crossover` should emit `tracing::event!()` (equivalent to `debug!()`) rather than span operations

**Detection:** Run integration tests with `RUST_LOG=trace` and a `tracing-subscriber` configured with thread tracking. Any span-open-without-close log indicates the bug.

**Phase:** Phase 3 (TracingObserver). Must be addressed before implementing any observer hooks called from rayon parallel regions.

---

### Pitfall 3: metrics Counter/Histogram Contention from Rayon Worker Threads

**What goes wrong:** If MetricsObserver calls `counter.increment()` or `histogram.record()` from inside `par_iter()` closures, all rayon threads will contend on the same metrics shard. Most metrics crates (metrics-rs facade) use `RwLock` or `DashMap` internally for their global registry. Under high parallelism, these locks become bottlenecks that can account for more overhead than the genetic operators themselves.

**Why it happens:** `metrics::counter!("ga.crossover.count")` looks like a free macro call but invokes a global registry lookup plus an atomic increment under a reader lock. Multiply by population_size × rayon_threads per generation.

**Consequences:**
- Non-obvious performance regression: the GA completes correctly, but 10x–50x slower at high population sizes
- The regression is only visible under profiling — the user sees "slow GA" without understanding the metrics layer is the cause
- Library reputation damaged since users do not know MetricsObserver is the cause

**Prevention:**
- Never call `metrics::counter!` or similar inside `par_iter()` closures
- Accumulate counts per-thread as local integers, then record the batch total after the parallel region completes (`counter!("ga.crossover.count", batch_count)`)
- MetricsObserver hooks that are called from hot parallel paths must receive pre-aggregated `GenerationStats` (which already collects population-wide stats), not per-chromosome events
- Design `on_generation_end(stats: &GenerationStats)` as the correct hook for metrics recording — it is called once per generation from the sequential outer loop

**Detection:** Benchmark `ga_run` bench with MetricsObserver attached vs no observer. A regression proportional to `population_size × threads` (not just generation count) confirms the contention pattern.

**Phase:** Phase 5 (MetricsObserver). The hook granularity must be documented clearly during Phase 1 trait design: which hooks are "hot" (rayon context) and which are "cold" (sequential context).

---

### Pitfall 4: LogObserver Breaks Existing env_logger Initialization

**What goes wrong:** `ga.rs:554` calls `env_logger::Builder::from_default_env().filter_level(log_level).try_init()`. LogObserver, if it also attempts to initialize a logger, will conflict. `try_init()` silently fails if a logger is already initialized — but the inverse is also true: if LogObserver initializes env_logger first, the GA's programmatic log level configuration will have no effect, breaking the backward-compatibility guarantee.

**Why it happens:** The migration from hardcoded `log!()` macros to LogObserver requires deciding who owns logger initialization. The current GA owns it (try_init per run). If LogObserver is a user-constructed object, users may initialize their own logger before calling `ga.run()`, causing silent level-filter conflicts.

**Consequences:**
- `LogLevel::Off` in GaConfiguration no longer silences output when LogObserver is active (backward-compat breakage)
- Users who initialize `env_logger` before constructing GA will see unexpected output level changes — or silence when they expect logs
- The "identical log output" requirement (PROJECT.md constraint) is violated

**Prevention:**
- LogObserver must NOT own or initialize a logger. It is a bridge: it receives events from the `GaObserver` hook system and calls `log::info!()`, `log::debug!()`, etc.
- Logger initialization stays owned by the GA configuration path (`ga.rs`), exactly as today
- The `LogLevel` in `GaConfiguration` should gate which observer events produce log output (LogObserver checks the configured level before emitting)
- Add a test: run GA with `LogLevel::Off` + LogObserver attached, capture log output, assert it is empty

**Detection:** Unit test: construct GA with LogLevel::Off, attach LogObserver, run, assert no log lines emitted. Fails if LogObserver bypasses the level filter.

**Phase:** Phase 2 (LogObserver). Must be validated before shipping — the backward-compatibility test is a hard requirement.

---

### Pitfall 5: Option<Arc<dyn GaObserver<U>>> Overhead When None — Missing the Zero-Cost Branch

**What goes wrong:** The project mandates "zero overhead when no observer is set." `Option::None` does eliminate the observer call, but only if the call site is structured so the compiler can eliminate it entirely. If observer hooks are called inside hot rayon closures via `if let Some(obs) = observer { obs.on_crossover(...)  }`, the branch plus the potential `Arc` dereference remain in the compiled closure — the compiler cannot statically eliminate them because `Arc<dyn Trait>` is not monomorphized.

**Why it happens:** Trusting that `Option::None` = zero overhead is correct for single-threaded sequential code but assumes the compiler can inline and eliminate the branch. With dynamic dispatch (`dyn GaObserver<U>`), LLVM cannot see through the vtable to determine the call is a no-op and cannot eliminate the branch. In hot loops this means a branch miss per iteration even when `None`.

**Consequences:**
- The zero-overhead guarantee is violated when the observer is `None` but the check is inside a hot parallel closure (100–1000 iterations per generation)
- Benchmark regressions will be subtle: ~1–5ns per chromosome pair, significant only at high population sizes
- No correctness bug, just a silently broken performance promise

**Prevention:**
- Observer hooks inside the rayon hot path (`parent_crossover`, fitness evaluation) should be gated at the call site **before** entering the parallel region: if `observer.is_none()`, set a boolean flag and skip the hook dispatch entirely using `if self.has_observer { ... }`
- Hot-path hooks (per-chromosome events) should be considered `#[inline]` free functions that the compiler can analyze
- Prefer observer hooks at the per-generation level (already sequential) over per-chromosome level. `on_generation_end(stats)` is cold; `on_chromosome_evaluated()` is hot
- Review which hooks are called inside `par_iter()` vs the outer `for i in 0..max_generations` loop. Outer-loop hooks can safely use `Option<Arc<dyn ...>>`. Inner-loop hooks need the boolean short-circuit guard

**Detection:** Criterion benchmark: run with `observer = None` (default). Compare against a baseline with no observer field on the struct. Any statistically significant regression indicates the branch is not being eliminated.

**Phase:** Phase 1 (GaObserver trait design). Hook granularity decisions (which hooks are per-generation vs per-individual) must be made here.

---

## Moderate Pitfalls

---

### Pitfall 6: Default No-Op Methods Break dyn Dispatch for New Events

**What goes wrong:** The plan is to use default no-op methods on `GaObserver` for forward compatibility. However, adding a new method with a default no-op body to a `dyn Trait` after the trait is published adds a new vtable entry. Any user who compiled their `impl GaObserver for MyObserver` against the old trait will have a vtable that doesn't include the new method — this is a **compile-time break**, not a silent runtime issue.

**Prevention:**
- This is the standard Rust sealed-trait / version-bump tradeoff. Default methods provide forward compat for new implementors, but changing an existing method signature or adding methods to a published trait is still a minor semver bump (v2.1.x → v2.2.0)
- Plan for this: design the initial `GaObserver` method set to be comprehensive enough to avoid adding methods in patch releases
- Document in CHANGELOG and README: "Adding new default methods to GaObserver is a minor version bump"

**Phase:** Phase 1 (trait design). Enumerate the full set of desired hooks before publishing.

---

### Pitfall 7: Observer Called Before Population is Initialized

**What goes wrong:** `ga.run()` calls `initialization()` which calls `info!("Initialization started")`. If the observer's `on_run_start` hook is fired before `initialize()` (reasonable design), and the observer tries to read `population.size()` or similar state, it will encounter an empty population.

**Prevention:**
- Define hook firing order explicitly in trait documentation: `on_run_start` fires before population is ready; `on_generation_start` fires when population is valid
- Hooks that receive population data should only be `on_generation_*` hooks, not `on_run_start`
- Never pass `&Population<U>` to `on_run_start`

**Phase:** Phase 1 (trait design). Document the lifecycle contract.

---

### Pitfall 8: Island GA and NSGA-II Sub-Traits Add Vtable Complexity

**What goes wrong:** The island GA evolves islands in `par_iter_mut()` (island/mod.rs:404). If `IslandGaObserver` extends `GaObserver` and is called from within that parallel closure, it faces all the same vtable/Arc contention issues as the base observer — but they are harder to notice because island observers may be added in a later phase when the base observer pattern seems "proven."

**Prevention:**
- Apply the same "clone Arc before parallel region" rule to island observers
- `IslandGaObserver::on_island_generation_end(island_idx, stats)` is the right hook granularity — it is called once per island per generation, from within the rayon closure, but that closure is not itself fine-grained (one island per rayon task, not one chromosome)
- `on_island_generation_end` being called once per island per generation is acceptable overhead; `on_chromosome_evaluated` inside the island crossover loop is not

**Phase:** Phase 4 (Island/NSGA-II observer sub-traits).

---

### Pitfall 9: Feature Flag Compilation and Re-export Errors

**What goes wrong:** `TracingObserver` behind `observer-tracing` and `MetricsObserver` behind `observer-metrics` must not be re-exported from `lib.rs` without their feature gate. If `pub use observer::TracingObserver` is added unconditionally to `prelude.rs` or `lib.rs`, it will cause a compilation error in the common case (feature not enabled) and confuse IDE auto-complete.

**Prevention:**
- Wrap all observer type exports in `#[cfg(feature = "observer-tracing")]` / `#[cfg(feature = "observer-metrics")]`
- Add CI matrix entries: `cargo build --no-default-features`, `cargo build --features observer-tracing`, `cargo build --features observer-metrics`, `cargo build --all-features`
- Ensure `tracing` and `metrics` crates are `optional = true` in Cargo.toml and not transitively pulled in by default

**Phase:** Phase 3 (TracingObserver) and Phase 5 (MetricsObserver). The CI matrix must be added as part of Phase 3, before Phase 5 adds a second optional feature.

---

## Minor Pitfalls

---

### Pitfall 10: env_logger Initialized Multiple Times in Tests

**What goes wrong:** `try_init()` is silent on re-init failure, but running multiple GA tests in the same process (Rust's default test runner) will cause all but the first to silently get the log level from the first test's configuration. Tests that call `ga.run()` with `LogLevel::Debug` will not produce debug output if an earlier test used `LogLevel::Off`.

**Prevention:**
- Use `once_cell::sync::Lazy` or `std::sync::Once` for logger initialization in tests
- Document in the test module: logger initialization is one-shot per process; use `RUST_LOG` env var to control test output

**Phase:** Phase 2 (LogObserver). Affects test reliability during backward-compat validation.

---

### Pitfall 11: CompositeObserver Must Be Send + Sync

**What goes wrong:** `CompositeObserver` holds a `Vec<Box<dyn GaObserver<U>>>`. `Box<dyn GaObserver<U>>` is only `Send + Sync` if the trait bound includes `+ Send + Sync`. Forgetting the bounds will cause a compile error when users try to attach `CompositeObserver` to a GA that uses rayon.

**Prevention:**
- Define `GaObserver<U>: Send + Sync` as a supertrait bound from the beginning
- `CompositeObserver` field: `Vec<Box<dyn GaObserver<U> + Send + Sync>>`
- Equivalently, add `Send + Sync` to the trait definition so they are always implied

**Phase:** Phase 1 (GaObserver trait). The `Send + Sync` supertrait cannot be added later without a semver break.

---

### Pitfall 12: Checkpoint Log Warning Regression After Observer Migration

**What goes wrong:** Line 764 in ga.rs: `log::warn!("Failed to save checkpoint...")` is a direct `log::warn!` call inside a `#[cfg(feature = "serde")]` block. When migrating logging to LogObserver, this call may be missed because it is in a feature-gated block that only compiles with `serde`. The checkpoint warning would be silenced after migration if only `ga_events` log target is migrated.

**Prevention:**
- Audit all `log::*!` call sites with `grep -rn 'log::' src/` before beginning the migration
- Create a migration checklist of every log call site (ga.rs, island/mod.rs, nsga2/mod.rs, traits/chromosome.rs) before starting Phase 2
- Run `cargo build --features serde` as part of LogObserver validation

**Phase:** Phase 2 (LogObserver). Must be part of the pre-migration audit.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| GaObserver trait (Phase 1) | Arc clone inside par_iter (Pitfall 1) | Decide access pattern before writing any hook call site |
| GaObserver trait (Phase 1) | Missing Send+Sync supertrait (Pitfall 11) | Add to trait definition on day one |
| GaObserver trait (Phase 1) | Zero-overhead None branch in hot path (Pitfall 5) | Document which hooks are hot vs cold; prefer per-generation hooks |
| LogObserver (Phase 2) | Breaks env_logger initialization (Pitfall 4) | LogObserver is a bridge, not a logger owner |
| LogObserver (Phase 2) | Missed serde-gated log calls (Pitfall 12) | Run pre-migration audit before writing code |
| TracingObserver (Phase 3) | span.enter() in rayon closures (Pitfall 2) | Use in_scope or event! only inside parallel regions |
| TracingObserver (Phase 3) | Feature flag re-export errors (Pitfall 9) | Add CI matrix as part of this phase |
| Island/NSGA-II observers (Phase 4) | Same Arc contention, harder to notice (Pitfall 8) | Apply base observer access rules explicitly |
| MetricsObserver (Phase 5) | Counter contention in par_iter (Pitfall 3) | Only record aggregated stats from on_generation_end |
| CompositeObserver (Phase 5) | New observer vtable entries break implementations (Pitfall 6) | Lock down initial hook set; treat additions as minor version bumps |

---

## Sources

- Codebase analysis: `/src/ga.rs` (lines 554–780, 958–1067), `/src/island/mod.rs` (lines 403–464), `/src/traits/chromosome.rs`
- Rust standard library documentation on `Arc::clone` and dynamic dispatch overhead (HIGH confidence — official docs)
- `tracing` crate threading model: `Span::enter()` is not `Send`; `in_scope` is the thread-safe alternative (HIGH confidence — well-documented in tracing crate)
- Rayon `par_iter` closure capture semantics and task-stealing model (HIGH confidence — rayon documentation)
- `metrics` crate global registry uses `RwLock`/`DashMap` internally — contention under parallel writes (MEDIUM confidence — based on metrics-rs source and common patterns; validate with benchmark)
- env_logger `try_init()` one-shot initialization behavior (HIGH confidence — env_logger documentation)
- Semver implications of adding default methods to published Rust traits (HIGH confidence — Rust API compatibility guidelines)
