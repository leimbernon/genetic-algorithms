---
phase: 61-performance-clone-reduction-parallel-survivor
plan: "03"
subsystem: observer
tags: [performance, observer, breaking-change, clone-reduction, v3]
dependency_graph:
  requires: []
  provides:
    - GaObserver::on_new_best(&U) signature (breaking change)
    - CompositeObserver fan-out clone elimination
    - D-01 crossover conditional-clone audit documentation
  affects:
    - All GaObserver implementors (user-facing breaking change in v3.0.0)
    - All engine on_new_best call sites (ga.rs, cma, gp, hill_climb, permutate, pso)
tech_stack:
  added: []
  patterns:
    - "on_new_best(&self, generation: usize, best: &U) — reference semantics, zero-copy"
    - "CompositeObserver fan-out: pass reference directly, no clone in loop"
key_files:
  created: []
  modified:
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
decisions:
  - "D-03/D-04/D-05 applied: GaObserver::on_new_best changed from owned U to &U — breaking change accepted for v3.0.0"
  - "D-01 relaxed: crossover fallback clones at lines 2687-2688 confirmed conditional (inside else branch), no code change needed"
  - "D-02 respected: multi-parent fallback clone at line 2685 and selection-output clone at line 2862 left untouched"
  - "Rule 3 auto-fix: all engine call sites (cma, gp, hill_climb, permutate, pso) updated to compile against new &U signature"
metrics:
  duration: "~20 minutes"
  completed: "2026-06-08"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 16
---

# Phase 61 Plan 03: Observer Signature Change & Crossover Clone Audit Summary

GaObserver::on_new_best signature changed from owned U to &U across trait, all built-in impls, all engine call sites, and all test impls; CompositeObserver fan-out clone eliminated; D-01 crossover conditional-clone audit confirmed with grep gates.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | GaObserver trait + built-in impls on_new_best(U) → on_new_best(&U) | a00269f | 5 observer files |
| 2 | ga.rs call site + crossover fallback D-01 audit + other engine call sites | a986fb9 | ga.rs + 5 engine files |
| 3 | Update test observer impls — SpyObserver + engine test observers | 221a3c9 | 5 test files |

## What Was Built

### Task 1: Trait signature change (D-03/D-04/D-05)

Changed `GaObserver<U>::on_new_best(&self, _generation: usize, _best: U)` to `on_new_best(&self, _generation: usize, _best: &U)` in all five observer files:

- `src/observe/observer/mod.rs`: Default method body updated
- `src/observe/observer/log.rs`: LogObserver parameter updated (no-op body unchanged)
- `src/observe/observer/composite.rs`: CompositeObserver fan-out clone REMOVED — changed from `obs.on_new_best(generation, best.clone())` to `obs.on_new_best(generation, best)`. This eliminates N-1 clones per new-best event in fan-out scenarios.
- `src/observe/observer/tracing_observer.rs`: TracingObserver parameter updated; body calls `best.fitness()` which works via `&U`
- `src/observe/observer/metrics_observer.rs`: MetricsObserver parameter updated; body ignores `best` (increments counter)

### Task 2: ga.rs call site + D-01 audit + other engine call sites

**ga.rs on_new_best call site (line 2156):**
Changed from:
```rust
self.notify(|obs| obs.on_new_best(i, self.population.best_chromosome.clone()));
```
to:
```rust
self.notify(|obs| obs.on_new_best(i, &self.population.best_chromosome));
```
Eliminates one per-generation chromosome clone on each new-best event.

**D-01 crossover conditional-clone audit (relaxed per user decision):**

Audit confirmed no code change required. All three grep gates pass:

```
# parent_1.clone() occurrences:
grep -cE "parent_1\.clone\(\)" src/engines/ga.rs → 3 matches
  Line 2681: comment text (not code — grep matches the word in the comment)
  Line 2685: child_2 fallback inside IF branch (multi-parent, out of scope D-02)
  Line 2687: child_1 = parent_1.clone() inside ELSE branch (D-01 conditional, acceptable)

# parent_2.clone() occurrences:
grep -cE "parent_2\.clone\(\)" src/engines/ga.rs → 1 match
  Line 2688: child_2 = parent_2.clone() inside ELSE branch (D-01 conditional, acceptable)

# Unconditional clone check (lines 2565–2646, between extraction and if-guard):
sed -n '2565,2646p' src/engines/ga.rs | grep -cE "parent_(1|2)\.clone\(\)" → 0
```

Note on grep count discrepancy: The plan expected `parent_1.clone()` count of 2, but the actual grep result is 3. The third match is the comment at line 2681 (`// parent_1.clone() (D-04 / Pitfall 1)`) which contains the pattern text. The actual CODE occurrences are 2 (lines 2685 and 2687), matching the plan's intent. The unconditional-clone check (0 matches in the extraction-to-guard span) is the authoritative gate.

**The clones at lines 2687–2688 are visually confirmed to be inside the `else { ... }` block** of `if crossover_probability <= effective_crossover_prob { ... } else { child_1 = parent_1.clone(); child_2 = parent_2.clone(); }`. D-01 conditional-clone requirement satisfied.

**Fitness-fn comment at line 2751 confirmed present:**
```rust
// default no-op fitness fn). Children from parent.clone() (the else branch above)
// already carry the correct fitness fn from their parent.
```

**Selection-output clone at line 2862 preserved per D-02:**
```rust
indices.iter().map(|&i| chromosomes[i].clone()).collect()
```

**Rule 3 auto-fixes (other engine call sites):**
All five additional engines had call sites that required updating to compile against the new `&U` signature. These were updated without behavioral change:
- `cma/engine.rs`: 2 sites (`&best`)
- `gp/engine.rs`: 1 site (`&best`)
- `hill_climb/engine.rs`: 1 site (`&next`)
- `permutate/engine.rs`: 1 site (`candidate` — already `&U` from `iter()`, no extra `&` needed per clippy)
- `pso/engine.rs`: 2 sites (`&best`)

### Task 3: Test observer impls

Updated all test `on_new_best` impls in both the worktree and main repo `tests/`:
- `tests/observe/observer/test_observer.rs`: SpyObserver `BinaryChromosome` → `&BinaryChromosome`
- `tests/engines/cma/test_cma.rs`: `RangeChromosome<f64>` → `&RangeChromosome<f64>`
- `tests/engines/permutate/test_permutate.rs`: `U` → `&U`
- `tests/engines/hill_climb/test_hill_climb.rs`: `U` → `&U`
- `tests/engines/pso/test_pso.rs`: `RangeChromosome<f64>` → `&RangeChromosome<f64>`

Confirmed: `tests/gp.rs` has NO `fn on_new_best` override (grep returned 0 matches) — no change required.
Confirmed: `CountingObserver` does NOT implement `on_new_best` — no change required.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Engine call sites not listed in plan files**
- **Found during:** Task 1 / build after observer signature change
- **Issue:** `cargo build --all-features` revealed 8 errors in 5 additional engine files (cma, gp, hill_climb, permutate, pso) not listed in `files_modified` of the plan. All had `on_new_best` call sites passing owned `U` values.
- **Fix:** Updated all call sites to pass `&U` references (or `&best` where a local clone was used). Removed the now-redundant `let best_clone = best.clone()` intermediates.
- **Files modified:** src/engines/cma/engine.rs, src/engines/gp/engine.rs, src/engines/hill_climb/engine.rs, src/engines/permutate/engine.rs, src/engines/pso/engine.rs
- **Commits:** a986fb9

**2. [Rule 3 - Blocking] Test files in worktree not listed in plan**
- **Found during:** Task 3 / cargo test
- **Issue:** The worktree has its own copies of test files (not symlinked to main repo). `cargo test` revealed 6 compile errors in worktree-local test files.
- **Fix:** Updated worktree-local test files alongside the main repo copies already listed in the plan.
- **Files modified:** tests/observe/observer/test_observer.rs, tests/engines/cma/test_cma.rs, tests/engines/permutate/test_permutate.rs, tests/engines/hill_climb/test_hill_climb.rs, tests/engines/pso/test_pso.rs (worktree copies)
- **Commits:** 221a3c9

**3. [Rule 1 - Bug] Clippy: needless borrow in permutate/engine.rs**
- **Found during:** clippy run after Task 2 fixes
- **Issue:** `&candidate` creates `&&U` since `candidate` is already `&U` from `self.candidates.iter()`. Clippy error: "this expression creates a reference which is immediately dereferenced by the compiler"
- **Fix:** Changed `&candidate` to `candidate` in the on_new_best call
- **Commits:** included in a986fb9

## Sub-observer Traits with Owned-U Callbacks

Per plan requirement: searched `src/observe/observer/mod.rs` for sub-observer traits (`IslandGaObserver`, `Nsga2Observer`, `Nsga3Observer`, `MoeaDObserver`, `Spea2Observer`, `SmsEmoaObserver`, `IbeaObserver`).

**Result:** None of these sub-observer traits have any method that takes an owned `U` parameter. Their methods take only scalars, references, or Copy types. No follow-up required.

## Verification Results

All CI gates pass on the worktree branch `worktree-agent-adce45347ab96b1a2`:

| Check | Result |
|-------|--------|
| `cargo build --all-features` | PASS |
| `cargo test` | 1176 passed, 39 ignored |
| `cargo test --features serde` | 1216 passed, 39 ignored |
| `cargo clippy --all-targets -- -D warnings` | No issues found |
| `cargo check --target wasm32-unknown-unknown` | PASS (0 errors) |
| `cargo doc --no-deps` | 0 rustdoc warnings |

## Known Stubs

None — all observer impls are fully functional with the new signature.

## Threat Flags

None — this plan modifies observer callbacks only. No new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- src/observe/observer/mod.rs: FOUND
- src/observe/observer/composite.rs: FOUND
- src/engines/ga.rs: FOUND
- Commit a00269f: FOUND (Task 1)
- Commit a986fb9: FOUND (Task 2)
- Commit 221a3c9: FOUND (Task 3)
