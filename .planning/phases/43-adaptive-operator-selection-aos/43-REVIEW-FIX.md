---
phase: 43-adaptive-operator-selection-aos
fixed_at: 2026-06-02T00:00:00Z
review_path: .planning/phases/43-adaptive-operator-selection-aos/43-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 43: Code Review Fix Report

**Fixed at:** 2026-06-02
**Source review:** `.planning/phases/43-adaptive-operator-selection-aos/43-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: Reward signal blows up at convergence

**Files modified:** `src/aos.rs`, `tests/engines/aos/test_aos.rs`
**Commit:** a1006e4
**Applied fix:** Replaced `best_fitness.abs().max(f64::EPSILON)` denominator with
`1.0 + parent_fitness.abs()`, bounding rewards in `(-1, 1)` regardless of fitness
magnitude. Updated the doc comment, inline tests, and integration tests to reflect
the new formula. The `_best_fitness` parameter is kept for API compatibility.

### CR-02: Only child_1 reward recorded; child_2 silently discarded

**Files modified:** `src/engines/ga.rs`
**Commit:** 02dbd87
**Applied fix:** Both `child_1` and `child_2` rewards are now accumulated against
their respective parents (`parent_1` and `parent_2`) for both the crossover and
mutation reward accumulators. Each couple now contributes two reward entries per
generation instead of one, doubling the signal fidelity.

### WR-01: `learning_rate` field in `ProbabilityMatching` is unused

**Files modified:** `src/aos.rs`, `tests/engines/aos/test_aos.rs`
**Commit:** 16d470b
**Applied fix:** Removed `learning_rate` field from the `ProbabilityMatching` variant
definition, `pm_default()` constructor, `update()` match arm, and all test call sites
that constructed `ProbabilityMatching` with the extra field. Doc comment updated to
remove the reference to `learning_rate = 0.3`.

### WR-02: Mutex contention on rayon hot path (advisory)

**Files modified:** `src/engines/ga.rs`
**Commit:** 4dc932d
**Applied fix:** Added a `TODO(perf)` comment at the AOS operator selection lock
sites (crossover and mutation) documenting the known per-couple Mutex contention
and recommending pre-selection of operator indices before `par_iter` dispatch as the
future optimization path. No structural refactor performed per review guidance.

### WR-03: `with_reward_window(0)` causes panic

**Files modified:** `src/aos.rs`, `src/engines/ga.rs`
**Commit:** 2305bcb
**Applied fix:** Added validation in `Ga::build()` that rejects `aos_reward_window == 0`
when a crossover or mutation portfolio is configured, returning a `ConfigurationError`.
Added defensive `window_size.max(1)` guard in `ArmState::new` as a belt-and-suspenders
fallback to prevent index-out-of-bounds even if the build guard is bypassed.

### WR-04: AOS state not re-initialized on checkpoint resume — undocumented

**Files modified:** `src/engines/ga.rs`
**Commit:** 21c35f8
**Applied fix:** Added an explanatory comment at the AOS initialization block in
`run_with_callback` documenting that: (1) AOS state is always reset to fresh on
every invocation, and (2) on checkpoint resume the exploration phase is immediately
skipped when `checkpoint_generation > exploration_generations`, which is intentional
behavior.

---

_Fixed: 2026-06-02_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
