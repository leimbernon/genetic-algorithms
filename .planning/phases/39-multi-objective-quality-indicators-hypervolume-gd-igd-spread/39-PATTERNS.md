# Phase 39: Multi-objective Quality Indicators — Hypervolume, GD, IGD, Spread - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 11 (6 source files, 4 test files, 1 directory module)
**Analogs found:** 11 / 11

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/multi_objective/indicators/mod.rs` | config | n/a (module entry) | `src/engines/multi_objective/mod.rs` | exact (same role, same module pattern) |
| `src/engines/multi_objective/indicators/hypervolume.rs` | utility | computation | `src/engines/multi_objective/non_dominated_sort.rs` | exact (multi_objective utility, pure computation with `&[Vec<f64>]` input) |
| `src/engines/multi_objective/indicators/generational_distance.rs` | utility | computation | `src/engines/multi_objective/non_dominated_sort.rs` | exact (multi_objective utility, pure computation with `&[Vec<f64>]` input) |
| `src/engines/multi_objective/indicators/inverted_generational_distance.rs` | utility | computation | `src/engines/multi_objective/non_dominated_sort.rs` | exact (multi_objective utility, pure computation with `&[Vec<f64>]` input) |
| `src/engines/multi_objective/indicators/spread.rs` | utility | computation | `src/engines/multi_objective/non_dominated_sort.rs` | exact (multi_objective utility, pure computation with `&[Vec<f64>]` input) |
| `src/engines/multi_objective/mod.rs` | config | n/a (module entry) | File itself — add one `pub mod indicators;` line | exact |
| `src/error.rs` | utility | n/a (type defn) | File itself — add one `InvalidIndicatorConfiguration(String)` variant + Display arm | exact |
| `tests/engines/multi_objective/indicators/test_hypervolume.rs` | test | computation | `tests/engines/nsga2/test_non_dominated_sort.rs` | exact (MOO utility test with inline `Vec<Vec<f64>>` test data) |
| `tests/engines/multi_objective/indicators/test_generational_distance.rs` | test | computation | `tests/engines/nsga2/test_non_dominated_sort.rs` | exact (MOO utility test with inline `Vec<Vec<f64>>` test data) |
| `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` | test | computation | `tests/engines/nsga2/test_non_dominated_sort.rs` | exact (MOO utility test with inline `Vec<Vec<f64>>` test data) |
| `tests/engines/multi_objective/indicators/test_spread.rs` | test | computation | `tests/engines/nsga2/test_non_dominated_sort.rs` | exact (MOO utility test with inline `Vec<Vec<f64>>` test data) |

## Pattern Assignments

### `src/engines/multi_objective/indicators/mod.rs` (config, module entry)

**Analog:** `src/engines/multi_objective/mod.rs`

**Module entry point pattern** (entire file, lines 1-29):
```rust
//! Shared multi-objective optimization primitives.
//!
//! This module hosts the building blocks shared by the NSGA-II, NSGA-III,
//! and future MOEA engines: non-dominated sorting, Pareto individual/front
//! types, dominance predicates, and the `ObjectiveFn<G>` type alias.
//!
//! NSGA-II re-exports these symbols via `pub use crate::multi_objective::*`
//! for full backward compatibility — existing user code that uses paths like
//! `genetic_algorithms::nsga2::pareto::ParetoIndividual` continues to work.

pub mod non_dominated_sort;
pub mod pareto;

// ... (enums / type aliases follow, but indicators/mod.rs is simpler:
//      it only needs doc comments + pub mod / pub use lines)
```

**Pattern for indicators/mod.rs:**
- Doc comment describing the module's purpose (quality indicators for MOO)
- `mod` declarations for each indicator submodule (private; functions re-exported)
- `pub use` re-exports for each public function, following the same convention as `multi_objective/mod.rs` uses for `pub mod`

---

### `src/engines/multi_objective/indicators/hypervolume.rs` (utility, computation)

**Analog:** `src/engines/multi_objective/non_dominated_sort.rs`

**Imports pattern:**
```rust
// non_dominated_sort.rs lines 1-2:
use super::pareto::{constrained_dominates, dominates, dominates_with_directions};
use super::ObjectiveDirection;
```
Indicator files will use `use super::...` for sibling module helpers (if any validation helpers are factored into a shared file). If validation helpers are inlined, no cross-module imports are needed — pure std only.

**Doc comment pattern** (lines 1-6 of non_dominated_sort.rs):
```rust
/// Performs non-dominated sorting on a population.
///
/// Returns a list of fronts, where each front is a vector of indices
/// into the original objectives list. Front 0 is the first (best) Pareto front.
///
/// # Arguments
///
/// * `objectives` - A slice of objective vectors, one per individual.
```

**Core pure-function pattern** (lines 1-31 of non_dominated_sort.rs):
```rust
pub fn non_dominated_sort(objectives: &[&[f64]]) -> Vec<Vec<usize>> {
    non_dominated_sort_inner(objectives, dominates)
}
```
For indicators: stateless `pub fn` taking `&[Vec<f64>]` (not `&[&[f64]]` — input points are owned `Vec<f64>` from user code), returning `Result<f64, GaError>`.

**Error handling pattern:** Not applicable in non_dominated_sort (it never fails, returns `Vec`). Indicators return `Result<f64, GaError>` with validation upfront (see Shared Patterns below).

---

### `src/engines/multi_objective/indicators/generational_distance.rs` (utility, computation)

**Analog:** `src/engines/multi_objective/non_dominated_sort.rs`

Same pattern as hypervolume.rs — pure function with input validation. Uses `f64::powi(2)` for squared Euclidean distance, `f64::powf(power)` for configurable exponent, and `f64::sqrt()`.

---

### `src/engines/multi_objective/indicators/inverted_generational_distance.rs` (utility, computation)

**Analog:** `src/engines/multi_objective/non_dominated_sort.rs`

Same structural pattern as `generational_distance.rs` with swapped input argument roles. Reuses `nearest_distance()` if factored as a shared helper, or inlines the computation.

---

### `src/engines/multi_objective/indicators/spread.rs` (utility, computation)

**Analog:** `src/engines/multi_objective/non_dominated_sort.rs`

Same structural pattern. Sort by first objective, compute consecutive Euclidean distances, then Deb 2002 delta formula with edge case (division by zero returns 0.0).

---

### `src/engines/multi_objective/mod.rs` (config, n/a — one-line addition)

**Pattern:** Add `pub mod indicators;` after line 12 (`pub mod pareto;`), following the same declaration style:

```rust
// Line 11-12:
pub mod non_dominated_sort;
pub mod pareto;
// NEW:
pub mod indicators;
```

No other changes needed. The existing `#[path = "engines/multi_objective/mod.rs"] pub mod multi_objective;` in `src/lib.rs` already covers the indicators submodule transitively. Users import via `crate::multi_objective::indicators::*`.

---

### `src/error.rs` (utility, n/a — one new variant + Display arm)

**Analog:** Existing `GaError` enum in `src/error.rs`

**New variant** (add after line 41 `InvalidSpea2Configuration(String)`):
```rust
/// An indicator configuration parameter is invalid.
InvalidIndicatorConfiguration(String),
```

**Display arm** (add after the `InvalidSpea2Configuration` arm at line 75):
```rust
GaError::InvalidIndicatorConfiguration(msg) => {
    write!(f, "Invalid indicator configuration: {}", msg)
}
```

**Pattern:** Uses `Invalid{Feature}Configuration(String)` convention consistent with existing variants:
- `InvalidIslandConfiguration(String)` (line 31)
- `InvalidNichingConfiguration(String)` (line 33)
- `InvalidNsga2Configuration(String)` (line 35)
- `InvalidSpea2Configuration(String)` (line 41)
- etc.

---

### `tests/engines/multi_objective/indicators/test_hypervolume.rs` (test, computation)

**Analog:** `tests/engines/nsga2/test_non_dominated_sort.rs`

**Import pattern** (lines 1-5):
```rust
use genetic_algorithms::nsga2::configuration::ObjectiveDirection;
use genetic_algorithms::nsga2::non_dominated_sort::{
    assign_ranks, non_dominated_sort, non_dominated_sort_constrained,
    non_dominated_sort_with_directions,
};
```
For indicators: import from `crate::multi_objective::indicators::*` (via `use genetic_algorithms::multi_objective::indicators::{hypervolume, generational_distance, inverted_generational_distance, spread}`).

**Test function pattern** (lines 8-15 of test_non_dominated_sort.rs):
```rust
#[test]
fn test_non_dominated_sort_single_front() {
    // Three non-dominated points
    let objectives: Vec<Vec<f64>> = vec![vec![1.0, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
    let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
    let fronts = non_dominated_sort(&refs);
    assert_eq!(fronts.len(), 1);
    assert_eq!(fronts[0].len(), 3);
}
```
For indicators: inline `Vec<Vec<f64>>` test data with analytically-known expected values. Use `matches!` macro for error assertions:

```rust
// From SPEA2 test (test_spea2.rs line 22):
assert!(matches!(result, Err(GaError::InvalidSpea2Configuration(_))));
```
For indicator tests: assert exact float matches with a small epsilon tolerance (`(a - b).abs() < 1e-10`).

**Test data helpers** (pattern from SPEA2's `build_test_spea2`, test_spea2.rs lines 139-166):
Define inline helper function generating reference front data:
```rust
fn zdt1_reference_front(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let f1 = i as f64 / (n - 1) as f64;
            let f2 = 1.0 - f1.sqrt();
            vec![f1, f2]
        })
        .collect()
}
```

---

### `tests/engines/multi_objective/indicators/test_generational_distance.rs` (test, computation)

**Analog:** `tests/engines/nsga2/test_non_dominated_sort.rs`

Same pattern as test_hypervolume.rs. Tests include:
- Identical fronts produce GD=0.0
- Shifted front produces positive GD
- Dimension mismatch returns error (`matches!` macro)
- Empty sets return error

---

### `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` (test, computation)

**Analog:** `tests/engines/nsga2/test_non_dominated_sort.rs`

Same pattern. Key test: IGD > GD for sparse approx (coverage asymmetry).

---

### `tests/engines/multi_objective/indicators/test_spread.rs` (test, computation)

**Analog:** `tests/engines/nsga2/test_non_dominated_sort.rs`

Same pattern. Tests include:
- Uniformly spaced points produce spread=0.0 (perfect distribution)
- Non-uniform points produce spread>0
- Single point returns error
- Edge case when denominator is 0

---

## Shared Patterns

### 1. GaError Variant Addition

**Source:** `src/error.rs` lines 17-46 (enum variants) and lines 49-78 (Display impl)
**Apply to:** `src/error.rs` modification

Pattern for adding a new variant:
```rust
/// (doc comment)
InvalidIndicatorConfiguration(String),  // add after line 41
```

Pattern for adding Display arm (follow existing match structure):
```rust
GaError::InvalidIndicatorConfiguration(msg) => {
    write!(f, "Invalid indicator configuration: {}", msg)
},
```

### 2. Input Validation Pattern (upfront, returns Err)

All indicator functions must validate inputs before computation. Use `let ... = ...?` pattern or early return:
```rust
if points.is_empty() {
    return Err(GaError::InvalidIndicatorConfiguration(
        "approx_front must not be empty".to_string(),
    ));
}
```

### 3. Float Comparison in Tests

Use `f64::EPSILON` or `1e-10` tolerance for assertion:
```rust
let expected = 0.666666666;
assert!((result - expected).abs() < 1e-10,
    "Hypervolume mismatch: expected {}, got {}", expected, result);
```

### 4. Test Module Structure

**Source:** `tests/engines/nsga2/test_non_dominated_sort.rs` (entire file)
**Apply to:** All test files under `tests/engines/multi_objective/indicators/`

- No `mod.rs` needed (Cargo discovers individual `*.rs` files)
- One file per indicator function
- Import via `use genetic_algorithms::multi_objective::indicators::*;`
- Inline test data (no hardcoded constants from external libraries)
- `#[test]` attribute on each test function
- Function names: `test_{indicator}_{scenario}` (e.g., `test_hypervolume_basic_zdt1`)

### 5. f64 Comparison Traits

Use `f64::EPSILON` and `f64::INFINITY`:
```rust
let mut nearest = f64::INFINITY;  // for min-distance initialization
```
```rust
if (a - b).abs() < f64::EPSILON { ... }  // for equality comparison
```

## No Analog Found

All 11 files have exact or role-match analogs in the codebase. No unmatched files.

| File | Closest Analog | Match Quality |
|------|----------------|---------------|
| All indicator source files | `non_dominated_sort.rs` | exact (multi_objective utility, pure computation) |
| indicators/mod.rs | `multi_objective/mod.rs` | exact (same module entry pattern) |
| mod.rs modification | The file itself | exact |
| error.rs modification | The file itself | exact |
| All test files | `test_non_dominated_sort.rs` | exact (MOO utility tests) |

## Metadata

**Analog search scope:** `src/engines/multi_objective/`, `src/error.rs`, `tests/engines/`
**Files scanned:** 11 (multi_objective module, existing tests, error module, lib.rs)
**Pattern extraction date:** 2026-05-10
