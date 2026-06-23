# Phase 76: Parallelize Survivor Selection and Non-Dominated Sorting - Pattern Map

**Mapped:** 2026-06-19
**Files analyzed:** 5 (modify/delete) + 2 (import updates)
**Analogs found:** 3 / 3

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/multi_objective/non_dominated_sort.rs` | utility (algorithm) | transform | `src/operations/survivor/fitness.rs` | cfg-gate pattern |
| `src/engines/nsga2/non_dominated_sort.rs` | (DELETE) | — | — | duplicate |
| `src/engines/nsga2/mod.rs` | config (module wiring) | — | `src/engines/multi_objective/mod.rs` | re-export pattern |
| `src/engines/island/nsga2.rs` | import fixup | — | — | trivial |
| `tests/engines/nsga2/test_non_dominated_sort.rs` | test | transform | — | no change needed |
| `benches/nsga2.rs` | benchmark | transform | — | add args only |

## Pattern Assignments

### `src/engines/multi_objective/non_dominated_sort.rs` (utility, transform)

**Analog:** `src/operations/survivor/fitness.rs` (lines 12-13, 46-57) — the canonical cfg-gate pattern for parallel/sequential dual-path.

**Imports pattern** — add rayon import gated on parallel feature (copy from `fitness.rs` lines 12-13):
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;
```

**Core change:** Modify `non_dominated_sort_inner()` (lines 87-131) to add parallel path. The `F` bound must gain `+ Sync` for `par_iter` compatibility:

```rust
// BEFORE (line 87-89):
fn non_dominated_sort_inner<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool,

// AFTER:
fn non_dominated_sort_inner<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,
```

**Parallel inner function** — add after `non_dominated_sort_inner`, gated on the parallel feature. The sequential body is preserved as the else-path:

```rust
// Add between non_dominated_sort_inner and assign_ranks:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
fn non_dominated_sort_inner_parallel<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,
{
    let n = objectives.len();
    // Phase 1: parallel pairwise comparison — each thread handles one i
    let results: Vec<(Vec<usize>, Vec<usize>)> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut dominates = Vec::new();
            let mut dominated_by = Vec::new();
            for j in (i + 1)..n {
                if dom(objectives[i], objectives[j]) {
                    dominates.push(j);
                } else if dom(objectives[j], objectives[i]) {
                    dominated_by.push(j);
                }
            }
            (dominates, dominated_by)
        })
        .collect();

    // Phase 2: sequential merge (O(N²) integer ops, no floating-point)
    let mut domination_count: Vec<usize> = vec![0; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    for i in 0..n {
        dominated_set[i] = results[i].0.clone();
        domination_count[i] = results[i].1.len();
    }
    for i in 0..n {
        for &j in &results[i].1 {
            dominated_set[j].push(i);
        }
    }

    // Front extraction (unchanged from sequential)
    let mut fronts: Vec<Vec<usize>> = vec![];
    let mut current_front: Vec<usize> = (0..n).filter(|&i| domination_count[i] == 0).collect();
    while !current_front.is_empty() {
        let mut next_front: Vec<usize> = vec![];
        for &i in &current_front {
            for &j in &dominated_set[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                }
            }
        }
        fronts.push(current_front);
        current_front = next_front;
    }
    fronts
}
```

**Threshold dispatch** — insert inside `non_dominated_sort_inner` after the `n == 0` early return (after line 94):

```rust
    // Parallel path for large populations
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    if n >= 100 {
        return non_dominated_sort_inner_parallel(objectives, dom);
    }
```

**Constrained variant** — `non_dominated_sort_constrained()` (lines 39-84) gets the same treatment: add a `_parallel` variant with `violations` captured in the closure, same merge logic.

---

### `src/engines/nsga2/non_dominated_sort.rs` (DELETE — D-01)

**Action:** Delete this file entirely. It is an exact duplicate of `multi_objective/non_dominated_sort.rs` (confirmed: identical algorithm, identical function signatures; only difference is extra doc-test examples in the nsga2 copy).

---

### `src/engines/nsga2/mod.rs` (module wiring — D-02)

**Current** (line 109):
```rust
pub mod non_dominated_sort;
```

**Replace with:**
```rust
pub use crate::multi_objective::non_dominated_sort;
```

This re-exports the shared module, preserving all existing import paths like `nsga2::non_dominated_sort::non_dominated_sort`. No downstream files (tests, benches, island engine) need import changes.

---

### `src/engines/island/nsga2.rs` (import fixup — trivial)

**Current** (line 53):
```rust
use crate::nsga2::non_dominated_sort::{assign_ranks, non_dominated_sort};
```

**No change needed** — after D-02 re-export, this path still resolves correctly. The re-export makes `crate::nsga2::non_dominated_sort` an alias for `crate::multi_objective::non_dominated_sort`.

---

### `tests/engines/nsga2/test_non_dominated_sort.rs` (test — no change needed)

**Current imports** (lines 1-5):
```rust
use genetic_algorithms::nsga2::configuration::ObjectiveDirection;
use genetic_algorithms::nsga2::non_dominated_sort::{
    assign_ranks, non_dominated_sort, non_dominated_sort_constrained,
    non_dominated_sort_with_directions,
};
```

**No change needed** — the re-export from `nsga2/mod.rs` preserves these paths. All 7 existing tests continue to compile and pass.

---

### `benches/nsga2.rs` (benchmark — add args only)

**Current args** (lines 27-31):
```rust
#[divan::bench(args = [
    (10usize, 2usize), (10, 3), (10, 5),
    (50, 2), (50, 3), (50, 5),
    (100, 2), (100, 3), (100, 5),
    (500, 2), (500, 3), (500, 5),
])]
```

**Add after line 31:**
```rust
    (1000, 2), (1000, 5),
```

This adds population sizes above the parallelization threshold (100) to demonstrate speedup.

---

## Shared Patterns

### WASM cfg-gate (applies to all parallel paths)
**Source:** `src/operations/survivor/fitness.rs` lines 46-57
**Apply to:** `non_dominated_sort_inner()` and `non_dominated_sort_constrained()` in `multi_objective/non_dominated_sort.rs`
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| { ... });
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| { ... });
```
**Adaptation for this phase:** Instead of dual cfg-gated statements on the same operation, use the threshold-dispatch pattern: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))] if n >= 100 { return parallel_fn(...); }` followed by the sequential body.

### Sync bound on closures
**Source:** `src/engines/nsga2/mod.rs` line 484 (`into_par_iter` with `Arc<ObjectiveFn>`)
**Apply to:** `non_dominated_sort_inner` signature — add `+ Sync` to the `F` bound. All existing call sites satisfy this: `dominates` is a fn pointer (always `Sync`); direction closures capture `&[ObjectiveDirection]` (which is `Sync`).

### Module re-export for backward compatibility
**Source:** `src/engines/multi_objective/mod.rs` line 44-46 (doc-comment explains re-export pattern)
**Apply to:** `nsga2/mod.rs` — replace `pub mod non_dominated_sort;` with `pub use crate::multi_objective::non_dominated_sort;`

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `non_dominated_sort_inner_parallel()` (new function) | utility (algorithm) | transform | New function; pattern derived from RESEARCH.md §Code Examples |

## Metadata

**Analog search scope:** `src/operations/survivor/`, `src/engines/multi_objective/`, `src/engines/nsga2/`
**Files scanned:** 5
**Pattern extraction date:** 2026-06-19
