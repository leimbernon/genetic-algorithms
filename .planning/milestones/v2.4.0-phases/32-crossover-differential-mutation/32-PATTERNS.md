# Phase 32: Crossover & Differential Mutation - Pattern Map

**Mapped:** 2026-05-04
**Files analyzed:** 11
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/operations/crossover/edge_recombination.rs` | operator | transform | `src/operations/crossover/pmx.rs` | exact |
| `src/operations/crossover.rs` | dispatch | request-response | self (modify) | exact |
| `src/operations/mutation/differential.rs` | operator | transform | `src/operations/mutation/gaussian.rs` | exact |
| `src/operations/mutation.rs` | dispatch | request-response | self (modify) | exact |
| `src/operations.rs` | config/enum | — | self (modify) | exact |
| `src/configuration.rs` | config | — | self (modify) | exact |
| `src/traits/configuration.rs` | trait | — | self (modify) | exact |
| `src/engines/ga.rs` | engine | event-driven | self (modify) | exact |
| `tests/operations/test_crossover_edge_recombination.rs` | test | — | `tests/operations/test_crossover_pmx.rs` | exact |
| `tests/operations/test_mutation_differential.rs` | test | — | `tests/operations/test_mutation_polynomial.rs` | exact |
| `tests/observe/test_serde.rs` | test | — | self (modify) | exact |

---

## Pattern Assignments

### `src/operations/crossover/edge_recombination.rs` (operator, transform)

**Analog:** `src/operations/crossover/pmx.rs`

**Imports pattern** (`pmx.rs` lines 1–8):
```rust
use crate::error::GaError;
use crate::traits::{ChromosomeT, GeneT};
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::collections::HashMap;
```
Add `HashSet` to the `std::collections` import for visited-gene tracking and duplicate-ID validation.

**Public function signature** (mirror of `pmx.rs` line 23):
```rust
pub fn erx<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Length + equality validation** (`pmx.rs` lines 26–38):
```rust
let len = parent_1.dna().len();
if len != parent_2.dna().len() {
    return Err(GaError::CrossoverError(format!(
        "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
        len,
        parent_2.dna().len()
    )));
}
if len < 2 {
    return Err(GaError::CrossoverError(
        "PMX crossover requires DNA of length >= 2".to_string(),
    ));
}
```
Change the minimum-length message to reference `EdgeRecombination` instead of `PMX`.

**Gene-uniqueness check (D-08)** — derived from `order.rs` `segment_ids` HashSet pattern:
```rust
let ids_p1: std::collections::HashSet<i32> =
    parent_1.dna().iter().map(|g| g.id()).collect();
if ids_p1.len() != parent_1.dna().len() {
    return Err(GaError::CrossoverError(
        "EdgeRecombination crossover requires unique gene IDs (permutation chromosomes only)".to_string(),
    ));
}
// repeat for parent_2
```

**Debug log + RNG init** (`pmx.rs` lines 40–42):
```rust
debug!(target="crossover_events", method="edge_recombination"; "Starting ERX crossover");
let mut rng = crate::rng::make_rng();
```

**Two-child construction + Cow::Owned set_dna** (`pmx.rs` lines 53–63):
```rust
let child_dna_1 = erx_build_child(start_gene_1, &mut adj.clone(), &all_ids, &mut rng);
let child_dna_2 = erx_build_child(start_gene_2, &mut adj, &all_ids, &mut rng);

let mut child_1 = U::new();
let mut child_2 = U::new();
child_1.set_dna(Cow::Owned(child_dna_1));
child_2.set_dna(Cow::Owned(child_dna_2));

debug!(target="crossover_events", method="edge_recombination"; "ERX crossover finished");
Ok(vec![child_1, child_2])
```

**Private helper signature pattern** (`pmx.rs` line 70 — `pmx_build_child`):
```rust
fn erx_build_child<G: GeneT>(
    start: i32,
    adj: &mut HashMap<i32, HashSet<i32>>,
    all_ids: &[i32],
    rng: &mut impl Rng,
) -> Vec<G>
```
`all_ids` is the ordered slice of gene IDs from parent_1 (used for fallback random selection). Return type is `Vec<G>` — caller wraps in `Cow::Owned` and calls `set_dna`.

---

### `src/operations/crossover.rs` (dispatch, request-response)

**Analog:** `src/operations/crossover.rs` (self-modification)

**New module declaration** (after line 30 `pub mod pmx;`):
```rust
pub mod edge_recombination;
```

**Re-export** (after line 13 `pub use self::pmx::pmx;`):
```rust
pub use self::edge_recombination::erx;
```

**New match arm in `CrossoverOperator for Crossover`** (lines 153–192 pattern, after `Crossover::Pmx` arm):
```rust
Crossover::EdgeRecombination => erx(parent_1, parent_2),
```

**New match arm in `CrossoverOperator for CrossoverConfiguration`** (lines 194–240 pattern, after `Crossover::Pmx` arm):
```rust
Crossover::EdgeRecombination => erx(parent_1, parent_2),
```

---

### `src/operations/mutation/differential.rs` (operator, transform)

**Analog:** `src/operations/mutation/gaussian.rs`

**Imports pattern** (`gaussian.rs` lines 1–14):
```rust
use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use rand::Rng;
use std::fmt::Debug;
```
Add `use std::any::Any;` and `use crate::operations::mutation::gaussian::GaussianConvertible;` (reuse trait for `to_f64`/`from_f64`).

**Public function signature**:
```rust
pub fn differential_mutation<U>(
    individual: &mut U,
    chromosomes: &[U],
    target_idx: usize,
    f: f64,
) -> Result<(), crate::error::GaError>
where
    U: ChromosomeT + 'static,
```

**Population size guard (D-03)**:
```rust
if chromosomes.len() < 4 {
    return Err(GaError::MutationError(
        "Differential mutation requires at least 4 chromosomes in the population (target + 3 distinct donors)".to_string(),
    ));
}
```

**Range<T> downcast macro pattern** (`src/operations/crossover.rs` `try_sbx` lines 46–71, adapted for `&mut dyn Any`):
```rust
macro_rules! try_differential {
    ($t:ty) => {
        if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
            // ... compute and clamp mutant vector, return Some(Ok(()))
            return Some(Ok(()));
        }
    };
}
try_differential!(f64);
try_differential!(f32);
try_differential!(i32);
try_differential!(i64);
// if None → return Err(GaError::MutationError("Differential mutation requires Range<T>..."))
```

**Range clamping pattern** (`gaussian.rs` lines 46–58):
```rust
let (lo, hi) = gene.ranges[range_idx];
let lo_f64 = T::to_f64(lo);
let hi_f64 = T::to_f64(hi);
let new_val_f64 = (mutant_val).clamp(lo_f64, hi_f64);
gene.value = T::from_f64(new_val_f64);
```

**Three-index sampling** (derived from `crate::rng::make_rng()` pattern in `gaussian.rs` line 36):
```rust
let mut rng = crate::rng::make_rng();
let pop_len = chromosomes.len();
let mut r1 = rng.random_range(0..pop_len);
while r1 == target_idx { r1 = rng.random_range(0..pop_len); }
let mut r2 = rng.random_range(0..pop_len);
while r2 == target_idx || r2 == r1 { r2 = rng.random_range(0..pop_len); }
let mut r3 = rng.random_range(0..pop_len);
while r3 == target_idx || r3 == r1 || r3 == r2 { r3 = rng.random_range(0..pop_len); }
```

**set_dna with Cow::Owned** (set all genes at once, since mutant depends on r1/r2/r3):
```rust
individual.set_dna(std::borrow::Cow::Owned(new_dna));
```

---

### `src/operations/mutation.rs` (dispatch, request-response)

**Analog:** `src/operations/mutation.rs` (self-modification)

**New module declaration** (after line 32 `pub mod value;`):
```rust
pub mod differential;
```

**New safety-net arm in `MutationOperator for Mutation`** (lines 155–162 pattern — `Mutation::NonUniform` arm):
```rust
Mutation::Differential => {
    return Err(GaError::MutationError(
        "Mutation::Differential requires population context. \
         It is applied automatically by the GA engine when configured — \
         do not call factory_with_params() directly.".to_string(),
    ));
}
```

**New arm in `factory_non_value`** (lines 259–272 pattern — `Mutation::NonUniform` arm):
```rust
Mutation::Differential => Err(GaError::MutationError(
    "Mutation::Differential requires Range<T> chromosomes and population context. \
     Use Swap, Inversion, or Scramble instead.".to_string(),
)),
```

---

### `src/operations.rs` (enum variants)

**Analog:** `src/operations.rs` (self-modification)

**Enum variant pattern** (lines 51–82 `Crossover` enum, lines 88–119 `Mutation` enum):
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Crossover {
    // ... existing variants ...
    /// Edge Recombination Crossover for permutation-based chromosomes (TSP, scheduling).
    /// Builds a union adjacency list from both parents and constructs offspring that
    /// preserve adjacency relationships found in either parent.
    EdgeRecombination,
}

pub enum Mutation {
    // ... existing variants ...
    /// DE-style differential mutation for `Range<T>` chromosomes.
    /// Computes mutant vector as `x_r1 + F * (x_r2 - x_r3)` from three
    /// distinct random population members, clamped to gene ranges.
    /// Configure F via `MutationConfiguration::differential_f` (default 0.5).
    Differential,
}
```

---

### `src/configuration.rs` (config struct)

**Analog:** `src/configuration.rs` (self-modification)

**Existing `Option<f64>` field pattern** (`MutationConfiguration` lines 152–155):
```rust
/// Distribution index for Polynomial mutation. ...
pub polynomial_eta: Option<f64>,
/// Decay parameter for NonUniform mutation. ...
pub non_uniform_b: Option<f64>,
```

**New field to add** (after `non_uniform_b`, before `dynamic_mutation`):
```rust
/// F scale factor for Differential mutation. Controls perturbation magnitude.
/// Typical range: 0.4–1.0. Default is 0.5 when `None`.
/// Only used when `method` is `Mutation::Differential`.
pub differential_f: Option<f64>,
```

**New `Default` impl entry** (lines 166–181 `Default for MutationConfiguration`):
```rust
differential_f: None,
```

---

### `src/traits/configuration.rs` (builder trait)

**Analog:** `src/traits/configuration.rs` (self-modification)

**Existing builder method pattern** (`MutationConfig` trait lines 51–52):
```rust
/// Sets the sigma for Gaussian mutation.
fn with_mutation_sigma(self, sigma: f64) -> Self;
```

**New method to add** (after `with_mutation_probability_step`):
```rust
/// Sets the F scale factor for Differential mutation (DE-style).
/// Typical range: 0.4–1.0. Default is 0.5.
fn with_differential_f(self, f: f64) -> Self;
```

---

### `src/engines/ga.rs` (engine dispatch)

**Analog:** `src/engines/ga.rs` (self-modification)

**Existing builder impl pattern** (lines 215–243 — `MutationConfig for Ga<U>` implementations):
```rust
fn with_mutation_sigma(mut self, sigma: f64) -> Self {
    self.configuration.mutation_configuration.sigma = Some(sigma);
    self
}
```

**New builder impl to add**:
```rust
fn with_differential_f(mut self, f: f64) -> Self {
    self.configuration.mutation_configuration.differential_f = Some(f);
    self
}
```

**Existing mutation dispatch in `parent_crossover`** (lines 1389–1406):
```rust
if mutation_probability < effective_mutation_prob {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_1,
        configuration.mutation_configuration.step,
        configuration.mutation_configuration.sigma,
    )?;
}

mutation_probability = rng.random_range(0.0..1.0);
if mutation_probability <= effective_mutation_prob {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_2,
        configuration.mutation_configuration.step,
        configuration.mutation_configuration.sigma,
    )?;
}
```

**Replace with Differential branch (D-01)**:
```rust
if mutation_probability < effective_mutation_prob {
    if configuration.mutation_configuration.method == Mutation::Differential {
        let f = configuration.mutation_configuration.differential_f.unwrap_or(0.5);
        mutation::differential::differential_mutation(&mut child_1, chromosomes, *key, f)?;
    } else {
        mutation::factory_with_params(
            configuration.mutation_configuration.method,
            &mut child_1,
            configuration.mutation_configuration.step,
            configuration.mutation_configuration.sigma,
        )?;
    }
}

mutation_probability = rng.random_range(0.0..1.0);
if mutation_probability <= effective_mutation_prob {
    if configuration.mutation_configuration.method == Mutation::Differential {
        let f = configuration.mutation_configuration.differential_f.unwrap_or(0.5);
        mutation::differential::differential_mutation(&mut child_2, chromosomes, *value, f)?;
    } else {
        mutation::factory_with_params(
            configuration.mutation_configuration.method,
            &mut child_2,
            configuration.mutation_configuration.step,
            configuration.mutation_configuration.sigma,
        )?;
    }
}
```
Note: `*key` is parent_1's population index (passed as `target_idx` for child_1), `*value` is parent_2's index (for child_2). The `Mutation` import must be in scope at the top of `ga.rs` — check existing imports.

---

### `tests/operations/test_crossover_edge_recombination.rs` (test)

**Analog:** `tests/operations/test_crossover_pmx.rs`

**Imports pattern** (`test_crossover_pmx.rs` lines 1–6):
```rust
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::operations::crossover::edge_recombination::erx;
use genetic_algorithms::traits::{ChromosomeT, GeneT};
use std::borrow::Cow;
use std::collections::HashSet;
```

**Helper factory pattern** (`test_crossover_pmx.rs` lines 8–38):
```rust
fn make_permutation_parents() -> (BinaryChromosome, BinaryChromosome) {
    // p1: [1,2,3,4,5], p2: [3,5,1,2,4] — distinct IDs, valid permutations
}
```

**Test coverage map** (mirror of `test_crossover_pmx.rs` structure):
- `erx_produces_two_children` — `children.len() == 2`
- `erx_preserves_length` — both children same length as parents
- `erx_produces_valid_permutations` — no duplicate IDs, all parent IDs present (run 50 iterations)
- `erx_error_on_different_lengths` — `GaError::CrossoverError`
- `erx_error_too_short` — length 1 → `GaError::CrossoverError` (D-07)
- `erx_error_duplicate_ids` — duplicate ID in parent → `GaError::CrossoverError` (D-08)
- `erx_fallback_exhausted_neighbors` — construct a case where adjacency list is guaranteed to exhaust (e.g., linear 3-gene chromosome where one gene has no unvisited neighbors mid-build); verify offspring is still a valid permutation (D-06)

---

### `tests/operations/test_mutation_differential.rs` (test)

**Analog:** `tests/operations/test_mutation_polynomial.rs`

**Imports pattern** (`test_mutation_polynomial.rs` lines 1–5):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::differential::differential_mutation;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;
```

**Helper factory pattern** (`test_mutation_polynomial.rs` lines 7–14):
```rust
fn build_f64_chromosome(id: i32, n: usize) -> RangeChromosome<f64> {
    // RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0)
}
fn make_population(size: usize) -> Vec<RangeChromosome<f64>> {
    (0..size).map(|i| build_f64_chromosome(i as i32, 5)).collect()
}
```

**Test coverage map** (mirror of `test_mutation_polynomial.rs` structure):
- `differential_mutation_stays_within_range` — all gene values remain in `[lo, hi]` after mutation (run 200 iterations, `test_mutation_polynomial.rs` lines 16–32 pattern)
- `differential_mutation_can_change_value` — at least one gene changes across 200 iterations
- `differential_error_small_population` — pop size 3 → `GaError::MutationError` (D-03)
- `differential_error_non_range` — pass `BinaryChromosome` → `GaError::MutationError` (D-02)
- `differential_f_parameter` — F=0.0 produces no change; F=2.0 still clamps to range
- `differential_mutation_with_i32` — Range<i32> chromosomes stay within bounds (`test_mutation_polynomial.rs` lines 85–106 pattern)

---

### `tests/observe/test_serde.rs` (test — modify existing)

**Analog:** `tests/observe/test_serde.rs` (self-modification)

**`serde_crossover_enum` array** (lines 52–63) — add after `Crossover::Clone`:
```rust
Crossover::Rejuvenate,       // already there (verify)
Crossover::EdgeRecombination, // ADD
```

**`serde_mutation_enum` array** (lines 71–85) — add after `Mutation::Insertion`:
```rust
Mutation::ListValue,   // already there (verify)
Mutation::Differential, // ADD
```

**`serde_ga_configuration_with_values` struct literal** (lines 140–151) — add field to `MutationConfiguration`:
```rust
mutation_configuration: MutationConfiguration {
    probability_max: Some(0.05),
    probability_min: Some(0.01),
    method: Mutation::Polynomial,
    step: Some(0.5),
    sigma: Some(1.5),
    polynomial_eta: Some(30.0),
    non_uniform_b: Some(3.0),
    dynamic_mutation: false,
    target_cardinality: None,
    probability_step: None,
    differential_f: None,  // ADD — prevents "missing field" compile error (Pitfall 3)
},
```

---

## Shared Patterns

### RNG initialization
**Source:** `src/operations/mutation/gaussian.rs` line 36 and `src/operations/crossover/pmx.rs` line 42
**Apply to:** `edge_recombination.rs`, `differential.rs`
```rust
let mut rng = crate::rng::make_rng();
```
Never use `rand::thread_rng()` directly.

### Crossover error format
**Source:** `src/operations/crossover/pmx.rs` lines 27–38
**Apply to:** `edge_recombination.rs`
```rust
return Err(GaError::CrossoverError(format!("...")));
return Err(GaError::CrossoverError("...".to_string()));
```

### Mutation error format
**Source:** `src/operations/mutation.rs` lines 155–161
**Apply to:** `differential.rs`, new arms in `mutation.rs`
```rust
return Err(GaError::MutationError("...".to_string()));
```

### Range<T> downcast macro (Any)
**Source:** `src/operations/crossover.rs` lines 46–71 (`try_sbx`) and `src/operations/mutation.rs` lines 44–57 (`try_polynomial`)
**Apply to:** `differential.rs`
```rust
macro_rules! try_type {
    ($t:ty) => {
        if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
            // operate on concrete type
            return Some(Ok(()));
        }
    };
}
try_type!(f64); try_type!(f32); try_type!(i32); try_type!(i64);
```

### GaussianConvertible reuse
**Source:** `src/operations/mutation/gaussian.rs` lines 67–108
**Apply to:** `differential.rs`
```rust
use crate::operations::mutation::gaussian::GaussianConvertible;
// T::to_f64(gene.value) and T::from_f64(new_val_f64)
```
Do not duplicate these conversion impls. Import and reuse.

### Cow::Owned set_dna
**Source:** `src/operations/crossover/pmx.rs` lines 58–59
**Apply to:** `edge_recombination.rs`, `differential.rs`
```rust
child.set_dna(Cow::Owned(new_dna));
// or: individual.set_dna(std::borrow::Cow::Owned(new_dna));
```

### Debug logging target
**Source:** `src/operations/crossover/pmx.rs` line 40
**Apply to:** `edge_recombination.rs` (use `target="crossover_events"`), `differential.rs` (use `target="mutation_events"`)
```rust
debug!(target="crossover_events", method="edge_recombination"; "...");
debug!(target="mutation_events", method="differential"; "...");
```

### Option<f64> field + Default
**Source:** `src/configuration.rs` lines 152–180 (`MutationConfiguration`)
**Apply to:** `configuration.rs` (add `differential_f`)
```rust
pub polynomial_eta: Option<f64>,   // existing pattern
pub differential_f: Option<f64>,   // new — same convention
// In Default:
polynomial_eta: None,
differential_f: None,
```

---

## No Analog Found

All files have close analogs in the codebase. No files require research-only patterns.

---

## Metadata

**Analog search scope:** `src/operations/`, `src/engines/`, `src/configuration.rs`, `src/traits/configuration.rs`, `src/operations.rs`, `tests/operations/`, `tests/observe/`
**Files scanned:** 14
**Pattern extraction date:** 2026-05-04
