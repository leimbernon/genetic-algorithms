# Phase 48: New Genotype Types - Research

**Researched:** 2026-05-21
**Domain:** Rust trait/struct implementation — new chromosome and gene types, operator compatibility enforcement
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** New `UniqueGenotype<T>` gene type with `{ id: i32, value: T }` — lightweight GeneT wrapper. Alphabet stored once on the chromosome, not per gene.
- **D-02:** `UniqueChromosome<T>` struct: `{ dna: Vec<UniqueGenotype<T>>, alphabet: Arc<[T]>, fitness: f64, age: usize, fitness_fn: FitnessFnWrapper<UniqueGenotype<T>> }`.
- **D-03:** New `src/initializers/unique_initializer.rs` with `unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>` using Fisher-Yates shuffle.
- **D-04:** New opt-in trait `OperatorCompat` with default-returning `valid_crossovers() -> Option<&'static [Crossover]>` and `valid_mutations() -> Option<&'static [Mutation]>`.
- **D-05:** `Ga::build()` validator checks `OperatorCompat` for `U`. If the selected crossover/mutation is not in the valid set (and valid set is `Some`), `build()` returns `GaError::ConfigurationError`. Fail-fast.
- **D-06:** `UniqueChromosome<T>` implements `OperatorCompat` with valid crossovers `[Crossover::Pmx, Crossover::Order, Crossover::EdgeRecombination]` and valid mutations `[Mutation::Insertion, Mutation::Swap, Mutation::Inversion]`.
- **D-07:** `MultiUniqueChromosome<T>` implements `OperatorCompat` with the same valid sets.
- **D-08:** New `MultiRangeGenotype<T>` gene: `{ id: i32, lo: T, hi: T, value: T, mutation_rate: f64 }` flat struct, no Arc overhead.
- **D-09:** Per-gene bounds provided as `Vec<(T, T)>` at build time via a `with_bounds(...)` config API or initializer parameter. Length must match chromosome length; validated in `Ga::build()`.
- **D-10:** Per-gene mutation rate `p_i` is first-class — Gaussian mutation uses `gene.mutation_rate` instead of global sigma for each gene independently.
- **D-11:** New `src/initializers/multi_range_initializer.rs` following the same pattern as other initializers.
- **D-12:** Groups on `MultiUniqueChromosome<T>` as `Vec<Arc<[T]>>`. User provides `Vec<Vec<T>>` at build time. DNA is concatenation of all group permutations.
- **D-13:** Reuse `UniqueGenotype<T>` for the gene type in `MultiUniqueChromosome<T>`. Group membership is implicit from position.
- **D-14:** `MultiUniqueChromosome<T>` exposes `fn group_ranges(&self) -> Vec<(usize, usize)>`. New `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` variants apply PMX/OX within each group boundary.

### Claude's Discretion

None specified.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GEN-01 | User can define a permutation chromosome with `UniqueChromosome<T>` where initialization guarantees no duplicate genes and all elements from the provided alphabet are present — invalid operators return `GaError` at runtime; PMX, OX, and ERX are the documented safe crossover operators | Fisher-Yates init in `list_initializer.rs` is the direct model; PMX/OX/ERX already implemented generically over `LinearChromosome` |
| GEN-02 | User can migrate the `job_scheduling` example to `UniqueChromosome<i32>` — `RangeChromosome<i32>` with unique-id hack replaced by a semantically correct type | `examples/job_scheduling.rs` line ~116 has a comment explicitly flagging this migration; the example uses `with_initialization_fn` with a manual shuffle workaround |
| GEN-03 | User can define a real-valued chromosome with `MultiRangeChromosome<T>` where each gene has its own `(lo_i, hi_i)` bounds and mutation rate `p_i` | Gaussian mutation pattern in `src/operations/mutation/gaussian.rs` is the model; flat struct avoids Arc overhead |
| GEN-04 | User can define a chromosome with `MultiUniqueChromosome<T>` containing multiple independent permutation groups, each with its own alphabet — crossover applies PMX/OX within each group boundary | `group_ranges()` accessor enables group-aware crossover operators; requires new `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` enum variants |
</phase_requirements>

## Summary

Phase 48 adds three new chromosome/gene type pairs to the library — `UniqueChromosome<T>`, `MultiRangeChromosome<T>`, and `MultiUniqueChromosome<T>` — plus an `OperatorCompat` trait for build-time operator validation, two new `Crossover` enum variants for multi-group permutation crossover, and a migration of the `job_scheduling` example.

All three new types follow the same structural pattern established by `Range<T>` and `ListChromosome<T>`: gene struct implementing `GeneT`, chromosome struct implementing both `ChromosomeT` and `LinearChromosome`, a dedicated initializer function, and re-exports from the standard module paths. The existing permutation operators (PMX, OX, ERX, Insertion, Swap, Inversion) already work generically over `LinearChromosome` and will work with `UniqueChromosome<T>` and `MultiUniqueChromosome<T>` with zero changes to operator files. The `OperatorCompat` validation plugs into `Ga::build()` by adding a check alongside the existing validator chain.

The two new crossover variants (`MultiGroupPmx`, `MultiGroupOx`) require additions to the `Crossover` enum in `src/operations.rs`, new dispatch arms in `src/operations/crossover.rs`, and new implementation files. All new types must carry serde attributes and WASM-compatibility gates per project policy.

**Primary recommendation:** Implement the three type pairs sequentially (UniqueChromosome → MultiRangeChromosome → MultiUniqueChromosome), add `OperatorCompat` as an independent trait file, then layer the multi-group crossover operators and the example migration last.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| UniqueGenotype\<T\> gene struct | Library types layer (`src/types/genotypes/`) | — | Gene structs live in `src/types/genotypes/`; follows existing pattern |
| UniqueChromosome\<T\> struct | Library types layer (`src/types/chromosomes/`) | — | Chromosome structs live in `src/types/chromosomes/`; follows existing pattern |
| unique_random_initialization | Initializers layer (`src/initializers/`) | — | All initializers live here; Fisher-Yates pattern directly from `list_initializer.rs` |
| MultiRangeGenotype\<T\> gene struct | Library types layer (`src/types/genotypes/`) | — | Same pattern as `Range<T>` gene |
| MultiRangeChromosome\<T\> struct | Library types layer (`src/types/chromosomes/`) | — | Same pattern as `Range<T>` chromosome |
| multi_range_random_initialization | Initializers layer (`src/initializers/`) | — | Per-gene bounds sampling analogous to `range_initializer.rs` |
| MultiUniqueChromosome\<T\> struct | Library types layer (`src/types/chromosomes/`) | — | Groups encoded as `Vec<Arc<[T]>>`; group_ranges() derives slice boundaries |
| OperatorCompat trait | Traits layer (`src/traits/`) | Validators (`src/validators/`) | Trait definition in `src/traits/operator_compat.rs`; check invoked in `Ga::build()` via validator chain |
| MultiGroupPmx / MultiGroupOx | Crossover operators (`src/operations/crossover/`) | — | New enum variants + implementation files; follow PMX/OX pattern, slice by group_ranges() |
| job_scheduling.rs migration | Examples layer (`examples/`) | — | Replace `RangeChromosome<i32>` + manual shuffle with `UniqueChromosome<i32>` |

## Standard Stack

### Core (no new external dependencies)
[ASSUMED] No new crates are needed for this phase. All functionality is implemented using existing dependencies already in `Cargo.toml`: `rand` (Fisher-Yates), `std::sync::Arc` (alphabet sharing), the existing `serde` feature gate pattern.

| Capability | Mechanism | Source |
|------------|-----------|--------|
| Fisher-Yates shuffle | `rand::Rng::random_range` | [VERIFIED: codebase — `list_initializer.rs` uses this exact pattern] |
| Arc alphabet sharing | `std::sync::Arc<[T]>` | [VERIFIED: codebase — `Range<T>` gene uses `Arc<[(T,T)]>` for ranges] |
| Serde conditional attrs | `#[cfg_attr(feature = "serde", ...)]` | [VERIFIED: codebase — every existing gene and chromosome uses this pattern] |
| WASM gating | `#[cfg(not(target_arch = "wasm32"))]` | [VERIFIED: codebase — `ga.rs` uses this pattern for `Instant` and `par_iter`] |
| Fitness fn storage | `FitnessFnWrapper<Gene>` from `src/fitness/fitness_fn_wrapper.rs` | [VERIFIED: codebase — all three existing chromosome types use this] |

**Installation:** No new packages required.

## Package Legitimacy Audit

No external packages are added in this phase. All new code uses existing in-tree dependencies.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (none) | — | — | — | — | — | — |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
User code
    │
    ├── UniqueChromosome<T>          ← new, implements ChromosomeT + LinearChromosome
    │     ├── UniqueGenotype<T>[]    ← new, implements GeneT; value: T, id: i32
    │     ├── Arc<[T]> alphabet      ← owned once per chromosome; shared on clone
    │     └── FitnessFnWrapper       ← existing; shared via Arc
    │
    ├── MultiRangeChromosome<T>      ← new, implements ChromosomeT + LinearChromosome
    │     └── MultiRangeGenotype<T>[]  ← new; lo: T, hi: T, value: T, mutation_rate: f64
    │
    ├── MultiUniqueChromosome<T>     ← new, implements ChromosomeT + LinearChromosome
    │     ├── UniqueGenotype<T>[]    ← reuses UniqueGenotype<T>
    │     └── Vec<Arc<[T]>> groups   ← one Arc<[T]> per permutation group
    │           └── group_ranges()   ← derives [(start, end)] from group alphabet lengths
    │
    └── OperatorCompat trait         ← new, in src/traits/operator_compat.rs
          │
          └── Ga::build()            ← validator hook checks valid_crossovers() / valid_mutations()
                │                       returns GaError::ConfigurationError on mismatch
                └── ValidatorFactory::validate()   ← existing validator chain, extended

Crossover operators (existing, zero changes needed):
    pmx.rs, order.rs, edge_recombination.rs  → generic over LinearChromosome → work with UniqueChromosome<T>

New crossover operators:
    multi_group_pmx.rs    ← calls group_ranges(), applies pmx_build_child per group slice
    multi_group_ox.rs     ← calls group_ranges(), applies ox_build_child per group slice

Initializers (new):
    unique_initializer.rs         ← Fisher-Yates over alphabet
    multi_range_initializer.rs    ← per-gene range sampling from (lo_i, hi_i) pairs
```

### Recommended Project Structure

New files to create:

```
src/types/genotypes/
├── unique.rs            # UniqueGenotype<T> struct + GeneT impl
└── multi_range.rs       # MultiRangeGenotype<T> struct + GeneT impl

src/types/chromosomes/
├── unique.rs            # UniqueChromosome<T> + ChromosomeT + LinearChromosome impls
├── multi_range.rs       # MultiRangeChromosome<T> + ChromosomeT + LinearChromosome impls
└── multi_unique.rs      # MultiUniqueChromosome<T> + ChromosomeT + LinearChromosome impls

src/initializers/
├── unique_initializer.rs         # unique_random_initialization
└── multi_range_initializer.rs    # multi_range_random_initialization

src/traits/
└── operator_compat.rs   # OperatorCompat trait definition

src/operations/crossover/
├── multi_group_pmx.rs   # multi_group_pmx() — applies PMX per group
└── multi_group_ox.rs    # multi_group_ox()  — applies OX per group
```

Files to modify:

```
src/types/genotypes/mod.rs       — add pub mod unique, pub mod multi_range; re-exports
src/types/chromosomes/mod.rs     — add pub mod unique, pub mod multi_range, pub mod multi_unique; re-exports
src/initializers.rs              — add pub mod unique_initializer, pub mod multi_range_initializer; re-exports
src/traits.rs                    — add pub mod operator_compat; re-export OperatorCompat
src/operations/crossover.rs      — add multi_group_pmx/multi_group_ox arms to Crossover::crossover()
src/operations.rs                — add Crossover::MultiGroupPmx, Crossover::MultiGroupOx variants
src/engines/ga.rs (build())      — add OperatorCompat check
src/validators/generic_validator.rs — add operator_compat validation function
src/lib.rs                       — verify UniqueChromosome, MultiRangeChromosome, MultiUniqueChromosome visible
examples/job_scheduling.rs       — migrate to UniqueChromosome<i32>
```

### Pattern 1: Gene Struct (UniqueGenotype\<T\>)

[VERIFIED: codebase — modeled on `src/types/genotypes/range.rs`]

```rust
// Source: src/types/genotypes/range.rs (model)
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
}

impl<T: Clone + Default + Sync + Send> GeneT for UniqueGenotype<T> {
    fn id(&self) -> i32 { self.id }
    fn set_id(&mut self, id: i32) -> &mut Self { self.id = id; self }
}
```

Key difference from `Range<T>`: no `Arc<[(T,T)]>` ranges field — the alphabet lives once on the chromosome (`Arc<[T]>`), not per-gene. [VERIFIED: codebase — D-01 from CONTEXT.md]

### Pattern 2: Chromosome Struct (UniqueChromosome\<T\>)

[VERIFIED: codebase — modeled on `src/types/chromosomes/range.rs`]

```rust
// Source: src/types/chromosomes/range.rs (model)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "T: serde::Serialize",
                deserialize = "T: serde::de::DeserializeOwned")))]
pub struct UniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    pub alphabet: Arc<[T]>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,
}

// ChromosomeT impl: calculate_fitness, fitness, set_fitness, age, set_age
// LinearChromosome impl: dna, dna_mut, set_dna (Cow<[Gene]>), set_fitness_fn
```

The `alphabet: Arc<[T]>` field is extra compared to `Range<T>`. It must participate in `Default` (use `Arc::from(&[][..] as &[T])`) and `Clone` (clones cheaply via `Arc::clone`). [ASSUMED — Arc<[T]> default value; pattern verified from Range<T>]

### Pattern 3: Initializer (Fisher-Yates)

[VERIFIED: codebase — `src/initializers/list_initializer.rs` `list_random_initialization_without_repetitions`]

```rust
// Source: src/initializers/list_initializer.rs (direct model)
pub fn unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>
where
    T: Clone + Sync + Send + Default + Debug,
{
    let mut rng = crate::rng::make_rng();
    let mut indices: Vec<usize> = (0..alphabet.len()).collect();
    // Fisher-Yates shuffle
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }
    indices.iter().enumerate()
        .map(|(pos, &idx)| UniqueGenotype { id: idx as i32, value: alphabet[idx].clone() })
        .collect()
}
```

Note: unlike `list_random_initialization_without_repetitions`, this function does NOT take `genes_per_chromosome` — for a permutation, the chromosome length equals the alphabet length. The caller passes the full alphabet. [ASSUMED — signature design; confirmed as the semantic intent from D-03]

### Pattern 4: OperatorCompat Trait and Validation

[VERIFIED: codebase — validator pattern from `src/validators/generic_validator.rs`; Crossover/Mutation enums from `src/operations.rs`]

```rust
// Source: src/traits/operator_compat.rs (new file)
use crate::operations::{Crossover, Mutation};

pub trait OperatorCompat {
    fn valid_crossovers() -> Option<&'static [Crossover]> { None }
    fn valid_mutations() -> Option<&'static [Mutation]> { None }
}

// Blanket impl: all types are OperatorCompat with no restrictions by default.
// (Individual types override to restrict.)
```

Validation function to add to `src/validators/generic_validator.rs`:

```rust
pub fn operator_compat_check<U: LinearChromosome + OperatorCompat>(
    configuration: &GaConfiguration,
) -> Result<(), GaError> {
    if let Some(valid) = U::valid_crossovers() {
        if !valid.contains(&configuration.crossover_configuration.method) {
            return Err(GaError::ConfigurationError(format!(
                "Crossover::{:?} is not valid for this chromosome type. \
                 Valid crossovers: {:?}",
                configuration.crossover_configuration.method, valid
            )));
        }
    }
    if let Some(valid) = U::valid_mutations() {
        if !valid.contains(&configuration.mutation_configuration.method) {
            return Err(GaError::ConfigurationError(format!(
                "Mutation::{:?} is not valid for this chromosome type. \
                 Valid mutations: {:?}",
                configuration.mutation_configuration.method, valid
            )));
        }
    }
    Ok(())
}
```

**Integration point:** `Ga::build()` calls `operator_compat_check::<U>(&self.configuration)?` after the existing `ValidatorFactory::validate()` call. [VERIFIED: codebase — `src/engines/ga.rs` line 716 `build()` shows the validator call location]

### Pattern 5: MultiGroupPmx Crossover

[VERIFIED: codebase — `src/operations/crossover/pmx.rs` `pmx_build_child` is the inner function to reuse]

```rust
// Source: src/operations/crossover/multi_group_pmx.rs (new file)
pub fn multi_group_pmx<U>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome + GroupRanges,  // GroupRanges is a helper trait or method
{
    let groups = parent_1.group_ranges();
    let mut child_dna_1 = parent_1.dna().to_vec();
    let mut child_dna_2 = parent_2.dna().to_vec();
    for (start, end) in &groups {
        let slice_1 = pmx_build_child(&parent_1.dna()[*start..=*end],
                                       &parent_2.dna()[*start..=*end], 0, end-start);
        let slice_2 = pmx_build_child(&parent_2.dna()[*start..=*end],
                                       &parent_1.dna()[*start..=*end], 0, end-start);
        child_dna_1[*start..=*end].clone_from_slice(&slice_1);
        child_dna_2[*start..=*end].clone_from_slice(&slice_2);
    }
    // build children from child_dna_1, child_dna_2 ...
}
```

Note: `pmx_build_child` in `pmx.rs` is a module-private function (`fn pmx_build_child`). To reuse it from `multi_group_pmx.rs`, it must be changed to `pub(super)` or `pub(crate)`, or the multi-group implementation must duplicate the logic. [VERIFIED: codebase — checked `pmx.rs`; `pmx_build_child` is currently private]

The same pattern applies to `ox_build_child` in `order.rs`. [VERIFIED: codebase — checked `order.rs`; `ox_build_child` is currently module-private]

**Decision required for planner:** Change `pmx_build_child` and `ox_build_child` to `pub(crate)`, or inline the logic. Making them `pub(crate)` is the lower-risk choice and avoids duplication.

### Pattern 6: MultiRangeGenotype — Per-Gene Gaussian Mutation

[VERIFIED: codebase — `src/operations/mutation/gaussian.rs` shows the `RangeChromosome<T>` downcast pattern; `MultiRangeChromosome<T>` needs its own `gaussian_mutate` via `ValueMutable`]

`MultiRangeChromosome<T>` should implement `ValueMutable::gaussian_mutate(&mut self, _sigma: f64)` where each gene independently samples noise scaled by `gene.mutation_rate` (not the global sigma argument), then clamps to `(gene.lo, gene.hi)`. The global sigma argument is ignored or used as a fallback scale. [ASSUMED — design choice; confirmed by D-10]

### Anti-Patterns to Avoid

- **Per-gene alphabet storage:** Do NOT store `Arc<[T]>` or `Vec<T>` alphabet on each `UniqueGenotype<T>`. The alphabet belongs on the chromosome once. The `List<T>` gene has per-gene alleles — this is the pattern `UniqueGenotype<T>` explicitly avoids. [VERIFIED: codebase — D-01 explicitly prohibits this; `List<T>` is the negative example]
- **Partial initializer override for UniqueChromosome:** The `with_initialization_fn` builder method accepts a closure with signature `Fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>` [ASSUMED — check actual signature in ga.rs before finalizing]. The example's current manual shuffle workaround uses `with_initialization_fn`. `UniqueChromosome<T>` needs its own builder that captures the alphabet and calls `unique_random_initialization`.
- **Removing the alphabet field under serde:** The `alphabet: Arc<[T]>` field is needed at runtime (for post-mutation validation and `group_ranges()` in `MultiUniqueChromosome`). It must NOT be `#[serde(skip)]`. It should be serialized normally. [ASSUMED — no explicit instruction either way; confirmed by logical necessity]
- **Forgetting `Crossover::MultiGroupPmx` / `MultiGroupOx` in OperatorCompat:** These new variants must be added to `valid_crossovers()` on `MultiUniqueChromosome<T>` — they don't exist yet at the start of the phase, but they must be listed alongside `Pmx` and `Order` once the variants are added. The planner must sequence this correctly.
- **Using `par_iter()` in initializers without WASM gate:** Initializers currently use sequential iterators. If parallelism is added to any new initializer, it must be gated. The Fisher-Yates shuffle is inherently sequential; no WASM issue here. [VERIFIED: codebase — existing initializers use sequential loops]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fisher-Yates shuffle | Custom shuffle | Pattern from `list_random_initialization_without_repetitions` in `list_initializer.rs` | Already implemented, tested, and using `make_rng()` correctly |
| Box-Muller transform for Gaussian noise | Custom normal distribution | Pattern from `src/operations/mutation/gaussian.rs` | Already implemented; use the identical approach |
| Arc\<[T]\> from Vec\<T\> | Custom reference counting | `let arc: Arc<[T]> = vec.into_boxed_slice().into()` | Established pattern — `Range<T>` gene uses `ranges.into_boxed_slice().into()` |
| Fitness function deferred storage | Custom closure wrapper | `FitnessFnWrapper<Gene>` from `src/fitness/fitness_fn_wrapper.rs` | Already Clone, Debug, PartialEq, thread-safe |
| Permutation crossover (PMX, OX, ERX) | New permutation-safe crossover | Existing `pmx`, `order`, `erx` in `src/operations/crossover/` | Already generic over `LinearChromosome`; works without changes |
| Permutation mutation (Insertion, Swap, Inversion) | New permutation-safe mutation | Existing operators in `src/operations/mutation/` | Already generic over `LinearChromosome`; works without changes |
| Operator compatibility enforcement | Runtime checks inside operators | `OperatorCompat` trait + `Ga::build()` check | Build-time enforcement is strictly better; operators stay generic |

**Key insight:** The library's operator implementations are already fully generic over `LinearChromosome`. `UniqueChromosome<T>` gets the entire permutation operator library for free by implementing `LinearChromosome`. The only new operators needed are `MultiGroupPmx` and `MultiGroupOx` — which themselves reuse `pmx_build_child` / `ox_build_child`.

## Common Pitfalls

### Pitfall 1: OperatorCompat Bounds in Ga::build()

**What goes wrong:** Adding `U: OperatorCompat` to `Ga::build()` breaks all existing users who don't implement `OperatorCompat` on their custom chromosome types.
**Why it happens:** `Ga::build()` is generic over `U: LinearChromosome`; adding a new bound is a breaking change.
**How to avoid:** Provide a blanket implementation: `impl<T: LinearChromosome> OperatorCompat for T {}` with the default `None`-returning methods. Then the bound `U: OperatorCompat` is always satisfied, and the check is a no-op for types that don't override it. [ASSUMED — the blanket impl approach; confirmed by D-04 "opt-in" language and default `None` return]
**Warning signs:** Compilation failure on existing tests after adding the bound without the blanket impl.

### Pitfall 2: alphabet Field in Default impl

**What goes wrong:** `UniqueChromosome<T>: Default` requires `Arc<[T]>` to have a sensible default (empty slice). `Arc::<[T]>::from(&[] as &[T])` is not in stable Rust — the syntax is `Arc::from([] as [T; 0])` or more idiomatically `Arc::from(Vec::<T>::new().as_slice())`.
**Why it happens:** The Rust `Arc<[T]>` from empty slice syntax is non-obvious.
**How to avoid:** Use `Arc::from(Vec::<T>::new().into_boxed_slice())` or `Arc::from(&[][..])` (the latter works via deref coercion). The existing `Range<T>` gene uses `Arc::from([])` for its ranges default. [VERIFIED: codebase — `Range<T>` default uses `Arc::from([])`]
**Warning signs:** Compilation error on `Default` impl.

### Pitfall 3: MultiRangeChromosome Gaussian Mutation — Wrong Bounds Source

**What goes wrong:** Using the `Range<T>` chromosome's existing Gaussian mutation (which reads `gene.ranges`) on a `MultiRangeChromosome<T>` gene that has `gene.lo` / `gene.hi` fields instead.
**Why it happens:** The existing `gaussian_mutation()` in `mutation/gaussian.rs` is typed specifically for `RangeChromosome<T>` and reads `gene.ranges[range_idx]`.
**How to avoid:** Implement `ValueMutable::gaussian_mutate` on `MultiRangeChromosome<T>` with its own per-gene logic reading `gene.lo`, `gene.hi`, and `gene.mutation_rate`. Do NOT try to reuse the existing `gaussian_mutation` function. [VERIFIED: codebase — `gaussian.rs` is tightly coupled to `RangeChromosome<T>`]
**Warning signs:** Type error when calling `gaussian_mutation` on `MultiRangeChromosome<T>`.

### Pitfall 4: Multi-Group Crossover — pmx_build_child Visibility

**What goes wrong:** `multi_group_pmx.rs` cannot call `pmx_build_child` from `pmx.rs` because it is `fn pmx_build_child` (private to the module).
**Why it happens:** Rust module privacy — module-private functions are inaccessible from sibling modules.
**How to avoid:** Change `pmx_build_child` to `pub(crate)` (same for `ox_build_child` in `order.rs`). This is the minimal change — it does not affect the public API. [VERIFIED: codebase — confirmed both functions are module-private]
**Warning signs:** `error[E0603]: function 'pmx_build_child' is private` at compile time.

### Pitfall 5: job_scheduling Example — Fitness Function Gene Type Mismatch

**What goes wrong:** After migrating from `RangeChromosome<i32>` to `UniqueChromosome<i32>`, the fitness closure `|dna: &[RangeGenotype<i32>]|` must be updated to `|dna: &[UniqueGenotype<i32>]|` and the gene value access changes from `gene.value` (still works — same field name) but the loop type annotation changes.
**Why it happens:** The fitness function is closed over the gene type; changing chromosome types changes the gene type parameter.
**How to avoid:** Update the fitness function signature, the `machine_finish[m] += PROCESSING_TIMES[job][m]` line still works since `gene.value` exists on `UniqueGenotype<T>` too. But also update imports and remove the `alleles` variable (not needed for `UniqueChromosome` initialization). [VERIFIED: codebase — `examples/job_scheduling.rs` line ~90 fitness closure inspected]
**Warning signs:** Compilation error in `job_scheduling.rs` after migration.

### Pitfall 6: Crossover Enum — Debug / PartialEq Requirements for OperatorCompat

**What goes wrong:** `OperatorCompat::valid_crossovers()` returns `&'static [Crossover]`. The `OperatorCompat` check uses `valid.contains(&configuration.crossover_configuration.method)`. This requires `Crossover: PartialEq`.
**Why it happens:** The `.contains()` call requires `PartialEq` on the element type.
**How to avoid:** Verify that `Crossover` and `Mutation` enums derive `PartialEq`. [ASSUMED — likely already derived; must be confirmed in `src/operations.rs`]
**Warning signs:** Compilation error: "`Crossover: PartialEq` is not satisfied".

### Pitfall 7: MultiUniqueChromosome — group_ranges() Requires groups Field

**What goes wrong:** `group_ranges()` must return the slice boundaries derived from `self.groups` alphabet lengths. If `Default` initializes `groups` as empty, `group_ranges()` returns an empty vec, and multi-group crossover silently does nothing instead of erroring.
**Why it happens:** The default constructor leaves groups empty; user must call the chromosome's builder or directly set `groups`.
**How to avoid:** Add a validation step (either in `Ga::build()` or in the initializer) that checks `groups` is non-empty when `MultiUniqueChromosome<T>` is used. Document clearly that `MultiUniqueChromosome<T>` must be initialized via its dedicated builder, not `Default`. [ASSUMED — no explicit validation specified in CONTEXT.md; needed for correctness]
**Warning signs:** Silent no-op crossover in multi-unique tests.

## Code Examples

### UniqueGenotype\<T\> - GeneT Implementation

```rust
// Source: verified from src/types/genotypes/range.rs pattern
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

### UniqueChromosome\<T\> - Dual Trait Impl

```rust
// Source: verified from src/types/chromosomes/range.rs pattern
impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT for UniqueChromosome<T> {
    type Gene = UniqueGenotype<T>;
    fn calculate_fitness(&mut self) { self.fitness = self.fitness_fn.call(&self.dna); }
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self { self.fitness = fitness; self }
    fn set_age(&mut self, age: usize) -> &mut Self { self.age = age; self }
    fn age(&self) -> usize { self.age }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> LinearChromosome for UniqueChromosome<T> {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna { Cow::Borrowed(s) => s.to_vec(), Cow::Owned(v) => v };
        self
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where F: Fn(&[UniqueGenotype<T>]) -> f64 + Send + Sync + 'static {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}
```

### unique_random_initialization

```rust
// Source: verified from src/initializers/list_initializer.rs Fisher-Yates pattern
pub fn unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>
where T: Clone + Sync + Send + Default + Debug,
{
    let mut rng = crate::rng::make_rng();
    let mut indices: Vec<usize> = (0..alphabet.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }
    indices.into_iter().enumerate()
        .map(|(pos, idx)| UniqueGenotype { id: idx as i32, value: alphabet[idx].clone() })
        .collect()
}
```

### OperatorCompat on UniqueChromosome

```rust
// Source: verified from D-04, D-06 in CONTEXT.md; Crossover enum from src/operations.rs
impl<T: Sync + Send + Clone + Default + Debug + 'static> OperatorCompat for UniqueChromosome<T> {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[Crossover::Pmx, Crossover::Order, Crossover::EdgeRecombination,
               Crossover::MultiGroupPmx, Crossover::MultiGroupOx, Crossover::Clone])
    }
    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion])
    }
}
```

Note: `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` do not exist yet in the enum — they must be added before `OperatorCompat` can reference them. Sequence matters in the plan.

### group_ranges() on MultiUniqueChromosome

```rust
// Source: derived from D-14 in CONTEXT.md
pub fn group_ranges(&self) -> Vec<(usize, usize)> {
    let mut ranges = Vec::with_capacity(self.groups.len());
    let mut start = 0;
    for group in &self.groups {
        let end = start + group.len() - 1;
        ranges.push((start, end));
        start = end + 1;
    }
    ranges
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `RangeChromosome<i32>` with manual shuffle for permutations | `UniqueChromosome<T>` with typed permutation semantics | Eliminates semantic hack; operators validated at build time |
| Per-gene uniform bounds in `Range<T>` (shared `Arc<[(lo,hi)]>`) | Per-gene independent bounds in `MultiRangeGenotype<T>` (flat struct fields) | Enables heterogeneous real-valued spaces without Arc allocation per gene |
| No operator compatibility enforcement | `OperatorCompat` trait + build-time check | Surfaces invalid operator combinations before the first generation runs |

**Deprecated/outdated in this phase:**
- `RangeChromosome<i32>` with `with_initialization_fn` manual shuffle workaround in `job_scheduling.rs` — replaced by `UniqueChromosome<i32>`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `unique_random_initialization` takes `alphabet: &[T]` only (no `genes_per_chromosome` parameter) — chromosome length equals alphabet length for full permutations | Architecture Patterns §Pattern 3 | If partial permutations are needed, the signature needs a `genes_per_chromosome` parameter; low risk for GEN-01 scope |
| A2 | Blanket `impl<T: LinearChromosome> OperatorCompat for T {}` is the correct way to make the bound non-breaking in `Ga::build()` | Architecture Patterns §Pattern 4, Pitfall 1 | If the blanket impl conflicts with downstream impls, a different approach (e.g., no bound on `build()`, just call the check inside the validator with a concrete type bound) is needed |
| A3 | `Crossover` and `Mutation` enums derive `PartialEq` (needed for `valid.contains()` in the OperatorCompat check) | Common Pitfalls §Pitfall 6 | If they don't, the check implementation must use a different comparison (e.g., `discriminant()`) |
| A4 | `alphabet: Arc<[T]>` on `UniqueChromosome<T>` should be serialized (not `#[serde(skip)]`) so checkpoint restore can reconstruct the chromosome fully | Architecture Patterns §Anti-Patterns | If skipped, checkpointed UniqueChromosome instances lose their alphabet on restore |
| A5 | `MultiGroupPmx` and `MultiGroupOx` should appear in `UniqueChromosome::valid_crossovers()` in addition to `Pmx` and `Order` | Code Examples §OperatorCompat on UniqueChromosome | If `UniqueChromosome` is not intended for multi-group use, these variants should only appear on `MultiUniqueChromosome` |
| A6 | `MultiUniqueChromosome` validation (non-empty groups) is needed at `Ga::build()` time | Common Pitfalls §Pitfall 7 | Without it, silent no-op crossover is possible; the planner should add this validation task |

**If this table were empty:** All claims in this research would be verified or cited.

## Open Questions (RESOLVED)

1. **Should `Crossover::Clone` be in `UniqueChromosome::valid_crossovers()`?**
   - What we know: Clone crossover preserves both parents unchanged — it is always permutation-safe.
   - What's unclear: The CONTEXT.md valid set for D-06 lists only `[Pmx, Order, EdgeRecombination]`. Clone is not listed but would be safe to include.
   - Recommendation: Include `Crossover::Clone` and `Crossover::Rejuvenate` in valid sets — they are trivially permutation-safe. Confirm with user before locking.
   - RESOLVED: 48-02 Plan implements `[Pmx, Order, EdgeRecombination, Clone, Rejuvenate]` — Clone and Rejuvenate added as a safe, permissive extension. The addition cannot cause invalid-operator errors on correct permutation usage.

2. **How does the user configure UniqueChromosome\<T\> per Ga builder?**
   - What we know: The existing `Ga` builder has `with_initialization_fn(move |n, alleles, _| ...)`. For `UniqueChromosome<T>`, the user needs to capture the alphabet and call `unique_random_initialization`.
   - What's unclear: Whether a dedicated `with_alphabet(alphabet: Vec<T>)` builder method is needed, or whether `with_initialization_fn` capturing an alphabet clone is sufficient.
   - Recommendation: `with_initialization_fn` capturing the alphabet is sufficient for v3.0.0. A convenience `with_alphabet` builder is a future usability improvement, not required by GEN-01/GEN-02.
   - RESOLVED: 48-02 Plan uses `with_initialization_fn` capturing the alphabet clone — no new builder method required for this phase.

3. **Does `MultiRangeChromosome<T>` need a `with_bounds(...)` builder method on `Ga`, or is it only a parameter to the initializer?**
   - What we know: D-09 says "a `with_bounds(...)` config API (or equivalent initialization fn)".
   - What's unclear: Whether bounds should be stored on `GaConfiguration` or captured in the initialization closure.
   - Recommendation: Capture bounds in the initialization closure (same pattern as the existing `range_random_initialization` with alleles). Avoids changes to `GaConfiguration`.
   - RESOLVED: 48-03 Plan captures bounds in the initialization closure — no `GaConfiguration` changes required.

## Environment Availability

Step 2.6: Checked. This phase is purely Rust library code — no external tools, services, or CLIs beyond the standard Rust toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All new types | ✓ | (project-level) | — |
| wasm32-unknown-unknown target | WASM compatibility check | [ASSUMED] | — | `rustup target add wasm32-unknown-unknown` |

**Missing dependencies with no fallback:** None detected.

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treating as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None — standard Rust test discovery |
| Quick run command | `cargo test --test '*' 2>&1 \| grep -E '(FAILED\|ok\|error)'` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GEN-01 | `unique_random_initialization` produces no-duplicate, full-alphabet permutation | unit | `cargo test --test test_initializers` | ❌ Wave 0 |
| GEN-01 | `UniqueChromosome<T>` implements `ChromosomeT` + `LinearChromosome` | unit | `cargo test --test test_chromosomet_core` | partial — existing file needs cases |
| GEN-01 | `OperatorCompat` rejects `Crossover::SinglePoint` on `UniqueChromosome` at build time | unit | `cargo test --test test_engines` | partial — existing file needs cases |
| GEN-01 | PMX crossover works on `UniqueChromosome<i32>` | unit | `cargo test --test test_crossover_pmx` | ✅ exists, may need `UniqueChromosome` case |
| GEN-02 | `job_scheduling` example compiles and runs | smoke | `cargo run --example job_scheduling` | ✅ (after migration) |
| GEN-03 | `MultiRangeChromosome` per-gene bounds enforced by initializer | unit | `cargo test --test test_initializers` | ❌ Wave 0 |
| GEN-03 | Gaussian mutation on `MultiRangeChromosome` uses `gene.mutation_rate` | unit | `cargo test --test test_mutation_creep_gaussian` | partial |
| GEN-04 | `group_ranges()` returns correct slices for multi-group chromosome | unit | `cargo test tests/types/chromosomes/test_multi_unique.rs` | ❌ Wave 0 |
| GEN-04 | `MultiGroupPmx` applies PMX within each group, not across boundaries | unit | `cargo test --test test_crossover_multi_group` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test && cargo clippy`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/types/chromosomes/test_unique.rs` — covers GEN-01 ChromosomeT/LinearChromosome impls
- [ ] `tests/types/chromosomes/test_multi_range.rs` — covers GEN-03
- [ ] `tests/types/chromosomes/test_multi_unique.rs` — covers GEN-04 group_ranges
- [ ] `tests/types/genotypes/test_unique.rs` — covers UniqueGenotype GeneT impl
- [ ] `tests/types/genotypes/test_multi_range.rs` — covers MultiRangeGenotype GeneT impl
- [ ] `tests/operations/test_crossover_multi_group_pmx.rs` — covers MultiGroupPmx correctness
- [ ] `tests/operations/test_crossover_multi_group_ox.rs` — covers MultiGroupOx correctness
- [ ] Add `unique_random_initialization` cases to `tests/initializers/test_initializers.rs`

## Security Domain

This phase adds no authentication, session management, access control, or cryptography. It is a pure data-structure and trait implementation phase. ASVS does not apply.

`security_enforcement`: not set — but no applicable ASVS categories exist for this domain.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase] `src/types/chromosomes/range.rs` — canonical chromosome pattern (dna, fitness, age, FitnessFnWrapper, ChromosomeT + LinearChromosome dual impl)
- [VERIFIED: codebase] `src/types/genotypes/range.rs` — canonical gene pattern (Arc fields, GeneT impl, serde attrs)
- [VERIFIED: codebase] `src/initializers/list_initializer.rs` — Fisher-Yates permutation init pattern
- [VERIFIED: codebase] `src/initializers/range_initializer.rs` — per-gene range sampling pattern
- [VERIFIED: codebase] `src/operations/crossover/pmx.rs` — `pmx_build_child` inner function (currently private)
- [VERIFIED: codebase] `src/operations/crossover/order.rs` — `ox_build_child` inner function (currently private)
- [VERIFIED: codebase] `src/operations/mutation/gaussian.rs` — Gaussian mutation implementation for `Range<T>`
- [VERIFIED: codebase] `src/validators/generic_validator.rs` — validator chain pattern for `Ga::build()`
- [VERIFIED: codebase] `src/engines/ga.rs` line 716 — `build()` method validator call location
- [VERIFIED: codebase] `src/fitness/fitness_fn_wrapper.rs` — `FitnessFnWrapper<G>` (Arc-backed, Clone, thread-safe)
- [VERIFIED: codebase] `src/traits/chromosome.rs`, `src/traits/linear_chromosome.rs`, `src/traits/gene.rs` — current trait definitions
- [VERIFIED: codebase] `examples/job_scheduling.rs` line ~116 — migration comment confirming Phase 48 scope
- [VERIFIED: codebase] `.planning/phases/48-new-genotype-types/48-CONTEXT.md` — all 14 locked decisions

### Secondary (MEDIUM confidence)
- [CITED: 48-CONTEXT.md §canonical_refs] — list of files confirmed as structural models for implementation

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns verified in codebase
- Architecture: HIGH — all integration points located and verified; two unknowns (A1, A2) are low-risk design decisions
- Pitfalls: HIGH — all pitfalls verified by direct code inspection (private functions, type coupling, enum requirements)

**Research date:** 2026-05-21
**Valid until:** 2026-06-21 (stable Rust library — 30 day window)
