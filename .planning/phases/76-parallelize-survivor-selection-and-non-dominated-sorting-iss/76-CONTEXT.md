# Phase 76: Parallelize Survivor Selection and Non-Dominated Sorting - Context

**Gathered:** 2026-06-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Parallelize survivor selection and non-dominated sorting with rayon where it pays off, keeping WASM single-threaded fallbacks (cfg-gated per CLAUDE.md). The primary bottleneck is the O(N^2) non-dominated sorting in multi-objective engines. Survivor operators that are already parallel (fitness, age, mu_plus_lambda, mu_comma_lambda) stay as-is. Deterministic crowding and parsimony stay sequential (low benefit).

**Already parallel — no action needed:**
- `fitness_based()` — uses `par_sort_unstable_by`
- `age_based()` — uses `par_sort_unstable_by`
- `mu_plus_lambda()` — uses `par_sort_unstable_by`
- `mu_comma_lambda()` — uses `par_sort_unstable_by`

**Out of scope:**
- Deterministic crowding parallelization (pairwise O(N), low benefit)
- Parsimony pressure parallelization (O(N) wrapper, low benefit)
- Crowding distance computation parallelization (O(N log N) per front, less critical than sort)
- `assign_ranks()` parallelization (O(N) after sort, not the bottleneck)
- Clone reduction in survivor operators (Phase 75 concern)

</domain>

<decisions>
## Implementation Decisions

### Non-dominated sorting deduplication
- **D-01:** Delete `src/engines/nsga2/non_dominated_sort.rs` (the duplicate copy). All engines re-export from `src/engines/multi_objective/non_dominated_sort.rs`. This eliminates code duplication and ensures parallel improvements apply everywhere.
- **D-02:** Update `nsga2/mod.rs` imports to use `crate::engines::multi_objective::non_dominated_sort::*` instead of the local module. Verify no other files reference the nsga2 copy.

### Non-dominated sorting parallelization
- **D-03:** Parallelize the outer `i` loop in `non_dominated_sort_inner()` using `par_iter()`. The inner `j` loop stays sequential per `i` — good cache locality, simpler code.
- **D-04:** Use the standard WASM cfg-gate: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]` for the parallel path, with `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]` for the sequential fallback.
- **D-05:** Population size threshold: parallelize when `n >= 100` (ROADMAP default). Below 100, sequential is faster due to rayon overhead. Use `if n >= 100 { /* parallel path */ } else { /* sequential path */ }` inside the cfg-gated blocks.

### Engine scope
- **D-06:** All 6 multi-objective engines get parallel non-dominated sorting: NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA. Since the sorting function is shared (D-01), all engines benefit automatically.

### Claude's Discretion
- Whether to add a brief comment explaining the parallelization strategy at the `par_iter()` call site
- Exact `par_iter()` vs `into_par_iter()` choice (depends on whether ownership is needed)
- Whether to benchmark before/after as part of this phase or rely on Phase 74 benchmarks

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Non-dominated sorting (primary target)
- `src/engines/multi_objective/non_dominated_sort.rs` — shared implementation to parallelize; `non_dominated_sort_inner()` at line 87 is the O(N^2) bottleneck
- `src/engines/nsga2/non_dominated_sort.rs` — duplicate copy to delete (D-01)

### Engine files that import non-dominated sorting
- `src/engines/nsga2/mod.rs` — imports from local `non_dominated_sort` module; update to use `multi_objective` path
- `src/engines/nsga3/mod.rs` — uses `multi_objective::non_dominated_sort`
- `src/engines/moead/mod.rs` — uses `multi_objective::non_dominated_sort`
- `src/engines/spea2/mod.rs` — uses `multi_objective::non_dominated_sort`
- `src/engines/sms_emoa/mod.rs` — uses `multi_objective::non_dominated_sort`
- `src/engines/ibea/mod.rs` — uses `multi_objective::non_dominated_sort`

### WASM cfg-gate pattern reference
- `src/operations/survivor/fitness.rs` — existing `par_sort_unstable_by` with cfg-gate (lines 46-51, 61-66)
- `CLAUDE.md` — WASM rule: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`

### Survivor operators (already parallel — no changes)
- `src/operations/survivor/fitness.rs` — `par_sort_unstable_by` pattern reference
- `src/operations/survivor/age.rs` — `par_sort_unstable_by` pattern reference
- `src/operations/survivor/mu_plus_lambda.rs` — `par_sort_unstable_by` pattern reference
- `src/operations/survivor/mu_comma_lambda.rs` — `par_sort_unstable_by` pattern reference

### Benchmarks (success criterion measurement)
- `benches/` — existing engine benchmarks for before/after comparison

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `non_dominated_sort_inner()` at `multi_objective/non_dominated_sort.rs:87` — the O(N^2) function to parallelize; clean double-loop structure ready for `par_iter`
- Survivor operator cfg-gate pattern in `fitness.rs` — copy-paste template for the parallel/sequential dual-path

### Established Patterns
- WASM cfg-gate: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]` / `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]`
- `par_sort_unstable_by` for parallel sorting in survivor operators
- `into_par_iter()` for parallel map+collect in engine init and offspring eval
- `par_iter()` for immutable parallel iteration

### Integration Points
- `non_dominated_sort()` / `non_dominated_sort_with_directions()` / `non_dominated_sort_constrained()` — all three public functions call `non_dominated_sort_inner()`; parallelizing the inner function covers all entry points
- All 6 multi-objective engines call these functions in their `run()` loops — no engine-level changes needed if the shared function is parallelized

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard rayon parallelization approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 76-parallelize-survivor-selection-and-non-dominated-sorting*
*Context gathered: 2026-06-19*
