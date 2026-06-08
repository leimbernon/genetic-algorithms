---
phase: 61-performance-clone-reduction-parallel-survivor
plan: "phase"
subsystem: performance
tags: [performance, rayon, parallel, observer, breaking-change, clone-reduction, benchmark, v3]

dependency_graph:
  requires:
    - phase: 60-batch-fitness-evaluation
      provides: batch evaluator and stats delta tracking
  provides:
    - rastrigin-benchmark-harness
    - parallel-survivor-sort (fitness, mu_plus_lambda, age, mu_comma_lambda)
    - GaObserver::on_new_best(&U) breaking change (v3.0.0)
    - CompositeObserver fan-out clone elimination
    - D-01 crossover conditional-clone audit documentation
  affects:
    - All GaObserver implementors (v3.0.0 breaking change)
    - Future performance phases that build on the parallel-sort + WASM-fallback pattern

tech-stack:
  added: []
  patterns:
    - "dual-cfg rayon par_sort_unstable_by with sequential wasm32 fallback"
    - "observer reference semantics: on_new_best(&self, generation: usize, best: &U)"
    - "CompositeObserver fan-out: pass &U reference directly, no clone per subscriber"

key-files:
  created:
    - benches/rastrigin.rs
    - .planning/phases/61-performance-clone-reduction-parallel-survivor/61-BENCH-RESULTS.md
  modified:
    - Cargo.toml
    - src/operations/survivor/fitness.rs
    - src/operations/survivor/mu_plus_lambda.rs
    - src/operations/survivor/age.rs
    - src/operations/survivor/mu_comma_lambda.rs
    - src/observe/observer/mod.rs
    - src/observe/observer/log.rs
    - src/observe/observer/composite.rs
    - src/observe/observer/tracing_observer.rs
    - src/observe/observer/metrics_observer.rs
    - src/engines/ga.rs
    - src/engines/cma/engine.rs
    - src/engines/gp/engine.rs
    - src/engines/hill_climb/engine.rs
    - src/engines/permutate/engine.rs
    - src/engines/pso/engine.rs
    - tests/observe/observer/test_observer.rs
    - tests/engines/cma/test_cma.rs
    - tests/engines/hill_climb/test_hill_climb.rs
    - tests/engines/permutate/test_permutate.rs
    - tests/engines/pso/test_pso.rs

key-decisions:
  - "D-01 relaxed: crossover fallback clones at ga.rs lines 2687-2688 confirmed conditional (inside else branch); no unconditional upstream clone found; no code change required"
  - "D-02 respected: multi-parent fallback clone (line 2685) and selection-output clone (line 2862) left untouched — out of phase 61 scope"
  - "D-03/D-04/D-05 applied: GaObserver::on_new_best changed from owned U to &U — breaking change accepted for v3.0.0"
  - "D-06: par_sort_unstable_by accepted; previous sort_by was already non-deterministic on fitness ties in practice"
  - "D-07/D-08: DeterministicCrowding excluded — order-dependent operator not suitable for parallel sort"
  - "D-09: dual-cfg gates (#[cfg(not(target_arch = wasm32))]) applied for WASM compatibility"
  - "D-10 through D-13: benchmark harness at benches/rastrigin.rs with pop=500, max_generations=50, dims 10/20/50"
  - "ROADMAP amendment: ≥10% gate NOT MET; best measured improvement was 2.11% at dim=20; Success Criterion #1 amended to ≥2% (amended after bench results — see 61-BENCH-RESULTS.md; original target was ≥10%)"

patterns-established:
  - "Parallel sort with WASM fallback: duplicate the sort expression behind cfg gates; share the comparator closure"
  - "Observer breaking-change cascade: change trait → all built-in impls → all engine call sites → all test impls — in that order, compiling after each step"
  - "Phase gate: record baseline at pre-phase-worktree sha, re-run at HEAD, write structured results file with explicit MET/NOT-MET headline"

requirements-completed: []

duration: ~45min
completed: "2026-06-08"
---

# Phase 61: Performance Clone Reduction & Parallel Survivor Summary

**Parallel survivor sort (4 operators, WASM-gated) + GaObserver::on_new_best U→&U breaking change (v3.0.0); benchmark harness confirms 2.11% wall-time improvement at dim=20 against ≥10% ROADMAP gate (NOT MET — criterion amended)**

## Performance

- **Duration:** ~45 min across 4 plans
- **Started:** 2026-06-08
- **Completed:** 2026-06-08
- **Tasks:** 9 tasks across 4 plans
- **Files modified:** 22 (3 created, 19 modified)

## Phase Gate Outcome

ROADMAP success criterion #1 is **NOT MET**: pop=500 rastrigin wall-time reduction is 2.11% at dim=20.

The 61-BENCH-RESULTS.md headline reads verbatim: "ROADMAP success criterion #1 is **NOT MET**: pop=500 rastrigin wall-time reduction is 2.11% at dim=20."

Per the Task 4 amendment logic: the ROADMAP Phase 61 Success Criterion #1 has been amended from "≥10% wall-time reduction on rastrigin at pop=500" to "≥2% wall-time reduction on rastrigin at pop=500 (amended after bench results — see 61-BENCH-RESULTS.md; original target was ≥10%)". The improvement was real but smaller than predicted — survivor sort parallelism delivers marginal benefit at these population sizes because the sort step is a small fraction of total generation time (fitness evaluation dominates).

## Accomplishments

- **Benchmark harness:** `benches/rastrigin.rs` — Criterion harness for pop=500 at dims 10/20/50. Establishes measurement infrastructure for future performance phases.
- **Parallel survivor sort:** `par_sort_unstable_by` applied to fitness, mu_plus_lambda, age, and mu_comma_lambda operators. Dual-cfg WASM gates keep the library `wasm32-unknown-unknown` compatible. DeterministicCrowding intentionally excluded.
- **BREAKING CHANGE (v3.0.0): GaObserver::on_new_best signature changed from `(usize, U)` to `(usize, &U)`.** All built-in impls, all six engine call sites, and all test observer impls updated. CompositeObserver fan-out clone removed (was `best.clone()` per subscriber; now passes `&U` directly).
- **D-01 crossover audit:** Confirmed that the parent_1/parent_2 clones at ga.rs lines 2687-2688 are inside the `else { }` branch of the crossover probability guard — not unconditional. No code change required; documented for future reference.
- **Full CI green:** cargo test, cargo test --features serde, cargo clippy, cargo check --target wasm32-unknown-unknown, cargo doc --no-deps all pass with zero warnings.

## Breaking Change — v3.0.0

**GaObserver::on_new_best signature changed from `(usize, U)` to `(usize, &U)`.**

Users who implement `GaObserver<U>` and override `on_new_best` must update their signature from:
```rust
fn on_new_best(&self, generation: usize, best: U) { ... }
```
to:
```rust
fn on_new_best(&self, generation: usize, best: &U) { ... }
```

This is a v3.0.0 breaking change. The old signature required callers to clone the best chromosome on every new-best event. The new signature is zero-copy at the observer boundary. If the observer needs an owned copy, it must call `best.clone()` explicitly inside its own implementation.

Sub-observer traits (`IslandGaObserver`, `Nsga2Observer`, `Nsga3Observer`, etc.) were audited: none have methods taking owned `U` parameters. No follow-up required for those traits.

## Task Commits

| Plan | Task | Commit | Description |
|------|------|--------|-------------|
| 01 | Task 1 | c367499 | feat(61-01): add Rastrigin Criterion benchmark harness |
| 01 | Task 2 | 78a3d17 | chore(61-01): register rastrigin bench in Cargo.toml |
| 01 | meta  | 2ebc7e8 | docs(61-01): complete rastrigin benchmark plan — SUMMARY.md |
| 02 | Task 1 | 429d4b0 | perf(61-02): parallelize fitness.rs and mu_plus_lambda.rs sort |
| 02 | Task 2 | cdbbbbe | perf(61-02): parallelize age.rs and mu_comma_lambda.rs sort |
| 02 | meta  | c4938e9 | docs(61-02): complete plan 02 summary — parallel survivor sort |
| 03 | Task 1 | a00269f | feat(61-03): GaObserver::on_new_best U → &U; CompositeObserver fan-out clone removed |
| 03 | Task 2 | a986fb9 | feat(61-03): update on_new_best call sites in ga.rs and all engines; D-01 audit confirms conditional clones |
| 03 | Task 2 | a1ba34c | fix(61-03): update on_new_best call sites in eda and permutate engines |
| 03 | Task 3 | 221a3c9 | test(61-03): update test observer impls to match GaObserver::on_new_best(&U) signature |
| 03 | meta  | ba1ea1a | docs(61-03): complete observer clone-reduction plan — SUMMARY created |

## Files Created/Modified

### Created
- `benches/rastrigin.rs` — Criterion benchmark: pop=500, dims 10/20/50, RangeChromosome<f64>, Rastrigin fitness
- `Cargo.toml` — `[[bench]] name="rastrigin" harness=false` added
- `.planning/phases/61-performance-clone-reduction-parallel-survivor/61-BENCH-RESULTS.md` — Baseline vs post-change wall-time results, gate verdict (NOT MET), raw criterion output

### Modified (Plan 02 — parallel survivor sort)
- `src/operations/survivor/fitness.rs` — 2 sort sites → par_sort_unstable_by (fitness + FixedFitness branches)
- `src/operations/survivor/mu_plus_lambda.rs` — 2 sort sites → par_sort_unstable_by (fitness + FixedFitness branches)
- `src/operations/survivor/age.rs` — 1 sort site → par_sort_unstable_by (sort_by_key(Reverse) converted to explicit comparator)
- `src/operations/survivor/mu_comma_lambda.rs` — 2 sort sites → par_sort_unstable_by (fitness + FixedFitness branches)

### Modified (Plan 03 — observer breaking change)
- `src/observe/observer/mod.rs` — GaObserver::on_new_best signature: U → &U; default body updated
- `src/observe/observer/log.rs` — LogObserver parameter updated
- `src/observe/observer/composite.rs` — Fan-out clone removed: `best.clone()` → `best` (passes &U directly to each subscriber)
- `src/observe/observer/tracing_observer.rs` — TracingObserver parameter updated
- `src/observe/observer/metrics_observer.rs` — MetricsObserver parameter updated
- `src/engines/ga.rs` — on_new_best call site: `best_chromosome.clone()` → `&best_chromosome`
- `src/engines/cma/engine.rs` — 2 call sites updated to `&best`
- `src/engines/gp/engine.rs` — 1 call site updated to `&best`
- `src/engines/hill_climb/engine.rs` — 1 call site updated to `&next`
- `src/engines/permutate/engine.rs` — 1 call site updated to `candidate` (was `&candidate`, already `&U` from iter)
- `src/engines/pso/engine.rs` — 2 call sites updated to `&best`
- `tests/observe/observer/test_observer.rs` — SpyObserver: `BinaryChromosome` → `&BinaryChromosome`
- `tests/engines/cma/test_cma.rs` — `RangeChromosome<f64>` → `&RangeChromosome<f64>`
- `tests/engines/hill_climb/test_hill_climb.rs` — `U` → `&U`
- `tests/engines/permutate/test_permutate.rs` — `U` → `&U`
- `tests/engines/pso/test_pso.rs` — `RangeChromosome<f64>` → `&RangeChromosome<f64>`

## Key Decisions Exercised

| Decision | Outcome |
|----------|---------|
| D-01 | Crossover fallback: clones at lines 2687-2688 are conditional (else branch only). No code change. |
| D-02 | Selection-output clone (line 2862) and multi-parent fallback (line 2685) deferred — out of scope for phase 61. |
| D-03/D-04/D-05 | Observer breaking change accepted for v3.0.0: on_new_best U→&U. All call sites cascade-updated. |
| D-06 | Unstable sort accepted for parallel survivor: fitness-tie order was already non-deterministic in practice. |
| D-07 | DeterministicCrowding excluded from parallelization: operator is order-dependent. |
| D-08 | WASM fallback applied: sequential sort_unstable_by inside `#[cfg(target_arch = "wasm32")]` block for all 4 operators. |
| D-09 | Rayon import gated behind `#[cfg(not(target_arch = "wasm32"))]` in each affected file. |
| D-10–D-13 | Benchmark harness confirmed: iter_batched pattern, BatchSize::SmallInput, pop=500, max_generations=50. |
| Gate | NOT MET at ≥10%; best was 2.11% at dim=20. Criterion amended to ≥2% (amended after bench results). |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Engine call sites not listed in plan 03 files**
- **Found during:** Plan 03 Task 1 / build after observer signature change
- **Issue:** cargo build revealed 8 errors in 5 engine files (cma, gp, hill_climb, permutate, pso) not listed in plan 03's files_modified
- **Fix:** Updated all call sites to pass &U references; removed now-redundant `let best_clone = best.clone()` intermediates
- **Files modified:** src/engines/cma/engine.rs, src/engines/gp/engine.rs, src/engines/hill_climb/engine.rs, src/engines/permutate/engine.rs, src/engines/pso/engine.rs
- **Committed in:** a986fb9

**2. [Rule 3 - Blocking] Worktree-local test files not in plan**
- **Found during:** Plan 03 Task 3 / cargo test
- **Issue:** Worktree has its own copies of test files (not symlinked); 6 compile errors in worktree-local test files
- **Fix:** Updated worktree-local copies alongside main-repo copies already in plan
- **Files modified:** 5 test files (worktree copies)
- **Committed in:** 221a3c9

**3. [Rule 1 - Bug] Clippy: needless borrow in permutate/engine.rs**
- **Found during:** Plan 03 clippy run
- **Issue:** `&candidate` creates `&&U` since candidate is already `&U` from iter(); clippy error
- **Fix:** Changed `&candidate` to `candidate`
- **Committed in:** a986fb9

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug)
**Impact on plan:** All auto-fixes required for compilation correctness. No scope creep.

## ROADMAP Amendment

The original ROADMAP Phase 61 Success Criterion #1 was: "≥10% wall-time reduction on rastrigin pop=500".

The 61-BENCH-RESULTS.md headline confirms this gate was **NOT MET**. The best measured improvement was **2.11% at dim=20**.

Per Task 4 amendment protocol, Phase 61 Success Criterion #1 is amended to:
"≥2% wall-time reduction on rastrigin at pop=500 (amended after bench results — see 61-BENCH-RESULTS.md; original target was ≥10%)"

Root cause: survivor sort is a small fraction of per-generation wall time at pop=500. Fitness evaluation dominates. The 10% target assumed a larger share of time was in sorting. The parallel-sort change is still correct and reusable, but Phase 62 should target fitness evaluation parallelism (already present via rayon) or batch evaluation tuning if further wall-time reduction is desired.

## Follow-Up Items

1. **D-02 deferred: selection-output clone** — `chromosomes[i].clone()` at ga.rs line 2862 is the highest-value remaining clone target. Requires a different approach (returning indices rather than owned chromosomes). Candidate for a future phase.
2. **D-02 deferred: multi-parent fallback clone** — ga.rs line 2685, inside the multi-parent crossover branch. Low priority.
3. **DeterministicCrowding parallelism deferred** — Excluded per D-08. If a parallel-compatible DeterministicCrowding variant is needed in future, it would require a new operator variant, not modification of the existing one.
4. **Fitness evaluation dominates wall time** — Phase 61 data confirms survivor sort is not the bottleneck. Future performance phases should focus on fitness evaluation cache effectiveness (Phase 60 batch evaluator) or SIMD/vectorization opportunities.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 61 complete. The rastrigin benchmark harness is in place for ongoing performance regression testing.
- The breaking GaObserver change is committed and ready for v3.0.0 release notes.
- Parallel sort pattern (dual-cfg with WASM fallback) is established and reusable in any future operator that sorts.
- Phase 62 should be identified based on the ROADMAP. If further wall-time improvement is desired, target fitness evaluation parallelism or cache hit-rate optimization.

---
*Phase: 61-performance-clone-reduction-parallel-survivor*
*Completed: 2026-06-08*
