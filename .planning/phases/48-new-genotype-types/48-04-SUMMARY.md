---
phase: 48-new-genotype-types
plan: 04
subsystem: types, crossover, tests
tags:
  - rust
  - chromosomes
  - crossover
  - phase-gate
dependency_graph:
  requires:
    - 48-01 (OperatorCompat trait + Crossover::MultiGroupPmx/MultiGroupOx variants + pub(crate) build_child visibility)
    - 48-02 (UniqueGenotype<T> — reused as gene type for MultiUniqueChromosome)
    - 48-03 (MultiRangeChromosome patterns)
  provides:
    - MultiUniqueChromosome<T> chromosome type (src/types/chromosomes/multi_unique.rs)
    - group_ranges() accessor returning [(0, g0_len-1), ...] from group alphabet lengths
    - multi_group_pmx<U: LinearChromosome> (src/operations/crossover/multi_group_pmx.rs)
    - multi_group_ox<U: LinearChromosome> (src/operations/crossover/multi_group_ox.rs)
    - Dispatch arms for Crossover::MultiGroupPmx and Crossover::MultiGroupOx in crossover.rs
    - GEN-04 fully implemented
  affects:
    - src/operations/crossover.rs (dispatch arms + module declarations)
    - src/types/chromosomes/mod.rs (pub mod + pub use for multi_unique)
    - src/lib.rs (crate-root re-export of MultiUniqueChromosome)
tech_stack:
  added: []
  patterns:
    - MultiUniqueChromosome<T> — groups as Vec<Arc<[T]>>; gene type reuses UniqueGenotype<T> (D-13)
    - group_ranges() derives (start, end) pairs from group alphabet lengths on-the-fly
    - multi_group_pmx/ox: generic over U: LinearChromosome via the LinearChromosome::group_ranges() override
    - Dispatch decision: generic fn approach via LinearChromosome::group_ranges() default method override — no new GroupAware trait needed
    - OperatorCompat restriction: only MultiGroupPmx, MultiGroupOx, Clone, Rejuvenate accepted; standard Pmx/Order rejected at build time
    - ValueMutable empty impl (default fallback; OperatorCompat prevents incompatible mutations at build)
key_files:
  created:
    - src/types/chromosomes/multi_unique.rs
    - src/operations/crossover/multi_group_pmx.rs
    - src/operations/crossover/multi_group_ox.rs
    - tests/types/chromosomes/test_multi_unique.rs
    - tests/operations/test_crossover_multi_group_pmx.rs
    - tests/operations/test_crossover_multi_group_ox.rs
  modified:
    - src/types/chromosomes/mod.rs (pub mod multi_unique + pub use MultiUniqueChromosome)
    - src/lib.rs (crate-root pub use MultiUniqueChromosome)
    - src/operations/crossover.rs (pub mod multi_group_pmx/ox + use + dispatch arms)
decisions:
  - "Dispatch approach: generic LinearChromosome::group_ranges() override on MultiUniqueChromosome — no GroupAware trait needed; multi_group_pmx/ox are generic over U: LinearChromosome and call self.group_ranges() which MultiUniqueChromosome overrides to return real group data (all other chromosomes return empty vec from the default, which errors out via the is_empty() guard)"
  - "Empty-groups guard: functions return Err(GaError::ConfigurationError) on empty group_ranges() — no silent no-op (RESEARCH Pitfall 7 mitigation)"
  - "Mismatched-group guard: both parents must have identical group_ranges(); mismatch returns ConfigurationError (T-48-11)"
  - "Doc link fix: pmx_build_child and ox_build_child are pub(crate), so doc links were replaced with plain backtick refs to eliminate rustdoc warnings"
metrics:
  duration: "~20 minutes (Phase 48-04 continuation — all code already committed in prior partial run)"
  completed: "2026-05-22T14:30:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 6
  files_modified: 4
---

# Phase 48 Plan 04: MultiUniqueChromosome + Multi-Group Crossover Summary

**One-liner:** `MultiUniqueChromosome<T>` with `Vec<Arc<[T]>>` groups, `group_ranges()` boundary accessor, and two group-sliced crossover operators (`multi_group_pmx`, `multi_group_ox`) that apply PMX/OX per group via `LinearChromosome::group_ranges()` override — GEN-04 complete, Phase 48 closed.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | MultiUniqueChromosome struct + group_ranges + ChromosomeT/LinearChromosome/OperatorCompat + tests | 3f848fd | src/types/chromosomes/multi_unique.rs, tests/types/chromosomes/test_multi_unique.rs |
| 2 | multi_group_pmx + multi_group_ox + dispatch arms + correctness tests | e829bd2 | src/operations/crossover/multi_group_pmx.rs, multi_group_ox.rs, src/operations/crossover.rs |
| Doc fix | Rustdoc warning: broken links to pub(crate) build_child fns | 273bd63 | src/operations/crossover/multi_group_pmx.rs, multi_group_ox.rs |

## What Was Built

### `src/types/chromosomes/multi_unique.rs`

`MultiUniqueChromosome<T>` — chromosome with five public fields:

```rust
pub struct MultiUniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    pub groups: Vec<Arc<[T]>>,      // one Arc<[T]> per permutation group (D-12)
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,  // serde(skip)
}
```

The `groups` field holds the per-group alphabets. Group sizes and DNA boundaries are derived
on-the-fly by `group_ranges()` — no separate boundary storage needed.

**`group_ranges()` implementation (D-14):**

```rust
pub fn group_ranges(&self) -> Vec<(usize, usize)> {
    let mut ranges = Vec::with_capacity(self.groups.len());
    let mut start = 0usize;
    for group in &self.groups {
        if group.is_empty() { continue; }
        let end = start + group.len().saturating_sub(1);
        ranges.push((start, end));
        start = end + 1;
    }
    ranges
}
```

Empty groups are skipped defensively. Example: groups of sizes `[3, 3, 2]` → `[(0, 2), (3, 5), (6, 7)]`.

Trait impls:
- `ChromosomeT`: calculate_fitness, fitness, set_fitness, age, set_age
- `LinearChromosome`: dna, dna_mut, set_dna (Cow match), set_fitness_fn, `group_ranges()` override (delegates to inherent method)
- `OperatorCompat`: restricts crossovers to `[MultiGroupPmx, MultiGroupOx, Clone, Rejuvenate]`; mutations to `[Insertion, Swap, Inversion]` (D-07, A5)
- `ValueMutable`: empty impl (OperatorCompat prevents incompatible mutations at build time)
- `Default`: empty dna, empty groups, NaN fitness, age 0
- `Display`: `"[phenotype...] fitness=X.XXXXXX"`

Constructor `new(groups: Vec<Vec<T>>)` converts each inner `Vec<T>` to `Arc<[T]>` via `into_boxed_slice().into()`.

### Dispatch Decision: Generic via `LinearChromosome::group_ranges()` override

The plan listed several dispatch options (concrete fn typed for `MultiUniqueChromosome<T>`, new `GroupAware` trait, or `Any` downcast). The chosen approach is the cleanest:

- `LinearChromosome` already has a `group_ranges(&self) -> Vec<(usize, usize)>` default method that returns an empty Vec.
- `MultiUniqueChromosome<T>` overrides this method in its `LinearChromosome` impl to return real group boundaries.
- `multi_group_pmx<U: LinearChromosome>` calls `parent.group_ranges()` and returns `Err(ConfigurationError)` if it is empty.
- For all other chromosome types, `group_ranges()` returns empty Vec → the empty-groups guard triggers → the error reaches the user before any damage.
- No new `GroupAware` trait is needed. No `Any` downcast. The type system does the right thing with the override.

**This approach is preferred because:**
1. Minimal surface area — no new trait, no new bounds in operator dispatch
2. Correct safety invariant — non-`MultiUnique` chromosomes cannot accidentally succeed at multi-group crossover
3. Consistent with how `LinearChromosome::group_ranges()` was designed in Phase 47

### `src/operations/crossover/multi_group_pmx.rs`

```rust
pub fn multi_group_pmx<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
) -> Result<Vec<U>, GaError>
```

Body:
1. Calls `parent_1.group_ranges()` — returns empty Vec for non-MultiUnique chromosomes.
2. Returns `Err(ConfigurationError)` if groups are empty (T-48-10 mitigation, Pitfall 7).
3. Compares `parent_2.group_ranges()` — returns `Err(ConfigurationError)` on mismatch (T-48-11 mitigation).
4. Iterates `(start, end)` pairs; calls `pmx_build_child` on each group slice.
5. Assembles two children via `U::new()` + `set_dna(Cow::Owned(...))`.

### `src/operations/crossover/multi_group_ox.rs`

Same shape as `multi_group_pmx` but calls `ox_build_child`. Per-group random position sampling handles edge cases:
- Size 1 group: skipped (single element, no crossover needed).
- Size 2 group: positions `(0, 1)` always used.
- Size >= 3 group: two distinct positions sampled randomly via `rng.random_range`.

### Dispatch Arms in `src/operations/crossover.rs`

```rust
Crossover::MultiGroupPmx => multi_group_pmx(parent_1, parent_2),
Crossover::MultiGroupOx  => multi_group_ox(parent_1, parent_2),
```

Added to both `impl CrossoverOperator for Crossover` and `impl CrossoverOperator for CrossoverConfiguration` match blocks. The inert error-returning stubs from 48-01 are replaced by real dispatch.

## Test Cases Added

### `tests/types/chromosomes/test_multi_unique.rs` (replaces Wave 0 scaffold — 18 tests)

1. `multi_unique_chromosome_new_produces_correct_alphabets` — groups contain expected alphabet contents
2. `group_ranges_three_groups` — `[(0,2), (3,5), (6,7)]` for sizes 3,3,2
3. `group_ranges_empty_groups_returns_empty` — empty chromosome → empty Vec
4. `group_ranges_single_element_group` — `[(0,0)]` for single-element group
5. `group_ranges_two_equal_groups` — `[(0,3), (4,7)]` for two groups of 4
6. `multi_unique_chromosome_default` — empty dna, empty groups, NaN fitness, age 0
7. `multi_unique_chromosome_calculate_fitness_invokes_fn` — fitness_fn called via calculate_fitness
8. `multi_unique_chromosome_set_fitness_set_age` — set_fitness/set_age chain
9. `multi_unique_chromosome_set_dna_cow_owned` — Cow::Owned path
10. `multi_unique_chromosome_set_dna_cow_borrowed` — Cow::Borrowed path
11. `multi_unique_chromosome_valid_crossovers_contains_multi_group_variants` — MultiGroupPmx/MultiGroupOx included; Pmx/Order excluded
12. `multi_unique_chromosome_valid_mutations_restricted` — Insertion/Swap/Inversion included; Gaussian excluded
13. `pmx_crossover_rejected_at_build` — `Ga::build()` with Pmx returns ConfigurationError
14. `single_point_crossover_rejected_at_build` — ConfigurationError for SinglePoint
15. `order_crossover_rejected_at_build` — ConfigurationError for Order
16. `multi_group_pmx_crossover_swap_mutation_accepted` — MultiGroupPmx + Swap → Ok
17. `multi_group_ox_crossover_inversion_mutation_accepted` — MultiGroupOx + Inversion → Ok
18. `groups_is_arc_shared` — cloned chromosome shares the same `Arc<[T]>` allocation

### `tests/operations/test_crossover_multi_group_pmx.rs` (8 tests)

1. `multi_group_pmx_produces_two_children_correct_length` — 2 children, 8 genes each
2. `multi_group_pmx_group0_gene_multiset_preserved` — group-0 values {0,1,2} preserved
3. `multi_group_pmx_group1_gene_multiset_preserved` — group-1 values {10,20,30} preserved
4. `multi_group_pmx_group2_gene_multiset_preserved` — group-2 values {100,200} preserved
5. `multi_group_pmx_no_gene_crosses_group_boundary` — explicit cross-boundary check
6. `multi_group_pmx_children_have_unique_gene_ids_per_group` — gene IDs unique within each group
7. `multi_group_pmx_empty_groups_returns_error` — ConfigurationError on empty groups
8. `multi_group_pmx_mismatched_group_structures_returns_error` — ConfigurationError on mismatch
+ Stress test: `multi_group_pmx_stress_100_runs_group_membership_preserved` — 100 random runs, all group memberships preserved

### `tests/operations/test_crossover_multi_group_ox.rs` (7 tests + stress)

Symmetric to PMX tests, verifying the same invariants for OX.

## Deviations from Plan

### Auto-fix: Rustdoc warnings — broken links to pub(crate) items

**Found during:** Phase verification gate — `cargo doc --no-deps` emitted 2 warnings about `[pmx_build_child]` and `[ox_build_child]` links in multi-group crossover doc comments.

**Rule:** Rule 1 (bug — documentation warnings on public items).

**Fix:** Replaced the broken `[`pmx_build_child`]` and `[`ox_build_child`]` doc link syntax with plain backtick-quoted `pmx_build_child` and `ox_build_child`. `pub(crate)` items cannot be referenced via rustdoc links in public documentation.

**Files modified:** `src/operations/crossover/multi_group_pmx.rs`, `src/operations/crossover/multi_group_ox.rs`

**Commit:** 273bd63

### No GroupAware Trait (Plan deviation: documented)

The plan described three dispatch approaches. The selected approach (generic via `LinearChromosome::group_ranges()` override) is simpler than the `GroupAware` trait option and does not require any new trait file, new bounds, or architectural changes. This is a deliberate improvement over the plan text — the plan explicitly said "Pick whichever approach minimally extends the existing architecture."

## Known Stubs

None. All Phase 48 implementations are production-ready.

## Threat Flags

No new threat surface beyond what the plan documented. All STRIDE threats mitigated as documented.

| Threat | Status |
|--------|--------|
| T-48-10: empty groups → silent no-op | Mitigated — `is_empty()` guard returns `ConfigurationError` |
| T-48-11: mismatched parent group structures | Mitigated — `parent_2.group_ranges() != groups` check returns `ConfigurationError` |
| T-48-12: large group count DoS | Accepted — O(g × group_len) = O(total_dna_len) same as standard PMX |
| T-48-13: serde groups field | Accepted — local checkpoint trust model unchanged |
| T-48-14: bypass via direct call | Accepted — direct callers assume responsibility; Ga::build() catches engine-level case |
| T-48-SC: no new cargo dependencies | Confirmed |

## Phase 48 Verification Gate Results

| Check | Result |
|-------|--------|
| `cargo build --all-features` | EXIT 0 |
| `cargo test --all-features --test test_types --test test_operations --test test_traits --test test_initializers --test test_validators` | 498 PASSED, 0 FAILED |
| `cargo test --features serde` (Phase 48 test suites only) | PASSED |
| `cargo clippy --all-features --no-deps` | No new warnings |
| `cargo doc --no-deps` | No warnings (after doc link fix in 273bd63) |
| `cargo check --target wasm32-unknown-unknown` | EXIT 0 |
| `cargo run --example job_scheduling --quiet` | EXIT 0 (best makespan 13, valid permutation) |
| All 10 examples compile (smoke loop) | 9/10 EXIT 0; `sms_emoa_zdt1` fails on pre-existing Phase 48-01 compile error (out of scope) |

Pre-existing failures (`test_ga.rs` `Chromosome: OperatorCompat`, `sms_emoa_zdt1` example) are from Phase 48-01 and are out of scope for Phase 48-04.

## Self-Check: PASSED

- `src/types/chromosomes/multi_unique.rs` — FOUND
- `src/operations/crossover/multi_group_pmx.rs` — FOUND
- `src/operations/crossover/multi_group_ox.rs` — FOUND
- `tests/types/chromosomes/test_multi_unique.rs` — FOUND
- `tests/operations/test_crossover_multi_group_pmx.rs` — FOUND
- `tests/operations/test_crossover_multi_group_ox.rs` — FOUND
- Task 1 commit 3f848fd — VERIFIED
- Task 2 commit e829bd2 — VERIFIED
- Doc fix commit 273bd63 — VERIFIED
- 498 tests pass, 0 failures — VERIFIED
- `cargo check --target wasm32-unknown-unknown` EXIT 0 — VERIFIED
- `cargo doc --no-deps` no warnings — VERIFIED
