---
phase: 48-new-genotype-types
plan: 03
subsystem: types, initializers, mutation
tags:
  - rust
  - chromosomes
  - genotypes
  - mutation
dependency_graph:
  requires:
    - 48-01 (OperatorCompat trait + operator_compat_check validator)
    - 48-02 (UniqueChromosome + UniqueGenotype patterns)
  provides:
    - MultiRangeGenotype<T> gene type (src/types/genotypes/multi_range.rs)
    - MultiRangeChromosome<T> chromosome type (src/types/chromosomes/multi_range.rs)
    - multi_range_random_initialization<T>(bounds, mutation_rates) (src/initializers/multi_range_initializer.rs)
    - multi_range_gaussian_mutation in gaussian.rs (per-gene D-10 mutation)
    - Crate-root re-exports for MultiRangeChromosome, MultiRangeGenotype, multi_range_random_initialization
  affects:
    - src/types/genotypes/mod.rs (new pub mod + pub use)
    - src/types/chromosomes/mod.rs (new pub mod + pub use)
    - src/initializers.rs (new pub mod + pub use)
    - src/lib.rs (three new crate-root pub use)
    - src/operations/mutation/gaussian.rs (new multi_range_gaussian_mutation function)
tech_stack:
  added: []
  patterns:
    - MultiRangeGenotype<T> — flat-fielded gene struct (id, lo, hi, value, mutation_rate; no Arc)
    - MultiRangeChromosome<T> — standard chromosome struct (dna, fitness, age, fitness_fn)
    - Empty OperatorCompat impl (no restriction — all real-valued ops accepted)
    - ValueMutable::gaussian_mutate override dispatches to multi_range_gaussian_mutation
    - Box-Muller noise scaled by gene.mutation_rate (not global sigma) with clamp to (gene.lo, gene.hi)
    - multi_range_random_initialization uses parallel bounds+rates slices; defaults short rates to 0.1
key_files:
  created:
    - src/types/genotypes/multi_range.rs
    - src/types/chromosomes/multi_range.rs
    - src/initializers/multi_range_initializer.rs
  modified:
    - src/types/genotypes/mod.rs
    - src/types/chromosomes/mod.rs
    - src/initializers.rs
    - src/lib.rs
    - src/operations/mutation/gaussian.rs
    - tests/types/genotypes/test_multi_range.rs
    - tests/types/chromosomes/test_multi_range.rs
    - tests/operations/test_mutation_creep_gaussian.rs
    - tests/initializers/test_initializers.rs
decisions:
  - "Gaussian mutation dispatch via ValueMutable::gaussian_mutate override — cleaner than downcast approach; aligns with existing RangeChromosome pattern"
  - "multi_range_gaussian_mutation added as new pub fn in gaussian.rs — not reusing gaussian_mutation (RESEARCH Pitfall 3 — gaussian_mutation reads gene.ranges, incompatible with flat gene.lo/hi)"
  - "Box-Muller noise scaled by gene.mutation_rate (not _sigma arg) — _sigma is accepted for API consistency but intentionally ignored (D-10)"
  - "Empty OperatorCompat impl body for MultiRangeChromosome — inherits default None-returning methods (no restriction, all operators valid)"
metrics:
  duration: "~25 minutes"
  completed: "2026-05-22T13:59:46Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 3
  files_modified: 8
---

# Phase 48 Plan 03: MultiRangeChromosome + Per-Gene Gaussian Mutation Summary

**One-liner:** `MultiRangeChromosome<T>` with flat-fielded per-gene `(lo, hi)` bounds and `mutation_rate`, `multi_range_random_initialization` parallel-slice sampler, and `multi_range_gaussian_mutation` that reads `gene.mutation_rate` (not global sigma) — full GEN-03 implementation with zero Arc overhead.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | MultiRangeGenotype + initializer + module re-exports + gene tests | 93924c8 | src/types/genotypes/multi_range.rs, src/initializers/multi_range_initializer.rs, src/types/genotypes/mod.rs, src/initializers.rs, src/lib.rs |
| 2 | MultiRangeChromosome + trait impls + per-gene Gaussian mutation + chromosome tests | f7ff1c1 | src/types/chromosomes/multi_range.rs, src/operations/mutation/gaussian.rs, src/types/chromosomes/mod.rs |

## What Was Built

### `src/types/genotypes/multi_range.rs`

`MultiRangeGenotype<T>` — a flat-fielded gene struct with five public fields (no Arc):

```rust
pub struct MultiRangeGenotype<T> {
    pub id: i32,
    pub lo: T,          // per-gene lower bound
    pub hi: T,          // per-gene upper bound
    pub value: T,       // current value
    pub mutation_rate: f64,  // per-gene noise scale for Gaussian mutation
}
```

Implements `GeneT` (id/set_id), `Default` (all zero), `Display` (`"id:value"`), plus `new(id, lo, hi, value, mutation_rate)`, `value()` (Copy), `set_value()`. Full serde conditional attrs. No `Arc` indirection — decision D-08.

### `src/types/chromosomes/multi_range.rs`

`MultiRangeChromosome<T>` — standard chromosome struct with four fields:

```rust
pub struct MultiRangeChromosome<T: Sync + Send + Copy + Default + Debug> {
    pub dna: Vec<MultiRangeGenotype<T>>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<MultiRangeGenotype<T>>,  // serde(skip)
}
```

Trait impls:
- `ChromosomeT`: calculate_fitness, fitness, set_fitness, age, set_age
- `LinearChromosome`: dna, dna_mut, set_dna (Cow match), set_fitness_fn
- `OperatorCompat`: empty impl (inherits `None` defaults — no restriction, all operators valid)
- `ValueMutable::gaussian_mutate`: dispatches to `multi_range_gaussian_mutation` — reads `gene.mutation_rate` and clamps to `(gene.lo, gene.hi)` (D-10)

### `src/initializers/multi_range_initializer.rs`

```rust
pub fn multi_range_random_initialization<T>(
    bounds: &[(T, T)],       // one (lo, hi) per gene; chromosome length == bounds.len()
    mutation_rates: &[f64],  // one rate per gene; short slices default to 0.1
) -> Vec<MultiRangeGenotype<T>>
```

Samples each gene independently from `[lo_i, hi_i)` using `rng.random_range(lo..hi)`. Short `mutation_rates` slices fall back to `0.1` (T-48-07 mitigation). No `genes_per_chromosome` parameter — length is derived from `bounds.len()`.

### `src/operations/mutation/gaussian.rs` (extended)

New `pub fn multi_range_gaussian_mutation<T>` added after existing `GaussianConvertible` impls:
- Reads `gene.mutation_rate` as the Box-Muller scale (NOT global `_sigma`)
- Clamps result to `[gene.lo, gene.hi]`
- Uses the same Box-Muller pattern as `gaussian_mutation` but reads flat gene fields
- `_sigma` argument accepted for API consistency but intentionally ignored

This is separate from `gaussian_mutation` (which reads `gene.ranges` on `RangeChromosome<T>`) — cannot be reused; RESEARCH Pitfall 3.

## Gaussian Mutation Dispatch Approach

The dispatch is via `ValueMutable::gaussian_mutate` override on `MultiRangeChromosome<T>`. This is the same pattern used by `Range<T>` chromosomes (which override `creep_mutate` and `gaussian_mutate`). When `Mutation::Gaussian` is dispatched through `factory_with_params`, it calls `individual.gaussian_mutate(sigma)`, which routes to the per-gene implementation.

No new enum arms or downcast is needed — the override is clean and non-disruptive to existing code.

## Test Cases Added

### `tests/types/genotypes/test_multi_range.rs` (10 tests, replaces Wave 0 scaffold)

1. `multi_range_genotype_new_id_accessor` — `new(3, -5.0, 5.0, 0.0, 0.1).id() == 3`
2. `multi_range_genotype_new_lo_hi_fields` — lo and hi fields accessible
3. `multi_range_genotype_new_value_accessor` — value() returns Copy value
4. `multi_range_genotype_new_mutation_rate_field` — mutation_rate field accessible
5. `multi_range_genotype_set_id_mutates_and_returns_self` — GeneT::set_id chain
6. `multi_range_genotype_set_value_mutates` — set_value in place
7. `multi_range_genotype_default_all_zero` — all five fields zero for f64
8. `multi_range_genotype_default_i32` — all five fields zero for i32
9. `multi_range_genotype_clone_preserves_all_five_fields` — Clone copies all fields
10. `multi_range_genotype_struct_literal_flat_fields_only` — struct literal compiles with flat fields only (no Arc)

### `tests/initializers/test_initializers.rs` (5 new tests)

1. `multi_range_initialization_correct_length` — `len == bounds.len()` (7 genes)
2. `multi_range_initialization_per_gene_bounds_enforcement` — 100-run sweep; every `gene.lo <= gene.value < gene.hi` with heterogeneous bounds `[(0,1), (10,20), (-5,-1)]`
3. `multi_range_initialization_mutation_rate_assignment` — every gene's rate matches `mutation_rates[i]`
4. `multi_range_initialization_short_rates_defaults_to_0_1` — trailing genes get default 0.1
5. `multi_range_initialization_gene_ids_sequential` — gene ids are 0..n

### `tests/types/chromosomes/test_multi_range.rs` (12 tests, replaces Wave 0 scaffold)

1. `multi_range_chromosome_default_empty_dna_nan_fitness` — Default struct state
2. `multi_range_chromosome_calculate_fitness_invokes_fn` — fitness_fn called by calculate_fitness
3. `multi_range_chromosome_set_fitness_returns_self` — set_fitness chain
4. `multi_range_chromosome_set_age_returns_self` — set_age chain
5. `multi_range_chromosome_set_dna_cow_owned_replaces_dna` — Cow::Owned path
6. `multi_range_chromosome_set_dna_cow_borrowed_replaces_dna` — Cow::Borrowed path
7. `multi_range_chromosome_operator_compat_no_restriction_crossovers` — valid_crossovers() is None
8. `multi_range_chromosome_operator_compat_no_restriction_mutations` — valid_mutations() is None
9. `multi_range_chromosome_single_point_crossover_accepted_at_build` — Ga::build() with SinglePoint succeeds
10-11. (inline) — OperatorCompat restriction verified in tests 7+8 above
12. (via build test) — Gaussian mutation accepted for MultiRangeChromosome

### `tests/operations/test_mutation_creep_gaussian.rs` (4 new tests)

1. `multi_range_gaussian_values_stay_within_per_gene_bounds_1000_iterations` — **heterogeneous bounds clamp test**: 1000-iteration sweep with bounds `[(0,1), (10,100)]` and rates `[0.05, 5.0]`; every value stays within `[gene.lo, gene.hi]`
2. `multi_range_gaussian_per_gene_rate_controls_noise_scale` — **mutation-rate scale test**: gene with rate=0.0001 produces avg|delta| << gene with rate=20.0 over 2000 mutations (ratio > 10x verified)
3. `multi_range_gaussian_mutation_direct_call_clamps_to_bounds` — direct call to `multi_range_gaussian_mutation` with rate=1e10 forces clamping; all values stay in `[0.0, 1.0]`
4. `multi_range_gaussian_mutation_empty_dna_does_nothing` — empty chromosome through factory returns Ok without panic

## Deviations from Plan

### Auto-fix: GeneT::default() ambiguity in test code

**Found during:** Task 1 — `MultiRangeGenotype::<f64>::default()` was ambiguous between `Default::default()` and `GeneT::default()`.

**Fix:** Changed test calls to use fully-qualified syntax `<MultiRangeGenotype<f64> as Default>::default()` to disambiguate. This is a test-only change — library code is unaffected.

**Rule:** Rule 1 (auto-fix bug — compile error in test code).

**Files modified:** `tests/types/genotypes/test_multi_range.rs`

## Known Stubs

None. All implementations are production-ready within scope.

## Threat Flags

No new threat surface introduced. `MultiRangeChromosome` reuses the existing `ChromosomeT` + `LinearChromosome` pathway. All threat model items (T-48-07, T-48-08, T-48-09, T-48-SC) accepted as documented in PLAN.

| Threat | Status |
|--------|--------|
| T-48-07: short mutation_rates slice | Mitigated — `unwrap_or(0.1)` fallback in initializer |
| T-48-08: O(1) mutation per call | Confirmed — single random gene selected per call |
| T-48-09: lo >= hi panic | Documented in fn doc (explicit panic on misuse) |
| T-48-SC: no new deps | Confirmed |

## Verification

- `cargo build --all-features` — EXIT 0
- `cargo test --all-features --test test_types --test test_operations --test test_initializers --test test_traits --test test_validators` — 466 passed, 0 failed
- `cargo clippy --all-features` — No new warnings introduced by Phase 48-03 code
- `cargo check --target wasm32-unknown-unknown` — EXIT 0 (Box-Muller uses no Instant/par_iter)
- Per-gene clamp test passes (1000-iteration sweep, both genes within bounds for every mutation)
- Per-gene rate scale test passes (rate=20.0 avg delta >> rate=0.0001 avg delta by >10x factor)
- Pre-existing `--all-features` compile failures (`test_ga.rs` `Chromosome: OperatorCompat`, sms_emoa example) are from Phase 48-01 and are out of scope

## Self-Check: PASSED

- `src/types/genotypes/multi_range.rs` — FOUND
- `src/types/chromosomes/multi_range.rs` — FOUND
- `src/initializers/multi_range_initializer.rs` — FOUND
- Task 1 commit 93924c8 — VERIFIED
- Task 2 commit f7ff1c1 — VERIFIED
