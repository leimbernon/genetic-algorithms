# Phase 18: Observer API Polish - Research

**Researched:** 2026-03-28
**Domain:** Rust observer trait system, ga.rs execution loop, crate public API surface
**Confidence:** HIGH

## Summary

Phase 18 closes five concrete tech-debt items discovered during the v2.2.0 milestone audit. Every item has been precisely located in the codebase — this is surgical work, not exploratory. The scope is four files: `src/observer/tracing_observer.rs` (two empty impl blocks), `src/ga.rs` (hook reordering and Duration timing), and `src/lib.rs` (three `pub use` lines). No new abstractions are introduced.

The most structurally significant change is the ga.rs hook ordering fix. Moving the extension block before `on_generation_end` requires understanding which variables are needed. The audit points at lines 964–984. `on_generation_end` receives `notify_stats` (a clone from `self.stats.last()`), which is computed after `gen_stats` is pushed to `self.stats` at line 920. The extension block at line 968 reads `gen_stats.diversity` but does not write to `self.stats`. Conclusion: the extension block can move before stats collection (or at minimum before `on_generation_end`) without data dependency conflicts.

The Duration::ZERO fix (lines 785–786) is the second structural change. `parent_crossover()` is an opaque rayon `par_iter()` call that performs both crossover and mutation internally. Adding separate timers for mutation and fitness evaluation requires wrapping the call site — a single `Instant::now()` for the combined crossover+mutation block can be split into separate timers only if they have distinct call sites. Since `parent_crossover` is a single function, the pragmatic fix is to add observer-gated timers around the entire call (same pattern as selection at lines 755–762), then fire `on_mutation_complete` and `on_fitness_evaluation_complete` with the elapsed `Duration` rather than `Duration::ZERO`. This produces accurate but combined timing rather than per-operator timing. Fitness evaluation also happens inside `parent_crossover` (chromosomes have their fitness calculated at creation time in the rayon block). The audit noted this constraint was accepted in Phase 13 — the fix produces non-zero values without requiring a deeper refactor.

**Primary recommendation:** Implement all five fixes as a single focused PR. Each fix is independent; there are no ordering dependencies among them except that the ga.rs changes should be verified together to avoid conflicting edits.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| OBS-01 | GaObserver hooks fire in correct order; TerminationCause/ExtensionEvent accessible at crate root | Hook ordering fix in ga.rs + lib.rs re-exports |
| OBS-02 | NoopObserver accessible from crate root | `pub use observer::NoopObserver` in lib.rs |
| LOG-01 | LogObserver reproduces pre-v2.2.0 log output including extension-before-generation ordering | ga.rs extension block must move before on_generation_end |
| TRAC-01 | TracingObserver works for all three GA engine types | Two empty impl blocks in tracing_observer.rs |
| COMP-01 | TracingObserver can be passed to CompositeObserver::add() | AllObserver blanket impl requires IslandGaObserver+Nsga2Observer — TRAC-01 fix enables this |
| COMP-02 | MetricsObserver mutation/fitness-eval histograms record non-zero values | ga.rs Duration::ZERO fix at lines 785–786 |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::time::Instant` | stdlib | High-resolution monotonic timer | Already used for selection/crossover/survivor timing in ga.rs |
| `std::time::Duration` | stdlib | Timer result type passed to observer hooks | Already the hook parameter type |
| `tracing` | 0.1 (existing) | TracingObserver spans | Already a dependency under `observer-tracing` feature |

No new dependencies. All changes use existing stdlib and already-imported crate APIs.

**Installation:** No `cargo add` needed — all dependencies already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure

No structural changes to the module layout. All edits are within existing files.

```
src/
├── observer/
│   ├── mod.rs              # AllObserver blanket impl — no changes needed
│   ├── tracing_observer.rs # ADD: impl IslandGaObserver<U> and impl Nsga2Observer<U>
│   └── metrics_observer.rs # REFERENCE: lines 120-122 are the exact template
├── ga.rs                   # FIX: hook ordering + Duration timing
└── lib.rs                  # ADD: 3 pub use lines for NoopObserver, ExtensionEvent, TerminationCause
```

### Pattern 1: Empty Sub-Trait Impl (TracingObserver AllObserver fix)

**What:** Add two empty impl blocks to `tracing_observer.rs` that satisfy `IslandGaObserver<U>` and `Nsga2Observer<U>`. Both traits have full default no-op implementations, so the impl body is empty.

**When to use:** Any GaObserver implementor that does not need engine-specific hooks but needs to be passable to `CompositeObserver`.

**Template from MetricsObserver (metrics_observer.rs lines 120–122):**
```rust
// Source: src/observer/metrics_observer.rs:120-122
impl<U: ChromosomeT> IslandGaObserver<U> for MetricsObserver {}
impl<U: ChromosomeT> Nsga2Observer<U> for MetricsObserver {}
```

**Applied to TracingObserver:**
```rust
// Add at the end of src/observer/tracing_observer.rs
// These must be inside #[cfg(feature = "observer-tracing")] scope (they are in the same file)
impl<U: ChromosomeT> IslandGaObserver<U> for TracingObserver {}
impl<U: ChromosomeT> Nsga2Observer<U> for TracingObserver {}
```

The import for `IslandGaObserver` and `Nsga2Observer` must be added to the use block at the top of `tracing_observer.rs`. Currently it imports `use crate::observer::{ExtensionEvent, GaObserver};` — add the two sub-traits to this use statement.

Once both impls exist, the `AllObserver<U>` blanket impl in `mod.rs` (lines 139–143) automatically applies, making `Arc::new(TracingObserver::new())` passable to `CompositeObserver::add()`.

### Pattern 2: Observer-Gated Instant (Duration::ZERO fix)

**What:** Gate `Instant::now()` on `self.observer.is_some()` before the `parent_crossover` call, then pass the real elapsed time to the mutation and fitness-eval hooks.

**Existing pattern (selection timing, ga.rs lines 755–763):**
```rust
// Source: src/ga.rs:755-763
let t_sel = if self.observer.is_some() { Some(Instant::now()) } else { None };
let parents = selection::factory(...)?;
if let Some(t) = t_sel {
    self.notify(|obs| obs.on_selection_complete(i, t.elapsed(), parents.len()));
}
```

**Current broken pattern (ga.rs lines 770–787):**
```rust
let t_cx = if self.observer.is_some() { Some(Instant::now()) } else { None };
let mut offspring = parent_crossover(...)?;  // crossover + mutation + fitness eval all inside
if let Some(t) = t_cx {
    let elapsed = t.elapsed();
    // ...
    self.notify(|obs| obs.on_crossover_complete(i, elapsed, offspring_count));
    self.notify(|obs| obs.on_mutation_complete(i, Duration::ZERO, pop_size));       // BUG
    self.notify(|obs| obs.on_fitness_evaluation_complete(i, Duration::ZERO, pop_size)); // BUG
}
```

**Fix:** `parent_crossover` is opaque — crossover, mutation, and fitness evaluation happen internally via `par_iter()`. The pragmatic fix is to pass the same `elapsed` Duration to mutation and fitness-eval hooks instead of `Duration::ZERO`. This is documented in REQUIREMENTS.md as the intended resolution (EXT-01 deferred, per-operator separation is v2.3+ work):

```rust
if let Some(t) = t_cx {
    let elapsed = t.elapsed();
    let offspring_count = offspring.len();
    let pop_size = self.population.chromosomes.len();
    self.notify(|obs| obs.on_crossover_complete(i, elapsed, offspring_count));
    self.notify(|obs| obs.on_mutation_complete(i, elapsed, pop_size));       // was ZERO
    self.notify(|obs| obs.on_fitness_evaluation_complete(i, elapsed, pop_size)); // was ZERO
}
```

This produces a non-zero Duration in all three hooks. Users who need per-operator granularity can use separate tracers; `MetricsObserver` histograms will record real values.

### Pattern 3: Hook Ordering Fix (extension before generation_end)

**What:** Move the extension block before the `on_generation_end` notification in ga.rs.

**Current order (ga.rs lines 960–984):**
1. `r.on_generation_complete(&gen_stats)` (Reporter, line 961)
2. `self.notify(|obs| obs.on_generation_end(&notify_stats))` (line 965)  ← fires first (wrong)
3. Extension block runs (lines 968–1029)
4. `self.notify(|obs| obs.on_extension_triggered(...))` (line 979)  ← fires second (wrong)

**Pre-v2.2.0 order was:** extension → stats → generation_end

**Key data dependency check:**
- `on_generation_end` receives `notify_stats = self.stats.last().cloned().unwrap_or(gen_stats.clone())`
- `gen_stats` is computed at line 918: `GenerationStats::from_fitness_values(i, &fitness_values, ...)`
- `gen_stats` is pushed to `self.stats` at line 920
- The dynamic mutation update (lines 922–958) writes `dynamic_mutation_probability` back to the last stats entry
- The extension block at line 968 reads `gen_stats.diversity` — it does NOT write to `gen_stats` or `self.stats`
- Conclusion: extension block has no data dependency on `on_generation_end`; reordering is safe

**Target order:**
1. Extension block (moves here — reads `gen_stats.diversity` which is already computed at line 918)
2. `on_extension_triggered` (inside the extension block)
3. `r.on_generation_complete(&gen_stats)` (Reporter)
4. `self.notify(|obs| obs.on_generation_end(&notify_stats))`

This restores the LOG-01 guarantee that extension events precede generation-end events in observer output.

**Reporter note:** The legacy `Reporter::on_generation_complete` also moves after extension. This was how the pre-v2.2.0 log output worked (extension log → "Best chromosome" log), so this is the correct historical ordering.

### Pattern 4: lib.rs Re-exports

**What:** Add three `pub use` lines to `src/lib.rs`.

**Current re-export block (lib.rs lines 95–104):**
```rust
pub use observer::LogObserver;
pub use observer::IslandGaObserver;
pub use observer::Nsga2Observer;
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
pub use observer::AllObserver;
pub use observer::CompositeObserver;
pub use observer::GaObserver;
```

**Missing items:**
- `NoopObserver` — defined at `src/observer/mod.rs:95`, public struct, just not re-exported
- `ExtensionEvent` — defined at `src/observer/mod.rs:40`, public struct
- `TerminationCause` — defined in `src/ga.rs` (exact line TBD; module is already `pub mod ga` in lib.rs, and `ga::TerminationCause` is currently reachable via `genetic_algorithms::ga::TerminationCause`)

**Correct additions:**
```rust
pub use observer::NoopObserver;
pub use observer::ExtensionEvent;
pub use ga::TerminationCause;
```

No visibility issues: all three types are already `pub` at their definition sites. `ga` module is already `pub mod ga` in lib.rs. `observer` module is already `pub mod observer`.

### Anti-Patterns to Avoid

- **Do not add methods to AllObserver**: It is a pure marker supertrait — adding methods breaks object safety and would require updates to all 4 existing impl types.
- **Do not change parent_crossover signature**: Adding per-operator timing to the rayon parallel block is EXT-01 deferred work. The Duration fix here is at the call site only.
- **Do not move stats collection**: `gen_stats` must be computed before `on_generation_end` fires. Only the extension block (which reads diversity from `gen_stats`) moves relative to `on_generation_end`.
- **Do not add `log::*` calls in tracing_observer.rs**: TRAC-03 constraint — zero log calls to prevent LogTracer infinite recursion.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-operator mutation timing | Custom timer inside `parent_crossover` rayon block | Pass `t_cx.elapsed()` to mutation hook | Rayon blocks cannot share `Instant` safely across threads; the combined time is good enough for v2.2.0 |
| AllObserver impl for TracingObserver | Custom impl with methods | Two empty impl blocks + blanket impl | AllObserver has no methods; blanket impl auto-applies once sub-trait impls exist |

**Key insight:** All five fixes are one-liner to five-liner changes. The complexity is understanding which change is safe; the implementation itself is minimal.

## Common Pitfalls

### Pitfall 1: Forgetting the Import Addition in tracing_observer.rs
**What goes wrong:** Adding `impl<U: ChromosomeT> IslandGaObserver<U> for TracingObserver {}` without adding `IslandGaObserver` and `Nsga2Observer` to the `use crate::observer::{...}` statement causes a compile error.
**Why it happens:** The file currently only imports `GaObserver` and `ExtensionEvent` from `crate::observer`.
**How to avoid:** Update the use line to `use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer};`.
**Warning signs:** `error[E0412]: cannot find type `IslandGaObserver` in this scope`

### Pitfall 2: Moving Extension Block Before gen_stats Is Computed
**What goes wrong:** If the extension block moves above line 918 (gen_stats computation), `gen_stats.diversity` is not yet available.
**Why it happens:** The extension check reads `gen_stats.diversity < ext_config.diversity_threshold`.
**How to avoid:** Move the extension block to after line 964 (`on_generation_end` call) but before... wait — the correct target is after gen_stats is computed (line 920) but BEFORE the `on_generation_end` notify (line 965). Gen_stats is already ready at that point.
**Warning signs:** Compile error if `gen_stats` is referenced before its binding.

### Pitfall 3: Changing Duration Breaks Existing Tests
**What goes wrong:** Tests in `tests/test_observer.rs` check that mutation/fitness-eval hooks fire N times — they do not assert Duration values. Changing Duration::ZERO to a real value will not break any existing assertion.
**Why it happens:** The SpyObserver ignores the Duration parameter (`_duration: Duration`).
**How to avoid:** No action needed — existing tests are safe.
**Warning signs:** (None — this is a non-issue, documented here to save verification time.)

### Pitfall 4: TerminationCause Already Importable via ga::
**What goes wrong:** A developer might argue re-exporting TerminationCause at crate root is unnecessary since `genetic_algorithms::ga::TerminationCause` already works.
**Why it happens:** `ga` is already a public module. But `on_run_end(cause: TerminationCause, ...)` requires users implementing custom observers to import it. The ergonomic expectation is that all types in hook signatures are available from the crate root.
**How to avoid:** Add the re-export anyway. It's additive, non-breaking, and fixes OBS-01.
**Warning signs:** N/A — include it.

### Pitfall 5: Reporter Also Moves with Extension Block
**What goes wrong:** After moving the extension block, the Reporter's `on_generation_complete` call (line 961) also changes position relative to the extension. Since Reporter is legacy and the pre-v2.2.0 order was extension-first, the Reporter call should also move after extension. Both should move together.
**Why it happens:** Reporter and Observer are dispatched in sequence; both need consistent ordering.
**How to avoid:** Move the entire block (Reporter call + on_generation_end notify) to after the extension block.

## Code Examples

### TracingObserver — Complete Addition
```rust
// Source: src/observer/tracing_observer.rs — add to use statement
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer};

// Source: src/observer/tracing_observer.rs — add after the GaObserver<U> impl block
#[cfg(feature = "observer-tracing")]  // already implied by file gating; included for clarity
impl<U: ChromosomeT> IslandGaObserver<U> for TracingObserver {}

#[cfg(feature = "observer-tracing")]
impl<U: ChromosomeT> Nsga2Observer<U> for TracingObserver {}
```

### ga.rs — Duration Fix (exact lines 785–786)
```rust
// BEFORE:
self.notify(|obs| obs.on_mutation_complete(i, Duration::ZERO, pop_size));
self.notify(|obs| obs.on_fitness_evaluation_complete(i, Duration::ZERO, pop_size));

// AFTER (elapsed is already computed on line 781 as t_cx.elapsed()):
self.notify(|obs| obs.on_mutation_complete(i, elapsed, pop_size));
self.notify(|obs| obs.on_fitness_evaluation_complete(i, elapsed, pop_size));
```

### ga.rs — Hook Ordering Fix (conceptual diff)
```rust
// BEFORE (lines 960-984):
// ... reporter on_generation_complete ...
// ... on_generation_end ...
// if extension needed { extension::factory(...); on_extension_triggered(...); }

// AFTER:
// if extension needed { extension::factory(...); on_extension_triggered(...); }
// ... reporter on_generation_complete ...
// ... on_generation_end ...
```

### lib.rs — Re-exports Addition
```rust
// Add after existing observer re-exports (after line 104):
pub use observer::NoopObserver;
pub use observer::ExtensionEvent;
pub use ga::TerminationCause;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Duration::ZERO` for mutation/fitness-eval hooks | Real elapsed Duration from combined crossover/mutation/fitness block | Phase 18 | MetricsObserver histograms record non-zero values |
| `on_extension_triggered` fires after `on_generation_end` | Extension block runs before stats dispatch | Phase 18 | Restores LOG-01 pre-v2.2.0 temporal ordering |
| `TracingObserver` excluded from `CompositeObserver` | `TracingObserver` satisfies `AllObserver<U>` via blanket impl | Phase 18 | COMP-01 satisfied; user can compose TracingObserver with MetricsObserver |
| `NoopObserver`/`ExtensionEvent`/`TerminationCause` only via internal paths | Re-exported at crate root | Phase 18 | `use genetic_algorithms::NoopObserver;` works |

**Deprecated/outdated:**
- Nothing is deprecated in Phase 18 — all changes are purely additive or in-place fixes.

## Open Questions

1. **Should mutation and fitness-eval hooks receive the same Duration value?**
   - What we know: `parent_crossover` is opaque; all three operators run inside it.
   - What's unclear: Whether users will find it surprising that `on_crossover_complete`, `on_mutation_complete`, and `on_fitness_evaluation_complete` all receive identical Duration values.
   - Recommendation: Document in the hook docstring that these three hooks share the combined `parent_crossover` duration until EXT-01 (per-operator timing) is implemented in v2.3+. Add a `// NOTE: elapsed covers combined crossover+mutation+fitness time (EXT-01)` comment.

2. **Does the extension block regrow logic (lines 986–1029) also move?**
   - What we know: The regrow block runs after `on_extension_triggered` (line 986). It refills the population when extension reduced it.
   - What's unclear: Whether the regrow block should fire before `on_generation_end` (so stats reflect the regrown population) or after.
   - Recommendation: Move the entire extension block including regrow before `on_generation_end`. Stats will then reflect post-extension population state, which is more accurate and matches pre-v2.2.0 behavior.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None — standard Rust test discovery |
| Quick run command | `cargo test --test test_observer -- 2>&1` |
| Full suite command | `cargo test && cargo test --features observer-tracing && cargo test --features observer-metrics` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRAC-01 | `Arc::new(TracingObserver::new())` can be added to `CompositeObserver` and used with `Ga<U>`, `IslandGa<U>`, `Nsga2Ga<U>` without compile error | compile/smoke | `cargo test --features observer-tracing --test test_tracing_observer` | ❌ Wave 0 |
| COMP-01 | CompositeObserver with TracingObserver inner runs full GA without panic | integration | `cargo test --features observer-tracing --test test_composite_observer` | Partial — existing composite tests lack TracingObserver |
| COMP-02 | `on_mutation_complete` and `on_fitness_evaluation_complete` receive `Duration > Duration::ZERO` | unit | `cargo test --test test_observer -- test_mutation_timing_nonzero` | ❌ Wave 0 |
| LOG-01 | `on_extension_triggered` fires before `on_generation_end` within the same generation | ordering | `cargo test --test test_observer -- test_extension_fires_before_generation_end` | ❌ Wave 0 |
| OBS-01 | `TerminationCause` importable as `genetic_algorithms::TerminationCause` | compile | `cargo test --test test_observer_reexports` | ❌ Wave 0 |
| OBS-02 | `NoopObserver` importable as `genetic_algorithms::NoopObserver` | compile | `cargo test --test test_observer_reexports` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test test_observer -- 2>&1`
- **Per wave merge:** `cargo test && cargo test --features observer-tracing && cargo test --features observer-metrics && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/test_observer_reexports.rs` — covers OBS-01 (TerminationCause, ExtensionEvent at crate root), OBS-02 (NoopObserver at crate root)
- [ ] `tests/test_observer.rs` — add `test_mutation_timing_nonzero` and `test_extension_fires_before_generation_end` to existing file (file already exists)
- [ ] `tests/test_tracing_observer.rs` — add test for `TracingObserver` inside `CompositeObserver` with all three GA engine types (requires `#[cfg(feature = "observer-tracing")]`)

**Notes on existing test safety:**
- `test_observer_operator_hooks_fire_each_generation` counts firings only — safe, no Duration assertions.
- `test_ga_has_no_direct_log_calls` scans `src/ga.rs` source text — safe, no ordering assertions.
- No existing test will break from any of the five Phase 18 changes.

## Sources

### Primary (HIGH confidence)
- Direct source reading: `src/observer/tracing_observer.rs` — all 12 hooks confirmed, missing IslandGaObserver/Nsga2Observer impls confirmed
- Direct source reading: `src/observer/mod.rs` — AllObserver blanket impl at lines 139–143 confirmed; IslandGaObserver/Nsga2Observer signatures confirmed
- Direct source reading: `src/observer/metrics_observer.rs:120–122` — template for empty sub-trait impls confirmed
- Direct source reading: `src/ga.rs:755–787` — selection timing pattern confirmed; Duration::ZERO at lines 785–786 confirmed
- Direct source reading: `src/ga.rs:960–984` — hook ordering confirmed; on_generation_end at line 965, extension block at 968, on_extension_triggered at 979
- Direct source reading: `src/ga.rs:818–920` — gen_stats computed at line 918, pushed at line 920; extension reads gen_stats.diversity (safe to keep this dependency)
- Direct source reading: `src/lib.rs:95–104` — current re-export block confirmed; NoopObserver/ExtensionEvent/TerminationCause absence confirmed
- Direct source reading: `tests/test_observer.rs` — no Duration assertions, no ordering assertions; reordering is safe

### Secondary (MEDIUM confidence)
- `.planning/v2.2.0-MILESTONE-AUDIT.md` — exact line numbers and fix descriptions verified against actual source

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all changes use stdlib and existing APIs
- Architecture: HIGH — all five changes are read directly from source; no inference required
- Pitfalls: HIGH — derived from actual code structure and test file analysis

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (stable codebase; only changes if ga.rs is modified independently)
