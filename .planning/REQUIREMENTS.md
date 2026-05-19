# Requirements — v3.0.0 Advanced Representations, Alternative Strategies & Architecture Simplification

## v1 Requirements

### ARCH — Architecture Audit & API Simplification

- [ ] **ARCH-01**: User can implement `ChromosomeT` for custom types using only the minimal core contract (fitness, age, calculate_fitness) — flat-slice methods (`dna()`, `set_dna()`, `set_fitness_fn()`) move to `LinearChromosome` supertrait, so tree and variable-length types are no longer forced to fake compliance
- [ ] **ARCH-02**: User can implement `LinearChromosome` to gain full compatibility with all existing operators — a mechanical bound change from `U: ChromosomeT` to `U: LinearChromosome` across ~30 operator files; all existing chromosome types (`Binary`, `Range<T>`, `ListChromosome<T>`) implement `LinearChromosome`
- [ ] **ARCH-03**: User can build without `Reporter<U>` — the trait removed entirely in v3.0.0; users who relied on it migrate to `GaObserver<U>` (available since v2.2.0); a `MIGRATION.md` is published
- [ ] **ARCH-04**: User cannot accidentally bypass builder validation — `GaConfiguration` fields are `pub(crate)` with read-only accessors; `needs_unique_ids` and `alleles_can_be_repeated` removed from `LimitConfiguration` (initialization concerns, not engine config)
- [ ] **ARCH-05**: User configures chromosome length via `ChromosomeLength::Fixed(n)` or `ChromosomeLength::Variable { min, max }` — replaces bare `genes_per_chromosome: usize`; existing `Fixed(n)` behavior is identical to current behavior
- [ ] **ARCH-06**: User can configure stopping behavior via flat builder methods (`.with_stagnation_limit(50)`, `.with_convergence_threshold(0.001)`) — `StoppingCriteria` struct flattened; `LocalSearchOperator` changed from `Arc<dyn ...>` to `Option<LocalSearch>` enum (consistent with all other operators)
- [ ] **ARCH-07**: User can run all 10 existing examples without modification after the architecture audit — CI compiles and runs each example with a short generation count on every PR to the milestone branch

### GEN — New Genotype Types

- [ ] **GEN-01**: User can define a permutation chromosome with `UniqueChromosome<T>` where initialization guarantees no duplicate genes and all elements from the provided alphabet are present — invalid operators (single-point, uniform, arithmetic crossover) return `GaError` at runtime; PMX, OX, and ERX are the documented safe crossover operators
- [ ] **GEN-02**: User can migrate the `job_scheduling` example to `UniqueChromosome<i32>` — `RangeChromosome<i32>` with unique-id hack replaced by a semantically correct type
- [ ] **GEN-03**: User can define a real-valued chromosome with `MultiRangeChromosome<T>` where each gene has its own `(lo_i, hi_i)` bounds and mutation rate `p_i` — initialization samples each gene from its own range; Gaussian mutation clamps to per-gene bounds
- [ ] **GEN-04**: User can define a chromosome with `MultiUniqueChromosome<T>` containing multiple independent permutation groups, each with its own alphabet — crossover applies PMX/OX within each group boundary and never across boundaries; initialization generates each group independently

### STR — Alternative Strategy Engines

- [ ] **STR-01**: User can implement the `Strategy<U>` trait for custom search algorithms — common interface (`run()`, `best()`) over `Ga<U>`, `HillClimbEngine<U>`, and `PermutateEngine<U>`; trait objects (`Box<dyn Strategy<U>>`) enable runtime strategy selection
- [ ] **STR-02**: User can run stochastic hill climbing with `HillClimbEngine::Stochastic` — accepts any neighbor with higher fitness; stops on no improvement within a configurable iteration limit; user provides `neighbor_fn: Fn(&U) -> Vec<U>`; `GaObserver` hooks fire per iteration
- [ ] **STR-03**: User can run steepest-ascent hill climbing with `HillClimbEngine::SteepestAscent` — evaluates all neighbors returned by `neighbor_fn`, accepts the single best; `GaObserver` hooks fire per iteration
- [ ] **STR-04**: User can run exhaustive permutation search with `PermutateEngine` — iterates all possible chromosomes up to a configurable safety gate (default 100,000); emits a warning and returns best-found if gate is exceeded; `GaObserver` hooks fire per candidate

### SEL — Advanced Selection

- [ ] **SEL-02**: User can configure `LexicaseSelection` on any chromosome type that implements `MultiCaseFitness: ChromosomeT` — selection shuffles test cases randomly per event and filters to elites case by case; scalar `fitness()` is set to the mean case score for survivor/stopping compatibility
- [ ] **SEL-03**: User can configure epsilon-lexicase selection (ε-lexicase) for continuous-valued case scores — filter keeps all individuals within epsilon of the best on each case; `epsilon` is user-configurable with a sensible default

### CRS — Advanced Crossover Operators

- [ ] **CRS-02**: User can configure `Crossover::Undx { num_parents }` for real-valued chromosomes — offspring centered at centroid, normally distributed along inter-parent direction; minimum 3 parents; gene bounds enforced post-crossover (clamp); binary/permutation chromosomes return `GaError` at build time via `RealValued` marker trait
- [ ] **CRS-03**: User can configure `Crossover::Spx { num_parents }` for real-valued chromosomes — parents define a simplex; offspring sampled uniformly from within the expanded simplex; configurable `epsilon` expansion factor
- [ ] **CRS-04**: User can configure `Crossover::Pcx { num_parents }` for real-valued chromosomes — offspring centered around the primary parent, perturbed in directions of other parents; more exploitative than UNDX/SPX

### MUT — Advanced Mutation

- [ ] **MUT-05**: User can configure `Mutation::SelfAdaptiveGaussian` on chromosomes implementing `SelfAdaptive: ChromosomeT` — per-chromosome sigma vector co-evolves via log-normal update (`σ' = σ × exp(τ' × N(0,1) + τ × N_i(0,1))`); sigma lower bound enforced (`sigma_min` configurable, default 1e-5); sigmas inherited via intermediate recombination during crossover
- [ ] **MUT-06**: User can configure variable-length mutation with `Mutation::Insertion` (add gene at random position within bounds) and `Mutation::Deletion` (remove gene at random position within bounds) — only valid when `ChromosomeLength::Variable` is configured; lengths clamp to `[min, max]`

### CHR — Advanced Chromosome Representations

- [ ] **CHR-01**: User can configure `ChromosomeLength::Variable { min, max }` to enable variable-length evolution — existing crossover operators return `GaError::IncompatibleChromosomeLength` for unequal-length parents; `Crossover::VariableLength(AlignmentStrategy)` handles variable-length parents explicitly; `ExtensionOperator` samples length distribution from current population (not fixed regrowth)
- [ ] **CHR-02**: User can optionally apply parsimony pressure to variable-length populations — configurable `length_penalty: f64` in survivor configuration penalizes longer chromosomes proportionally, preventing unbounded length growth
- [ ] **CHR-03**: User can define a tree-structured chromosome by implementing `TreeChromosome: ChromosomeT` — separate from `LinearChromosome`; no flat-slice methods; tree operators (subtree crossover, subtree/point/hoist mutation) operate against this trait
- [ ] **CHR-04**: User can run a GP optimization via `GpGa<U: TreeChromosome>` with ramped half-and-half initialization, configurable primitive set (arithmetic and boolean built-ins provided), and ephemeral random constants (ERC) as terminals
- [ ] **CHR-05**: User can enforce tree size limits via required `max_depth: usize` and `max_node_count: usize` in `GpConfiguration` — both enforced post-crossover and post-mutation; violations return `GaError::TreeDepthExceeded` or `GaError::TreeSizeExceeded`; population average node count tracked in `GenerationStats`
- [ ] **CHR-06**: User can checkpoint and restore GP runs via the `serde` feature — `TreeChromosome` serialization uses `serde_stacker` to prevent stack overflow on deep evolved trees (depth ≥ 64 validated in CI)
- [ ] **CHR-07**: User can read the result of a GP run as a human-readable expression string — `GpChromosome` implements `Display` rendering the tree as infix/prefix expression

### TRAITS — New Opt-In Trait Contracts

- [ ] **TRAITS-01**: User can make any chromosome type support multi-case fitness evaluation by implementing `MultiCaseFitness: ChromosomeT` with `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)` — enables `LexicaseSelection`; compatible with `TreeChromosome` for GP program synthesis
- [ ] **TRAITS-02**: User can make any real-valued chromosome type support self-adaptive mutation by implementing `SelfAdaptive: ChromosomeT` with `strategy_params() -> &[f64]` and `adapt_strategy_params(tau, tau_prime)` — enables `Mutation::SelfAdaptiveGaussian`

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| ARCH-01 | Phase 47 | Pending |
| ARCH-02 | Phase 47 | Pending |
| ARCH-03 | Phase 47 | Pending |
| ARCH-04 | Phase 47 | Pending |
| ARCH-05 | Phase 47 | Pending |
| ARCH-06 | Phase 47 | Pending |
| ARCH-07 | Phase 47 | Pending |
| GEN-01 | Phase 48 | Pending |
| GEN-02 | Phase 48 | Pending |
| GEN-03 | Phase 48 | Pending |
| GEN-04 | Phase 48 | Pending |
| STR-01 | Phase 49 | Pending |
| STR-02 | Phase 49 | Pending |
| STR-03 | Phase 49 | Pending |
| STR-04 | Phase 49 | Pending |
| SEL-02 | Phase 50 | Pending |
| SEL-03 | Phase 50 | Pending |
| TRAITS-01 | Phase 50 | Pending |
| CRS-02 | Phase 51 | Pending |
| CRS-03 | Phase 51 | Pending |
| CRS-04 | Phase 51 | Pending |
| MUT-05 | Phase 51 | Pending |
| TRAITS-02 | Phase 51 | Pending |
| MUT-06 | Phase 52 | Pending |
| CHR-01 | Phase 52 | Pending |
| CHR-02 | Phase 52 | Pending |
| CHR-03 | Phase 53 | Pending |
| CHR-04 | Phase 53 | Pending |
| CHR-05 | Phase 53 | Pending |
| CHR-06 | Phase 53 | Pending |
| CHR-07 | Phase 53 | Pending |

## Future Requirements

- Per-gene sigma vectors in self-adaptive mutation (scalar sigma only in v3.0.0)
- Grammar-guided / grammatical evolution (out of scope — separate paradigm)
- Strongly-typed GP as the only mode (loosely-typed f64 GP is the default)
- CMA-ES as an operator (scope for a future milestone as its own engine)
- `GpObserver` sub-trait for GP-specific events like `on_bloat_detected` — decision deferred to planning

## Out of Scope

- GUI/interactive visualization — library generates static PNG/SVG charts
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick
- Per-gene observer hooks — too granular, unacceptable overhead in hot loops
- DE-vs-GA head-to-head benchmark — deferred from v2.4.0; not a user-facing feature
- Grammar-guided evolution, grammatical evolution — separate paradigm
- Strongly-typed GP as the only option — too complex for v3.0.0; loosely-typed is the default
- CMA-ES as a mutation operator — it is a standalone algorithm (O(n²) storage); future milestone
