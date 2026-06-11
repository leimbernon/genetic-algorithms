# Phase 32: Crossover & Differential Mutation - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Add two new operators to the library:
- `Crossover::EdgeRecombination` — builds adjacency lists from both parents and constructs offspring that preserve adjacency relationships found in either parent; designed for permutation chromosomes (TSP, scheduling)
- `Mutation::Differential` — DE-style mutation available in the standard `Ga<U>` engine; computes a mutant vector as `x_r1 + F * (x_r2 - x_r3)` using three distinct random population members, clamped to gene ranges; for `Range<T>` chromosomes only

Both operators follow the existing enum + factory pattern. No new traits. All existing operators remain unaffected.

</domain>

<decisions>
## Implementation Decisions

### Differential Mutation: Population Access

- **D-01:** The GA engine (`src/engines/ga.rs`) detects `Mutation::Differential` **before** the standard per-offspring mutation loop and calls a population-aware function directly, bypassing `factory_with_params`. No trait change to `MutationOperator::mutate`. Non-Differential operators are completely unaffected.
- **D-02:** Differential mutation is **`Range<T>` chromosomes only** — requires `ValueMutable` (same constraint as Gaussian/Creep). If used with Binary or List chromosomes, return a clear `GaError::MutationError`.
- **D-03:** When the population is too small to draw 3 distinct members other than the target (i.e. `population_size < 4`), return `GaError::MutationError` with a clear message. Users who configure `Mutation::Differential` must ensure `population_size >= 4`.

### Differential Mutation: F Scale Factor

- **D-04:** Add `differential_f: Option<f64>` to `MutationConfiguration` with a default of `0.5` when `None`. This matches the existing `polynomial_eta` / `non_uniform_b` pattern — one `Option<f64>` field per operator-specific parameter.
- **D-05:** Add a corresponding `with_differential_f(f: f64)` builder method to the `ConfigurationT` builder trait, following the same pattern as `with_mutation_sigma`.

### Edge Recombination Crossover: Degenerate Handling

- **D-06:** When all of the current gene's neighbors are already visited (adjacency list exhausted mid-construction), fall back to **randomly picking any remaining unvisited gene**. This is the canonical ERX algorithm (Whitley 1989). The fallback is rare and keeps the offspring valid.
- **D-07:** Minimum chromosome length is **`len >= 2`** — error with `GaError::CrossoverError` for shorter chromosomes. Consistent with PMX's floor; ERX needs at least 2 genes to have any adjacency to preserve.

### Edge Recombination Crossover: Chromosome Constraints

- **D-08:** **Validate gene uniqueness at factory time**: if either parent contains duplicate gene IDs, return `GaError::CrossoverError` with a clear message. ERX's adjacency semantics are undefined for non-permutation chromosomes, and an early error is more helpful than silent wrong output. This is an O(n) check using a HashSet on gene IDs.

### Claude's Discretion

- Exact adjacency-list data structure (HashMap<gene_id, HashSet<gene_id>> or Vec-based adjacency)
- Tie-breaking when multiple neighbors have equal smallest remaining-neighbor count (any consistent policy)
- Whether ERX produces 1 child or 2 children per call (producing 2 is the norm; use parent2's start gene for the second)
- Log target names: follow existing patterns (`crossover_events`, `mutation_events`)
- Internal helper function names and loop structure within `clearing.rs` / `deterministic_crowding.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Operator Infrastructure

- `src/operations.rs` — `Crossover` enum (add `EdgeRecombination` variant) and `Mutation` enum (add `Differential` variant)
- `src/operations/crossover.rs` — `CrossoverOperator for Crossover` impl + factory; add `EdgeRecombination` match arm
- `src/operations/mutation.rs` — `MutationOperator for Mutation` impl + `factory_with_params`; add `Differential` match arm (even if engine branches early, the match arm must exist for trait completeness)
- `src/traits/operators.rs` — `CrossoverOperator` and `MutationOperator` trait definitions (interface contracts)

### Engine Integration (Differential Mutation)

- `src/engines/ga.rs` — the GA engine's mutation dispatch loop; add the `Mutation::Differential` branch here before the standard `factory_with_params` call
- `src/operations/mutation/gaussian.rs` — reference implementation for `Range<T>`-constrained mutation with value clamping; mirror the ValueMutable + range bounds pattern

### Configuration

- `src/configuration.rs` — `MutationConfiguration` struct (add `differential_f: Option<f64>` field with default `0.5`)
- `src/traits/configuration.rs` — builder trait methods (add `with_differential_f(f: f64)`)

### Reference Implementations (Crossover)

- `src/operations/crossover/order.rs` — canonical permutation crossover pattern: length validation, gene-order logic, logging, returns `Vec<U>`
- `src/operations/crossover/pmx.rs` — pattern for permutation crossover that uses gene ID mapping (mirrors what ERX needs for adjacency by ID)

### Chromosome Traits

- `src/traits/chromosome.rs` — `ChromosomeT`: `dna() -> &[Self::Gene]`, `dna_mut()`, `set_dna()`
- `src/traits/gene.rs` — `GeneT`: `id() -> i32` (used for ERX adjacency lists and Differential target identification)

### Requirements

- `.planning/REQUIREMENTS.md` §CRS-01 (Edge Recombination) and §MUT-04 (Differential) — exact acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crate::rng::make_rng()` — RNG factory used by all operators; use this, not `rand::thread_rng()`
- `src/operations/crossover/order.rs::order()` — permutation offspring construction pattern (gene-set tracking, `Cow<[Gene]>` for zero-copy DNA); reuse for ERX's visited-gene tracking
- `src/operations/mutation/gaussian.rs::gaussian_mutation()` — `Range<T>` value clamping to `(lo, hi)` bounds; the same clamping logic applies to Differential mutation's mutant vector
- `GeneT::id() -> i32` — available on every gene; use for ERX adjacency map keys and Differential's population sampling (identify target index to exclude)

### Established Patterns

- Enum variants use `Copy + Clone + Debug + PartialEq` + `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` — replicate for new variants
- Crossover factory in `src/operations/crossover.rs`: match arm delegates to a free function in `src/operations/crossover/<name>.rs`
- Mutation factory in `src/operations/mutation.rs`: same delegation pattern
- `MutationConfiguration` fields: `Option<f64>` with `None` default; engine passes `config.differential_f` (unwrap_or 0.5) to the operator function
- Length/type validation errors: `GaError::CrossoverError(format!(...))` or `GaError::MutationError(format!(...))`

### Integration Points

- `src/operations.rs` — new enum variants land here (ERX in `Crossover`, Differential in `Mutation`)
- `src/operations/crossover.rs` and `src/operations/mutation.rs` — match arm dispatch
- `src/engines/ga.rs` — Differential mutation population branch (D-01); reads `self.configuration.mutation_configuration` for `differential_f`
- `src/configuration.rs` — `differential_f` field on `MutationConfiguration`
- `src/traits/configuration.rs` — `with_differential_f` builder method
- `tests/observe/test_serde.rs` — add `Crossover::EdgeRecombination` and `Mutation::Differential` to serde round-trip test arrays (lesson from Phase 31 CR-01)

</code_context>

<specifics>
## Specific Ideas

- Differential mutation mutant vector formula: `mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])`, clamped to each gene's `(lo, hi)` range. r1, r2, r3 are distinct indices ≠ target index.
- ERX adjacency map: for each gene g in a parent, record g's left and right neighbors in the chromosome order. The union of both parents' adjacency lists forms the neighbor set for each gene. Use gene IDs (not positions) as map keys.
- ERX tie-breaking (D-discretion): when multiple neighbors have equal fewest remaining neighbors, pick arbitrarily (e.g., first in the set iteration order) — this is acceptable and consistent with the original algorithm.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 32-Crossover & Differential Mutation*
*Context gathered: 2026-05-04*
