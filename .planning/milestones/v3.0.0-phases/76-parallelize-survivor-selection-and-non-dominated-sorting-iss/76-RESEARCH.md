# Phase 76: Parallelize Survivor Selection and Non-Dominated Sorting - Research

**Researched:** 2026-06-19
**Domain:** Rayon parallelization of O(N²) non-dominated sorting; WASM fallback cfg-gating
**Confidence:** HIGH

## Summary

Phase 76 parallelizes the O(N²) non-dominated sorting algorithm used by all 6 multi-objective engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA). The survivor operators (fitness, age, mu+lambda, mu,lambda) already use `par_sort_unstable_by` and need no changes. The primary target is `non_dominated_sort_inner()` in `src/engines/multi_objective/non_dominated_sort.rs` (line 87).

A prerequisite step (D-01) removes the duplicate `src/engines/nsga2/non_dominated_sort.rs` file, since `nsga2/mod.rs` already imports from `multi_objective::non_dominated_sort` internally (line 114). External consumers (benches, tests, island engine) import via `nsga2::non_dominated_sort` — these must be updated to re-export from the shared module.

**Primary recommendation:** Parallelize the outer `i` loop in `non_dominated_sort_inner()` using `rayon::par_iter()`, collecting per-index results then merging sequentially. The merge is O(N²) integer operations only — the expensive `dom()` floating-point comparisons are fully parallelized. Apply the same pattern to `non_dominated_sort_constrained()`. Use `if n >= 100` threshold per D-05.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Delete `src/engines/nsga2/non_dominated_sort.rs` (the duplicate copy). All engines re-export from `src/engines/multi_objective/non_dominated_sort.rs`. This eliminates code duplication and ensures parallel improvements apply everywhere.
- **D-02:** Update `nsga2/mod.rs` imports to use `crate::engines::multi_objective::non_dominated_sort::*` instead of the local module. Verify no other files reference the nsga2 copy.
- **D-03:** Parallelize the outer `i` loop in `non_dominated_sort_inner()` using `par_iter()`. The inner `j` loop stays sequential per `i` — good cache locality, simpler code.
- **D-04:** Use the standard WASM cfg-gate: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]` for the parallel path, with `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]` for the sequential fallback.
- **D-05:** Population size threshold: parallelize when `n >= 100` (ROADMAP default). Below 100, sequential is faster due to rayon overhead. Use `if n >= 100 { /* parallel path */ } else { /* sequential path */ }` inside the cfg-gated blocks.
- **D-06:** All 6 multi-objective engines get parallel non-dominated sorting: NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA. Since the sorting function is shared (D-01), all engines benefit automatically.

### the agent's Discretion
- Whether to add a brief comment explaining the parallelization strategy at the `par_iter()` call site
- Exact `par_iter()` vs `into_par_iter()` choice (depends on whether ownership is needed)
- Whether to benchmark before/after as part of this phase or rely on Phase 74 benchmarks

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Phase Requirements

> No formal requirement IDs — this is a performance phase that closes GitHub issue #259.

| Criterion | Description | Research Support |
|-----------|-------------|------------------|
| SC-1 | `non_dominated_sort.rs` uses `rayon::par_iter` for dominance comparison on populations ≥100 | Parallelization strategy documented below; threshold per D-05 |
| SC-2 | SurvivorOperator implementations use parallel ranking where order-independent | Already parallel — `par_sort_unstable_by` in all 4 operators |
| SC-3 | WASM fallback preserves sequential path | Standard cfg-gate pattern used throughout codebase (80+ occurrences) |
| SC-4 | `cargo bench` confirms measurable improvement on ZDT1, DTLZ2 at pop ≥200 | Existing `benches/nsga2.rs` covers pop sizes 10–500; extend to 1000+ |
| SC-5 | All existing tests pass; `cargo check --target wasm32-unknown-unknown` passes | WASM target confirmed installed; current build compiles clean |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Non-dominated sorting | API/Backend | — | Pure computation on objective vectors; no I/O or UI concern |
| Survivor selection ranking | API/Backend | — | Sorting chromosomes by fitness; already parallel via `par_sort` |
| WASM fallback gating | Build/Config | API/Backend | `cfg` attributes at compile time; no runtime decision |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rayon | 1.10 | Parallel iterators (`par_iter`, `into_par_iter`) | Already a dependency; `par_sort_unstable_by` used in 4 survivor operators; `into_par_iter` used in engine init/offspring eval |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| — | — | — | No new dependencies needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rayon `par_iter` | Manual thread spawning | rayon handles work-stealing and thread pool; manual spawning adds complexity with no benefit |
| rayon `par_iter` | `crossbeam` scoped threads | crossbeam is lower-level; rayon already wraps it with better API |

**Installation:** No new dependencies — rayon is already in `Cargo.toml`:
```toml
rayon = { version = "1.10", optional = true }
```

**Version verification:** rayon 1.10 confirmed in `Cargo.toml` line 44. `par_iter()` and `into_par_iter()` are stable APIs available since rayon 1.0.

## Package Legitimacy Audit

> No new packages installed — phase only modifies existing code.

## Architecture Patterns

### Parallelization Strategy for `non_dominated_sort_inner()`

The algorithm has an O(N²) double loop: `for i in 0..n { for j in (i+1)..n { ... } }`. The `dom()` calls inside are the expensive part (floating-point comparisons on objective vectors).

**Parallel approach (D-03):** Parallelize the outer `i` loop. Each thread processes one `i` value and runs the inner `j` loop sequentially (good cache locality per D-03). Collect per-index results, then merge sequentially.

```rust
// Phase 1: parallel pairwise comparison
let results: Vec<(Vec<usize>, Vec<usize>)> = (0..n)
    .into_par_iter()
    .map(|i| {
        let mut dominates = Vec::new();    // j > i where i dominates j
        let mut dominated_by = Vec::new(); // j > i where j dominates i
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
for i in 0..n {
    dominated_set[i] = results[i].0.clone();
    domination_count[i] = results[i].1.len();
}
for i in 0..n {
    for &j in &results[i].1 {
        dominated_set[j].push(i);  // j dominates i → add i to j's dominated set
    }
}
```

**Why this works:**
- Each thread writes only to its own `results[i]` — no cross-thread writes during parallel phase
- The merge phase is O(N²) but only integer operations (Vec push, usize increment)
- The expensive `dom()` calls (floating-point comparisons on `&[f64]` slices) are fully parallelized
- Memory: O(N²) for results vector. For N=500: ~125K entries × ~24 bytes ≈ 3MB. For N=1000: ~500K × 24 bytes ≈ 12MB. Acceptable.

**Threshold (D-05):** `if n >= 100` — below this, rayon dispatch overhead exceeds parallelization benefit.

### Same pattern for `non_dominated_sort_constrained()`

The constrained variant has the same double-loop structure but also accesses `violations[i]` and `violations[j]`. The parallelization is identical — just add `violations` to the closure captures:

```rust
let results: Vec<(Vec<usize>, Vec<usize>)> = (0..n)
    .into_par_iter()
    .map(|i| {
        let vi = violations.get(i).copied().unwrap_or(0.0);
        let mut dominates = Vec::new();
        let mut dominated_by = Vec::new();
        for j in (i + 1)..n {
            let vj = violations.get(j).copied().unwrap_or(0.0);
            if constrained_dominates(objectives[i], objectives[j], vi, vj, directions) {
                dominates.push(j);
            } else if constrained_dominates(objectives[j], objectives[i], vj, vi, directions) {
                dominated_by.push(j);
            }
        }
        (dominates, dominated_by)
    })
    .collect();
// ... same merge ...
```

### WASM cfg-gate pattern (D-04)

Standard pattern used 80+ times across the codebase:

```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

pub fn non_dominated_sort_inner<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,  // Sync bound needed for par_iter
{
    let n = objectives.len();
    if n == 0 { return vec![]; }

    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    if n >= 100 {
        return non_dominated_sort_inner_parallel(objectives, dom);
    }

    // Sequential path (existing code, unchanged)
    // ...
}
```

The `Sync` bound on `F` is only needed for the parallel path. Since all call sites pass either fn pointers (`dominates`) or closures capturing `&[ObjectiveDirection]` (which is `Sync`), this is safe.

### Recommended Project Structure (no changes)

```
src/engines/multi_objective/
├── non_dominated_sort.rs    # MODIFY: add parallel path
├── pareto.rs                # unchanged
├── mod.rs                   # unchanged
└── indicators/              # unchanged

src/engines/nsga2/
├── mod.rs                   # MODIFY: remove pub mod non_dominated_sort, add re-export
├── non_dominated_sort.rs    # DELETE (D-01)
├── pareto.rs                # unchanged (not in scope for D-01)
├── crowding_distance.rs     # unchanged
└── configuration.rs         # unchanged
```

### Anti-Patterns to Avoid
- **Hand-rolling thread pools:** rayon manages this; don't use `std::thread::spawn`
- **Using Mutex for `dominated_set`:** Per-element Mutex is wasteful; the collect-then-merge approach avoids it entirely
- **Parallelizing the front extraction loop:** The front extraction is O(N) — sequential is faster than rayon overhead

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parallel iteration | Manual thread spawning + join | `rayon::par_iter` | Work-stealing, thread pool management, fork-join |
| Parallel sorting | Custom merge-sort | `rayon::par_sort_unstable_by` | Already used in survivor operators; battle-tested |
| WASM detection | Runtime feature check | `#[cfg(target_arch = "wasm32")]` | Compile-time; zero runtime cost |

## Common Pitfalls

### Pitfall 1: Missing `Sync` bound on closure parameter
**What goes wrong:** `non_dominated_sort_inner<F>` currently requires `F: Fn(...)`. Adding `par_iter()` requires `F: Sync`. If the bound isn't added, compilation fails with opaque error about `F` not being `Send + Sync`.
**Why it happens:** The sequential version doesn't need `Sync` — closures are only called on the main thread.
**How to avoid:** Add `+ Sync` to the `F` bound in `non_dominated_sort_inner`. All existing call sites satisfy this bound.
**Warning signs:** Compiler error: "the trait `Sync` is not implemented for `F`"

### Pitfall 2: Removing `nsga2::non_dominated_sort` breaks public API
**What goes wrong:** Deleting the module without re-exporting breaks `use genetic_algorithms::nsga2::non_dominated_sort::*` for external users.
**Why it happens:** The nsga2 module currently has `pub mod non_dominated_sort;` which exposes the duplicate. Removing it without `pub use` breaks the path.
**How to avoid:** After deleting the file and removing `pub mod non_dominated_sort;`, add `pub use crate::multi_objective::non_dominated_sort::*;` to `nsga2/mod.rs`. Verify with `cargo test --doc` that all doc-tests using `nsga2::non_dominated_sort::*` still compile.
**Warning signs:** Doc-test failures referencing `nsga2::non_dominated_sort`

### Pitfall 3: `dominated_set[j].push(i)` merge not accounted for
**What goes wrong:** In the parallel path, each thread computes `results[i].1` (which j > i dominate i). The merge must add `i` to `dominated_set[j]` for each such `j`. Forgetting this step means `domination_count` values are correct but `dominated_set` is incomplete — the front extraction loop produces wrong fronts.
**Why it happens:** The merge has two parts (direct results + cross-thread contributions), and it's easy to implement only the first.
**How to avoid:** After the direct merge loop, add the cross-thread merge: `for i in 0..n { for &j in &results[i].1 { dominated_set[j].push(i); } }`.
**Warning signs:** Test failures where fronts have wrong sizes or missing individuals.

### Pitfall 4: `into_par_iter` vs `par_iter` choice
**What goes wrong:** Using `par_iter()` on a range produces `&usize` references; `into_par_iter()` produces owned `usize` values. The closure needs owned indices for `dominated_set[i]` indexing.
**Why it happens:** Habitual use of `par_iter()` without considering ownership.
**How to avoid:** Use `(0..n).into_par_iter()` which yields owned `usize` values. Alternatively, `(0..n).par_iter().copied()`.
**Warning signs:** Compiler errors about borrowing from captured variables.

### Pitfall 5: WASM check without `parallel` feature
**What goes wrong:** The cfg-gate `#[cfg(target_arch = "wasm32")]` alone would disable parallelism even when `feature = "parallel"` is enabled on non-WASM targets.
**Why it happens:** Confusing the two conditions (WASM target vs parallel feature).
**How to avoid:** Always use the compound gate: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]` for parallel path, `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]` for sequential fallback.
**Warning signs:** Parallel code runs on WASM (crashes) or sequential code runs on native (no speedup).

## Code Examples

### Parallel non_dominated_sort_inner (the core change)

```rust
// Source: Pattern derived from existing codebase + rayon docs
// File: src/engines/multi_objective/non_dominated_sort.rs

#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

/// Generic non-dominated sorting driven by a domination predicate.
fn non_dominated_sort_inner<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,
{
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    // Parallel path for large populations (D-03, D-04, D-05)
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    if n >= 100 {
        return non_dominated_sort_inner_parallel(objectives, dom);
    }

    // Sequential path (existing code)
    let mut domination_count: Vec<usize> = vec![0; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    let mut fronts: Vec<Vec<usize>> = vec![];

    for i in 0..n {
        for j in (i + 1)..n {
            if dom(objectives[i], objectives[j]) {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if dom(objectives[j], objectives[i]) {
                dominated_set[j].push(i);
                domination_count[i] += 1;
            }
        }
    }

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

#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
fn non_dominated_sort_inner_parallel<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,
{
    let n = objectives.len();

    // Phase 1: parallel pairwise comparison — each i processes its j loop
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

    // Phase 2: sequential merge
    let mut domination_count: Vec<usize> = vec![0; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];

    for i in 0..n {
        dominated_set[i] = results[i].0.clone();
        domination_count[i] = results[i].1.len();
    }

    // Cross-thread merge: add i to dominated_set[j] for each j that dominates i
    for i in 0..n {
        for &j in &results[i].1 {
            dominated_set[j].push(i);
        }
    }

    // Front extraction (unchanged)
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

### WASM cfg-gate reference (existing pattern)

```rust
// Source: src/operations/survivor/fitness.rs lines 46-57
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| {
    b.fitness()
        .partial_cmp(&a.fitness())
        .unwrap_or(std::cmp::Ordering::Equal)
});
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| {
    b.fitness()
        .partial_cmp(&a.fitness())
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

### Re-export pattern for backward compatibility

```rust
// Source: src/engines/nsga2/mod.rs — after D-01
// Remove: pub mod non_dominated_sort;
// Add:
pub use crate::multi_objective::non_dominated_sort::*;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Sequential O(N²) non-dominated sort | Parallel outer loop + sequential merge | This phase | N²/threads speedup for N ≥ 100 |
| Duplicate `nsga2::non_dominated_sort.rs` | Shared `multi_objective::non_dominated_sort.rs` | This phase (D-01) | Single codebase for all engines |

**Deprecated/outdated:**
- `src/engines/nsga2/non_dominated_sort.rs`: Duplicate file — deleted in D-01. All imports redirected to `multi_objective::non_dominated_sort`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All 6 multi-objective engines import non-dominated sorting from `multi_objective::non_dominated_sort` (or will after D-02) | Architecture | If any engine has its own copy, it won't benefit from parallelization. Verified by grep: nsga3, moead, spea2, sms_emoa, ibea all use `multi_objective::non_dominated_sort`. NSGA-II uses it internally (line 114). Island/nsga2 uses `nsga2::non_dominated_sort` (the duplicate) — must be updated. |
| A2 | `dom()` closure is `Sync` for all call sites | Pitfall 1 | If not, parallel path won't compile. Verified: `dominates` is a fn pointer (always `Sync`); direction closures capture `&[ObjectiveDirection]` (which is `Sync`); constrained closures capture `&[ObjectiveDirection]` + `&[f64]` (both `Sync`). |
| A3 | The `nsga2::pareto` module is also a duplicate of `multi_objective::pareto` | D-01 scope | D-01 only mentions `non_dominated_sort.rs`. The `pareto.rs` duplicate is NOT in scope for this phase. If someone expects both to be cleaned up, they'll be surprised. Risk: low — D-01 is explicit about scope. |
| A4 | `cargo check --target wasm32-unknown-unknown` passes currently | SC-5 | Verified: confirmed installed and compiles clean. |

## Open Questions

1. **Should `nsga2::pareto` also be consolidated?**
   - What we know: `nsga2/pareto.rs` is a duplicate of `multi_objective/pareto.rs` (identical code with extra doc-tests)
   - What's unclear: D-01 scope is limited to `non_dominated_sort.rs` only
   - Recommendation: Out of scope for this phase. Note as technical debt for a future cleanup phase.

2. **Benchmark before/after within this phase?**
   - What we know: `benches/nsga2.rs` already benchmarks `non_dominated_sort` at pop sizes 10–500
   - What's unclear: Whether to extend benchmarks to 1000+ and run before/after within this phase
   - Recommendation: Extend bench args to include `(1000, 2)` and `(1000, 5)` to demonstrate speedup at the parallelization threshold. Run before/after as part of verification.

3. **Memory overhead for large populations**
   - What we know: The parallel path allocates O(N²) for results vector
   - What's unclear: Whether this causes issues at N=1000+ (12MB)
   - Recommendation: Acceptable for the use case. The sequential path is always available for memory-constrained environments.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rayon | Parallel non-dominated sort | ✓ | 1.10 (in Cargo.toml) | Sequential path (cfg-gated) |
| wasm32-unknown-unknown target | WASM check (SC-5) | ✓ | installed | — |
| cargo bench | SC-4 verification | ✓ | standard | — |

**Missing dependencies with no fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Built-in `cargo test` |
| Config file | none |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --doc` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | Parallel NDS produces same fronts as sequential | unit | `cargo test --test test_non_dominated_sort` | ✅ tests/engines/nsga2/test_non_dominated_sort.rs |
| SC-2 | Survivor operators still parallel | unit | `cargo test` (existing tests cover all operators) | ✅ |
| SC-3 | WASM target compiles | smoke | `cargo check --target wasm32-unknown-unknown` | ✅ |
| SC-4 | Benchmark shows improvement | manual | `cargo bench --bench nsga2` | ✅ benches/nsga2.rs |
| SC-5 | All tests pass | smoke | `cargo test` | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo test --doc && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green + `cargo bench --bench nsga2` shows improvement at N≥200

### Wave 0 Gaps
- [ ] Add `(1000, 2)` and `(1000, 5)` args to `benches/nsga2.rs` non_dominated_sort bench to demonstrate parallelization benefit at scale

## Security Domain

> Omitted — this phase involves no authentication, session management, access control, input validation, or cryptography. Pure performance optimization of a mathematical algorithm.

## Sources

### Primary (HIGH confidence)
- `src/engines/multi_objective/non_dominated_sort.rs` — the file to parallelize; read in full
- `src/engines/nsga2/non_dominated_sort.rs` — the duplicate to delete (D-01); read in full
- `src/operations/survivor/fitness.rs` — cfg-gate pattern reference; read in full
- `src/engines/nsga2/mod.rs` — import structure and re-exports; read in full
- `Cargo.toml` — dependency versions and feature flags; read in full

### Secondary (MEDIUM confidence)
- rayon documentation — `par_iter()` requires `F: Send + Sync` on closures; `into_par_iter()` consumes the iterator

### Tertiary (LOW confidence)
- None — all findings verified against codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — rayon already a dependency; no new packages
- Architecture: HIGH — parallelization strategy derived from algorithm analysis and existing codebase patterns
- Pitfalls: HIGH — all pitfalls identified from codebase patterns and rayon API requirements

**Research date:** 2026-06-19
**Valid until:** 2026-07-19 (stable — no fast-moving dependencies)
