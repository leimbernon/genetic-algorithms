---
phase: 45-memetic-algorithm-framework
plan: 02
subsystem: ga-engine
tags: [memetic, local-search, hill-climbing, GA-integration, builder-pattern]

# Dependency graph
requires:
  - phase: 45-01-local-search-operator-foundation
    provides: LocalSearchOperator trait, LocalSearch enum, HillClimbingConfig, application strategy and mode enums
provides:
  - LocalSearchConfiguration struct (method, strategy, mode, hill_climbing fields)
  - LocalSearchConfig trait in ConfigurationT supertrait
  - GaConfiguration.local_search_configuration field
  - Ga::with_local_search() builder method
  - Ga::with_local_search_configuration() trait method
  - Generation loop local search refinement block (all 4 strategies, both modes)
  - 6 integration tests covering all strategies, modes, and zero-overhead path
affects: [45-03, 45-04, 45-CONTEXT]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Enum-based local search storage in GaConfiguration (not Box<dyn>) due to generic improve() method"
    - "Extract-process-reinsert rayon pattern for parallel local search on offspring subset"
    - "DNA snapshot restore for Baldwinian mode"
    - "select_nth_unstable_by for BestN strategy (partial sort for top-k)"
    - "cfg-gated par_iter_mut/iter dispatch for WASM compatibility"

key-files:
  created:
    - tests/engines/local_search.rs
  modified:
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/traits.rs
    - src/engines/ga.rs
    - tests/test_engines.rs

key-decisions:
  - "LocalSearch stored as LocalSearch enum in LocalSearchConfiguration.method (not Box<dyn LocalSearchOperator>) because generic improve() makes the trait not dyn-compatible"
  - "Extract-process-reinsert pattern for parallel local search (clone candidates, par_iter_mut, move back) avoids borrow checker issue with &mut Vec<U> captures in rayon"
  - "Baldwinian mode saves DNA snapshot before parallel block, restores original DNA after while keeping improved fitness"
  - "No separate Ga struct field for local search — the enum lives in config, matching existing operator enum pattern"
  - "hill_climbing: HillClimbingConfig field in LocalSearchConfiguration provides custom params for the operator"

patterns-established:
  - "Pattern: sub-trait + builder method follows existing SelectionConfig/ExtensionConfig pattern"
  - "Pattern: GaConfiguration field with Option<LocalSearchConfiguration> for zero-overhead when None"
  - "Pattern: Application strategy dispatch matches operator dispatch pattern (enum match)"

requirements-completed:
  - MEM-01

# Metrics
duration: 22min
completed: 2026-05-14
---

# Phase 45 Plan 02: GA Engine Integration Summary

**LocalSearchConfiguration struct, LocalSearchConfig builder trait, Ga builder method, and generation loop refinement block with all 4 application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations) and both modes (Lamarckian, Baldwinian)**

## Performance

- **Duration:** 22 min
- **Started:** 2026-05-14T10:52:00Z (approx)
- **Completed:** 2026-05-14T11:14:00Z (approx)
- **Tasks:** 4
- **Files modified:** 6 (1 created, 5 modified)

## Accomplishments
- LocalSearchConfiguration struct with method, application_strategy, mode, and hill_climbing defaults
- LocalSearchConfig trait added to ConfigurationT supertrait (follows existing sub-trait pattern)
- Ga::with_local_search(LocalSearch) builder method sets method in config (not Box<dyn>)
- GaConfiguration.local_search_configuration: Option<LocalSearchConfiguration> for zero-overhead when None
- Generation loop block between constraint penalty and population merge with:
  - AllOffspring: refines all offspring every generation
  - BestN: partial sort via select_nth_unstable_by for top-k efficiency
  - Probabilistic: random-thread RNG filter at configurable probability
  - EveryNGenerations: interval-based dispatch (interval=0 means never)
  - Lamarckian: DNA and fitness both updated in-place
  - Baldwinian: DNA snapshot before parallel block, restore original DNA after, keep improved fitness
  - rayon par_iter_mut via extract-process-reinsert pattern
  - WASM sequential fallback via cfg-gated iter()
- 6 integration tests pass covering all strategies, both modes, and no-local-search baseline
- Full test suite: 984 passed, 25 ignored, 0 failures - no regressions

## Task Commits

Each task was committed atomically:

1. **Task 1+2: LocalSearchConfiguration struct + builder trait + Ga builder method** - `706f3db` (feat)
2. **Task 3: Generation loop integration** - `06c2427` (feat)
3. **Task 4: Integration tests** - `6fb72ab` (test)

**Plan metadata:** (pending docs commit)

## Files Created/Modified
- `src/configuration.rs` - Added LocalSearchConfiguration struct, field on GaConfiguration, import + default + LocalSearchConfig impl
- `src/traits/configuration.rs` - Added LocalSearchConfig trait, import, and ConfigurationT supertrait update
- `src/traits.rs` - Added LocalSearchConfig to re-exports
- `src/engines/ga.rs` - Added imports, with_local_search() builder, LocalSearchConfig impl for Ga, generation loop block
- `tests/test_engines.rs` - Added mod local_search to engine test hierarchy
- `tests/engines/local_search.rs` - Created new file with 6 integration tests

## Decisions Made
- **LocalSearch stored as enum in config, not Box<dyn>**: The generic improve() method makes LocalSearchOperator not dyn-compatible. Matches existing enum pattern (Selection, Crossover, etc.).
- **Extract-process-reinsert for parallel local search**: Instead of capturing &mut Vec<U> in rayon closures (which fails because &mut Vec<U> is !Sync), clone candidates to a separate Vec, process with par_iter_mut(), then move results back.
- **No separate Ga struct field**: The operator lives in LocalSearchConfiguration.method, accessed via self.configuration.local_search_configuration.as_ref()?.method. No build-time initialization needed since the enum is ready immediately.
- **Baldwinian DNA snapshot**: Original DNA is saved before the parallel block, then restored after. The improved fitness from local search is kept while DNA reverts to its original state.

## Deviations from Plan

**Plan deviation applied by orchestrator: LocalSearchOperator is not dyn-compatible**

This plan was executed with the critical deviation described in the prompt instructions:
- **Do NOT use `Box<dyn LocalSearchOperator<U>>`** - won't compile (E0038)
- **Do NOT add `local_search: Option<Box<dyn LocalSearchOperator<U>>>`** to Ga struct
- **Store `LocalSearch` enum directly in config** — matches existing operator pattern

All Task 2 and Task 3 code was adapted accordingly.

### Auto-fixed Issues

**1. [Rule 3 - Blocking] LocalSearchConfiguration imported from wrong module**
- **Found during:** Task 2 (import setup)
- **Issue:** Plan specified `use crate::operations::local_search::LocalSearchConfiguration` but LocalSearchConfiguration is defined in `src/configuration.rs`, not in `src/operations/local_search.rs`
- **Fix:** Changed import to `use crate::configuration::LocalSearchConfiguration`
- **Files modified:** src/engines/ga.rs
- **Verification:** Compiles cleanly
- **Committed in:** 706f3db (Tasks 1+2 commit)

**2. [Rule 3 - Blocking] Rayon borrow checker issue with &mut Vec<U> capture in for_each**
- **Found during:** Task 3 (generation loop implementation)
- **Issue:** `candidates.iter().map(|&idx| &mut offspring[idx]).collect::<Vec<&mut U>>()` fails because the FnMut closure captures `offspring` by mutable reference and tries to return a &mut U that escapes the closure body
- **Fix:** Used extract-process-reinsert pattern: clone candidates to `selected: Vec<U>`, process with `selected.par_iter_mut().for_each()`, then move results back into offspring
- **Files modified:** src/engines/ga.rs
- **Verification:** Compiles and tests pass (adds performance cost of cloning, but correct)
- **Committed in:** 06c2427 (Task 3 commit)

**3. [Rule 1 - Bug] best_chromosome is U, not Option<U>**
- **Found during:** Task 4 (integration test compilation)
- **Issue:** Test code used `population.best_chromosome.as_ref().expect(...)` but best_chromosome is a plain `U` field on Population, not `Option<U>`
- **Fix:** Changed to `&population.best_chromosome` and removed `.as_ref().expect()`
- **Files modified:** tests/engines/local_search.rs
- **Verification:** Tests compile and pass
- **Committed in:** 6fb72ab (Task 4 commit)

---

**Total deviations:** 3 auto-fixed (1 Rule 1 bug, 2 Rule 3 blocking) + 1 orchestrator-applied correction (not-dyn-compatible)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep. Plan intent preserved.

## Issues Encountered
- Rayon's `for_each` requires `Send + Sync` on the closure, but `&mut Vec<U>` is `!Sync`. The extract-process-reinsert pattern (`par_iter_mut` on a cloned Vec) is the correct Rust idiom for parallel mutation of selected elements.
- `best_chromosome` is a plain `U` field, not `Option<U>`, contrary to some test patterns from other codebases.

## Threat Surface Scan
No new threat flags. All threats from the plan's threat register are mitigated:
- T-45-04 (Tampering - BestN index selection): Mitigated via `n.min(indices.len())` and `k.saturating_sub(1)`
- T-45-05 (Denial - Parallel mutation): Accepted - each rayon task gets a distinct offspring from the candidates Vec
- T-45-06 (Denial - Baldwinian DNA restore): Mitigated via DNA snapshot captured BEFORE parallel block
- T-45-07 (Tampering - Probabilistic zero probability): Accepted - empty candidates Vec is correct zero-overhead behavior

## Stub Check
No stubs found. All functionality is fully wired.

## WASM Compatibility
- rayon usage is cfg-gated with `#[cfg(not(target_arch = "wasm32"))]`
- WASM fallback uses sequential `iter().for_each()` 
- No `std::time::Instant` usage in the added block
- `rand::thread_rng()` is used for Probabilistic strategy (consistent with existing codebase)

## Next Phase Readiness
- Local search integration into Ga engine is complete
- Ready for Plan 45-03 (advanced local search operators) or Plan 45-03 (integration tests / benchmarks)
- The `hill_climbing` field in LocalSearchConfiguration allows downstream customization of HillClimbing parameters

## Self-Check: PASSED

All verification criteria met:
- `cargo check --lib` passes (0 errors, 0 warnings)
- `cargo test` passes (984/984, 25 ignored - no regressions)
- `cargo test --test test_engines` passes (333/333, 2 ignored)
- GaConfiguration.local_search_configuration field: found in configuration.rs (line 332)
- with_local_search() builder method: found in ga.rs (line 927)
- LocalSearchConfig trait: found in traits/configuration.rs (line 117), in ConfigurationT supertrait (line 139)
- Local search block between constraint penalty and add_chromosomes: line 1439-1552 (3a-) before line 1553 (3-)
- All 6 integration tests pass (test_local_search_all_offspring, best_n, probabilistic, every_n_generations, baldwinian, not_configured)
- All 3 commits found in git history (706f3db, 06c2427, 6fb72ab)

---
*Phase: 45-memetic-algorithm-framework*
*Completed: 2026-05-14*
