# Phase 48: New Genotype Types - Pattern Map

**Mapped:** 2026-05-21
**Files analyzed:** 18 (10 new, 8 modified)
**Analogs found:** 18 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/types/genotypes/unique.rs` | model | transform | `src/types/genotypes/range.rs` | exact |
| `src/types/genotypes/multi_range.rs` | model | transform | `src/types/genotypes/range.rs` | exact |
| `src/types/chromosomes/unique.rs` | model | transform | `src/types/chromosomes/list.rs` | exact |
| `src/types/chromosomes/multi_range.rs` | model | transform | `src/types/chromosomes/range.rs` | exact |
| `src/types/chromosomes/multi_unique.rs` | model | transform | `src/types/chromosomes/list.rs` | exact |
| `src/initializers/unique_initializer.rs` | utility | batch | `src/initializers/list_initializer.rs` | exact |
| `src/initializers/multi_range_initializer.rs` | utility | batch | `src/initializers/range_initializer.rs` | exact |
| `src/traits/operator_compat.rs` | middleware | request-response | `src/validators/generic_validator.rs` | role-match |
| `src/operations/crossover/multi_group_pmx.rs` | service | transform | `src/operations/crossover/pmx.rs` | exact |
| `src/operations/crossover/multi_group_ox.rs` | service | transform | `src/operations/crossover/order.rs` | exact |
| `src/types/genotypes/mod.rs` | config | — | `src/types/genotypes/mod.rs` (self) | exact |
| `src/types/chromosomes/mod.rs` | config | — | `src/types/chromosomes/mod.rs` (self) | exact |
| `src/initializers.rs` | config | — | `src/initializers.rs` (self) | exact |
| `src/traits.rs` | config | — | `src/traits.rs` (self) | exact |
| `src/operations.rs` | config | — | `src/operations.rs` (self) | exact |
| `src/operations/crossover.rs` | config | — | `src/operations/crossover.rs` (self) | exact |
| `src/engines/ga.rs` | controller | request-response | `src/validators/generic_validator.rs` | role-match |
| `examples/job_scheduling.rs` | config | — | `examples/job_scheduling.rs` (self) | exact |

---

## Pattern Assignments

### `src/types/genotypes/unique.rs` (model, transform)

**Analog:** `src/types/genotypes/range.rs`

**Imports pattern** (lines 1-11):
```rust
use crate::traits::GeneT;
use std::fmt;
use std::sync::Arc;
```
Note: `UniqueGenotype<T>` does NOT import `Arc` — the alphabet lives on the chromosome, not the gene. Import only `crate::traits::GeneT` and `std::fmt`.

**Struct + serde pattern** (lines 35-48 of range.rs):
```rust
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct UniqueGenotype<T> {
    pub id: i32,
    pub value: T,
    // No `ranges` field — alphabet lives on UniqueChromosome, not per gene (D-01)
}
```

**Default impl pattern** (lines 69-77 of range.rs):
```rust
impl<T: Default> Default for UniqueGenotype<T> {
    fn default() -> Self {
        Self {
            id: 0,
            value: Default::default(),
        }
    }
}
```

**GeneT impl pattern** (lines 79-87 of range.rs):
```rust
impl<T: Clone + Default + Sync + Send> GeneT for UniqueGenotype<T> {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}
```

**Display pattern** (lines 63-67 of range.rs):
```rust
impl<T: fmt::Display> fmt::Display for UniqueGenotype<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.value)
    }
}
```

**Constructor `new` pattern** (lines 101-107 of range.rs — adapted, no ranges):
```rust
impl<T: Clone + Default> UniqueGenotype<T> {
    pub fn new(id: i32, value: T) -> Self {
        Self { id, value }
    }
    pub fn value(&self) -> T {
        self.value.clone()
    }
    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }
}
```

---

### `src/types/genotypes/multi_range.rs` (model, transform)

**Analog:** `src/types/genotypes/range.rs`

**Key difference from `Range<T>`:** Bounds (`lo`, `hi`) and `mutation_rate` are flat fields on the gene struct (D-08). No `Arc<[(T,T)]>` indirection — this enables per-gene independent values without Arc overhead.

**Struct + serde pattern** (modeled on lines 35-48 of range.rs):
```rust
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct MultiRangeGenotype<T> {
    pub id: i32,
    pub lo: T,
    pub hi: T,
    pub value: T,
    pub mutation_rate: f64,
}
```

**Default impl** (modeled on lines 69-77 of range.rs):
```rust
impl<T: Default> Default for MultiRangeGenotype<T> {
    fn default() -> Self {
        Self {
            id: 0,
            lo: Default::default(),
            hi: Default::default(),
            value: Default::default(),
            mutation_rate: 0.0,
        }
    }
}
```

**GeneT impl** — same shape as `Range<T>` (lines 79-87 of range.rs):
```rust
impl<T: Sync + Send + Copy + Default> GeneT for MultiRangeGenotype<T> {
    fn id(&self) -> i32 { self.id }
    fn set_id(&mut self, id: i32) -> &mut Self { self.id = id; self }
}
```

**Constructor** (modeled on lines 101-107 of range.rs):
```rust
impl<T: Copy + Default> MultiRangeGenotype<T> {
    pub fn new(id: i32, lo: T, hi: T, value: T, mutation_rate: f64) -> Self {
        Self { id, lo, hi, value, mutation_rate }
    }
    pub fn value(&self) -> T { self.value }
    pub fn set_value(&mut self, value: T) -> &mut Self { self.value = value; self }
}
```

---

### `src/types/chromosomes/unique.rs` (model, transform)

**Analog:** `src/types/chromosomes/list.rs` (uses `Clone` gene, not `Copy`) and `src/types/chromosomes/range.rs`

**Key difference:** Adds `alphabet: Arc<[T]>` field. The `Default` impl uses `Arc::from([])` for the empty alphabet — same pattern that `Range<T>` gene uses for its ranges default (`Arc::from([])`).

**Imports pattern** (lines 1-14 of list.rs):
```rust
use crate::fitness::FitnessFnWrapper;
use crate::genotypes::UniqueGenotype;
use crate::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
```

**Struct + serde pattern** (lines 34-50 of list.rs):
```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct UniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    pub alphabet: Arc<[T]>,          // extra field vs Range/List
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,
}
```

**Default impl** — `Arc::from([])` for the empty alphabet (same as `Range<T>` gene uses `Arc::from([])`):
```rust
impl<T: Sync + Send + Clone + Default + Debug> Default for UniqueChromosome<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            alphabet: Arc::from([]),   // Arc<[T]> empty slice default
            fitness: f64::NAN,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        }
    }
}
```

**ChromosomeT impl** (lines 116-140 of list.rs):
```rust
impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT for UniqueChromosome<T> {
    type Gene = UniqueGenotype<T>;
    fn calculate_fitness(&mut self) { self.fitness = self.fitness_fn.call(&self.dna); }
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self { self.fitness = fitness; self }
    fn set_age(&mut self, age: usize) -> &mut Self { self.age = age; self }
    fn age(&self) -> usize { self.age }
}
```

**LinearChromosome impl** (lines 142-169 of list.rs):
```rust
impl<T: Sync + Send + Clone + Default + Debug + 'static> LinearChromosome for UniqueChromosome<T> {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where F: Fn(&[UniqueGenotype<T>]) -> f64 + Send + Sync + 'static {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}
```

**OperatorCompat impl** (see Shared Patterns section — add after ChromosomeT/LinearChromosome):
```rust
impl<T: Sync + Send + Clone + Default + Debug + 'static> OperatorCompat for UniqueChromosome<T> {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[
            Crossover::Pmx,
            Crossover::Order,
            Crossover::EdgeRecombination,
            Crossover::Clone,
            Crossover::Rejuvenate,
        ])
    }
    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion])
    }
}
```
Note: `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` must be added to the enum before this impl can reference them. Add those variants first (in `src/operations.rs`), then add them to this list.

---

### `src/types/chromosomes/multi_range.rs` (model, transform)

**Analog:** `src/types/chromosomes/range.rs`

**Key difference:** Gene type is `MultiRangeGenotype<T>` (not `RangeGenotype<T>`). The struct has no extra fields beyond the standard `{ dna, fitness, age, fitness_fn }` — bounds live on each gene, not on the chromosome.

**Imports pattern** (lines 1-14 of range.rs):
```rust
use crate::fitness::FitnessFnWrapper;
use crate::genotypes::MultiRangeGenotype;
use crate::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
```

**Struct + serde pattern** (lines 34-49 of range.rs — substitute gene type):
```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct MultiRangeChromosome<T: Sync + Send + Copy + Default + Debug> {
    pub dna: Vec<MultiRangeGenotype<T>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<MultiRangeGenotype<T>>,
}
```

**ChromosomeT + LinearChromosome impls** — identical shape to `Range<T>` (lines 112-166 of range.rs), substituting `MultiRangeGenotype<T>` for `RangeGenotype<T>` throughout. Trait bound on `T` changes to `Clone` instead of `Copy` depending on `MultiRangeGenotype<T>`'s requirements.

**Per-gene Gaussian mutation** (NEW — analog is `src/operations/mutation/gaussian.rs` lines 27-60 but must NOT reuse `gaussian_mutation()` directly — that function is typed for `RangeChromosome<T>` and reads `gene.ranges`). Implement a dedicated method reading `gene.lo`, `gene.hi`, `gene.mutation_rate`:
```rust
// In multi_range.rs or in a dedicated mutation file
pub fn multi_range_gaussian_mutate<T>(individual: &mut MultiRangeChromosome<T>, _sigma: f64)
where
    T: Sync + Send + Copy + Default + Debug + PartialOrd + 'static + GaussianConvertible,
{
    let len = individual.dna().len();
    if len == 0 { return; }
    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);
    let mut gene = individual.dna()[idx].clone();

    let current = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(gene.lo);
    let hi_f64 = T::to_f64(gene.hi);

    // Box-Muller (same as gaussian.rs lines 53-55)
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * gene.mutation_rate; // per-gene rate
    gene.value = T::from_f64((current + noise).clamp(lo_f64, hi_f64));

    individual.set_gene(idx, gene);
}
```
Key: uses `gene.mutation_rate` as the sigma scale, not the global `_sigma` argument (D-10).

---

### `src/types/chromosomes/multi_unique.rs` (model, transform)

**Analog:** `src/types/chromosomes/list.rs`

**Key difference:** Adds `groups: Vec<Arc<[T]>>` field encoding the per-group alphabets. Exposes `group_ranges(&self) -> Vec<(usize, usize)>`. Gene type reuses `UniqueGenotype<T>` (D-13).

**Imports** (modeled on list.rs lines 1-14):
```rust
use crate::fitness::FitnessFnWrapper;
use crate::genotypes::UniqueGenotype;
use crate::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
```

**Struct + serde pattern** (modeled on list.rs lines 34-50):
```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct MultiUniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    pub groups: Vec<Arc<[T]>>,       // one Arc<[T]> per permutation group (D-12)
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,
}
```

**group_ranges method** (derived from D-14):
```rust
impl<T: Sync + Send + Clone + Default + Debug> MultiUniqueChromosome<T> {
    pub fn group_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = Vec::with_capacity(self.groups.len());
        let mut start = 0usize;
        for group in &self.groups {
            let end = start + group.len().saturating_sub(1);
            ranges.push((start, end));
            start = end + 1;
        }
        ranges
    }
}
```

**ChromosomeT + LinearChromosome impls** — identical shape to `ListChromosome<T>` (lines 116-169 of list.rs), substituting `UniqueGenotype<T>` for `ListGenotype<T>`.

**OperatorCompat impl** (D-07 — same valid sets as UniqueChromosome, plus MultiGroupPmx/MultiGroupOx once added):
```rust
impl<T: Sync + Send + Clone + Default + Debug + 'static> OperatorCompat for MultiUniqueChromosome<T> {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[
            Crossover::MultiGroupPmx,
            Crossover::MultiGroupOx,
            Crossover::Clone,
            Crossover::Rejuvenate,
        ])
    }
    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion])
    }
}
```

---

### `src/initializers/unique_initializer.rs` (utility, batch)

**Analog:** `src/initializers/list_initializer.rs` — `list_random_initialization_without_repetitions` (lines 98-135)

**Fisher-Yates pattern** (lines 114-134 of list_initializer.rs):
```rust
pub fn unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>
where
    T: Clone + Sync + Send + Default + Debug,
{
    let mut rng = crate::rng::make_rng();

    // Fisher-Yates shuffle (same structure as list_random_initialization_without_repetitions)
    let mut indices: Vec<usize> = (0..alphabet.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    // Full permutation: chromosome length == alphabet length (D-03)
    let mut dna = Vec::with_capacity(alphabet.len());
    for &idx in &indices {
        dna.push(UniqueGenotype { id: idx as i32, value: alphabet[idx].clone() });
    }
    dna
}
```
Key difference from list analog: takes only `alphabet: &[T]` — no `genes_per_chromosome` parameter because a full permutation always has length equal to the alphabet (D-03).

---

### `src/initializers/multi_range_initializer.rs` (utility, batch)

**Analog:** `src/initializers/range_initializer.rs` (lines 36-56)

**Per-gene bounds sampling pattern** (lines 43-56 of range_initializer.rs):
```rust
pub fn multi_range_random_initialization<T>(
    bounds: &[(T, T)],  // one (lo, hi) per gene position; length == chromosome length
    mutation_rates: &[f64],  // one mutation_rate per gene; same length as bounds
) -> Vec<MultiRangeGenotype<T>>
where
    T: Sync + Send + Clone + Default + Debug + 'static + PartialOrd + SampleUniform + Copy,
{
    let mut rng = crate::rng::make_rng();
    let mut genes = Vec::with_capacity(bounds.len());
    for (i, &(lo, hi)) in bounds.iter().enumerate() {
        let value = rng.random_range(lo..hi);
        let rate = mutation_rates.get(i).copied().unwrap_or(0.1);
        genes.push(MultiRangeGenotype::new(i as i32, lo, hi, value, rate));
    }
    genes
}
```
Key differences from range analog: takes explicit `bounds: &[(T, T)]` and `mutation_rates: &[f64]` slices (D-09/D-10); no `genes_per_chromosome` needed since length is derived from `bounds.len()`.

---

### `src/traits/operator_compat.rs` (middleware, request-response)

**Analog:** `src/traits/linear_chromosome.rs` (trait file shape) + `src/validators/generic_validator.rs` (validation pattern)

**Trait definition** — new file, no prior content to diff against:
```rust
//! Operator compatibility trait for build-time operator validation.
use crate::operations::{Crossover, Mutation};

/// Opt-in trait that restricts which operators are valid for a chromosome type.
///
/// Default implementations return `None`, meaning no restriction. Override
/// `valid_crossovers()` and/or `valid_mutations()` to restrict the allowed
/// operator set. `Ga::build()` checks these at build time, failing fast with
/// `GaError::ConfigurationError` if an invalid operator is selected.
pub trait OperatorCompat {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        None
    }
    fn valid_mutations() -> Option<&'static [Mutation]> {
        None
    }
}

/// Blanket impl: all LinearChromosome types are OperatorCompat with no restrictions.
/// This avoids a breaking change to Ga::build()'s generic bounds (Pitfall 1 in RESEARCH.md).
use crate::traits::LinearChromosome;
impl<T: LinearChromosome> OperatorCompat for T {}
```

**Validation function** — to be added to `src/validators/generic_validator.rs`, modeled on the function shape at lines 83-98 (unique_gene_ids):
```rust
pub fn operator_compat_check<U: LinearChromosome + OperatorCompat>(
    configuration: &GaConfiguration,
) -> Result<(), GaError> {
    if let Some(valid) = U::valid_crossovers() {
        if !valid.contains(&configuration.crossover_configuration.method) {
            return Err(GaError::ConfigurationError(format!(
                "Crossover::{:?} is not valid for this chromosome type. Valid: {:?}",
                configuration.crossover_configuration.method, valid
            )));
        }
    }
    if let Some(valid) = U::valid_mutations() {
        if !valid.contains(&configuration.mutation_configuration.method) {
            return Err(GaError::ConfigurationError(format!(
                "Mutation::{:?} is not valid for this chromosome type. Valid: {:?}",
                configuration.mutation_configuration.method, valid
            )));
        }
    }
    Ok(())
}
```

---

### `src/operations/crossover/multi_group_pmx.rs` (service, transform)

**Analog:** `src/operations/crossover/pmx.rs`

**Key prerequisite:** `pmx_build_child` in `pmx.rs` (line 70) is currently `fn pmx_build_child` (module-private). Must be changed to `pub(crate)` before this file can call it.

**Core pattern** (modeled on pmx.rs lines 23-64, sliced by group_ranges):
```rust
use crate::error::GaError;
use crate::traits::LinearChromosome;
use std::borrow::Cow;

pub fn multi_group_pmx<U>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome + GroupRangesAccessor,  // see note below
{
    // Call group_ranges() via a helper trait or concrete downcast
    let groups = parent_1.group_ranges();
    let p1_dna = parent_1.dna();
    let p2_dna = parent_2.dna();

    let mut child_dna_1 = p1_dna.to_vec();
    let mut child_dna_2 = p2_dna.to_vec();

    for (start, end) in &groups {
        // Reuse pmx_build_child (must be pub(crate) in pmx.rs)
        let slice_1 = pmx::pmx_build_child(
            &p1_dna[*start..=*end], &p2_dna[*start..=*end], 0, end - start,
        );
        let slice_2 = pmx::pmx_build_child(
            &p2_dna[*start..=*end], &p1_dna[*start..=*end], 0, end - start,
        );
        child_dna_1[*start..=*end].clone_from_slice(&slice_1);
        child_dna_2[*start..=*end].clone_from_slice(&slice_2);
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));
    Ok(vec![child_1, child_2])
}
```
Note: `group_ranges()` is a method on `MultiUniqueChromosome<T>`, not on `LinearChromosome`. The planner must decide the dispatch approach: either a `GroupRangesAccessor` helper trait, or implement `multi_group_pmx` as a concrete function typed for `MultiUniqueChromosome<T>` directly. The latter avoids a new trait and is simpler for v3.0.0.

**Visibility change required in pmx.rs** (line 70):
```rust
// Change from:
fn pmx_build_child<G: GeneT>(donor: &[G], other: &[G], start: usize, end: usize) -> Vec<G>
// Change to:
pub(crate) fn pmx_build_child<G: GeneT>(donor: &[G], other: &[G], start: usize, end: usize) -> Vec<G>
```

---

### `src/operations/crossover/multi_group_ox.rs` (service, transform)

**Analog:** `src/operations/crossover/order.rs`

**Key prerequisite:** `ox_build_child` in `order.rs` (line 58) is currently `fn ox_build_child` (module-private). Must be changed to `pub(crate)`.

**Core pattern** (modeled on order.rs lines 19-55, sliced by group_ranges):
```rust
pub fn multi_group_ox<U>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome,  // concrete type for MultiUniqueChromosome<T> or trait-based
{
    let groups = parent_1.group_ranges(); // same dispatch decision as multi_group_pmx
    let p1_dna = parent_1.dna();
    let p2_dna = parent_2.dna();

    let mut child_dna_1 = p1_dna.to_vec();
    let mut child_dna_2 = p2_dna.to_vec();

    let mut rng = crate::rng::make_rng();

    for (start, end) in &groups {
        let group_len = end - start + 1;
        let mut p1_pos = rng.random_range(0..group_len);
        let mut p2_pos = rng.random_range(0..group_len);
        while p1_pos == p2_pos { p2_pos = rng.random_range(0..group_len); }
        if p1_pos > p2_pos { std::mem::swap(&mut p1_pos, &mut p2_pos); }

        // Reuse ox_build_child (must be pub(crate) in order.rs)
        let slice_1 = order::ox_build_child(
            &p1_dna[*start..=*end], &p2_dna[*start..=*end], p1_pos, p2_pos,
        );
        let slice_2 = order::ox_build_child(
            &p2_dna[*start..=*end], &p1_dna[*start..=*end], p1_pos, p2_pos,
        );
        child_dna_1[*start..=*end].clone_from_slice(&slice_1);
        child_dna_2[*start..=*end].clone_from_slice(&slice_2);
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));
    Ok(vec![child_1, child_2])
}
```

**Visibility change required in order.rs** (line 58):
```rust
// Change from:
fn ox_build_child<G: crate::traits::GeneT>(donor: &[G], filler: &[G], p1: usize, p2: usize) -> Vec<G>
// Change to:
pub(crate) fn ox_build_child<G: crate::traits::GeneT>(donor: &[G], filler: &[G], p1: usize, p2: usize) -> Vec<G>
```

---

### Module Re-export Files (config role)

#### `src/types/genotypes/mod.rs` (modified)

**Analog:** current file (lines 1-16)
```rust
// ADD alongside existing entries:
pub mod unique;
pub mod multi_range;

pub use unique::UniqueGenotype;
pub use multi_range::MultiRangeGenotype;
```

#### `src/types/chromosomes/mod.rs` (modified)

**Analog:** current file (lines 1-18)
```rust
// ADD alongside existing entries:
pub mod unique;
pub mod multi_range;
pub mod multi_unique;

pub use unique::UniqueChromosome;
pub use multi_range::MultiRangeChromosome;
pub use multi_unique::MultiUniqueChromosome;
```
Note: `mod range` is currently private (`mod range` without `pub`). New chromosome modules should be `pub mod` to match the pattern of `binary`, `list`.

#### `src/initializers.rs` (modified)

**Analog:** current file (lines 14-22)
```rust
// ADD alongside existing entries:
pub mod unique_initializer;
pub mod multi_range_initializer;

pub use unique_initializer::*;
pub use multi_range_initializer::*;
```

#### `src/traits.rs` (modified)

**Analog:** current file (lines 37-55)
```rust
// ADD alongside existing entries:
pub mod operator_compat;

pub use operator_compat::OperatorCompat;
```

#### `src/operations.rs` (modified — Crossover enum)

**Analog:** current file (lines 75-111). Add two new variants after `EdgeRecombination` (line 110):
```rust
/// Multi-group PMX crossover for `MultiUniqueChromosome<T>`.
/// Applies PMX independently within each permutation group defined by `group_ranges()`.
MultiGroupPmx,
/// Multi-group OX crossover for `MultiUniqueChromosome<T>`.
/// Applies OX independently within each permutation group defined by `group_ranges()`.
MultiGroupOx,
```

#### `src/operations/crossover.rs` (modified — dispatch arms)

**Analog:** current file lines 155-194. Add two new match arms in `impl CrossoverOperator for Crossover`:
```rust
Crossover::MultiGroupPmx => multi_group_pmx(parent_1, parent_2),
Crossover::MultiGroupOx => multi_group_ox(parent_1, parent_2),
```
And add the same in `impl CrossoverOperator for CrossoverConfiguration`.

Also add module declarations and imports at the top of crossover.rs:
```rust
pub mod multi_group_pmx;
pub mod multi_group_ox;
use multi_group_pmx::multi_group_pmx;
use multi_group_ox::multi_group_ox;
```

#### `src/engines/ga.rs` (modified — build() method)

**Analog:** current file lines 716-755. Add the operator compat check after the existing `ValidatorFactory::validate` call (line 726-734) and before the fitness cache wrapping (line 737):
```rust
// After ValidatorFactory::validate::<U>(...)?; (line 734)
// Add:
crate::validators::generic_validator::operator_compat_check::<U>(&self.configuration)?;
```
The `build()` function already has `U: LinearChromosome` in scope (via the `Ga<U>` struct bound). Adding `U: OperatorCompat` to the function is safe because of the blanket impl in `operator_compat.rs`.

---

### `examples/job_scheduling.rs` (migration)

**Analog:** current file (all 188 lines)

**Changes required:**
1. Replace imports (lines 47-59): swap `Range as RangeChromosome`, `Range as RangeGenotype` for `UniqueChromosome`, `UniqueGenotype`
2. Replace fitness function signature (line 90): `|dna: &[RangeGenotype<i32>]|` → `|dna: &[UniqueGenotype<i32>]|`; `gene.value` still works (same field name on `UniqueGenotype<T>`)
3. Replace alleles + manual shuffle initializer (lines 107-125) with `unique_random_initialization`:
```rust
// Replace:
let alleles = vec![RangeGenotype::new(0, vec![(0, N_JOBS as i32 - 1)], 0)];
let _alleles_clone = alleles.clone();
// ...and the with_initialization_fn closure...

// With:
use genetic_algorithms::initializers::unique_random_initialization;
let alphabet: Vec<i32> = (0..N_JOBS as i32).collect();
// In the builder:
.with_initialization_fn({
    let alphabet = alphabet.clone();
    move |_n, _| unique_random_initialization(&alphabet)
})
```
4. Replace `RangeChromosome<i32>` type annotation in `run_with_callback` closure (line 156): → `UniqueChromosome<i32>`
5. Remove the `rand::seq::SliceRandom` import that was only needed for the manual shuffle
6. Update module doc comment lines 15-16 to reflect `UniqueChromosome<i32>`

---

## Shared Patterns

### Serde Conditional Attrs
**Source:** `src/types/genotypes/range.rs` lines 36-43 and `src/types/chromosomes/range.rs` lines 35-41
**Apply to:** All new gene and chromosome structs
```rust
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
```

### `fitness_fn` Field Skip Under Serde
**Source:** `src/types/chromosomes/range.rs` line 47, `src/types/chromosomes/list.rs` line 48
**Apply to:** All new chromosome structs
```rust
#[cfg_attr(feature = "serde", serde(skip, default))]
pub fitness_fn: FitnessFnWrapper<SomeGeneType<T>>,
```

### FitnessFnWrapper Usage
**Source:** `src/types/chromosomes/range.rs` lines 112-136, `src/types/chromosomes/list.rs` lines 116-140
**Apply to:** All new chromosome `ChromosomeT` impls
```rust
fn calculate_fitness(&mut self) {
    self.fitness = self.fitness_fn.call(&self.dna);
}
```

### Cow<[Gene]> in set_dna
**Source:** `src/types/chromosomes/range.rs` lines 150-157, `src/types/chromosomes/list.rs` lines 154-161
**Apply to:** All new chromosome `LinearChromosome` impls
```rust
fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
    self.dna = match dna {
        Cow::Borrowed(slice) => slice.to_vec(),
        Cow::Owned(vec) => vec,
    };
    self
}
```

### make_rng() in Initializers
**Source:** `src/initializers/list_initializer.rs` line 114, `src/initializers/range_initializer.rs` line 44
**Apply to:** Both new initializer files
```rust
let mut rng = crate::rng::make_rng();
```

### Arc<[T]> from Vec<T>
**Source:** `src/types/genotypes/range.rs` line 104 (`ranges.into_boxed_slice().into()`)
**Apply to:** `UniqueChromosome::new()`, `MultiUniqueChromosome` group construction
```rust
let arc: Arc<[T]> = vec.into_boxed_slice().into();
// For empty default:
let arc: Arc<[T]> = Arc::from([]);  // as used in Range<T>::default() line 73
```

### WASM Gate (no new code needed in this phase)
**Source:** `src/engines/ga.rs` (par_iter gates)
**Apply to:** Any code that would use `Instant::now()` or `par_iter()` — none expected in this phase since initializers are sequential (Fisher-Yates is inherently sequential)

### Validator Function Shape
**Source:** `src/validators/generic_validator.rs` lines 83-98 (`unique_gene_ids`)
**Apply to:** `operator_compat_check` in generic_validator.rs — same function signature style: standalone `pub fn` accepting references, returning `Result<(), GaError>`, called from `validate()` or directly from `build()`.

### GaError::ConfigurationError
**Source:** `src/validators/generic_validator.rs` lines 107-112 (`fitness_target_is_some`), lines 150-152 (`chromosome_length_not_bigger_than_alleles`)
**Apply to:** `operator_compat_check` error returns and any `Ga::build()` validation additions
```rust
Err(GaError::ConfigurationError(format!("...")))
```

---

## No Analog Found

All files have close analogs in the codebase. No files require falling back to RESEARCH.md patterns exclusively.

| File | Role | Data Flow | Note |
|------|------|-----------|------|
| `src/traits/operator_compat.rs` | middleware | request-response | New trait with no prior analog; RESEARCH.md pattern is authoritative (D-04). Trait file shape borrowed from `src/traits/linear_chromosome.rs`. |

---

## Metadata

**Analog search scope:** `src/types/`, `src/initializers/`, `src/traits/`, `src/operations/crossover/`, `src/operations/mutation/`, `src/validators/`, `src/engines/`, `examples/`
**Files scanned:** 18
**Pattern extraction date:** 2026-05-21
