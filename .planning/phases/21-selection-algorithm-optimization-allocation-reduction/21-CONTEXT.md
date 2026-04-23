# Phase 21: Selection Algorithm Optimization + Allocation Reduction - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Four optimizations to the hot generation loop:
1. **ALGO-03**: Rank Selection replaces O(n) `.iter().position()` with `partition_point()` binary search
2. **ALGO-04**: Boltzmann Selection replaces O(n) `.iter().position()` with `partition_point()` binary search
3. **ALLOC-01**: Two redundant `fitness_values: Vec<f64>` allocations in `ga.rs` generation loop merged into one
4. **ALLOC-02**: Niching distance matrix computation made on-the-fly via a new `apply_fitness_sharing_with_dna()` function — no full O(n²) matrix allocated

No public API breaking changes. No benchmarks (deferred to Phase 25).

</domain>

<decisions>
## Implementation Decisions

### ALGO-03: Rank Selection binary search
- Replace `cumulative.iter().position(|&(_, cp)| cp >= r).unwrap_or(n - 1)` with `partition_point()` binary search
- `cumulative` is `Vec<(original_idx, f64)>` — use `cumulative.partition_point(|&(_, cp)| cp < r)`, then clamp to `n - 1` for float drift safety
- Claude handles the exact edge case (clamp vs min)

### ALGO-04: Boltzmann Selection binary search
- Replace `cumulative.iter().position(|&cp| cp >= r).unwrap_or(n - 1)` with `partition_point()` binary search
- `cumulative` is `Vec<f64>` — use `cumulative.partition_point(|&cp| cp < r)`, then clamp to `n - 1`
- Last entry is already clamped to `1.0` in the existing code — float drift protection preserved

### ALLOC-01: Fitness values collected once per generation
- Collect `fitness_values: Vec<f64>` ONCE before the niching block in the generation loop in `src/ga.rs`
- Pass the same `Vec` into the niching block (currently re-collects at line ~833) and into `GenerationStats::from_fitness_values()` (currently re-collects at line ~914)
- Scope: only the two in-loop allocations — extension does not collect fitness independently, no extension change needed
- The niching block already modifies `fitness_values` in-place (apply_fitness_sharing adjusts them), so order matters: collect → niching → stats (stats see the shared/adjusted values, which is the existing behavior)

### ALLOC-02: Niching distance matrix on-the-fly
- Add a new `pub fn apply_fitness_sharing_with_dna<G, F>(fitness_values: &mut [f64], dna_slices: &[&[G]], distance_fn: F, sigma_share: f64, alpha: f64)` in `src/niching/sharing.rs`
- This function computes the sharing function value for each (i, j) pair on-the-fly instead of pre-allocating a full `Vec<Vec<f64>>` distance matrix
- `ga.rs` switches from the two-call pattern (`compute_distance_matrix` → `apply_fitness_sharing`) to the single call `apply_fitness_sharing_with_dna`
- Keep `apply_fitness_sharing(&distances)` and `compute_distance_matrix` as existing pub functions — no breaking change for external callers
- `compute_distance_matrix` may remain unused internally after this change (external API only)

### Claude's Discretion
- Exact `partition_point` clamping strategy (`.min(n-1)` vs explicit check)
- Whether to add a `#[deprecated]` annotation to `compute_distance_matrix` since it's no longer used internally
- Variable naming in the merged fitness collection

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — ALGO-03, ALGO-04, ALLOC-01, ALLOC-02 acceptance criteria

### Files to modify
- `src/operations/selection/rank.rs` — replace `.iter().position()` with `partition_point()` (ALGO-03)
- `src/operations/selection/boltzmann.rs` — replace `.iter().position()` with `partition_point()` (ALGO-04)
- `src/ga.rs` — merge two `fitness_values` allocations into one (ALLOC-01); switch niching from `compute_distance_matrix` + `apply_fitness_sharing` to `apply_fitness_sharing_with_dna` (ALLOC-02)
- `src/niching/sharing.rs` — add `apply_fitness_sharing_with_dna` function (ALLOC-02)

### Key code locations
- `src/ga.rs` line ~833: niching block with first `fitness_values` allocation
- `src/ga.rs` line ~914: stats block with second `fitness_values` allocation
- `src/operations/selection/rank.rs` line ~72: `cumulative.iter().position()` to replace
- `src/operations/selection/boltzmann.rs` line ~82: `cumulative.iter().position()` to replace
- `src/niching/sharing.rs` lines 72–97: `apply_fitness_sharing` (keep as-is)
- `src/niching/sharing.rs` lines 120+: `compute_distance_matrix` (keep as-is, keep pub)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `partition_point()` — Rust std slice method, available on any sorted slice; returns first index where predicate is false (i.e. first index where `cp >= r`)
- `apply_fitness_sharing(fitness_values, &distances, sigma_share, alpha)` — existing pub API, keep unchanged
- `compute_distance_matrix(dna_slices, distance_fn)` — existing pub API, keep unchanged
- The distance closure in `ga.rs` (Hamming distance over gene IDs) can be reused directly in `apply_fitness_sharing_with_dna`

### Established Patterns
- `boltzmann.rs` already clamps last cumulative entry to `1.0` — `partition_point` result still needs `.min(n-1)` for the edge case where `r > last cumulative` due to extreme float drift
- Both selection files use the same `cumulative.iter().position(|..| cp >= r).unwrap_or(n-1)` pattern — identical fix in both
- `fitness_values` in the niching block is `mut` (apply_fitness_sharing modifies in-place) — the merged collection must also be `mut` so stats get the post-sharing values (matching current behavior)

### Integration Points
- `GenerationStats::from_fitness_values(i, &fitness_values, is_maximization)` — signature unchanged, just reuses the existing Vec
- The niching block reads/writes `fitness_values` then writes back to chromosomes via `set_fitness` — the merged Vec must appear before the niching block in the generation loop

</code_context>

<specifics>
## Specific Ideas

- The merged `fitness_values` collection should be `let mut fitness_values: Vec<f64> = ...` placed just before the niching `if let Some(ref niching_config)` block. If niching is disabled, it's still used by stats — no conditional needed.
- `apply_fitness_sharing_with_dna` inner loop: for each pair `(i, j)` where `j != i`, call `sharing_function(distance_fn(dna[i], dna[j]), sigma_share, alpha)` and accumulate `niche_count[i]`. Same O(n²) compute, zero O(n²) allocation.

</specifics>

<deferred>
## Deferred Ideas

- `#[deprecated]` on `compute_distance_matrix` — Claude can add this or defer to Phase 23 cleanup; mentioned as Claude's discretion
- Parallelizing the O(n²) niching loop with rayon — out of scope for this phase
- Criterion benchmarks — deferred to Phase 25

</deferred>

---

*Phase: 21-selection-algorithm-optimization-allocation-reduction*
*Context gathered: 2026-03-31*
