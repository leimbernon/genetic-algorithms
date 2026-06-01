---
phase: 55-rfc-multi-valued-fitness
reviewed: 2026-05-31T00:00:00Z
depth: standard
files_reviewed: 46
files_reviewed_list:
  - src/configuration.rs
  - src/engines/alps/configuration.rs
  - src/engines/alps/engine.rs
  - src/engines/cellular/configuration.rs
  - src/engines/cellular/engine.rs
  - src/engines/ga.rs
  - src/engines/gp/chromosome.rs
  - src/engines/gp/engine.rs
  - src/engines/ibea/mod.rs
  - src/engines/island/mod.rs
  - src/engines/island/nsga2.rs
  - src/engines/moead/mod.rs
  - src/engines/nsga2/mod.rs
  - src/engines/nsga3/mod.rs
  - src/engines/sms_emoa/mod.rs
  - src/engines/spea2/mod.rs
  - src/lib.rs
  - src/operations.rs
  - src/operations/crossover.rs
  - src/operations/crossover/pcx.rs
  - src/operations/crossover/spx.rs
  - src/operations/crossover/undx.rs
  - src/operations/mutation.rs
  - src/operations/mutation/self_adaptive_gaussian.rs
  - src/operations/selection.rs
  - src/operations/selection/boltzmann.rs
  - src/operations/selection/clearing.rs
  - src/operations/selection/fitness_proportionate.rs
  - src/operations/selection/lexicase.rs
  - src/operations/selection/random.rs
  - src/operations/selection/rank.rs
  - src/operations/selection/tournament.rs
  - src/operations/selection/truncation.rs
  - src/traits.rs
  - src/traits/configuration.rs
  - src/traits/operators.rs
  - src/traits/real_valued.rs
  - src/traits/self_adaptive.rs
  - src/traits/vector_fitness.rs
  - src/types/chromosomes/binary.rs
  - src/types/chromosomes/list.rs
  - src/types/chromosomes/multi_range.rs
  - src/types/chromosomes/multi_unique.rs
  - src/types/chromosomes/range.rs
  - src/types/chromosomes/unique.rs
findings:
  critical: 5
  warning: 5
  info: 2
  total: 12
status: issues_found
---

# Phase 55: Code Review Report

**Reviewed:** 2026-05-31T00:00:00Z
**Depth:** standard
**Files Reviewed:** 46
**Status:** issues_found

## Summary

Reviewed 46 source files spanning the v3.0.0 milestone changes: ChromosomeT/LinearChromosome split, new genotype types, N-ary selection, VectorFitness trait, multi-parent crossover operators (UNDX/SPX/PCX), self-adaptive mutation, GP engine, and multi-objective engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA).

The new abstractions are structurally sound and the WASM gating is generally correct. However, five critical defects were identified: one WASM violation that breaks compilation on `wasm32-unknown-unknown` (tournament selection unconditionally uses rayon), one algorithm-correctness bug that makes SPEA2 binary tournament purely random (rank field never populated during the run), one operator-configuration mismatch in roulette-wheel selection (ignores the configured `number_of_couples`), and two panic paths in AlpsEngine and CellularEngine when `layer_size` or grid dimensions are zero (no input validation).

---

## Critical Issues

### CR-01: Tournament selection unconditionally uses `rayon::par_iter` — breaks WASM

**File:** `src/operations/selection/tournament.rs:10-55`
**Issue:** The file unconditionally imports `use rayon::prelude::*` (line 10) and calls `.into_par_iter()` (line 55) with no `#[cfg(not(target_arch = "wasm32"))]` guard. Every other module that uses rayon is properly gated (e.g., `src/engines/ga.rs:158`, `src/engines/nsga2/mod.rs:123`). Tournament selection is the default selection method for `Ga`, NSGA-II, NSGA-III, ALPS, and Cellular engines, making this a hard compilation failure for any wasm32-unknown-unknown build.

**Fix:**
```rust
use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

// Inside tournament_impl:
#[cfg(not(target_arch = "wasm32"))]
let winners: Vec<usize> = (0..total_contestants)
    .into_par_iter()
    .map(|_| { /* ... */ })
    .collect();

#[cfg(target_arch = "wasm32")]
let winners: Vec<usize> = (0..total_contestants)
    .map(|_| { /* ... */ })
    .collect();
```

---

### CR-02: SPEA2 binary tournament ignores SPEA2 fitness — pure random parent selection

**File:** `src/engines/spea2/mod.rs:421-446`
**Issue:** `binary_tournament_from_archive` compares `pool[i].rank < pool[j].rank`. The `rank` field on `ParetoIndividual` defaults to `0` and is only assigned from a non-dominated sort at the very end of `run()` (lines 570–573), never during the generation loop. During all generations, every individual in the archive has `rank == 0`, so both branches of the rank comparison are always `false` and every tournament falls back to `rng.random::<bool>()`. SPEA2 parent selection is therefore purely random — the algorithm does not behave as SPEA2 at all.

The correct approach is to compare by the SPEA2 fitness value returned by `assign_spea2_fitness`, which is already computed each generation and stored in the `fitness` vector. This value needs to be propagated into the archive individuals or passed into the tournament function.

**Fix:** Store SPEA2 fitness on the `ParetoIndividual` before calling `create_offspring`, and use it in the tournament instead of `rank`:

```rust
// After environmental_selection, tag each archive member with its SPEA2 fitness
// Option A: store spea2_fitness on a per-individual field (add field or use existing fitness)
// Minimal fix: update the archive fitness field before calling create_offspring
let combined_fitness = Self::assign_spea2_fitness(&population, &archive, &directions);
// ...environmental selection...
// Then set a per-individual fitness proxy so the tournament can read it:
for (i, ind) in archive.iter_mut().enumerate() {
    // Use the stored spea2_fitness in the tournament (pass as parameter or assign to a field)
    ind.rank = if combined_fitness.get(population.len() + i).copied().unwrap_or(1.0) < 1.0 {
        0
    } else {
        1
    };
}
```
Or refactor `binary_tournament_from_archive` to accept `spea2_fitness: &[f64]` and compare directly by fitness value (lower is better) rather than by rank.

---

### CR-03: `roulette_wheel_selection` ignores the configured `number_of_couples`

**File:** `src/operations/selection/fitness_proportionate.rs:29-75`
**Issue:** The function signature is `roulette_wheel_selection<U>(chromosomes: &[U], num_parents: usize)` — it takes no `couples` parameter. On line 56, the number of selections is computed as `(chromosomes.len() / num_parents) * num_parents`, which depends solely on population size. When called from `selection::factory` (which correctly passes `configuration.number_of_couples`), the `number_of_couples` argument is silently dropped because the call site is `roulette_wheel_selection(chromosomes, num_parents)`. This produces `population_size / num_parents` parent groups regardless of configuration, which is typically 2× or more than the requested `number_of_couples`. The offspring count is therefore wrong, causing the population to grow unchecked before survivor selection or to produce far more evaluations than expected.

**Fix:**
```rust
pub fn roulette_wheel_selection<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,          // add this parameter
    num_parents: usize,
) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
    let num_selections = couples * num_parents;   // use couples, not chromosomes.len()
    // ...rest unchanged...
}
```
Update the call site in `selection::factory` and in `SelectionOperator for Selection`:
```rust
Selection::RouletteWheel => roulette_wheel_selection(chromosomes, number_of_couples, num_parents),
```

---

### CR-04: `AlpsEngine::run` panics with index-out-of-bounds when `layer_size == 0`

**File:** `src/engines/alps/engine.rs:129-137`
**Issue:** When `config.layer_size == 0`, `fresh_individuals(0)` returns an empty `Vec`. The call to `max_by` on an empty iterator returns `None`, and the fallback `unwrap_or_else(|| layers[0][0].clone())` immediately panics with an index-out-of-bounds error because `layers[0]` is empty. There is no validation that `layer_size > 0` or `n_layers > 0` anywhere in `AlpsConfiguration` or `AlpsEngine`.

**Fix:** Add a validation guard at the start of `run()`:
```rust
pub fn run(&mut self) -> AlpsResult<U> {
    if self.config.layer_size == 0 {
        panic!("AlpsEngine: layer_size must be > 0");
    }
    if self.config.n_layers == 0 {
        panic!("AlpsEngine: n_layers must be > 0");
    }
    // ...rest unchanged...
```
Or preferably return `Result<AlpsResult<U>, GaError>` and use `Err(GaError::ConfigurationError(...))` instead of panicking.

---

### CR-05: `CellularEngine::run` panics with index-out-of-bounds when grid has zero cells

**File:** `src/engines/cellular/engine.rs:120-121`
**Issue:** Lines 120–121 unconditionally access `pop[0]` to initialize `best_fitness` and `best`. If `rows == 0` or `cols == 0`, the population has zero elements (`pop_size = rows * cols = 0`) and the `init_fn` returns an empty `Vec`. `pop[0]` immediately panics. Like `AlpsEngine`, there is no configuration validation before the engine starts running.

**Fix:**
```rust
if pop.is_empty() {
    return CellularResult {
        population: vec![],
        best: panic!("CellularEngine: grid must have at least 1 cell (rows > 0 && cols > 0)"),
        best_fitness: f64::NAN,
        generations: 0,
    };
}
// Or return Result<_, GaError> and propagate an error.
```
The correct approach is a validation step at the start of `run()` that returns `Err(GaError::ConfigurationError(...))` if `rows == 0 || cols == 0`.

---

## Warnings

### WR-01: `stochastic_universal_sampling` logs to wrong target (`mutation_events`)

**File:** `src/operations/selection/fitness_proportionate.rs:153`
**Issue:** The final debug log at line 153 uses `target="mutation_events"` instead of `target="selection_events"`. This causes SUS completion events to appear in mutation log streams and be absent from selection log streams, making per-operator log filtering incorrect.

**Fix:**
```rust
debug!(target="selection_events", method="stochastic_universal_sampling"; "Stochastic universal sampling finished");
```

---

### WR-02: `lexicase_selection` assumes all chromosomes have the same number of fitness values

**File:** `src/operations/selection/lexicase.rs:129,133`
**Issue:** Line 129 early-returns if `chromosomes[0].fitness_values().is_empty()`, and line 133 sets `num_cases = chromosomes[0].fitness_values().len()`. If individual chromosomes have different-length `fitness_values` vectors (e.g., user error or partial update), subsequent indexing of `chromosomes[i].fitness_values()[case]` will panic with index-out-of-bounds for any chromosome with fewer cases than `chromosomes[0]`. The same issue applies to `epsilon_lexicase_selection` and `compute_mad_epsilons`.

**Fix:** Add a runtime check or an assertion:
```rust
let num_cases = chromosomes[0].fitness_values().len();
if chromosomes.iter().any(|c| c.fitness_values().len() != num_cases) {
    log::warn!(target="selection_events", "lexicase: fitness_values length mismatch — truncating to min");
    // or return Vec::new() / return Err
}
```

---

### WR-03: `GpChromosome` implements `LinearChromosome` with unconditional panics — no trait-level signal

**File:** `src/engines/gp/chromosome.rs:269-312`
**Issue:** `GpChromosome<N>` provides `LinearChromosome` implementations for `dna()`, `dna_mut()`, and `set_dna()` that always panic. This satisfies the trait bound required by `survivor::factory` and other operators, but means that any code path that mistakenly calls these methods (e.g., passing a `GpChromosome` to a crossover or mutation operator that reads DNA) will silently compile and only fail at runtime with a panic. There is no type-system enforcement preventing misuse. The comment in the code acknowledges this is intentional but documents it only in prose.

No code path in the reviewed files invokes these methods inadvertently for GP, but the design creates a latent panic surface that is invisible at compile time.

**Fix:** Consider a newtype wrapper or feature-gate to prevent the `LinearChromosome` impl from being registered, or at minimum convert the panics to `unreachable!()` with a more informative message and document the invariant as a compile-time bound (e.g., a separate `GpEngine` trait that does not require `LinearChromosome`).

---

### WR-04: `GpGa::run` passes fixed `num_parents=2` to `selection::factory` — multi-parent crossover unsupported silently

**File:** `src/engines/gp/engine.rs:262`
**Issue:** Line 262 calls `selection::factory(&pop, sel_cfg, 1, 2)` with a hardcoded `num_parents=2`. The GP engine does not support multi-parent crossover (UNDX/SPX/PCX), which is correct, but if a user configures `Crossover::Undx { num_parents: 5 }` in the `GpConfiguration`'s crossover field, the engine silently selects only 2-parent groups while the crossover operator will reject them later with a `CrossoverError("UNDX requires at least 3 parents")`. There is no validation at build or run time that rejects multi-parent variants for GP.

**Fix:** In `GpConfiguration::build()` or in `GpGa::run()`, add:
```rust
match self.config.crossover {
    GpCrossover::Subtree | GpCrossover::Hoist | GpCrossover::PointMutation => {}
    // If GP exposes standard crossovers:
    // reject Undx/Spx/Pcx variants with ConfigurationError
}
```

---

### WR-05: `self_adaptive_gaussian_mutation` accesses `individual.strategy_params` (private field) directly

**File:** `src/operations/mutation/self_adaptive_gaussian.rs:85`
**Issue:** Line 85 accesses `individual.strategy_params.get(idx)` directly on the concrete `RangeChromosome<T>` field, bypassing the `SelfAdaptive::strategy_params()` trait method. This is described in a comment ("Defensive fallback: if strategy_params is shorter than dna") but creates a coupling to the concrete type's field layout rather than the trait interface. More importantly, after `adapt_strategy_params()` updates the sigmas via `set_strategy_params()`, the direct field access on line 85 reads from the same field, so it should be consistent — but this design makes refactoring `RangeChromosome` fragile (renaming the field breaks this code silently until compilation).

**Fix:** Replace the direct field access with the trait method:
```rust
let sigma = individual.strategy_params().get(idx).copied().unwrap_or(1.0);
```

---

## Info

### IN-01: `roulette_wheel_selection` missing `couples` parameter is a silent API inconsistency

**File:** `src/operations/selection/fitness_proportionate.rs:29`
**Issue:** All other selection functions (`boltzmann_selection`, `clearing_selection`, `rank_selection`, `stochastic_universal_sampling`, `truncation_selection`) accept an explicit `couples: usize` parameter. `roulette_wheel_selection` is the only one that omits it, making the public API inconsistent and making the call-site bug in CR-03 easy to miss in code review.

**Fix:** The fix for CR-03 also resolves this inconsistency.

---

### IN-02: `Selection::Lexicase`/`Selection::EpsilonLexicase` through `SelectionOperator` trait panics instead of returning error

**File:** `src/operations/selection.rs:67-73`
**Issue:** When `Selection::Lexicase` or `Selection::EpsilonLexicase` is invoked through the `SelectionOperator` trait (used by island model and cellular GA paths), the implementation panics rather than returning `Err`. All other operator dispatch through this trait returns gracefully. The panic is documented in the code, but it means that any island-model user who configures `Selection::Lexicase` gets an unrecoverable runtime crash rather than a `GaError`.

**Fix:** Prefer returning a sentinel value or logging an error and falling back to `Selection::Tournament`, or document clearly in `SelectionOperator::select` that this method must not be called with lexicase variants. Consider replacing the `panic!` with a `debug_assert!` and a silent fallback for production builds.

---

_Reviewed: 2026-05-31T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
