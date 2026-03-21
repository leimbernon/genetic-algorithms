# Phase 7: List Genotype - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `List<T>` gene type and `ListChromosome<T>` for problems over finite symbolic alphabets
(e.g., colors, directions, symbols, categories). All existing selection, crossover, mutation,
and survivor operators must work without modification. A dedicated initializer and one new
mutation operator (list value replacement) are in scope.

</domain>

<decisions>
## Implementation Decisions

### Allele set placement
- Each `List<T>` gene carries its own allele set: `List { id: i32, alleles: Vec<T>, value: T }`
- Follows the `Range<T>` pattern (ranges stored per-gene), not per-chromosome
- `alleles` is cloned on gene clone — no `Arc` needed (allele sets are small in practice)
- `List::new(id, alleles, value)` returns `Result<Self, GaError>` — validates that `value` is a member of `alleles` at construction time (always, not just debug builds)

### Value-replacement mutation
- Add a new `Mutation` enum variant: `ListValue` (or `ListRandom`)
- Implementation: pick one random gene, replace its value with a **different** allele from its set
- "Different" is determined by allele index (gene.id = index of current allele) — no `PartialEq` on T required
- Exactly one gene changes per call — consistent with `swap`, `inversion`, `scramble` conventions
- The GA's `mutation_probability` controls how often the operator fires (same as all other mutation ops)

### Type bounds on T
- `T: Clone + Sync + Send + Debug` — minimal, matching existing operator requirements
- No `PartialEq` or `Hash` required on `T`
- `gene.id` stores the **index** of the current value in `alleles` — used by mutation to pick a different index without value comparison

### Initializer
- Add a dedicated `list_random_initialization` function in `src/initializers/` (new file: `list_initializer.rs`)
- Two variants, mirroring the generic initializer:
  - `list_random_initialization` — with repetition (same allele may appear multiple times)
  - `list_random_initialization_without_repetitions` — without repetition (permutation problems)
- Both set `gene.id` = allele index of the chosen value
- Signature: `list_random_initialization(genes_per_chromosome: usize, alleles: Option<&[T]>, _: Option<bool>) -> Vec<List<T>>`

### Claude's Discretion
- Module structure: `src/genotypes/list.rs` and `src/chromosomes/list.rs` (following existing pattern)
- Serde support: `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` with appropriate bounds
- Error variant reuse: use existing `GaError::ValidationError` for invalid initial value
- Public re-exports: follow existing `pub use` patterns in `src/genotypes/mod.rs`, `src/chromosomes/mod.rs`, `src/initializers/mod.rs`
- Diversity computation: works automatically (fitness std-dev) — no List-specific changes needed

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements are fully captured in decisions above.

### Requirements
- `.planning/REQUIREMENTS.md` §List Genotype — LIST-01 through LIST-04

### Existing patterns to follow
- `src/genotypes/range.rs` — per-gene data pattern (`id`, domain data, `value`); bounds; `GeneT` impl
- `src/chromosomes/binary.rs` — `ChromosomeT` impl structure; serde cfg_attr pattern
- `src/initializers/generic_initializer.rs` — initializer signature and allele-index logic
- `src/operations/mutation/swap.rs` — single-gene mutation convention (one op per call)
- `src/operations/mutation/value.rs` — value-replacement mutation pattern (for Range)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/initializers/generic_initializer.rs` — `generic_random_initialization` handles allele sampling; `list_random_initialization` will follow the same logic but return `Vec<List<T>>` with `gene.id = allele_index`
- `GaError::ValidationError(String)` — existing error variant for `List::new` validation
- `src/operations/mutation/` — all of `swap`, `inversion`, `scramble`, `insertion` operate on `ChromosomeT` generically and will work on `ListChromosome` for free

### Established Patterns
- Gene struct: `pub struct X { pub id: i32, pub <domain_data>, pub value: Y }` + `GeneT` impl + `Default` + `Clone` + serde cfg_attr
- Chromosome struct: `pub struct X<...> { pub dna: Vec<Gene>, pub fitness: f64, pub age: usize, pub fitness_fn: FitnessFnWrapper<Gene> }` + `ChromosomeT` impl + `Default` + `new()` + serde cfg_attr with `#[serde(skip, default)]` on fitness_fn
- Mutation operator: function in own file under `src/operations/mutation/`, registered as `Mutation` enum variant, dispatched via `mutation::factory_with_params()`
- Initializer: function in `src/initializers/<type>_initializer.rs`, signature `fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>`, re-exported from `src/initializers/mod.rs`

### Integration Points
- `src/genotypes/mod.rs` — add `pub mod list; pub use list::List;`
- `src/chromosomes/mod.rs` — add `pub mod list; pub use list::ListChromosome;`
- `src/initializers/mod.rs` — add `pub mod list_initializer; pub use list_initializer::*;`
- `src/operations/mutation/mod.rs` and `mutation.rs` — add `ListValue` variant and dispatch
- `src/lib.rs` — update module-level docs to mention List genotype

</code_context>

<specifics>
## Specific Ideas

- `gene.id` doubles as the allele index — this is the key design decision that avoids `PartialEq` on `T` while still enabling "always pick a different allele" in `list_value_mutation`
- `List::new(id, alleles, value) -> Result<Self, GaError>` — constructors return `Result` unlike Range/Binary which are infallible; this is intentional for correctness

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-list-genotype*
*Context gathered: 2026-03-21*
