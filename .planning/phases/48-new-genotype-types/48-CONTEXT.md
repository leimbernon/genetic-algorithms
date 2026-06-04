# Phase 48: New Genotype Types - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 48 delivers three new semantically correct chromosome types that replace ad-hoc hacks: `UniqueChromosome<T>` for permutation problems (TSP, scheduling), `MultiRangeChromosome<T>` for real-valued problems with per-gene independent bounds and mutation rates, and `MultiUniqueChromosome<T>` for multi-group permutation problems. It also migrates the `job_scheduling` example from the `RangeChromosome<i32>` unique-id hack to `UniqueChromosome<i32>`. Accompanied by a new `OperatorCompat` trait that enforces build-time operator validation.

</domain>

<decisions>
## Implementation Decisions

### UniqueChromosome Gene Type

- **D-01:** New `UniqueGenotype<T>` gene type with `{ id: i32, value: T }` — a lightweight wrapper that implements `GeneT`. No alphabet per gene; the chromosome holds the alphabet once.
- **D-02:** `UniqueChromosome<T>` struct: `{ dna: Vec<UniqueGenotype<T>>, alphabet: Arc<[T]>, fitness: f64, age: usize, fitness_fn: FitnessFnWrapper<UniqueGenotype<T>> }`. The `alphabet` field allows post-mutation permutation validation.
- **D-03:** New `src/initializers/unique_initializer.rs` with `unique_random_initialization<T>(alphabet: &[T]) -> Vec<UniqueGenotype<T>>` using Fisher-Yates shuffle. Follows the exact pattern of `range_initializer.rs`, `list_initializer.rs`.

### Invalid Operator Guard (`OperatorCompat` trait)

- **D-04:** New opt-in trait `OperatorCompat` (in `src/traits/` or adjacent to chromosome types) with two default-returning methods:
  - `fn valid_crossovers() -> Option<&'static [Crossover]>` — default `None` (no restriction)
  - `fn valid_mutations() -> Option<&'static [Mutation]>` — default `None` (no restriction)
- **D-05:** `Ga::build()` validator checks `OperatorCompat` for the chromosome type `U`. If the selected crossover/mutation is not in the valid set (and valid set is `Some`), `build()` returns `GaError::ConfigurationError`. Fails fast before any run.
- **D-06:** `UniqueChromosome<T>` implements `OperatorCompat`:
  - Valid crossovers: `[Crossover::Pmx, Crossover::Order, Crossover::EdgeRecombination]`
  - Valid mutations: `[Mutation::Insertion, Mutation::Swap, Mutation::Inversion]`
- **D-07:** `MultiUniqueChromosome<T>` also implements `OperatorCompat` with the same valid sets (same permutation semantics).

### MultiRangeChromosome Gene Type

- **D-08:** New `MultiRangeGenotype<T>` gene with `{ id: i32, lo: T, hi: T, value: T, mutation_rate: f64 }` — flat struct, no Arc overhead, explicit per-gene bounds and mutation rate. Implements `GeneT`.
- **D-09:** Per-gene bounds provided by the user as `Vec<(T, T)>` at build time. A `with_bounds(vec![(0.0, 1.0), (-5.0, 5.0)])` config API (or equivalent initialization fn) maps each tuple to a gene's `(lo, hi)` fields.
- **D-10:** Per-gene mutation rate `p_i` is a first-class field (`mutation_rate: f64`) per GEN-03 — NOT deferred. Gaussian mutation uses `gene.mutation_rate` instead of the global sigma for each gene independently.
- **D-11:** New `src/initializers/multi_range_initializer.rs` following the same pattern as other initializers.

### MultiUniqueChromosome Group Boundaries

- **D-12:** Groups represented as `Vec<Arc<[T]>>` on the chromosome. User provides `Vec<Vec<T>>` at build time — one inner `Vec<T>` per group alphabet. Group sizes and boundaries are derived from alphabet lengths automatically. DNA is the concatenation of all group permutations.
- **D-13:** Reuse `UniqueGenotype<T>` for the gene type (same lightweight gene, no `group_id` field). Group membership is implicit from position in DNA and derived from `group_ranges()`.
- **D-14:** `MultiUniqueChromosome<T>` exposes `fn group_ranges(&self) -> Vec<(usize, usize)>` — returns `[(0, g0_len-1), (g0_len, g0_len+g1_len-1), ...]`. PMX/OX crossover operators call this to apply within each group independently. This requires a `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` variant (or a wrapper) that internally slices and recombines by group.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §GEN — GEN-01 through GEN-04 (authoritative scope for this phase)

### Prior Phase Context (breaking changes that Phase 48 builds on)
- `.planning/phases/47-architecture-audit-chromosomet-split/47-CONTEXT.md` — D-02 (`LinearChromosome` supertrait), D-06 (`needs_unique_ids` removal), D-07 (`ChromosomeLength` enum)

### Existing Chromosome Implementations (structural pattern to follow)
- `src/types/chromosomes/range.rs` — canonical chromosome struct pattern: `dna`, `fitness`, `age`, `fitness_fn`, serde attrs, `ChromosomeT` + `LinearChromosome` impls
- `src/types/chromosomes/list.rs` — `ListChromosome<T>` with non-Copy gene type pattern
- `src/types/genotypes/range.rs` — `Range<T>` gene: GeneT impl, Arc fields, serde pattern
- `src/types/genotypes/list.rs` — `List<T>` gene: allele-set pattern (for contrast with `UniqueGenotype<T>`)

### Initializers (follow these patterns)
- `src/initializers/list_initializer.rs` — `list_random_initialization_without_repetitions` (Fisher-Yates permutation init, directly analogous to `unique_random_initialization`)
- `src/initializers/range_initializer.rs` — per-gene random sampling pattern

### Permutation Crossover Operators (already implemented, must work with UniqueChromosome)
- `src/operations/crossover/pmx.rs` — PMX crossover, generic over `U: LinearChromosome`
- `src/operations/crossover/order.rs` — OX crossover, generic over `U: LinearChromosome`
- `src/operations/crossover/edge_recombination.rs` — ERX crossover

### Example Migration Target
- `examples/job_scheduling.rs` — migrate from `RangeChromosome<i32>` hack to `UniqueChromosome<i32>`; comment at line ~90 explicitly calls out this migration

### Trait Architecture
- `src/traits/chromosome.rs` — `ChromosomeT` (new chromosomes implement this)
- `src/traits/linear_chromosome.rs` — `LinearChromosome` (new chromosomes implement this as supertrait)

### Module Re-export Points
- `src/types/chromosomes/mod.rs` — re-export new chromosome types here
- `src/types/genotypes/mod.rs` — re-export new gene types here
- `src/initializers.rs` — re-export new initializer functions here
- `src/lib.rs` — verify new public types are visible from crate root

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/operations/crossover/pmx.rs`: PMX crossover already generic over `U: LinearChromosome`; works with `UniqueChromosome<T>` with zero changes
- `src/operations/crossover/order.rs`: OX crossover already generic; same
- `src/operations/crossover/edge_recombination.rs`: ERX already available
- `src/initializers/list_initializer.rs` `list_random_initialization_without_repetitions`: Fisher-Yates permutation init is the direct model for `unique_random_initialization`
- `src/fitness/mod.rs` `FitnessFnWrapper`: existing wrapper for fitness closures — all three new chromosome types reuse it
- `src/rng.rs` `make_rng()`: RNG factory — all initializers use this

### Established Patterns
- Chromosome struct layout: `{ dna: Vec<Gene>, fitness: f64, age: usize, fitness_fn: FitnessFnWrapper<Gene> }` — all new types follow this exactly
- `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(bound(...)))]` — mandatory on all chromosome and gene structs; `fitness_fn` field uses `serde(skip, default)`
- `#[cfg(not(target_arch = "wasm32"))]` gates on any `Instant::now()` or `par_iter()` calls — required for WASM compatibility
- `ChromosomeT` + `LinearChromosome` dual impl: every new chromosome implements both (Phase 47 split)
- Operator trait bounds: `U: LinearChromosome` throughout operator files — no change needed

### Integration Points
- `src/validators/` — the `OperatorCompat` check plugs into the existing validator chain called by `Ga::build()`
- `src/types/chromosomes/mod.rs` — add `pub mod unique`, `pub mod multi_range`, `pub mod multi_unique` and re-exports
- `src/types/genotypes/mod.rs` — add `pub mod unique`, `pub mod multi_range` and re-exports
- `src/initializers.rs` — re-export `unique_random_initialization`, `multi_range_random_initialization`
- `examples/job_scheduling.rs` — migration from `Range<i32>` to `UniqueChromosome<i32>` in `main()`

</code_context>

<specifics>
## Specific Ideas

- `unique_random_initialization` uses Fisher-Yates shuffle — direct analog to `list_random_initialization_without_repetitions` in `src/initializers/list_initializer.rs`
- `job_scheduling.rs` already has a comment at the initialization block: "Phase 48 will migrate to `UniqueChromosome<i32>` for cleaner permutation support" — follow through on this
- `MultiRangeGenotype<T>` bounds in `with_bounds(vec![(lo0, hi0), (lo1, hi1), ...])` — length must match chromosome length; validate in `Ga::build()`
- `group_ranges()` on `MultiUniqueChromosome` is derived from `groups: Vec<Arc<[T]>>` lengths — no separate storage needed
- `OperatorCompat` trait placement: `src/traits/operator_compat.rs` (new file), re-exported via `src/traits.rs`

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 48-new-genotype-types*
*Context gathered: 2026-05-21*
