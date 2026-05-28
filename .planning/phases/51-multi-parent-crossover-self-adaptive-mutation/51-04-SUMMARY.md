---
phase: 51-multi-parent-crossover-self-adaptive-mutation
plan: "04"
subsystem: engine-integration
tags:
  - multi-parent-crossover
  - self-adaptive-mutation
  - UNDX
  - SPX
  - PCX
  - ga-engine
  - integration-tests
  - wasm-safe

# Dependency graph
requires:
  - phase: 51-02
    provides: factory_multi_parent, factory_multi_parent_dispatch, try_undx/spx/pcx dispatchers
  - phase: 51-03
    provides: self_adaptive_gaussian_mutation, try_self_adaptive dispatcher
  - phase: 51-01
    provides: RealValued/SelfAdaptive traits, Crossover::Undx/Spx/Pcx variants, MutationConfiguration SA fields
provides:
  - src/engines/ga.rs multi-parent dispatch branch in process_pair closure
  - tests/test_multi_parent_integration.rs (5 end-to-end integration tests)
  - crossover::factory_multi_parent_dispatch (public, no RealValued bound)
  - mutation::factory_self_adaptive (public, explicit ES param forwarding)
affects:
  - all engines that depend on ga.rs process_pair for crossover dispatch

# Tech tracking
tech-stack:
  added: []
  patterns:
    - factory_multi_parent_dispatch without RealValued bound (downcast approach instead of bound expansion)
    - factory_self_adaptive for explicit ES param forwarding from ga.rs
    - effective_method pattern (AOS-selected OR configured) before crossover dispatch
    - 1-vs-2 child handling: first pop = child_1, second pop = fallback clone for child_2

key-files:
  created:
    - tests/test_multi_parent_integration.rs
  modified:
    - src/engines/ga.rs (multi-parent dispatch + SelfAdaptiveGaussian branch)
    - src/operations/crossover.rs (factory_multi_parent_dispatch added)
    - src/operations/mutation.rs (factory_self_adaptive added)
    - src/operations/mutation/self_adaptive_gaussian.rs (doc fix)
    - src/traits/real_valued.rs (doctest fix + doc fix)

key-decisions:
  - "Used factory_multi_parent_dispatch<U: LinearChromosome + 'static> (not RealValued bound) so parent_crossover stays generic over all chromosome types — binary/permutation chromosomes not broken"
  - "First pop = child_1 (actual offspring), second pop = child_2 (fallback parent_1.clone()) — this is the 1-vs-2 child contract for multi-parent path"
  - "SelfAdaptiveGaussian added as a dedicated else-if branch in both child_1 and child_2 mutation sections to forward user-configured tau/tau_prime/sigma_min"
  - "Integration test sigma drift check on population.chromosomes (not just best_chromosome) — best may be an initial parent never mutated as offspring"

patterns-established:
  - "factory_*_dispatch pattern: public fn without restrictive bound, uses private downcast dispatchers internally (mirrors factory_lexicase model)"
  - "effective_method = selected_crossover.map(|(_, op)| op).unwrap_or(config.method) — AOS-aware dispatch pattern"

requirements-completed:
  - CRS-02
  - CRS-03
  - CRS-04
  - MUT-05
  - TRAITS-02

# Metrics
duration: 45min
completed: "2026-05-23"
---

# Phase 51 Plan 04: ga.rs Integration + End-to-End Tests — Summary

**UNDX/SPX/PCX multi-parent crossover and SelfAdaptiveGaussian wired into `Ga<RangeChromosome<f64>>.run()` via downcast-based dispatch functions, with 5 integration tests verifying end-to-end behavior.**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-23T19:00:00Z
- **Completed:** 2026-05-23T19:44:02Z
- **Tasks:** 3 (Task 4 is a human checkpoint — paused)
- **Files modified:** 6

## Accomplishments

- Added `factory_multi_parent_dispatch<U: LinearChromosome + 'static>` to `crossover.rs` — same downcast approach as `try_undx/spx/pcx` but public, usable from `ga.rs` without requiring `RealValued` bound on `parent_crossover`
- Added `factory_self_adaptive` to `mutation.rs` — forwards user-configured ES parameters (tau/tau_prime/sigma_min) from `MutationConfiguration` to the downcast dispatcher
- Modified `parent_crossover` in `ga.rs`: `process_pair` closure now detects `Crossover::Undx|Spx|Pcx` via `effective_method`, collects `num_parents` references (primary pair + random extras via `saturating_sub(2)`), calls `factory_multi_parent_dispatch`, and handles 1-vs-2 child contract via `unwrap_or_else(|| parent_1.clone())`
- Added dedicated `SelfAdaptiveGaussian` else-if branches in both child_1 and child_2 mutation sections, reading `mutation_configuration.self_adaptive_tau/tau_prime/sigma_min`
- All 5 integration tests pass: UNDX/SPX/PCX runs-without-panic, SelfAdaptiveGaussian sigma drift, serde round-trip
- Zero doc warnings after fixing rustdoc link issues in three files

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire multi-parent dispatch + SelfAdaptiveGaussian into ga.rs** - `2c1aadf` (feat)
2. **Task 2: Add end-to-end integration tests + doctest fixes** - `e2edde5` (feat)
3. **Task 3: Verification gate + SUMMARY** — this commit (docs)

## Files Created/Modified

- `src/operations/crossover.rs` — Added `factory_multi_parent_dispatch` public function (no RealValued bound, uses downcast dispatchers)
- `src/operations/mutation.rs` — Added `factory_self_adaptive` public function for explicit ES param forwarding
- `src/engines/ga.rs` — Multi-parent dispatch branch in `process_pair` closure; SelfAdaptiveGaussian branches in child_1/child_2 mutation sections
- `tests/test_multi_parent_integration.rs` — 5 integration tests: UNDX/SPX/PCX/SelfAdaptiveGaussian/serde
- `src/traits/real_valued.rs` — Fixed doctest (added `#[derive(Clone, Default)]`); fixed module doc links
- `src/operations/mutation/self_adaptive_gaussian.rs` — Fixed redundant explicit rustdoc link

## Requirements Closed

| ID | Description | Verified by |
|----|-------------|-------------|
| CRS-02 | UNDX multi-parent crossover operator | `end_to_end_undx_runs_without_panic` |
| CRS-03 | SPX multi-parent crossover operator | `end_to_end_spx_runs_without_panic` |
| CRS-04 | PCX multi-parent crossover operator | `end_to_end_pcx_runs_without_panic` |
| MUT-05 | SelfAdaptiveGaussian mutation | `end_to_end_self_adaptive_gaussian_sigmas_evolve` |
| TRAITS-02 | RealValued + SelfAdaptive marker traits compile-time enforcement | compile-time (impl on Range<T> only) |

## Gate Output

**Gate 1: `cargo build`**
```
Compiling genetic_algorithms v3.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.48s
```

**Gate 2: `cargo build --features serde`**
```
Compiling genetic_algorithms v3.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.97s
```

**Gate 3: `cargo test`**
```
cargo test: 1111 passed, 32 ignored (25 suites, 12.08s)
```

**Gate 4: `cargo test --features serde`**
```
cargo test: 1149 passed, 32 ignored (25 suites, 8.94s)
```

**Gate 5: `cargo clippy -- -D warnings`** (lib only; see note below)
```
cargo clippy: No issues found
```

**Gate 6: `cargo check --target wasm32-unknown-unknown`**
```
Compiling genetic_algorithms v3.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.78s
```

**Gate 7: `cargo doc --no-deps`**
```
Finished `dev` profile
Generated genetic_algorithms/index.html
(0 warnings after fixing doc link issues)
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pop order reversed — "Crossover returned no children" on multi-parent path**
- **Found during:** Task 2 (integration test run)
- **Issue:** The initial implementation had `child_2 = children.pop()` first, then `child_1 = children.pop()`. For a 1-element Vec (from `factory_multi_parent_dispatch`), the first pop consumed the only element into `child_2`, then the second pop failed with "Crossover returned no children". Both children are semantically equivalent so the naming is symmetric — fix inverts the pop order.
- **Fix:** Changed to `child_1 = children.pop().ok_or_else(...)` first (gets the actual offspring), `child_2 = children.pop().unwrap_or_else(|| parent_1.clone())` second (fallback for 1-child path).
- **Files modified:** `src/engines/ga.rs`
- **Committed in:** e2edde5 (Task 2 commit)

**2. [Rule 1 - Bug] Pre-existing doctest failure in `src/traits/real_valued.rs`**
- **Found during:** Task 2 (full test run via `cargo test`)
- **Issue:** The doctest in `real_valued.rs` (created in Plan 01) defined `MyGene` and `MyChromosome` without `#[derive(Clone, Default)]`. These derives are required because `Cow<[MyGene]>` in `LinearChromosome::set_dna` needs `[MyGene]: ToOwned` which requires `MyGene: Clone`. The doctest failed at compile time.
- **Fix:** Added `#[derive(Clone, Default)]` to the hidden `# struct MyGene` and `# struct MyChromosome` lines in the doctest.
- **Files modified:** `src/traits/real_valued.rs`
- **Committed in:** e2edde5 (Task 2 commit)

**3. [Rule 1 - Bug] Integration test sigma drift check on population (not best_chromosome)**
- **Found during:** Task 2 (sigma drift test failure)
- **Issue:** The plan specified checking `ga.get_best_individual().strategy_params()` for drift. But `best_chromosome` may remain an initial individual (selected early when close to origin) that was never selected as an offspring and thus never had `adapt_strategy_params` called on it. Its sigmas remain `[1.0, 1.0, 1.0, 1.0, 1.0]`.
- **Fix:** Changed the assertion to check `population.chromosomes` (the final population) — all individuals in the final population have been through at least one generation as offspring, so some will have adapted sigmas.
- **Files modified:** `tests/test_multi_parent_integration.rs`
- **Committed in:** e2edde5 (Task 2 commit)

**4. [Rule 1 - Bug] 5 rustdoc warnings in files created during this plan**
- **Found during:** Task 3 (Gate 7: `cargo doc --no-deps`)
- **Issue:** (a) `mutation.rs` `factory_self_adaptive` doc linked to private `try_self_adaptive`. (b) `real_valued.rs` module doc had unresolved links `[Crossover::Undx]` etc. (c) `self_adaptive_gaussian.rs` had redundant explicit link target.
- **Fix:** Replaced private item link with prose; changed `[Crossover::Undx]` links to backtick text; simplified the redundant explicit link.
- **Files modified:** `src/operations/mutation.rs`, `src/traits/real_valued.rs`, `src/operations/mutation/self_adaptive_gaussian.rs`
- **Committed in:** This commit (Task 3)

**5. [Deviation - Architecture] Chose factory_multi_parent_dispatch without RealValued bound instead of bound expansion**
- **Found during:** Task 1 (Read step for parent_crossover callers)
- **Issue:** The plan's first-choice approach was to expand `parent_crossover`'s bound from `U: LinearChromosome` to `U: LinearChromosome + RealValued`. However, `Ga<U>` is used with `BinaryChromosome`, `ListChromosome`, and `UniqueChromosome` — none of which implement `RealValued`. Expanding the bound would break all non-real-valued GA users at compile time.
- **Fix:** Used the plan's documented fallback: added `pub fn factory_multi_parent_dispatch<U: LinearChromosome + 'static>` that calls the existing private `try_undx/spx/pcx` downcast dispatchers internally. This mirrors the established downcast pattern from `try_cauchy`, `try_polynomial`, etc. `parent_crossover`'s `U: LinearChromosome + ... + 'static` already satisfies the `'static` bound needed for the `Any` downcasts.
- **Files modified:** `src/operations/crossover.rs`, `src/engines/ga.rs`
- **Committed in:** 2c1aadf (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (3 Rule 1 bugs, 1 Rule 1 doc warnings) + 1 architecture deviation (documented fallback path)
**Impact on plan:** All auto-fixes required for correctness or compilation. Architecture deviation is explicitly documented as acceptable fallback in the plan. No scope creep.

## Known Pre-existing Issues (Out of Scope)

**`cargo clippy --all-targets -- -D warnings` fails on bench targets:** Files in `benches/` implement `ChromosomeT` methods that belong in `LinearChromosome` (the two traits were split in Phase 47). This is a pre-existing issue not caused by this plan's changes. Verified by `git stash` + re-run. `cargo clippy -- -D warnings` (lib only) passes cleanly.

**`cargo clippy --tests -- -D warnings` warns on pre-existing test files:** `tests/traits/test_self_adaptive.rs` (Plan 01) has an unused `ChromosomeT` import and unnecessary `i32 as i32` casts. `tests/operations/test_mutation_self_adaptive.rs` (Plan 01/02) has unused `Result` values. Not caused by this plan.

## Open Follow-ups

- `SelfAdaptive` implementation on `MultiRangeChromosome` (Phase 48 scope — Phase 51 ships only `RealValued` for `MultiRangeChromosome`)
- Multi-parent dispatch for NSGA-II/III, MOEA/D, SPEA2, SMS-EMOA, IBEA (future phase — only `ga.rs` wired in Phase 51)
- Per-gene UNDX Gram-Schmidt orthogonalization vs. simplified approximation used in `undx.rs` (research assumption A5 — current approximation is valid but differs from strict UNDX literature)

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. All changes are in-process operator dispatch and integration test code.

## Self-Check: PASSED

Files exist:
- `src/operations/crossover.rs` (factory_multi_parent_dispatch) — FOUND
- `src/operations/mutation.rs` (factory_self_adaptive) — FOUND
- `src/engines/ga.rs` (multi-parent dispatch) — FOUND
- `tests/test_multi_parent_integration.rs` — FOUND

Commits verified:
- 2c1aadf (feat(51-04): wire multi-parent crossover dispatch) — FOUND
- e2edde5 (feat(51-04): add end-to-end integration tests) — FOUND

---
*Phase: 51-multi-parent-crossover-self-adaptive-mutation*
*Completed: 2026-05-23*
