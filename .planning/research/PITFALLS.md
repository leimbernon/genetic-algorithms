# Pitfalls Research

**Domain:** Adding observability (GaObserver trait, tracing, metrics) to a concurrent Rust GA library
**Researched:** 2026-03-25
**Confidence:** HIGH (based on codebase analysis + official Rust/tracing/rayon documentation)

---

## Critical Pitfalls

Mistakes that cause rewrites, race conditions, or silent performance regressions.

---

### Pitfall 1: Arc Clone Inside the Rayon Parallel Closure

**What goes wrong:**
`GaObserver<U>` is planned as `Option<Arc<dyn GaObserver<U> + Send + Sync>>`. If `Arc::clone` is called inside the `par_iter()` closure in `parent_crossover` (ga.rs), every parent pair processed in parallel performs an atomic reference-count increment/decrement on the same cache line. With populations of 1000, this creates hot cache-line bouncing across all rayon worker threads — defeating rayon parallelism at the exact use case this library targets.

**Why it happens:**
The existing `fitness_fn` pattern in island/mod.rs does `let ff = Arc::clone(&fitness_fn)` inside a for-loop, which looks the same but executes sequentially. Developers see this precedent and copy it into `par_iter()` closures. The two sites look identical in code but behave very differently under parallelism.

**How to avoid:**
- Clone `Arc<dyn GaObserver<U>>` exactly **once before** entering the parallel region
- Pass a shared borrow into the closure: `let obs_ref = observer.as_deref();` captures `Option<&dyn GaObserver<U>>` — no Arc clone inside the closure
- Observer methods must be `&self` (immutable) to allow shared borrows across rayon threads without a Mutex
- If stateful observers need mutation (e.g., counting), use `AtomicU64` internally; if they need `&mut self`, they cannot be called from inside `par_iter()` at all

**Warning signs:**
- Benchmark `ga_run` criterion bench with a no-op `Arc<dyn GaObserver>` attached; any overhead proportional to `population_size × thread_count` indicates Arc contention
- Observer field type is `Arc<dyn ...>` but `Arc::clone` appears inside a `par_iter().for_each(...)` or `par_iter().map(...)` closure

**Phase to address:**
Phase 1 (GaObserver trait definition). The `&self` vs `&mut self` decision for observer methods determines the access pattern for all downstream phases. Must be locked before writing any hook call site.

---

### Pitfall 2: `tracing` `Span::enter()` Guard in Rayon Closures

**What goes wrong:**
`tracing::span.enter()` returns an `Entered<'_>` guard that must be dropped on the same thread it was created. Rayon uses work-stealing: a closure may be enqueued on one thread and executed on another. If `let _guard = span.enter()` is placed inside a rayon parallel iterator, the guard may be created on thread A and dropped on thread B, violating tracing's threading contract.

**Why it happens:**
The existing `debug!(target="ga_events", ...)` calls inside `parent_crossover`'s parallel region work correctly because `log::debug!()` is stateless — it emits and forgets. `TracingObserver` developers see these call sites and assume tracing macros behave identically. They do not: `tracing::instrument` macro and `Span::enter()` maintain per-thread stack state.

**How to avoid:**
- Never use `span.enter()` or `#[tracing::instrument]` inside rayon closures
- Use `tracing::Span::in_scope(|| { ... })` — it enters and exits within a single call stack frame, safe under work-stealing
- For cross-thread causality, use `span.follows_from(&parent_span)` instead of span nesting
- Observer hooks invoked from inside parallel regions must document: "emit `tracing::event!()` only — do not enter spans"
- `TracingObserver` hooks called from `parent_crossover`'s parallel path should emit events, not spans

**Warning signs:**
- `tracing-subscriber` with thread tracking logs unclosed spans or mismatched thread IDs
- Spans appear as orphans in Jaeger/OTLP collectors (no parent, open-ended duration)
- The bug only manifests under load (actual rayon task-stealing) and not in single-threaded tests

**Phase to address:**
Phase 3 (TracingObserver). Must be addressed before writing any observer hook call sites inside the rayon parallel region. Add an explicit test: run GA under `cargo test -- --test-threads=1` vs `--test-threads=4` and verify no span warnings.

---

### Pitfall 3: Metrics Counter/Histogram Contention from Rayon Worker Threads

**What goes wrong:**
If `MetricsObserver` calls `metrics::counter!("ga.crossover.count")` or `metrics::histogram!("ga.fitness")` from inside `par_iter()` closures, all rayon threads contend on the same metrics shard. The `metrics` crate's global registry uses `RwLock`/`DashMap` internally. Under 8+ rayon threads × 500 parent pairs per generation, the lock becomes the bottleneck — accounting for more overhead than the genetic operators themselves.

**Why it happens:**
`metrics::counter!("x")` looks like a free macro call. The atomic increment is hidden behind the facade. Developers do not realize that every call involves a global registry lookup plus an atomic or lock operation. Multiplied by `population_size × threads` per generation, this degrades from O(1) to O(N threads) contention.

**How to avoid:**
- Never call `metrics::counter!` or `metrics::histogram!` inside `par_iter()` closures
- Accumulate counts as thread-local or local integers within the parallel closure; record the batch total **after** the parallel region: `counter!("ga.crossover.count", batch_total)`
- `MetricsObserver` hooks must only be attached to per-generation sequential hooks (`on_generation_end(stats)`) — `GenerationStats` already provides pre-aggregated population-wide values
- Design rule: if a hook fires N times per generation (N = population size), it is a hot hook and must never call the metrics facade directly

**Warning signs:**
- Benchmark shows runtime proportional to `population_size × thread_count` rather than just `population_size`
- `perf` or `cargo flamegraph` shows rayon worker threads spending significant time in `parking_lot::RwLock::read` or `dashmap::DashMap::get`

**Phase to address:**
Phase 5 (MetricsObserver). The hot/cold hook distinction must be documented in Phase 1 trait design so Phase 5 implementors know which hooks are safe for metrics calls.

---

### Pitfall 4: LogObserver Breaks Existing env_logger Initialization

**What goes wrong:**
`ga.rs` calls `env_logger::Builder::from_default_env().filter_level(log_level).try_init()` inside `run_with_callback`. LogObserver, if it also attempts to initialize or reconfigure a logger, will conflict. `try_init()` silently fails if a logger is already initialized — the inverse is also true: if the user initializes `env_logger` before calling `ga.run()`, the GA's programmatic `LogLevel` configuration has no effect. The backward-compatibility guarantee ("identical log output") is broken silently.

**Why it happens:**
`LogObserver` appears to be a logging component, so developers instinctively give it logger initialization responsibility. The existing `Reporter<U>` (which `LogObserver` conceptually replaces) has no logger ownership, but that boundary is not obvious.

**How to avoid:**
- `LogObserver` is a **bridge**, not a logger owner. It receives events from `GaObserver` hooks and emits them via `log::info!()`, `log::debug!()`, etc.
- Logger initialization stays owned by the GA configuration path in `ga.rs` — exactly as it is today
- `LogLevel` in `GaConfiguration` gates which observer events produce log output; `LogObserver` checks the configured level before emitting
- Add a regression test: construct GA with `LogLevel::Off` + `LogObserver` attached, capture stdout/log output, assert it is empty

**Warning signs:**
- `LogObserver` struct has any `init`, `builder`, or `env_logger` calls in its constructor or `new()` method
- Running two GA instances in sequence in the same test produces different log levels in the second run

**Phase to address:**
Phase 2 (LogObserver). The backward-compatibility test (`LogLevel::Off` + `LogObserver` = no output) is a hard requirement that must pass before the phase is marked complete.

---

### Pitfall 5: LogObserver + Old `log!()` Calls Produce Duplicate Output During Migration

**What goes wrong:**
`ga.rs`, `island/mod.rs`, `nsga2/mod.rs`, and operator files contain hardcoded `info!()`, `debug!()`, `trace!()` calls. If `LogObserver` is attached and emits the same events via the observer hook system before the old direct `log!()` calls are removed, every logged event appears twice — once from the direct macro and once from `LogObserver`.

**Why it happens:**
The migration strategy of "add observer hooks, then remove old log calls" creates a window where both systems are active. In practice, the removal step is deferred ("we'll clean that up") and the duplicate output ships to users.

**How to avoid:**
- Migration is atomic per module, not incremental: for each file, either all `log!()` calls go through the observer OR none do — never both simultaneously
- Audit all log call sites before starting migration: `grep -rn 'log::\|info!\|debug!\|trace!\|warn!' src/` — as of this writing, 40+ files contain log calls
- The `#[cfg(feature = "serde")]`-gated `log::warn!` in ga.rs (checkpoint failure) is easy to miss; run `cargo build --features serde` explicitly
- Add an integration test that attaches `LogObserver` and counts log lines per generation; assert the count matches the known baseline (not double the baseline)

**Warning signs:**
- Running `RUST_LOG=debug cargo test` after attaching `LogObserver` shows each event appearing twice in output
- Log count per generation is approximately 2× what it was before observer migration

**Phase to address:**
Phase 2 (LogObserver). Pre-migration audit must be completed before writing `LogObserver`. Remove each `log!()` call site at the same commit it is replaced by an observer event — never in a separate cleanup commit.

---

### Pitfall 6: `tracing-log` Bridge Causes Infinite Recursion with `TracingObserver`

**What goes wrong:**
If users enable `tracing-log`'s `LogTracer` (which routes `log::*!` calls into the tracing subscriber) AND the `TracingObserver` internally emits `log::*!` calls, the result is an infinite loop: `log::info!` → `LogTracer` → tracing event → `TracingObserver::on_generation_end` → `log::info!` → ... The call stack overflows.

This is documented in `tracing-log`'s own docs: "Log::Logger implementations that convert log records to trace events should not be used with Subscribers that convert trace events back into log records."

**Why it happens:**
Users enabling full structured tracing often enable `LogTracer::init()` to capture all `log::*` output. If `TracingObserver` also emits via `log::*` internally (attempting to be compatible with users who don't use tracing subscribers), the bridge closes into a cycle.

**How to avoid:**
- `TracingObserver` must ONLY emit via `tracing::event!()` / `tracing::span!()` — never via `log::*!`
- Document `TracingObserver` contract: "This observer requires a tracing subscriber. Do not initialize `tracing_log::LogTracer` if any observer also emits via the `log` crate."
- The existing direct `log!()` calls in `ga.rs` and the `TracingObserver` path must be treated as two separate output channels — they cannot share a common sink when `LogTracer` is active
- After full migration to observer hooks, the direct `log!()` calls in `ga.rs` should be removed to eliminate the bridge risk entirely

**Warning signs:**
- Stack overflow during GA run when `RUST_LOG=trace` is set and `LogTracer` is initialized
- Test hangs that only occur with `--features observer-tracing` and a tracing subscriber initialized

**Phase to address:**
Phase 3 (TracingObserver). Document the bridge risk in the `TracingObserver` rustdoc. Add a CI test that initializes `LogTracer` + attaches `TracingObserver` and runs for 5 generations without overflow.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Keep old `log!()` calls during LogObserver migration | Faster Phase 2 delivery | Permanent duplicate output; impossible to silence the old path | Never — migration must be atomic per module |
| `Box<dyn GaObserver<U> + Send>` instead of `Arc<dyn ...>` | Matches existing `Reporter<U>` pattern; no reference counting | Cannot share observer across rayon threads in island GA; forces per-island observer duplication | Never for GaObserver — island GA requires shared ownership |
| Per-chromosome observer hooks (`on_chromosome_evaluated`) | Rich telemetry | Called N times per generation per thread; breaks zero-overhead guarantee; locks in performance regression | Never — use `on_generation_end(stats)` with pre-aggregated data |
| `#[cfg(feature = "observer-tracing")]` only on the impl, not the re-export | Less code change | `use genetic_algorithms::TracingObserver` fails to compile without the feature; user confusion | Never — gate both the type definition AND the re-export |
| Sealing `GaObserver` trait to prevent external implementations | Makes future method additions non-breaking | Prevents users from writing custom observers (contradicts the library's purpose) | Never — the trait must be unsealed; accept that method additions are minor version bumps |

---

## Integration Gotchas

Common mistakes when connecting to external systems.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `env_logger` + `LogObserver` | LogObserver calls `env_logger::init()` in constructor | LogObserver only calls `log::info!()` etc.; logger init stays in `ga.rs` `run_with_callback` |
| `tracing-subscriber` + `tracing-log` bridge | Enabling `LogTracer` when `TracingObserver` emits via `log::*` | `TracingObserver` emits only via `tracing::event!`; never via `log::*` |
| `metrics` crate + rayon | Calling `metrics::counter!` inside `par_iter()` | Only record metrics in sequential per-generation hooks; batch counts from parallel regions |
| `Reporter<U>` (existing) + `GaObserver<U>` (new) | Attaching both independently without knowing they overlap | Document that `LogObserver` is the canonical migration path; `Reporter<U>` hooks and `GaObserver<U>` hooks should not both emit the same log lines |
| Island GA `par_iter_mut()` + observer | Calling observer from inside island parallel closure without pre-cloning Arc | Clone `Arc<dyn IslandGaObserver<U>>` once before the `par_iter_mut()` call; pass `&Arc` into closure |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| `Arc::clone` inside `par_iter()` closure | Benchmark overhead proportional to `pop_size × threads`; flamegraph shows `Arc::drop` hot | Clone once before parallel region; use `&Arc` inside closure | Population > 200 on 4+ core machine |
| Metrics facade calls inside hot operator hooks | Runtime 10x–50x slower at large populations; profiler shows lock contention in metrics registry | Only record in `on_generation_end`; use batch aggregation | Population > 500 with 8+ rayon threads |
| `dyn GaObserver<U>` None-branch inside hot loop | ~1–5ns overhead per chromosome pair even when `observer = None`; no correctness bug | Pre-check `observer.is_some()` before the parallel region; use a boolean flag | Population > 1000 per generation (noticeable at benchmark precision) |
| `Span::enter()` in rayon closure | Intermittent panics or hang under load; orphaned spans in trace collector | Use `in_scope(|| ...)` or `event!()` only inside rayon closures | Any multi-threaded run with work-stealing active |
| `Vec<Box<dyn GaObserver<U>>>` in `CompositeObserver` without `Send + Sync` bounds | Compile error when attaching `CompositeObserver` to GA | Define `GaObserver<U>: Send + Sync` as supertrait from Phase 1 | First attempt to use `CompositeObserver` with rayon-backed GA |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **LogObserver migration complete:** All `log!()` call sites in `ga.rs`, `island/mod.rs`, `nsga2/mod.rs`, and feature-gated serde blocks have been replaced. Verify with `grep -rn 'info!\|debug!\|trace!\|warn!' src/` — only `LogObserver` itself and operator files should remain.
- [ ] **Zero-overhead guarantee verified:** Run criterion bench `ga_run` with `observer = None` (default) against baseline without any observer field. Statistically indistinguishable results confirms the None branch is eliminated.
- [ ] **`Send + Sync` supertrait on `GaObserver`:** Verify `CompositeObserver` compiles when attached to `IslandGa<U>`. If `Send + Sync` is missing from the supertrait, this only fails at the `IslandGa` attachment site — not at `Ga<U>`.
- [ ] **Feature flag CI matrix:** Four CI jobs must pass: `--no-default-features`, `--features serde`, `--features observer-tracing`, `--features observer-metrics`. A missing matrix entry means the default feature set was tested but the flag-gated code was not.
- [ ] **TracingObserver in parallel region:** Every `TracingObserver` hook that fires inside `par_iter()` uses `event!()` not `span.enter()`. Review by grepping `TracingObserver` impl for `.enter()` calls.
- [ ] **Reporter<U> and GaObserver<U> coexistence documented:** The library ships with both. A user attaching `LogObserver` (as `GaObserver`) while also having a `SimpleReporter` (as `Reporter`) will see duplicate per-generation output. The README or `with_reporter`/`with_observer` doc must warn about this.
- [ ] **`tracing-log` bridge test passes:** CI runs a test that initializes `LogTracer::init()` + attaches `TracingObserver` and runs 10 generations without stack overflow.

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Arc contention discovered post-release | MEDIUM | Add `Arc::clone` before `par_iter()` in a patch release; add benchmark regression test to CI |
| Duplicate log output from both `log!()` and `LogObserver` | LOW | Remove old `log!()` call sites in a patch release; bump minor version to signal migration complete |
| `Span::enter()` panic in TracingObserver | HIGH | Requires replacing all `span.enter()` calls inside parallel regions with `in_scope()`; needs a TracingObserver minor version bump |
| Metrics contention performance regression | MEDIUM | Move `counter!` calls from per-chromosome to per-generation hooks; add benchmark gate to CI |
| `tracing-log` infinite recursion stack overflow | HIGH | TracingObserver must switch from `log::*` to `tracing::event!` exclusively; breaking change if public API exposed log calls |
| Missing `Send + Sync` on `GaObserver` discovered after release | HIGH | Adding `Send + Sync` supertrait is a breaking change for existing `impl GaObserver for T` where T is not Send/Sync; requires major version bump |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Arc clone inside `par_iter()` (Pitfall 1) | Phase 1 — GaObserver trait design | Criterion bench: observer=None vs observer=no-op shows <1% overhead difference |
| `Span::enter()` in rayon closures (Pitfall 2) | Phase 3 — TracingObserver | Grep `TracingObserver` impl for `.enter()` calls; run integration test with thread tracking subscriber |
| Metrics counter contention in hot path (Pitfall 3) | Phase 5 — MetricsObserver | Benchmark with MetricsObserver vs no observer at pop=1000, threads=8 |
| LogObserver breaks env_logger init (Pitfall 4) | Phase 2 — LogObserver | Test: GA with `LogLevel::Off` + LogObserver = zero log lines |
| Duplicate output during log migration (Pitfall 5) | Phase 2 — LogObserver | Grep for `info!\|debug!\|trace!` in migrated modules after Phase 2 — only LogObserver file should remain |
| `tracing-log` infinite recursion (Pitfall 6) | Phase 3 — TracingObserver | CI test: LogTracer::init() + TracingObserver + 10 generations = no stack overflow |
| Default no-op methods break vtable on upgrade (Pitfall 7) | Phase 1 — trait design | Lock down initial hook set; treat additions as minor semver bumps; document in CHANGELOG |
| Observer called before population initialized (Pitfall 8) | Phase 1 — trait design | Hook lifecycle documented: `on_run_start` receives no population reference |
| Island/NSGA-II sub-traits repeat Arc contention (Pitfall 9) | Phase 4 — Island/NSGA-II observers | Same benchmark gate as Pitfall 1; apply to `par_iter_mut()` in island evolve path |
| Feature flag re-export without cfg guard (Pitfall 10) | Phase 3 (TracingObserver) + Phase 5 (MetricsObserver) | CI matrix: `--no-default-features` must compile without tracing/metrics types visible |
| `CompositeObserver` missing `Send + Sync` (Pitfall 11) | Phase 1 — GaObserver trait design | Compile test: `CompositeObserver` assigned to `IslandGa<U>` field |
| Missed serde-gated `log::warn!` in checkpoint code (Pitfall 12) | Phase 2 — LogObserver migration | `cargo build --features serde` in CI; audit checklist before migration starts |

---

## Sources

- Codebase analysis: `src/ga.rs` (lines 554–1097), `src/island/mod.rs` (lines 404–464), `src/reporter/mod.rs`, `Cargo.toml` — HIGH confidence (direct inspection)
- `tracing-log` crate official docs on `LogTracer` and infinite recursion warning: https://docs.rs/tracing-log/latest/tracing_log/ — HIGH confidence
- Rust SemVer guidelines on adding default methods to traits: https://doc.rust-lang.org/cargo/reference/semver.html — HIGH confidence
- Effective Rust on feature flag hygiene and additive feature requirement: https://effective-rust.com/features.html — HIGH confidence
- Rayon `par_iter` closure Send/Sync constraints and task-stealing model: https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html — HIGH confidence
- `tracing` crate `Span::enter()` threading contract (must drop on same thread): https://docs.rs/tracing — HIGH confidence
- `Arc` atomic reference count contention under parallel write workloads: Rust standard library documentation — HIGH confidence
- `metrics` crate facade global registry implementation (RwLock/DashMap internals): MEDIUM confidence — based on `metrics-rs` source; validate with benchmark before asserting in docs

---
*Pitfalls research for: GaObserver observability system, genetic_algorithms Rust library*
*Researched: 2026-03-25*
