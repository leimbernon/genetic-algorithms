---
phase: 39-multi-objective-quality-indicators-hypervolume-gd-igd-spread
plan: 02
subsystem: multi_objective::indicators
tags:
  - quality-indicators
  - moo-05
  - hypervolume
  - generational-distance
dependency-graph:
  requires:
    - plans/39-01-foundation (error variant + module scaffolding + validation helpers)
  provides:
    - pub fn hypervolume() — 2D Lebesgue measure
    - pub fn generational_distance() — GD indicator
    - Inline unit tests (6 per function)
    - Integration tests (8 per function, tests/ directory)
  affects:
    - Phase 38 (SMS-EMOA consumes hypervolume)
    - User code (post-run Pareto-front analysis)
tech-stack:
  added:
    - src/engines/multi_objective/indicators/hypervolume.rs
    - src/engines/multi_objective/indicators/generational_distance.rs
    - tests/engines/multi_objective/indicators/test_hypervolume.rs
    - tests/engines/multi_objective/indicators/test_generational_distance.rs
  patterns:
    - Sort-then-sweep with running minimum for 2D Lebesgue measure (Zitzler & Thiele 1999)
    - nearest_distance helper shared via super:: path
    - Configurable power parameter for GD norm (default 2.0)
key-files:
  created:
    - src/engines/multi_objective/indicators/hypervolume.rs
    - src/engines/multi_objective/indicators/generational_distance.rs
    - tests/engines/multi_objective/indicators/test_hypervolume.rs
    - tests/engines/multi_objective/indicators/test_generational_distance.rs
decisions:
  - hypervolume() restricted to exactly 2 objectives (returns Err for 3D+)
  - Reference point must strictly dominate all points (f1 < ref[0] AND f2 < ref[1])
  - Points sorted by f1 ascending before sweep; running-minimum f2 handles dominated points correctly
  - GD uses outer root (sum/n)^(1/power) to complete (1/n * sum d_i^p)^{1/p} formula
  - Integration tests use inline zdt1_reference_front() helper (no hardcoded library constants)
  - DEFAULT_POWER constant removed after clippy warning — unused since power is always explicit
metrics:
  duration: multiple sessions (committed 2026-05-10)
  completed_date: 2026-05-10
  tasks:
    total: 3
    completed: 3
    checkpoint: 0
---

# Phase 39 Plan 02: Hypervolume + Generational Distance

**One-liner:** Implemented `hypervolume()` (2D Lebesgue measure via sort-then-sweep) and `generational_distance()` (configurable-power nearest-distance metric) with full validation, inline unit tests, and integration tests against ZDT1 reference fronts.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Implement hypervolume.rs — 2D Lebesgue measure | `ea20b86` | src/engines/multi_objective/indicators/hypervolume.rs |
| 2 | Implement generational_distance.rs | `ea20b86` | src/engines/multi_objective/indicators/generational_distance.rs |
| 3 | Create integration tests for hypervolume and GD | `ea20b86` | tests/engines/multi_objective/indicators/test_hypervolume.rs, tests/engines/multi_objective/indicators/test_generational_distance.rs |

### Task 1 Details

Created `src/engines/multi_objective/indicators/hypervolume.rs` with `pub fn hypervolume(points: &[Vec<f64>], reference_point: &[f64]) -> Result<f64, GaError>`:
- Validates non-empty, 2D-only, reference point strictly dominates all points
- Sort by f1 ascending, sweep with running-minimum f2 (correct Lebesgue measure for dominated interior points)
- Returns area of union of rectangles bounded by [f1_i, ref[0]] x [f2_i, ref[1]]
- 6 inline unit tests covering single point, two-point front, 3D rejection, empty rejection, non-dominating reference, dimension mismatch

### Task 2 Details

Created `src/engines/multi_objective/indicators/generational_distance.rs` with `pub fn generational_distance(approx_front: &[Vec<f64>], true_front: &[Vec<f64>], power: f64) -> Result<f64, GaError>`:
- Validates both fronts non-empty, dimension-consistent, matching dimensions, positive power
- Uses `nearest_distance()` helper: `min_sq_dist.powf(power / 2.0)` = Euclidean distance raised to power
- GD = (1/n * sum d_i^p)^{1/p}
- O(n*m) pairwise distance computation
- 6 inline unit tests covering identical fronts, shifted fronts (GD = sqrt(2)), power=1, dimension mismatch, empty fronts, non-positive power
- Import path fix: `use crate::error::GaError` (not `crate::GaError` — GaError is not re-exported at crate root)

### Task 3 Details

Created integration test files under `tests/engines/multi_objective/indicators/`:
- `test_hypervolume.rs`: 8 test functions — single point, two-point front, ZDT1 analytical (HV = 2/3), wider reference point, 3D rejection, empty rejection, non-dominating reference, dimension mismatch
- `test_generational_distance.rs`: 8 test functions — identical fronts, shifted fronts, ZDT1 subset, power=1, dimension mismatch, empty approx, empty true, zero power
- Both use inline `zdt1_reference_front()` helper (no hardcoded constants)

## Deviations from Plan

### Post-Implementation Cleanup

**1. Removed unused `DEFAULT_POWER` constant**
- **Found during:** Phase verification gate (clippy)
- **Issue:** `pub(crate) const DEFAULT_POWER: f64 = 2.0` in generational_distance.rs was unused — both GD and IGD take `power` as an explicit parameter
- **Fix:** Removed the constant and its unused import in inverted_generational_distance.rs
- **Commit:** Uncommitted (working tree)

## Verification Gate

| Check | Result |
|-------|--------|
| `cargo test` (all) | 856 passed, 23 ignored |
| `cargo test --features serde` | 886 passed, 23 ignored |
| `cargo clippy` | No issues |
| `cargo doc --no-deps` | 0 new warnings (15 pre-existing) |
| `cargo check --target wasm32-unknown-unknown` | Not tested — pre-existing getrandom WASM issue |

## Known Stubs

None. Both functions are fully implemented with complete validation and test coverage.

## Self-Check: PASSED

- `grep 'pub fn hypervolume' src/engines/multi_objective/indicators/hypervolume.rs` = 1
- `grep 'pub fn generational_distance' src/engines/multi_objective/indicators/generational_distance.rs` = 1
- `grep -r "pub fn hypervolume\|pub fn generational_distance\|pub fn inverted_generational_distance\|pub fn spread" src/engines/multi_objective/indicators/` = 4
- All 16 integration tests in `tests/engines/multi_objective/indicators/` pass
