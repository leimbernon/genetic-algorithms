# Phase 21: Selection Algorithm Optimization + Allocation Reduction - Research

**Researched:** 2026-03-31
**Domain:** Rust slice methods (`partition_point`), allocation patterns in hot loops, niching on-the-fly computation
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **ALGO-03**: Replace `cumulative.iter().position(|&(_, cp)| cp >= r).unwrap_or(n - 1)` with `cumulative.partition_point(|&(_, cp)| cp < r)`, then clamp to `n - 1`. `cumulative` is `Vec<(usize, f64)>`.
- **ALGO-04**: Replace `cumulative.iter().position(|&cp| cp >= r).unwrap_or(n - 1)` with `cumulative.partition_point(|&cp| cp < r)`, then clamp to `n - 1`. `cumulative` is `Vec<f64>`. Last entry already clamped to `1.0`.
- **ALLOC-01**: Collect `fitness_values: Vec<f64>` ONCE before the niching block in `src/ga.rs`. Pass it into the niching block (currently re-collects at line ~833) and into `GenerationStats::from_fitness_values()` (currently re-collects at line ~914). Extension does NOT need changes. The `Vec` must be `mut` because niching modifies it in-place. Order: collect → niching → stats.
- **ALLOC-02**: Add `pub fn apply_fitness_sharing_with_dna<G, F>(fitness_values: &mut [f64], dna_slices: &[&[G]], distance_fn: F, sigma_share: f64, alpha: f64)` in `src/niching/sharing.rs`. Computes (i, j) sharing on-the-fly — no O(n²) matrix. Keep `apply_fitness_sharing` and `compute_distance_matrix` as existing pub functions (no breaking change).
- No public API breaking changes. No benchmarks (deferred to Phase 25).

### Claude's Discretion
- Exact `partition_point` clamping strategy (`.min(n-1)` vs explicit check)
- Whether to add a `#[deprecated]` annotation to `compute_distance_matrix` since it's no longer used internally
- Variable naming in the merged fitness collection

### Deferred Ideas (OUT OF SCOPE)
- `#[deprecated]` on `compute_distance_matrix` — may add or defer to Phase 23
- Parallelizing the O(n²) niching loop with rayon
- Criterion benchmarks — deferred to Phase 25
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ALGO-03 | Rank Selection uses `partition_point()` binary search instead of O(M×N) linear scan | `partition_point` semantics verified; exact predicate form and clamping strategy documented in Code Examples |
| ALGO-04 | Boltzmann Selection uses binary search and single-pass cumulative probability computation | Same `partition_point` fix; Boltzmann-specific notes on last-entry `1.0` clamp documented |
| ALLOC-01 | Fitness values collected once per generation and reused across niching and stats (eliminates 2 redundant O(n) allocations) | Exact insertion point identified (before line ~830 niching block); `mut` requirement and order dependency documented |
| ALLOC-02 | Niching distance matrix computed on-the-fly instead of full O(n²) memory allocation | New function signature and inner loop pattern documented; existing pub API preserved |
</phase_requirements>

---

## Summary

Phase 21 makes four targeted changes inside two hot paths: the roulette-wheel sampling in Rank and Boltzmann selection, and the generation loop in `ga.rs` that applies niching then records stats. All four changes are mechanical refactors of existing, working code — no new algorithms or public API additions are required beyond the single new `apply_fitness_sharing_with_dna` function.

The `partition_point` change in both selection files is a one-line swap. The linear scan `iter().position(|..| cp >= r).unwrap_or(n-1)` is O(n) per sample; `partition_point` on a sorted cumulative slice is O(log n) per sample, making the inner loop O(k log n) instead of O(k·n). Both cumulative vectors are already built in sorted ascending order by construction, so the precondition for binary search is guaranteed.

The ALLOC-01 and ALLOC-02 changes are inside `ga.rs` and `src/niching/sharing.rs`. They do not alter any public trait, struct, or enum — they are purely internal implementation changes.

**Primary recommendation:** Implement each change in a separate task in the order ALGO-03 → ALGO-04 → ALLOC-01 → ALLOC-02 to keep each diff minimal and bisect-friendly.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std (`slice::partition_point`) | stable since 1.52 | Binary search on sorted slice by predicate | Zero-dependency; part of std; exactly matches the use case |
| `rand` (already in Cargo.toml) | existing | RNG for roulette sampling | Already used in both selection files |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `log` (already in Cargo.toml) | existing | Structured event logging | Preserve existing `trace!` and `debug!` calls as-is |

No new dependencies are required for this phase.

---

## Architecture Patterns

### Files to Modify

```
src/
├── operations/selection/rank.rs       # ALGO-03: partition_point swap
├── operations/selection/boltzmann.rs  # ALGO-04: partition_point swap
├── niching/sharing.rs                 # ALLOC-02: add apply_fitness_sharing_with_dna
└── ga.rs                              # ALLOC-01: merge fitness_values; ALLOC-02: switch call
```

### Pattern 1: partition_point Binary Search (ALGO-03, ALGO-04)

**What:** `slice::partition_point(predicate)` returns the first index where the predicate is `false`. For a cumulative probability vector sorted ascending, `partition_point(|&cp| cp < r)` returns the first index where `cp >= r` — identical semantics to `position(|&cp| cp >= r)` but O(log n) instead of O(n).

**When to use:** Any sorted slice where you need the insertion point of a value.

**Clamping rationale:** `partition_point` may return `n` if all entries satisfy the predicate (i.e. `r` lies beyond the last cumulative value due to float drift). `.min(n - 1)` is the idiomatic clamp — a single expression, no branch, correct for all cases. The existing `.unwrap_or(n - 1)` on `position` handled the same edge case; `.min(n - 1)` is the direct equivalent for `partition_point`.

**rank.rs — before (line 70–73):**
```rust
let idx = cumulative
    .iter()
    .position(|&(_, cp)| cp >= r)
    .unwrap_or(n - 1);
selected.push(cumulative[idx].0);
```

**rank.rs — after:**
```rust
let idx = cumulative
    .partition_point(|&(_, cp)| cp < r)
    .min(n - 1);
selected.push(cumulative[idx].0);
```

**boltzmann.rs — before (line 94):**
```rust
let idx = cumulative.iter().position(|&cp| cp >= r).unwrap_or(n - 1);
selected.push(idx);
```

**boltzmann.rs — after:**
```rust
let idx = cumulative.partition_point(|&cp| cp < r).min(n - 1);
selected.push(idx);
```

Note: `partition_point` is called directly on the `Vec<f64>` (or `Vec<(usize, f64)>`), not on `.iter()`. The predicate receives `&f64` or `&(usize, f64)` by reference as the slice element type.

### Pattern 2: Merged Fitness Collection (ALLOC-01)

**What:** A single `let mut fitness_values: Vec<f64>` placed immediately before the niching `if let Some(ref niching_config)` block replaces two separate `.iter().map(|c| c.fitness()).collect()` calls (one inside the niching block at line ~833, one at the stats block at line ~914).

**Placement in ga.rs generation loop (after elite reinsertion, before niching block):**
```rust
// Collect fitness values once; reused by niching and stats.
let mut fitness_values: Vec<f64> = self
    .population
    .chromosomes
    .iter()
    .map(|c| c.fitness())
    .collect();

// Apply niching / fitness sharing if configured
if let Some(ref niching_config) = self.configuration.niching_configuration {
    if niching_config.enabled {
        // Remove the local fitness_values collection here — use the outer one.
        // ... (dna_slices extraction stays as-is)
        apply_fitness_sharing_with_dna(
            &mut fitness_values,
            &dna_slices,
            |dna_a, dna_b| { /* existing hamming closure */ },
            niching_config.sigma_share,
            niching_config.alpha,
        );
        for (chromosome, &shared_fitness) in self
            .population
            .chromosomes
            .iter_mut()
            .zip(fitness_values.iter())
        {
            chromosome.set_fitness(shared_fitness);
        }
    }
}

// ... best chromosome scan ...

// Stats — reuse the same fitness_values (already adjusted by niching if enabled)
let gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
```

**Critical ordering invariant:** Niching modifies `fitness_values` in-place. Stats must consume it *after* niching, so stats record the post-sharing fitness values — matching current behavior exactly.

**When niching is disabled:** `fitness_values` is still collected (no conditional needed) and used only by stats. This is the same number of allocations as before for the no-niching path, and one fewer when niching is active.

### Pattern 3: On-the-Fly Fitness Sharing (ALLOC-02)

**What:** A new `apply_fitness_sharing_with_dna` function in `src/niching/sharing.rs` computes the sharing niche count for each individual by iterating all pairs on-the-fly, without constructing an O(n²) distance matrix. Memory footprint: O(n) for `niche_count` vs O(n²) for the matrix.

**Function signature (to add to sharing.rs):**
```rust
pub fn apply_fitness_sharing_with_dna<G, F>(
    fitness_values: &mut [f64],
    dna_slices: &[&[G]],
    distance_fn: F,
    sigma_share: f64,
    alpha: f64,
)
where
    F: Fn(&[G], &[G]) -> f64,
{
    let n = fitness_values.len();
    if n == 0 {
        return;
    }

    let raw_fitnesses: Vec<f64> = fitness_values.to_vec();
    let mut niche_counts = vec![0.0f64; n];

    for i in 0..n {
        for j in 0..n {
            let d = if i < dna_slices.len() && j < dna_slices.len() {
                distance_fn(dna_slices[i], dna_slices[j])
            } else {
                f64::INFINITY
            };
            niche_counts[i] += sharing_function(d, sigma_share, alpha);
        }
    }

    for i in 0..n {
        if niche_counts[i] > 0.0 {
            fitness_values[i] = raw_fitnesses[i] / niche_counts[i];
        }
    }

    debug!(
        target: "niching_events",
        "Applied fitness sharing (with_dna) to {} individuals with sigma_share={}, alpha={}",
        n,
        sigma_share,
        alpha
    );
}
```

**Calling site in ga.rs replaces:**
```rust
// REMOVE:
let distances = crate::niching::sharing::compute_distance_matrix(&dna_slices, |...| {...});
crate::niching::sharing::apply_fitness_sharing(&mut fitness_values, &distances, ...);

// ADD:
crate::niching::sharing::apply_fitness_sharing_with_dna(
    &mut fitness_values,
    &dna_slices,
    |dna_a: &[U::Gene], dna_b: &[U::Gene]| {
        let max_len = dna_a.len().max(dna_b.len());
        if max_len == 0 { return 0.0; }
        let mut diff = 0usize;
        for idx in 0..max_len {
            let id_a = dna_a.get(idx).map(|g| g.id()).unwrap_or(-1);
            let id_b = dna_b.get(idx).map(|g| g.id()).unwrap_or(-1);
            if id_a != id_b { diff += 1; }
        }
        diff as f64
    },
    niching_config.sigma_share,
    niching_config.alpha,
);
```

**The `dna_slices` extraction stays as-is** — only the two-call pattern (`compute_distance_matrix` + `apply_fitness_sharing`) is replaced by the single call.

### Anti-Patterns to Avoid

- **Calling `partition_point` on an iterator:** `partition_point` is a method on `&[T]` (slice), not on `Iterator`. Call it directly on the vec/slice, e.g. `cumulative.partition_point(...)`, not `cumulative.iter().partition_point(...)`.
- **Removing `compute_distance_matrix` or `apply_fitness_sharing`:** These are public API. Keep them. The internal call in `ga.rs` switches to `apply_fitness_sharing_with_dna`; external callers continue to work.
- **Placing the merged `fitness_values` after the niching block:** Stats would then read pre-sharing values, changing observable behavior. Must be before niching.
- **Making `fitness_values` non-mut:** Niching modifies it in-place. Must be `let mut`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Binary search on sorted CDF | Custom binary search loop | `slice::partition_point` | Std library; correctly handles edge cases; single expression |
| O(n²) niche count accumulation | New data structure / matrix | Simple nested loop with `niche_counts: Vec<f64>` | No allocation benefit from complexity; the O(n²) compute is inherent to niching |

**Key insight:** `partition_point` is the idiomatic Rust way to find an insertion point in a sorted slice by predicate. It exists precisely for this use case.

---

## Common Pitfalls

### Pitfall 1: partition_point Returns n (not n-1) at Boundary
**What goes wrong:** When `r` is exactly `1.0` or floats push all cumulative values below `r`, `partition_point` returns `n` (one past the end). Indexing `cumulative[n]` panics.
**Why it happens:** `partition_point` returns the insertion point, which can be equal to the slice length.
**How to avoid:** Always clamp: `let idx = cumulative.partition_point(...).min(n - 1);`
**Warning signs:** Intermittent index-out-of-bounds panics, especially at extreme RNG values.

### Pitfall 2: Predicate Direction Mismatch
**What goes wrong:** Using `|cp| cp <= r` or `|cp| cp > r` instead of `|cp| cp < r` yields wrong result — off-by-one in which individual is selected.
**Why it happens:** `partition_point` finds the first index where the predicate is `false`. The predicate must be `cp < r` (all-true zone = "still less than r") so that the returned index is the first `cp >= r`.
**How to avoid:** Keep predicate as `cp < r` matching the pattern in CONTEXT.md.
**Warning signs:** Statistical selection tests failing (wrong distribution shape).

### Pitfall 3: fitness_values Declared After Niching Block
**What goes wrong:** Stats would see pre-niching fitness values, silently changing the statistics recorded per generation.
**Why it happens:** Mis-reading the code structure around lines 830–920 in ga.rs.
**How to avoid:** Place `let mut fitness_values: Vec<f64> = ...` immediately before the `if let Some(ref niching_config)` line.
**Warning signs:** Existing `test_ga.rs` integration tests may still pass (they don't always verify stats content), so this is a silent behavioral regression.

### Pitfall 4: Borrowing dna_slices and fitness_values Simultaneously
**What goes wrong:** `apply_fitness_sharing_with_dna` takes `&mut fitness_values` and `&dna_slices` — both from `self.population.chromosomes`. The borrow checker may reject if these are not separated.
**Why it happens:** `dna_slices` is built from `self.population.chromosomes.iter().map(|c| c.dna())`, and `fitness_values` is already its own `Vec<f64>` — it does not borrow from chromosomes. This is fine. The `fitness_values` and `dna_slices` are independent owned/borrowed values.
**How to avoid:** `dna_slices: Vec<&[U::Gene]>` borrows chromosomes immutably. `fitness_values: Vec<f64>` is an owned copy. The mutation is on `fitness_values`, not on chromosomes during the call. No borrow conflict exists.

### Pitfall 5: apply_fitness_sharing_with_dna Missing the raw_fitnesses Snapshot
**What goes wrong:** If fitness values are read from `fitness_values[j]` while also writing `fitness_values[i]`, the result depends on loop order — incorrect shared fitness.
**Why it happens:** `apply_fitness_sharing` (the existing function) correctly takes a snapshot `raw_fitnesses: Vec<f64> = fitness_values.to_vec()` before the loop. The new function must do the same.
**How to avoid:** Copy the snapshot pattern from `apply_fitness_sharing` — capture `raw_fitnesses` before any writes.

---

## Code Examples

### partition_point on Vec<(usize, f64)> (rank.rs)
```rust
// Source: Rust std docs — slice::partition_point (stable since 1.52)
// cumulative: Vec<(original_idx, f64)>, sorted ascending by cum prob
let idx = cumulative.partition_point(|&(_, cp)| cp < r).min(n - 1);
let selected_original_idx = cumulative[idx].0;
```

### partition_point on Vec<f64> (boltzmann.rs)
```rust
// Source: Rust std docs — slice::partition_point
// cumulative: Vec<f64>, sorted ascending, last entry clamped to 1.0
let idx = cumulative.partition_point(|&cp| cp < r).min(n - 1);
selected.push(idx);
```

### apply_fitness_sharing_with_dna inner accumulation pattern
```rust
// Accumulate niche_counts in a separate pass to avoid read-while-write
let raw_fitnesses: Vec<f64> = fitness_values.to_vec();
let mut niche_counts = vec![0.0f64; n];
for i in 0..n {
    for j in 0..n {
        let d = distance_fn(dna_slices[i], dna_slices[j]);
        niche_counts[i] += sharing_function(d, sigma_share, alpha);
    }
}
for i in 0..n {
    if niche_counts[i] > 0.0 {
        fitness_values[i] = raw_fitnesses[i] / niche_counts[i];
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `iter().position(predicate)` on sorted CDF | `partition_point(predicate)` | Phase 21 | O(log n) per sample vs O(n) |
| Two `fitness_values` allocations per generation | One shared allocation | Phase 21 | Eliminates 1 Vec alloc per generation (when niching enabled) |
| O(n²) distance matrix in `compute_distance_matrix` | On-the-fly per-pair in `apply_fitness_sharing_with_dna` | Phase 21 | Eliminates O(n²) Vec allocation per generation (when niching enabled) |

---

## Open Questions

1. **`#[deprecated]` on `compute_distance_matrix`**
   - What we know: After ALLOC-02, `ga.rs` no longer calls it; it remains public.
   - What's unclear: Whether to annotate now or defer to Phase 23 cleanup.
   - Recommendation: Add `#[deprecated(since = "2.2.1", note = "Use apply_fitness_sharing_with_dna for in-loop use")]` — it communicates intent and causes no breaking change. If the team prefers to defer, omit it.

2. **Variable name for merged fitness_values**
   - What we know: The existing name `fitness_values` is used in both locations being merged.
   - What's unclear: Whether a name like `gen_fitness_values` would be clearer.
   - Recommendation: Keep `fitness_values` — it is already the name in both blocks, minimizing diff noise.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | `Cargo.toml` |
| Quick run command | `cargo test selection` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ALGO-03 | Rank selection produces valid parent pairs after partition_point change | unit | `cargo test test_selection_rank` | Yes (`tests/operations/test_selection_rank.rs`) |
| ALGO-03 | Rank selection favors higher-fitness individuals (distribution correct) | unit | `cargo test test_rank_selection_favors_higher_fitness` | Yes |
| ALGO-04 | Boltzmann selection produces correct number of pairs | unit | `cargo test test_selection_boltzmann` | Yes (`tests/operations/test_selection_boltzmann.rs`) |
| ALGO-04 | Boltzmann at high/low temperature produces correct distributions | unit | `cargo test test_boltzmann_selection_high_temperature_approaches_uniform` | Yes |
| ALLOC-01 | GA generation loop completes with niching enabled (fitness values flow correctly) | integration | `cargo test test_ga` | Yes (`tests/test_ga.rs`) |
| ALLOC-02 | Niching fitness sharing produces same adjusted values with on-the-fly computation | unit | `cargo test test_niching` | Yes (`tests/niching/test_niching_sharing.rs`) |
| ALLOC-02 | New `apply_fitness_sharing_with_dna` function matches `apply_fitness_sharing` output | unit | `cargo test test_apply_fitness_sharing_with_dna` | No — Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test selection` and `cargo test niching`
- **Per wave merge:** `cargo test && cargo clippy`
- **Phase gate:** `cargo test && cargo test --features serde && cargo clippy` — all green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/niching/test_niching_sharing.rs` — add `test_apply_fitness_sharing_with_dna_matches_matrix_version` test that calls both the old matrix path and the new on-the-fly function on identical data and asserts equal fitness outputs (covers ALLOC-02 correctness)

---

## Sources

### Primary (HIGH confidence)
- Rust std docs — `slice::partition_point` (stable since Rust 1.52, 2021-05-06). Semantics confirmed by direct reading of source files and matching predicate form.
- Source code inspection: `src/operations/selection/rank.rs`, `src/operations/selection/boltzmann.rs`, `src/niching/sharing.rs`, `src/ga.rs` lines 800–930 — all read directly.

### Secondary (MEDIUM confidence)
- Existing test files in `tests/operations/test_selection_rank.rs`, `tests/operations/test_selection_boltzmann.rs`, `tests/niching/test_niching_sharing.rs` — confirmed test coverage and identified Wave 0 gap.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all changes use std Rust + existing project libraries
- Architecture: HIGH — all file locations and line numbers verified by direct source reading
- Pitfalls: HIGH — derived from direct inspection of the code being changed, not from external sources

**Research date:** 2026-03-31
**Valid until:** Stable — changes are to internal Rust std methods and pure Rust code with no external dependency updates needed
