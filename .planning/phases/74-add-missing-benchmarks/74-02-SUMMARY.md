---
phase: 74-add-missing-benchmarks
plan: 02
subsystem: benchmarks
tags: [divan, eda, gp, benchmark, symbolic-regression]

# Dependency graph
requires:
  - phase: 74-add-missing-benchmarks
    provides: "Cargo.toml [[bench]] pattern, divan harness setup"
provides:
  - "benches/eda.rs — EDA engine divan benchmark (Gaussian dims axis + Bernoulli binary group)"
  - "benches/gp.rs — GP engine divan benchmark, population-size axis, symbolic regression"
  - "Two new [[bench]] entries in Cargo.toml for eda and gp"
affects: [benchmarks, engine-coverage]

# Tech tracking
tech-stack:
  added: []
  patterns: [divan-bench-values, eda-real-engine-gaussian, gp-turbofish-constructor]

key-files:
  created:
    - benches/eda.rs
    - benches/gp.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Used EdaRealEngine::new (not EdaEngine::new) for Gaussian path — EdaEngine is Bernoulli-only per Pitfall 3"
  - "Used explicit turbofish GpGa::<MathNode>::with_ramped_half_and_half to avoid type-inference failure"
  - "GpConfiguration built inside with_inputs closure (not .build()) — build() returns Result<(), GaError>, not the config"
  - "max_generations=20 and max_depth=6 for GP bench — keeps pop_500 CI-friendly per Pitfall 6"

patterns-established:
  - "EDA bench: split Gaussian (EdaRealEngine) and Bernoulli (EdaEngine::bernoulli) into separate mod groups"
  - "GP bench: pass config directly to constructor, discard Result with let _ = engine.run()"

requirements-completed: []

# Metrics
duration: 1min
completed: 2026-06-19
---

# Phase 74 Plan 02: EDA and GP Engine Benchmarks Summary

**EDA Gaussian/Bernoulli benchmarks (sphere dims 10/30/100 + OneMax-64) and GP symbolic regression benchmark (pop 50/200/500) using divan**

## Performance

- **Duration:** 1 min
- **Started:** 2026-06-19T06:56:13Z
- **Completed:** 2026-06-19T06:57:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created `benches/eda.rs` with two groups: Gaussian (EdaRealEngine sphere, dims 10/30/100) and Bernoulli (EdaEngine::bernoulli, OneMax-64)
- Created `benches/gp.rs` with GpGa\<MathNode\> symbolic regression (f(x)=x²+x+1), population-size axis (50/200/500)
- Added `[[bench]]` entries for both `eda` and `gp` in Cargo.toml

## Task Commits

Each task was committed atomically:

1. **Task 1: Create benches/eda.rs** - `0ba5b3f` (feat)
2. **Task 2: Create benches/gp.rs** - `3850e0b` (feat)

## Files Created/Modified
- `benches/eda.rs` - EDA engine divan benchmark: Gaussian (EdaRealEngine, sphere, dims 10/30/100) + Bernoulli (EdaEngine, OneMax-64)
- `benches/gp.rs` - GP engine divan benchmark: symbolic regression (MathNode), population-size axis (50/200/500)
- `Cargo.toml` - Two new `[[bench]]` entries: eda and gp, both harness=false

## Decisions Made
- Used `EdaRealEngine::new` (not `EdaEngine::new`) for the Gaussian path — EdaEngine is Bernoulli-only (Pitfall 3 from RESEARCH.md)
- Used explicit turbofish `GpGa::<MathNode>::with_ramped_half_and_half` to avoid type-inference failure (Open Questions 2)
- Built `GpConfiguration` directly without `.build()` — `build()` returns `Result<(), GaError>` (validation only), not the config itself
- Set GP `max_generations=20` and `max_depth=6` to keep pop_500 runtime CI-friendly (Pitfall 6)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- EDA and GP engine benchmarks complete — engine coverage gap closed for these two engines
- Ready for plan 74-03 (remaining feature benchmarks: AOS, surrogate, batch_fitness)

## Self-Check: PASSED

- `benches/eda.rs` exists: FOUND
- `benches/gp.rs` exists: FOUND
- `Cargo.toml` contains `[[bench]] name = "eda"`: FOUND
- `Cargo.toml` contains `[[bench]] name = "gp"`: FOUND
- Commit `0ba5b3f` exists: FOUND
- Commit `3850e0b` exists: FOUND
- `cargo bench --no-run --bench eda --bench gp` exits 0: PASS

---
*Phase: 74-add-missing-benchmarks*
*Completed: 2026-06-19*
