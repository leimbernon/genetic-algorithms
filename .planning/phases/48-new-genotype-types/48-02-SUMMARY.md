---
phase: 48-new-genotype-types
plan: 02
subsystem: types, initializers, examples
tags:
  - rust
  - chromosomes
  - genotypes
  - initializers
  - examples
  - permutation
dependency_graph:
  requires:
    - 48-01 (OperatorCompat trait + operator_compat_check validator)
  provides:
    - UniqueGenotype<T> gene type (src/types/genotypes/unique.rs)
    - UniqueChromosome<T> chromosome type (src/types/chromosomes/unique.rs)
    - unique_random_initialization<T>(alphabet) (src/initializers/unique_initializer.rs)
    - Crate-root re-exports for UniqueChromosome, UniqueGenotype, unique_random_initialization
  affects:
    - src/types/genotypes/mod.rs (new pub mod + pub use)
    - src/types/chromosomes/mod.rs (new pub mod + pub use)
    - src/initializers.rs (new pub mod + pub use)
    - src/lib.rs (three new crate-root pub use)
    - examples/job_scheduling.rs (migrated from RangeChromosome to UniqueChromosome)
tech_stack:
  added: []
  patterns:
    - UniqueGenotype<T> — minimal gene struct (id, value) with no per-gene alphabet
    - UniqueChromosome<T> — chromosome with shared Arc<[T]> alphabet field
    - Arc::from([]) for empty alphabet default (same as Range<T> ranges default)
    - Fisher-Yates shuffle in unique_random_initialization (full permutation semantics)
    - ValueMutable empty impl (inherits default swap fallback; OperatorCompat enforces at build)
key_files:
  created:
    - src/types/genotypes/unique.rs
    - src/types/chromosomes/unique.rs
    - src/initializers/unique_initializer.rs
  modified:
    - src/types/genotypes/mod.rs
    - src/types/chromosomes/mod.rs
    - src/initializers.rs
    - src/lib.rs
    - examples/job_scheduling.rs
    - tests/types/genotypes/test_unique.rs
    - tests/types/chromosomes/test_unique.rs
    - tests/initializers/test_initializers.rs
decisions:
  - "ValueMutable empty impl required on UniqueChromosome<T>: Ga::build() has bound U: ValueMutable; OperatorCompat restriction at build time prevents incompatible mutations from running"
  - "test_examples.rs not updated: existing pattern is single example test; ARCH-07 CI workflow covers all examples"
  - "Mutation variants on one line in valid_mutations() — all three (Insertion, Swap, Inversion) present in static slice"
metrics:
  duration: "~30 minutes"
  completed: "2026-05-22T13:42:32Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 3
  files_modified: 8
---

# Phase 48 Plan 02: UniqueChromosome + Initializer + Example Migration Summary

**One-liner:** `UniqueChromosome<T>` with shared `Arc<[T]>` alphabet, Fisher-Yates `unique_random_initialization`, full `OperatorCompat` restriction (Pmx/Order/EdgeRecombination/Clone/Rejuvenate + Insertion/Swap/Inversion), and `job_scheduling` example migrated from RangeChromosome hack to semantically correct permutation type.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | UniqueGenotype + initializer + module re-exports + tests | b98ed0c | src/types/genotypes/unique.rs, src/initializers/unique_initializer.rs, src/types/genotypes/mod.rs, src/initializers.rs, src/lib.rs |
| 2 | UniqueChromosome + ChromosomeT + LinearChromosome + OperatorCompat + tests | cffd548 | src/types/chromosomes/unique.rs, src/types/chromosomes/mod.rs, tests/types/chromosomes/test_unique.rs |
| 3 | Migrate job_scheduling example to UniqueChromosome + smoke test | 7da247d | examples/job_scheduling.rs |

## What Was Built

### `src/types/genotypes/unique.rs`

`UniqueGenotype<T>` — a minimal gene struct with two fields:

```rust
pub struct UniqueGenotype<T> {
    pub id: i32,
    pub value: T,
    // No per-gene alphabet — alphabet lives on UniqueChromosome (D-01)
}
```

Implements `GeneT` (id/set_id), `Default`, `Display` (`"id:value"`), plus `new(id, value)`, `value()` (clone), `set_value()`. Full serde conditional attrs.

### `src/types/chromosomes/unique.rs`

`UniqueChromosome<T>` — chromosome with five fields:

```rust
pub struct UniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    pub alphabet: Arc<[T]>,          // shared alphabet, O(1) clone
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,  // serde(skip)
}
```

Trait impls:
- `ChromosomeT`: calculate_fitness, fitness, set_fitness, age, set_age
- `LinearChromosome`: dna, dna_mut, set_dna (Cow match), set_fitness_fn
- `OperatorCompat`: restricts crossovers to `[Pmx, Order, EdgeRecombination, Clone, Rejuvenate]` and mutations to `[Insertion, Swap, Inversion]`
- `ValueMutable`: empty impl (default fallback to swap; OperatorCompat prevents incompatible mutations at build time)
- `Default`: `Arc::from([])` for empty alphabet; `f64::NAN` fitness; age 0

### `src/initializers/unique_initializer.rs`

```rust
pub fn unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>
where T: Clone + Sync + Send + Default + Debug
```

Fisher-Yates shuffle of `(0..alphabet.len())` indices, then maps to `UniqueGenotype { id: idx as i32, value: alphabet[idx].clone() }`. Empty alphabet returns empty Vec.

### `examples/job_scheduling.rs` (migration)

Before:
- Chromosome type: `RangeChromosome<i32>` with `RangeGenotype<i32>` using `ranges: Arc<[(i32,i32)]>`
- Initialization: manual `rand::seq::SliceRandom` shuffle inside `with_initialization_fn` closure
- Fitness closure: `|dna: &[RangeGenotype<i32>]|`
- TODO comment: "Phase 48 will migrate to UniqueChromosome"

After:
- Chromosome type: `UniqueChromosome<i32>` with `UniqueGenotype<i32>`
- Initialization: `unique_random_initialization(&alphabet)` — clean, no manual shuffle
- Fitness closure: `|dna: &[UniqueGenotype<i32>]|` (gene.value still works — same field name)
- TODO comment: removed
- `rand::seq::SliceRandom` import: removed

## Test Cases Added

### `tests/types/genotypes/test_unique.rs` (6 tests)
1. `unique_genotype_new_id` — `UniqueGenotype::new(7, 42i32).id() == 7`
2. `unique_genotype_set_id_mutates` — `set_id` mutates and returns `&mut Self`
3. `unique_genotype_default` — Default returns `id: 0, value: 0i32`
4. `unique_genotype_display` — Display formats as `"3:99"`
5. `unique_genotype_value_clone` — `value()` clones the value
6. `unique_genotype_has_no_alphabet_field` — struct literal with only id+value compiles

### `tests/initializers/test_initializers.rs` (3 new tests)
1. `unique_initializer_empty_alphabet` — empty alphabet → empty vec
2. `unique_initializer_correct_length` — result length == alphabet length
3. `unique_initializer_permutation_property` — multiset of values == multiset of alphabet

### `tests/types/chromosomes/test_unique.rs` (10 tests)
1. `unique_chromosome_default` — empty dna, empty alphabet, NaN fitness, age 0
2. `single_point_crossover_rejected_at_build` — `Ga::build()` with SinglePoint returns `Err(ConfigurationError)`
3. `pmx_crossover_accepted_at_build` — `Ga::build()` with Pmx returns Ok
4. `gaussian_mutation_rejected_at_build` — `Ga::build()` with Gaussian mutation returns `Err(ConfigurationError)`
5. `order_crossover_swap_mutation_accepted` — Order + Swap → Ok
6. `calculate_fitness_invokes_fn` — fitness_fn called via calculate_fitness
7. `set_dna_cow_owned_replaces_dna` — Cow::Owned replaces dna, returns &mut Self
8. `set_dna_cow_borrowed_replaces_dna` — Cow::Borrowed also replaces dna
9. `alphabet_is_arc_shared` — clone shares same Arc allocation
10. `valid_crossovers_is_restricted` + `valid_mutations_is_restricted` — OperatorCompat restriction verified

## Deviations from Plan

### Auto-fix: ValueMutable impl required by Ga::build() bound

**Found during:** Task 2 — `Ga::build()` has bound `U: mutation::ValueMutable`. The plan did not mention this requirement.

**Fix:** Added empty `impl<T: ...> ValueMutable for UniqueChromosome<T> {}` to satisfy the bound. The default fallback behavior (swap) is acceptable — `OperatorCompat` enforces the actual restriction at build time, preventing any incompatible mutation from being configured before execution.

**Files modified:** `src/types/chromosomes/unique.rs`

**Commit:** cffd548 (included in Task 2 commit)

## Known Stubs

None. All implementations are production-ready within scope.

## Threat Flags

No new threat surface introduced. `UniqueChromosome` reuses the existing `ChromosomeT` + `LinearChromosome` pathway. No new network endpoints, auth paths, or schema changes.

## Verification

- `cargo build --all-features` — EXIT 0
- `cargo test --all-features --test test_types --test test_initializers --test test_traits` — 83 passed
- `cargo build --example job_scheduling --all-features` — EXIT 0
- `cargo run --example job_scheduling --quiet` — EXIT 0 (produces valid permutation orderings, best makespan 13)
- `cargo clippy --all-features` — No issues found
- `cargo check --target wasm32-unknown-unknown` — EXIT 0
- Pre-existing failures in `test_ga.rs` (`Chromosome: OperatorCompat` not satisfied) are from 48-01 and out of scope

## Self-Check: PASSED

- `src/types/genotypes/unique.rs` — FOUND
- `src/types/chromosomes/unique.rs` — FOUND
- `src/initializers/unique_initializer.rs` — FOUND
- Task 1 commit b98ed0c — VERIFIED
- Task 2 commit cffd548 — VERIFIED
- Task 3 commit 7da247d — VERIFIED
