# Pitfalls Research: v3.0.0

**Project:** genetic_algorithms v3.0.0
**Mode:** Ecosystem / Integration Pitfalls
**Confidence:** HIGH for per-feature risks (grounded in direct trait inspection); MEDIUM for cross-feature interactions (no prior precedent in this codebase)

---

## Summary

`ChromosomeT`'s `dna() -> &[Self::Gene]` flat-slice contract and `fitness() -> f64` scalar contract are the two load-bearing assumptions that every v3.0.0 feature violates. The correct response is separate subtraits (`TreeChromosomeT`, `MultiFitnessChromosomeT`), not modifications to `ChromosomeT` itself.

`CrossoverOperator::crossover(&U, &U)` is a two-parent hard contract. UNDX, SPX, and PCX require N parents. A new `MultiParentCrossoverOperator` trait is needed alongside — not replacing — the existing trait.

Serde checkpoint stack overflow is a concrete risk for tree chromosomes. `serde_stacker` exists for this and must be opted in explicitly.

The architecture audit and the feature phases all touch `ChromosomeT`. If they run in parallel, the merge conflicts will be unresolvable. The audit must stabilize the `ChromosomeT` skeleton before any feature phase begins modifying that trait.

GP tree bloat is a known catastrophic failure mode. Without `max_depth` enforcement at crossover and survivor selection, populations balloon in memory with each generation.

---

## Per-Feature Risks

### Tree Chromosome (#223)

**Risk:** `dna() -> &[Gene]` cannot be meaningfully implemented for a tree. The trap is a fake compliance: linearize the tree on every `dna()` call, store in a `Vec`. This silently allows every existing positional crossover operator to "work" on tree chromosomes, producing structurally corrupt offspring without any error.

**Prevention:** `TreeChromosomeT` is a separate supertrait that does NOT require `dna()` / `set_dna()`. Operators for GP implement against `TreeChromosomeT`. The existing `CrossoverOperator` is not extended to handle trees — new tree-specific operators are written against the new trait.

**Risk:** Recursive `Box<Node>` trees + serde `Serialize`/`Deserialize` derived = stack overflow on any realistically evolved tree (50+ levels deep).

**Prevention:** CI test that serializes a tree of depth ≥ 64. Use `serde_stacker::deserialize` at the checkpoint call site for tree populations.

**Risk:** Without `max_depth` enforcement, trees exhibit bloat — populations balloon without fitness gain. This is a well-documented GP failure mode (arxiv 1806.02112).

**Prevention:** `max_depth: usize` is a required method on `TreeChromosomeT`. Both subtree crossover and subtree mutation must check depth post-operation and return `GaError::TreeDepthExceeded` if exceeded.

**Warning signs:**
- `dna()` implementation contains `self.tree.traverse()` and stores the result in a `Vec`
- `set_dna(Cow<[Gene]>)` attempts to reconstruct a tree from a flat slice
- Population average node count increases monotonically across generations
- `cargo test --features serde` passes with shallow trees (depth < 10) but panics on evolved populations at generation 50+

---

### Variable-Length Chromosomes (#224)

**Risk:** All existing crossover operators silently truncate to `min(len_a, len_b)` when given different-length parents, then copy the tail of the longer parent wholesale. This is semantically wrong for positional encodings.

**Prevention:** Fixed-length crossover operators return `GaError::IncompatibleChromosomeLength` when parent lengths differ. A new `Crossover::VariableLength(AlignmentStrategy)` variant handles variable-length parents explicitly.

**Risk:** Without parsimony pressure, populations drift toward maximum chromosome length (variable-length analogue of GP bloat).

**Prevention:** Optional `length_penalty: f64` in `SurvivorOperator` configuration. Benchmark must include a plot of population length distribution across generations — monotonic growth is a red flag.

**Risk:** Extension operators (`MassGenesis`, `MassDeduplication`) assume fixed gene count for initialization. When diversity collapses and extension triggers, regrown individuals all have the same length, destroying length diversity.

**Prevention:** `ExtensionOperator` receives a `LengthDistribution` parameter (e.g., sampled from the current population's length histogram) for variable-length populations.

**Warning signs:**
- Extension trigger followed by a generation where 90%+ of chromosomes have identical length
- Average chromosome length monotonically increasing across generations without fitness improvement

---

### Lexicase Selection (#220)

**Risk:** `ChromosomeT::fitness() -> f64` appears in: trait definition, `SurvivorOperator` (sorts by fitness), `calculate_fitness()`, stopping criteria (convergence on fitness), `GenerationStats` (aggregates fitness values), `fitness_distance()`, and all multi-objective engines. Changing it to `Vec<f64>` breaks all of these simultaneously.

**Prevention:** Do not modify `ChromosomeT::fitness() -> f64`. Introduce `MultiFitnessChromosomeT: ChromosomeT` with `case_fitnesses() -> &[f64]` and `set_case_fitnesses(Vec<f64>)`. `LexicaseSelection` has a `where U: MultiFitnessChromosomeT` bound.

**Risk:** Bridging by averaging case fitnesses into the scalar `fitness()` field is a silent correctness failure — it produces tournament-equivalent behavior while claiming to be lexicase.

**Prevention:** A behavioral diversity test: a population evolved with lexicase must show measurably higher specialist count than the same population evolved with tournament selection under the same conditions.

**Risk:** Case shuffling under rayon can produce non-reproducible case orderings.

**Prevention:** Per-selection-event seeded RNG derived from the global seed + selection event index.

**Warning signs:**
- `LexicaseSelection::select` calls `chromosome.fitness()` internally (getting a single `f64`) — the implementation is wrong
- Lexicase selection produces same behavioral diversity as tournament selection in benchmarks

---

### Multi-Parent Crossover — UNDX, SPX, PCX (#221)

**Risk:** `CrossoverOperator::crossover(&U, &U)` is a hard two-parent contract. Forcing UNDX and SPX into a two-parent API produces a degenerate variant that loses their exploration properties.

**Prevention:** `MultiParentCrossoverOperator` is a new trait: `fn crossover_multi<U: ChromosomeT>(&self, parents: &[&U]) -> Result<Vec<U>, GaError>`. This is separate from `CrossoverOperator` — all existing two-parent operators remain untouched.

**Risk:** `SelectionOperator::select` returns `Vec<(usize, usize)>` — pairs. Multi-parent crossover needs N-tuples.

**Prevention:** Extend `SelectionOperator` with an optional `select_n` method: `fn select_n<U>(&self, chromosomes: &[U], n_parents: usize, n_groups: usize) -> Vec<Vec<usize>>`. Provide a default implementation that calls `select()` repeatedly.

**Risk:** UNDX and SPX are mathematically undefined for binary, list, or symbolic gene types.

**Prevention:** `RealValued` marker trait bound on `MultiParentCrossoverOperator` implementations for UNDX/SPX/PCX. Attempting to use these with a `BinaryChromosome` is a compile error.

---

### Self-Adaptive Mutation (#222)

**Risk:** Adding strategy parameters to the chromosome struct increases its size. Every crossover call clones the chromosome. v2.2.1 spent significant effort eliminating clone overhead (`Cow<[Gene]>`, `dna_mut()`, zero-copy operators). Strategy parameters in the chromosome directly undo this work.

**Prevention:** Benchmark gate in CI: `SelfAdaptiveMutation` throughput vs `GaussianMutation` with equivalent sigma should be within 2x overhead, not 10x. Use `f32` for strategy parameters. Add benchmark as a required CI gate before the phase is marked complete.

**Risk:** `MutationOperator::mutate(individual, step, sigma)` takes step and sigma as call-site parameters. Strategy parameters inside the chromosome require a fundamentally different calling convention.

**Prevention:** `SelfAdaptive` marker trait (or associated type `StrategyParams`) on `ChromosomeT`. `SelfAdaptiveMutationOperator` has a `where U: ChromosomeT + SelfAdaptive` bound. It does not implement `MutationOperator` — it is a separate trait.

**Risk:** `Instant::now()` or `SystemTime::now()` inside mutation logic for time-based adaptation schedules will fail to compile on `wasm32-unknown-unknown`.

**Prevention:** `cargo check --target wasm32-unknown-unknown` is required before any PR for this phase.

**Warning signs:**
- Benchmark shows crossover throughput drop >30% after adding strategy params to the chromosome
- After 100 generations with self-adaptive mutation, all chromosomes have identical sigma values (strategy params cloned but not independently mutated)
- `cargo check --target wasm32-unknown-unknown` fails in the mutation module

---

## Cross-Feature Integration Risks

**Tree + Lexicase:** GP practitioners will want lexicase for program synthesis. `TreeChromosomeT` must explicitly implement `MultiFitnessChromosomeT`; the per-case fitness vector is the common currency.

**Variable-Length + Self-Adaptive:** Per-gene sigma vectors would themselves need to be variable-length. Document that v3.0.0 uses scalar sigma for self-adaptive mutation only. Per-gene sigma vectors are future scope.

**Multi-Parent + Variable-Length:** UNDX/SPX centroid computation requires all parent vectors to have the same dimension. Add a `FixedLength` precondition check in `MultiParentCrossoverOperator` impls that returns `GaError::IncompatibleChromosomeLength` for variable-length parents.

**All new types + Observer:** `GaObserver` hooks receive `&U where U: ChromosomeT`. For tree chromosomes, observers cannot access tree depth or node count. Prevention: extend `GenerationStats` with `Option<TreeStats>` and `Option<CaseFitnessStats>` fields.

**Architecture Audit + Feature Phases (coordination):** The audit and every major feature both modify `ChromosomeT`. If they run concurrently, merge conflicts will be unresolvable. The audit must stabilize the `ChromosomeT` skeleton before feature phases begin modifying that trait.

---

## Published Library Risks

**Undocumented breaking changes:** Every user with `impl ChromosomeT for MyType` will get compile errors. A `MIGRATION.md` published before v3.0.0 is mandatory. `cargo-semver-checks` in CI against the v2.4.0 baseline catches accidental breaking changes.

**Checkpoint files from v2.x:** Users who checkpoint long-running runs will find v2.x checkpoint files fail to deserialize in v3.0.0. Prevention: `#[serde(default)]` for all new fields; `#[serde(rename = "old_name")]` for renamed fields; a forward-compatibility test.

**Examples:** The 10 runnable examples must compile and run in CI on every PR to the milestone branch.

---

## Architecture Audit Anti-Patterns

**Unbounded scope:** Audits expand continuously. Prevention: fixed list of items before the first commit; anything discovered but not on the list is filed as a new issue.

**Stranded refactor:** Three branches touch the same trait and the last to merge requires manual conflict resolution. Prevention: audit stabilizes the trait skeleton first; feature phases build against the stable skeleton.

**Shims that never get removed:** "Temporary" backwards-compatible shims survive every major version. Prevention: every shim has `#[deprecated(since = "3.0.0")]` and a filed GitHub issue for removal.

**Audit changes not covered by tests:** Builder method renames, struct restructuring break examples but not `cargo test`. Prevention: CI compiles and runs all 10 examples as part of every audit PR.

**Breaking everything at once:** A mega-PR is impossible to roll back selectively. Prevention: each audit concern is a separate PR. Each PR passes CI independently.

---

## Roadmap Implications (Suggested Phase Ordering)

1. **Architecture Audit first** — must stabilize `ChromosomeT` skeleton (subtrait split, `Reporter<U>` removal, `Strategy` trait interface) before any feature phase begins modifying that trait.
2. **New genotypes (`Unique<T>`, `MultiRange<T>`, `MultiUnique<T>`) and strategies (HillClimb, Permutate)** — additive, lower-risk; can run after audit. Ship early for a stable foundation.
3. **Lexicase Selection** — `MultiFitnessChromosomeT` is needed by both Lexicase and Tree Chromosome; design it in the Lexicase phase so it exists before Tree phase starts.
4. **Multi-Parent Crossover and Self-Adaptive Mutation** — neither depends on breaking chromosome changes; can run in parallel with Tree/Variable-Length if branches are kept separate.
5. **Tree Chromosome and Variable-Length Chromosomes last** — most architecturally disruptive. Having stable audit, stable new genotypes, and `MultiFitnessChromosomeT` in place reduces mid-phase reversal risk.

---

## Open Questions

- Does `serde_stacker` work for `wasm32-unknown-unknown`? The stack growth mechanism may rely on `std` threading primitives. Needs explicit verification before committing to it in the Tree Chromosome checkpoint design.
- What is the intended interaction between `Strategy` trait unification and the 4 alt-metaheuristic engines (DE, Scatter, Cellular, ALPS)? If `Strategy` wraps GA, HillClimb, and Permutate, does it also wrap the alt engines? This scope question should be resolved in the audit phase design document before implementation.
- The `Cow<[Gene]>` zero-copy optimization is incompatible with variable-length offspring (which always allocate). Is there a use case for `Cow` in variable-length crossover, or should the variable-length crossover path document that it always allocates?
