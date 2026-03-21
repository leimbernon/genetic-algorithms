# Phase 7: List Genotype - Research

**Researched:** 2026-03-21
**Domain:** Rust generic types, trait implementation, genetic algorithm operator integration
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Allele set placement:** Each `List<T>` gene carries its own allele set: `List { id: i32, alleles: Vec<T>, value: T }`. Follows the `Range<T>` pattern (ranges stored per-gene), not per-chromosome. `alleles` is cloned on gene clone — no `Arc` needed. `List::new(id, alleles, value)` returns `Result<Self, GaError>` — validates that `value` is a member of `alleles` at construction time (always, not just debug builds).
- **Value-replacement mutation:** Add new `Mutation` enum variant `ListValue` (or `ListRandom`). Implementation: pick one random gene, replace its value with a **different** allele from its set. "Different" determined by allele index — no `PartialEq` on T required. Exactly one gene changes per call.
- **Type bounds on T:** `T: Clone + Sync + Send + Debug` — minimal, matching existing operator requirements. No `PartialEq` or `Hash` required. `gene.id` stores the **index** of the current value in `alleles`.
- **Initializer:** Dedicated `list_random_initialization` in `src/initializers/list_initializer.rs`. Two variants mirroring generic initializer: with repetition and without repetition. Both set `gene.id = allele_index`. Signature: `list_random_initialization(genes_per_chromosome: usize, alleles: Option<&[T]>, _: Option<bool>) -> Vec<List<T>>`.
- **Module structure:** `src/genotypes/list.rs` and `src/chromosomes/list.rs`.
- **Serde support:** `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` with appropriate bounds.
- **Error variant reuse:** `GaError::ValidationError` for invalid initial value.
- **Public re-exports:** Follow existing `pub use` patterns in `src/genotypes/mod.rs` (which is `src/genotypes.rs`), `src/chromosomes/mod.rs` (which is `src/chromosomes.rs`), `src/initializers.rs`.

### Claude's Discretion

- Module structure (listed in decisions section above as "Claude's Discretion" in CONTEXT.md).
- Serde support, error variant reuse, public re-exports — all pattern-match existing code.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LIST-01 | User can define a `List<T>` gene drawn from a finite allele set | `src/genotypes/list.rs` — struct `List<T>` + `GeneT` impl; constructor validates membership |
| LIST-02 | User can create a `List<T>` chromosome compatible with `ChromosomeT` | `src/chromosomes/list.rs` — struct `ListChromosome<T>` + `ChromosomeT` impl; mirrors `Range<T>` chromosome |
| LIST-03 | List chromosomes work with all existing selection, crossover, mutation, and survivor operators | Operators are fully generic over `ChromosomeT` — no operator changes needed; `ListChromosome` + `ValueMutable` impl for `ListValue` variant |
| LIST-04 | User can initialize a List population with a built-in initializer | `src/initializers/list_initializer.rs` with two functions; `id` = allele index; re-exported from `src/initializers.rs` |

</phase_requirements>

## Summary

Phase 7 is a pure new-type addition: a `List<T>` gene and `ListChromosome<T>` chromosome for problems over finite symbolic alphabets. The work is well-bounded by the existing `Range<T>` pattern: one gene file, one chromosome file, one initializer file, one new mutation variant, and module wiring in four files.

The key design insight already decided in CONTEXT.md is that `gene.id` doubles as the allele-index. This lets the mutation operator pick "a different allele" without needing `PartialEq` on T — it just excludes the current index when sampling. This pattern has no precedent in the existing codebase (Range and Binary genes do not need it) but is straightforward to implement.

All existing operators (`Swap`, `Inversion`, `Scramble`, `Insertion`, `Order`, `Pmx`, `Uniform`, `MultiPoint`, `SinglePoint`, `Cycle`, all selection variants, all survivor variants) operate entirely through `ChromosomeT` and will work on `ListChromosome<T>` without modification. The only new operator plumbing is adding `Mutation::ListValue` to the `Mutation` enum and its dispatch arm in `mutation.rs`.

**Primary recommendation:** Follow `Range<T>` chromosome as the structural template; follow `binary_initializer.rs` signature convention for the initializer; add `ListValue` to the `Mutation` enum using the `ValueMutable` extension path already established by `Mutation::Value`.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | workspace version | RNG for allele sampling | Already used by all initializers and mutation operators |
| `serde` (feature-gated) | workspace version | Checkpoint serialization | Used by all existing chromosome and gene types |

No new dependencies needed. This phase adds only source files.

**Installation:** No `cargo add` required — all dependencies are already in `Cargo.toml`.

## Architecture Patterns

### Recommended Project Structure (new files only)

```
src/
├── genotypes/
│   └── list.rs              # List<T> gene struct + GeneT impl
├── chromosomes/
│   └── list.rs              # ListChromosome<T> struct + ChromosomeT impl
├── initializers/
│   └── list_initializer.rs  # list_random_initialization{,_without_repetitions}
└── operations/
    └── mutation/
        └── list_value.rs    # list_value_mutation<T> function
```

Modified files:
- `src/genotypes.rs` — `pub mod list; pub use list::List;`
- `src/chromosomes.rs` — `pub mod list; pub use list::ListChromosome;`
- `src/initializers.rs` — `pub mod list_initializer; pub use list_initializer::*;`
- `src/operations.rs` — add `ListValue` variant to `Mutation` enum
- `src/operations/mutation.rs` — add `ListValue` dispatch arm + `pub mod list_value;`
- `src/lib.rs` — update module-level doc comment

### Pattern 1: Gene Struct (List<T>)

**What:** Struct with `id: i32` (allele index), `alleles: Vec<T>`, `value: T`. Constructor validates membership by index range only (since `id` IS the index, validate `id < alleles.len()`). Alternatively, validate that `value` equals `alleles[id]` — but since `T` has no `PartialEq`, validate purely by index: user passes `id` and the constructor checks `id < alleles.len()`.

**IMPORTANT DESIGN NOTE:** `List::new(id, alleles, value)` must validate that `id < alleles.len()` (the index is in-bounds). The `value` in the struct is the cached value at `alleles[id]`. The constructor should store `alleles[id as usize].clone()` as the value, making `value` always consistent with `id`. There is no need to accept a separate `value` argument — or if you do, ignore it and derive it from `alleles[id]`. This avoids any inconsistency between `id` and `value`.

**When to use:** Single gene for any T that is Clone + Sync + Send + Debug.

```rust
// Source: mirrors src/genotypes/range.rs pattern
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct List<T> {
    pub id: i32,          // index of current value in alleles
    pub alleles: Vec<T>,
    pub value: T,
}

impl<T: Clone + Sync + Send + Debug> GeneT for List<T> {
    fn id(&self) -> i32 { self.id }
    fn set_id(&mut self, id: i32) -> &mut Self {
        // Update both id and value consistently
        if (id as usize) < self.alleles.len() {
            self.id = id;
            self.value = self.alleles[id as usize].clone();
        }
        self
    }
}

impl<T: Clone + Debug + Default + Sync + Send> Default for List<T> {
    fn default() -> Self {
        Self { id: 0, alleles: Vec::new(), value: T::default() }
    }
}

impl<T: Clone + Debug> List<T> {
    pub fn new(id: i32, alleles: Vec<T>, _value: T) -> Result<Self, GaError> {
        if alleles.is_empty() {
            return Err(GaError::ValidationError("Allele set must not be empty".to_string()));
        }
        if id < 0 || (id as usize) >= alleles.len() {
            return Err(GaError::ValidationError(format!(
                "id {} is out of bounds for allele set of length {}",
                id, alleles.len()
            )));
        }
        let value = alleles[id as usize].clone();
        Ok(Self { id, alleles, value })
    }
}
```

**Bounds clarification:** `GeneT` requires `Default + Clone + Sync + Send`. `Default` on `List<T>` needs `T: Default`. However, users may have `T` types without `Default`. Resolution: implement `Default` with an empty `alleles` vec and `id = 0`, relying on `T: Default` for the `value` field. The `Default` instance is a sentinel — it will never be used directly in a GA (genes are always constructed via `List::new` or the initializer). This matches how `Range<T>` handles `Default` (it requires `T: Default`).

### Pattern 2: Chromosome Struct (ListChromosome<T>)

**What:** Generic chromosome wrapping `Vec<List<T>>` with fitness, age, and fitness_fn. Bounds: `T: Clone + Sync + Send + Debug`.

**When to use:** Any symbolic-alphabet problem; plug-in replacement for Binary or Range chromosomes.

```rust
// Source: mirrors src/chromosomes/range.rs + src/chromosomes/binary.rs
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct ListChromosome<T: Clone + Sync + Send + Debug> {
    pub dna: Vec<List<T>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<List<T>>,
}

impl<T: Clone + Sync + Send + Debug + 'static> ChromosomeT for ListChromosome<T> {
    type Gene = List<T>;
    // ... standard impl matching Range<T> chromosome
}
```

Note: `ChromosomeT` requires `Clone + Default + Send + Sync + 'static`. `Default` for `ListChromosome<T>` needs `T: Default`. The same resolution as the gene applies.

### Pattern 3: Initializer (list_initializer.rs)

**What:** Two functions that sample from a `&[List<T>]` alleles slice, set `gene.id` to the allele index of the chosen value, and return `Vec<List<T>>`.

**Key difference from generic_initializer:** The generic initializer takes `Option<&[U::Gene]>` and is parameterized on the chromosome type `U`. The list initializer is parameterized directly on `T`, taking `Option<&[T]>` (the raw allele values, not gene templates). This avoids the user having to construct `List<T>` template genes for the allele list — they just pass the raw values.

```rust
// Source: mirrors src/initializers/range_initializer.rs + binary_initializer.rs
pub fn list_random_initialization<T>(
    genes_per_chromosome: usize,
    alleles: Option<&[T]>,
    _needs_unique_ids: Option<bool>,
) -> Vec<List<T>>
where
    T: Clone + Sync + Send + Debug + Default,
{
    let alleles = alleles.expect("Alleles must be provided for list_random_initialization");
    let mut rng = crate::rng::make_rng();
    let mut dna = Vec::with_capacity(genes_per_chromosome);

    for _ in 0..genes_per_chromosome {
        let index = rng.random_range(0..alleles.len());
        // id = allele index; value derived from alleles[index]
        let gene = List::new(index as i32, alleles.to_vec(), alleles[index].clone())
            .expect("list_random_initialization: allele index always valid");
        dna.push(gene);
    }
    dna
}
```

**IMPORTANT:** The initializer signature takes `Option<&[T]>` (raw allele values), not `Option<&[List<T>]>` (gene templates). This is a deliberate deviation from the `generic_initializer` signature. However, the `Ga` configuration uses `with_initialization_fn` which accepts `Fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>`. Since `Gene = List<T>`, the Ga API expects `Option<&[List<T>]>`.

**Resolution:** Provide BOTH forms:
1. A low-level `list_random_initialization<T>(usize, Option<&[T]>, Option<bool>) -> Vec<List<T>>` for direct use
2. A wrapper or the user wraps the call in a closure when passing to `with_initialization_fn`

Looking at the `range_random_initialization` pattern: it takes `Option<&[RangeGenotype<T>]>` (gene templates with ranges embedded). Similarly `list_random_initialization` should take `Option<&[List<T>]>` as gene templates so it fits the `Ga` API directly. The initializer extracts the allele set from the templates and picks randomly.

**Final signature (matching Ga API):**
```rust
pub fn list_random_initialization<T>(
    genes_per_chromosome: usize,
    alleles: Option<&[List<T>]>,  // template genes carrying the allele set
    _needs_unique_ids: Option<bool>,
) -> Vec<List<T>>
```
Each template gene's `alleles` field is used as the source. If all template genes share the same allele set, pick one template gene's alleles and sample from it.

### Pattern 4: List Value Mutation (list_value.rs)

**What:** Pick one random gene, pick a different allele index (exclude current `gene.id`), update both `id` and `value`.

**Key:** Uses `gene.id` as current index, picks a new index ≠ current index. No `PartialEq` on T needed.

```rust
// Source: mirrors src/operations/mutation/value.rs pattern
pub fn list_value_mutation<T>(individual: &mut ListChromosome<T>)
where
    T: Clone + Sync + Send + Debug + Default + 'static,
{
    let len = individual.dna().len();
    if len == 0 { return; }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut dna = individual.dna().to_vec();
    let gene = &mut dna[idx];

    if gene.alleles.len() < 2 { return; } // can't pick a different allele

    let current_index = gene.id as usize;
    // Pick a different index
    let mut new_index = rng.random_range(0..gene.alleles.len());
    while new_index == current_index {
        new_index = rng.random_range(0..gene.alleles.len());
    }

    gene.id = new_index as i32;
    gene.value = gene.alleles[new_index].clone();

    individual.set_dna(Cow::Owned(dna));
}
```

**Hooking into Mutation::ListValue:** `ListChromosome<T>` must implement `ValueMutable` with `value_mutate()` calling `list_value_mutation(self)`. The `Mutation::ListValue` arm in `mutation.rs` calls `individual.value_mutate()` — but wait, `ListValue` is a new variant, not reusing `Value`. The factory dispatch will need a specific arm:

```rust
Mutation::ListValue => individual.list_value_mutate(),
```

This requires a new method on `ValueMutable` trait: `list_value_mutate()` with a default implementation that warns and falls back to swap (matching the pattern of `value_mutate`, `bit_flip_mutate`, etc.).

**Alternative:** Add `ListValue` as a separate arm in `MutationOperator::mutate` that downcasts similar to `try_polynomial`. This avoids adding a new method to `ValueMutable`. Given that `try_polynomial` already demonstrates the downcast pattern for type-specific mutation, this is viable but adds complexity. The `ValueMutable` extension method is cleaner.

### Pattern 5: Mutation Enum Addition

Add to `src/operations.rs`:
```rust
pub enum Mutation {
    // ... existing variants ...
    /// List value mutation — replaces a single gene's value with a different allele.
    /// Requires a ListChromosome<T> implementing ValueMutable::list_value_mutate().
    ListValue,
}
```

Add to `src/operations/mutation.rs`:
- `pub mod list_value;`
- In `MutationOperator::mutate` match: `Mutation::ListValue => individual.list_value_mutate(),`
- In `factory_non_value` match: error arm for `Mutation::ListValue`
- New `ValueMutable` method: `fn list_value_mutate(&mut self)` with default warn + swap fallback

### Anti-Patterns to Avoid

- **Storing `Arc<Vec<T>>` for alleles:** Decided against in CONTEXT.md. Use `Vec<T>` per gene — allele sets are small.
- **Requiring `PartialEq` on T:** The `gene.id = allele_index` design removes all need for equality comparison.
- **Making `List::new` infallible:** Unlike `Range::new`, `List::new` MUST return `Result` to enforce `id < alleles.len()` invariant.
- **Separate `value` argument to `List::new` that could diverge from `alleles[id]`:** Derive value from id; ignore or validate the passed value.
- **Using `Default` impl as a real gene:** `Default` for `List<T>` is a sentinel used by framework internals; real genes always come from `List::new` or the initializer.
- **Adding `PartialEq` derive to `ListChromosome` without `PartialEq` bound on T:** Only derive `PartialEq` if `T: PartialEq`, or omit it. `Range<T>` chromosome derives `PartialEq` with an implicit bound. Check whether `PartialEq` is needed — `Binary` and `Range` chromosomes derive it for tests. For `List`, it is safe to derive `PartialEq` only for the chromosome (not required for operators). Given `T` has no `PartialEq` bound, either skip `PartialEq` derive or add `where T: PartialEq` to the derived impl.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thread-safe RNG | Custom RNG wrapper | `crate::rng::make_rng()` | Already used in every operator; seeded consistently |
| "Pick different index" loop | Complex exclusion algorithm | Simple rejection loop (≤2 retries on avg for allele sets ≥ 2) | Allele sets are small; rejection sampling is O(1) amortized |
| Serde bounds for generic T | Manual serialize/deserialize | `serde(bound(...))` attribute | Already established in `Range<T>` gene and chromosome |
| FitnessFnWrapper | Custom fn pointer storage | `crate::fitness::FitnessFnWrapper<List<T>>` | Handles `Send + Sync + 'static` boxing |

**Key insight:** All operator logic already works through `ChromosomeT` generics. Adding `ListChromosome<T>` requires zero operator code changes — the entire integration cost is a `ValueMutable` impl + one new factory dispatch arm.

## Common Pitfalls

### Pitfall 1: `set_id` inconsistency
**What goes wrong:** `GeneT::set_id` is called by `generic_random_initialization` (via `needs_unique_ids`) to assign position-based IDs. If `set_id` updates `id` but not `value`, the gene becomes inconsistent (id claims index 3, value is still from index 0).
**Why it happens:** The base `GeneT::set_id` contract only sets `id`. For `List<T>`, `id` has dual purpose as an allele index.
**How to avoid:** `List<T>::set_id` must also update `self.value = self.alleles[id as usize].clone()` — but only if `id` is a valid allele index. Add a bounds check: if `id < 0 || id as usize >= alleles.len()`, keep existing value and log a warning.
**Warning signs:** Tests that check `gene.value` after `set_id` will catch this immediately.

### Pitfall 2: Initializer signature mismatch with `Ga::with_initialization_fn`
**What goes wrong:** `with_initialization_fn` expects `Fn(usize, Option<&[List<T>]>, Option<bool>) -> Vec<List<T>>`. A helper taking `Option<&[T]>` cannot be passed directly.
**Why it happens:** The generic `Ga` API uses the gene type as the element type for the alleles argument.
**How to avoid:** Design `list_random_initialization` to accept `Option<&[List<T>]>` (gene templates) matching the `range_random_initialization` pattern exactly.
**Warning signs:** Type mismatch compiler error when user tries to pass `list_random_initialization` to `.with_initialization_fn`.

### Pitfall 3: `Default` bounds propagation
**What goes wrong:** `ChromosomeT` requires `Default`. `Default for ListChromosome<T>` requires `T: Default`. This pushes `T: Default` onto all operator paths even though operators only need `Clone + Sync + Send + Debug`.
**Why it happens:** Rust derives propagate bounds mechanically. `Range<T>` has the same issue.
**How to avoid:** Accept the bound (it matches `Range<T>`) and document that `T` must implement `Default` for use in `ListChromosome`. This is consistent with the existing library posture.
**Warning signs:** Compiler error "the trait `Default` is not implemented for T" when a user tries to use `ListChromosome` with a non-Default T.

### Pitfall 4: `PartialEq` on `List<T>` and `ListChromosome<T>`
**What goes wrong:** Deriving `PartialEq` on `List<T>` (which contains `alleles: Vec<T>`) requires `T: PartialEq`. This is a hidden bound that makes test assertions hard if T is a custom type.
**Why it happens:** `Range<T>` derives `PartialEq` with implicit `T: PartialEq` bound.
**How to avoid:** Either derive `PartialEq` (acceptable if `T: PartialEq` is added to the `#[derive]`), or add `impl<T: PartialEq> PartialEq for List<T>` manually. For test purposes, derive is simplest. Users with non-PartialEq T cannot use `==` on genes, which is unlikely to be a real problem.

### Pitfall 5: `list_value_mutation` with alleles.len() == 1
**What goes wrong:** If a gene's allele set has only one element, there is no "different" allele to pick. The rejection loop spins forever.
**Why it happens:** The rejection loop `while new_index == current_index` never terminates.
**How to avoid:** Guard: if `gene.alleles.len() < 2`, return early (no-op for that gene). Document this behavior.

### Pitfall 6: Serde skip/default on fitness_fn
**What goes wrong:** `FitnessFnWrapper` is not serializable. Forgetting `#[serde(skip, default)]` on the `fitness_fn` field causes serde compilation failure.
**Why it happens:** Easy to miss when copy-pasting from Binary chromosome (which also has this annotation).
**How to avoid:** Copy the exact annotation from `src/chromosomes/binary.rs` line 28–29.

## Code Examples

Verified patterns from existing source files:

### Gene struct (from src/genotypes/range.rs)
```rust
// Source: src/genotypes/range.rs
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct Range<T> {
    pub id: i32,
    pub ranges: Vec<(T, T)>,
    pub value: T,
}
```

### Chromosome serde skip pattern (from src/chromosomes/binary.rs)
```rust
// Source: src/chromosomes/binary.rs lines 28-29
#[cfg_attr(feature = "serde", serde(skip, default))]
pub fitness_fn: FitnessFnWrapper<BinaryGenotype>,
```

### Mutation dispatch arm (from src/operations/mutation.rs)
```rust
// Source: src/operations/mutation.rs — existing Value arm pattern
Mutation::Value => individual.value_mutate(),
// New ListValue arm will follow same pattern:
Mutation::ListValue => individual.list_value_mutate(),
```

### factory_non_value error arm pattern (from src/operations/mutation.rs)
```rust
// Source: src/operations/mutation.rs — existing BitFlip error arm pattern
Mutation::BitFlip => Err(GaError::MutationError(
    "Mutation::BitFlip requires a Binary chromosome type. \
         Use Swap, Inversion, or Scramble instead."
        .to_string(),
)),
// ListValue will follow same pattern
```

### RNG usage pattern (from src/operations/mutation/value.rs)
```rust
// Source: src/operations/mutation/value.rs
let mut rng = crate::rng::make_rng();
let idx = rng.random_range(0..len);
```

### Initializer return via Cow (from src/operations/mutation/value.rs)
```rust
// Source: src/operations/mutation/value.rs lines 51-54
individual.set_dna(Cow::Owned(dna));
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate allele set per chromosome | Per-gene allele set (Range pattern) | Decided in discuss-phase | Simpler ownership, no shared state |
| `PartialEq` for "different value" check | Index comparison via `gene.id` | Decided in discuss-phase | Removes T: PartialEq requirement, cleaner bounds |

**No deprecated items in this phase.** All new code follows current project patterns.

## Open Questions

1. **`List::new` value argument**
   - What we know: signature is `List::new(id, alleles, value) -> Result<Self, GaError>` per CONTEXT.md
   - What's unclear: if the constructor derives `value` from `alleles[id]`, the `value` argument is redundant. Accepting it makes the API more explicit but risks inconsistency if `value != alleles[id]`.
   - Recommendation: Accept `value` for API symmetry with `Range::new`, but ignore it and always derive `value = alleles[id as usize].clone()` internally. Document this behavior.

2. **`Display` for `List<T>`**
   - What we know: `Range<T>` implements `Display` with `T: Display` bound; `Binary` implements it too.
   - What's unclear: whether `List<T>` needs `Display` for `ListChromosome::phenotype()`.
   - Recommendation: Implement `Display for List<T>` where `T: Debug` (use `{:?}` for value like `Range<T>` uses for phenotype), matching the minimal bound approach.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `cargo test` |
| Config file | none (standard Cargo test runner) |
| Quick run command | `cargo test list` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LIST-01 | `List::new` constructs gene with valid id | unit | `cargo test list_gene` | ❌ Wave 0 |
| LIST-01 | `List::new` rejects id out of bounds | unit | `cargo test list_gene_validation` | ❌ Wave 0 |
| LIST-01 | `List::new` rejects empty alleles | unit | `cargo test list_gene_empty_alleles` | ❌ Wave 0 |
| LIST-01 | `GeneT` impl: `id()` returns allele index | unit | `cargo test list_gene_id` | ❌ Wave 0 |
| LIST-01 | `set_id` updates value consistently | unit | `cargo test list_gene_set_id` | ❌ Wave 0 |
| LIST-02 | `ListChromosome::new()` default-constructs | unit | `cargo test list_chromosome_new` | ❌ Wave 0 |
| LIST-02 | `ChromosomeT` impl: dna/fitness/age | unit | `cargo test list_chromosome_trait` | ❌ Wave 0 |
| LIST-03 | Swap mutation works on `ListChromosome` | unit | `cargo test list_chromosome_swap` | ❌ Wave 0 |
| LIST-03 | `Mutation::ListValue` changes exactly one gene | unit | `cargo test list_value_mutation` | ❌ Wave 0 |
| LIST-03 | `ListValue` picks different allele index | unit | `cargo test list_value_mutation_different` | ❌ Wave 0 |
| LIST-03 | `ListValue` no-ops on single-allele gene | unit | `cargo test list_value_single_allele` | ❌ Wave 0 |
| LIST-04 | `list_random_initialization` returns correct length | unit | `cargo test list_initializer` | ❌ Wave 0 |
| LIST-04 | `list_random_initialization` sets gene.id = allele index | unit | `cargo test list_initializer_id` | ❌ Wave 0 |
| LIST-04 | `list_random_initialization_without_repetitions` no duplicates | unit | `cargo test list_initializer_no_rep` | ❌ Wave 0 |
| LIST-04 | Serde roundtrip for `ListChromosome` | unit | `cargo test --features serde list_serde` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test list`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/chromosomes/test_list.rs` — covers LIST-02, LIST-03 (operator integration)
- [ ] Unit test module in `src/genotypes/list.rs` (`#[cfg(test)] mod tests`) — covers LIST-01
- [ ] Unit test module in `src/initializers/list_initializer.rs` — covers LIST-04
- [ ] Unit test module in `src/operations/mutation/list_value.rs` — covers LIST-03 (ListValue)
- [ ] `tests/test_chromosomes.rs` — add `mod test_list;` entry

## Sources

### Primary (HIGH confidence)
- `src/genotypes/range.rs` — gene struct pattern, bounds, GeneT impl, serde cfg_attr
- `src/chromosomes/binary.rs` — ChromosomeT impl structure, serde skip on fitness_fn
- `src/chromosomes/range.rs` — generic chromosome with T bounds
- `src/initializers/range_initializer.rs` — initializer signature with gene-template alleles
- `src/initializers/binary_initializer.rs` — simplest initializer pattern
- `src/initializers.rs` — pub mod + pub use re-export pattern
- `src/operations/mutation.rs` — Mutation enum dispatch, ValueMutable trait, factory functions
- `src/operations/mutation/value.rs` — value replacement mutation pattern
- `src/operations/mutation/insertion.rs` — ChromosomeT-generic mutation (no ValueMutable)
- `src/operations.rs` — Mutation enum definition with all existing variants
- `src/traits/chromosome.rs` — ChromosomeT contract and bounds
- `src/traits/gene.rs` — GeneT contract and bounds
- `src/error.rs` — GaError variants

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions — authoritative design decisions for this phase

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns verified in source
- Architecture: HIGH — all patterns directly read from existing source files
- Pitfalls: HIGH — identified from reading actual trait/impl contracts and dispatch code
- Validation Architecture: HIGH — test structure mirrors existing test files

**Research date:** 2026-03-21
**Valid until:** Stable — no external dependencies; only changes if codebase refactors occur
