---
phase: 11-advanced-mode-examples
plan: "01"
subsystem: examples
tags: [nsga2, multi-objective, pareto, zdt1, continuous-optimization, range-chromosome]

# Dependency graph
requires:
  - phase: 10-single-population-examples
    provides: Range chromosome patterns and rastrigin example style reference
provides:
  - NSGA-II ZDT1 multi-objective optimization example (examples/nsga2_zdt1.rs)
affects: [11-advanced-mode-examples]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Nsga2Ga builder: new(nsga2_config, ga_config) -> with_alleles -> with_initialization_fn -> with_objective_fns -> build -> run"
    - "GaConfiguration for NSGA-II: set limit_configuration fields directly (not via builder trait)"
    - "ZDT1 objective functions as closures passed to with_objective_fns"

key-files:
  created:
    - examples/nsga2_zdt1.rs
  modified: []

key-decisions:
  - "Nsga2Ga::run() has no callback hook — document API limitation in example doc block and stdout"
  - "ga_config.limit_configuration fields set directly (genes_per_chromosome, alleles_can_be_repeated) since Nsga2Ga does not expose the ConfigurationT builder"
  - "Sample ~10 evenly-spaced points from sorted Pareto front for readable output"

patterns-established:
  - "NSGA-II examples: use sort_by on objectives[0] then step_by sampling for Pareto front display"

requirements-completed: [EX-02]

# Metrics
duration: 5min
completed: 2026-03-22
---

# Phase 11 Plan 01: NSGA-II ZDT1 Multi-Objective Example Summary

**NSGA-II ZDT1 example running 100 individuals for 250 generations, printing a 10-point sampled Pareto front showing f2 = 1 - sqrt(f1) trade-off**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-22T10:33:47Z
- **Completed:** 2026-03-22T10:38:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Created `examples/nsga2_zdt1.rs` (141 lines) — self-contained, runnable NSGA-II multi-objective example
- Demonstrates ZDT1 benchmark with 30 continuous variables and two conflicting minimization objectives
- Output shows the Pareto trade-off curve: f1 spans ~0 to ~0.78, f2 decreases correspondingly from ~0.97 to ~0.14
- Documents the `Nsga2Ga::run()` API limitation (no callback) in both the doc block and stdout

## Task Commits

1. **Task 1: Create NSGA-II ZDT1 example** - `8b4cd3f` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `examples/nsga2_zdt1.rs` - Self-contained NSGA-II ZDT1 multi-objective example with Pareto front sampling

## Decisions Made

- Documented the `Nsga2Ga::run()` API limitation (no callback hook) in both the `/*!` doc block and a printed note so users understand why there is no per-generation progress output.
- Set `ga_config.limit_configuration` fields directly rather than using the builder trait, matching the NSGA-II contract documented in CONTEXT.md and the plan interfaces.
- Used `sort_by` on `objectives[0]` and `step_by(step).take(10)` to sample the front evenly, giving a readable Pareto curve overview.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Clippy produced zero warnings. `cargo run --example nsga2_zdt1` exits 0 and prints 10 sampled Pareto front points.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- NSGA-II example complete; Phase 11 plans 02 and 03 (Island Model and Adaptive GA examples) can proceed.
- No blockers.

---
*Phase: 11-advanced-mode-examples*
*Completed: 2026-03-22*
