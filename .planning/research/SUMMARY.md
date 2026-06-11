# Research Summary: v3.0.0

**Project:** genetic_algorithms v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification
**Synthesized:** 2026-05-19
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

v3.0.0 is fundamentally an **architectural redesign project** with feature additions layered on top — not a dependency expansion. Zero new external crates are required for the majority of features; `rand_distr` (already present) handles all numeric sampling, and tree chromosome representation is best served by an internal `Box<N>` recursive enum. The single structural truth driving the entire milestone is that `ChromosomeT`'s flat-slice contract (`dna() -> &[Self::Gene]`) and scalar fitness contract (`fitness() -> f64`) are load-bearing assumptions that multiple v3.0.0 features violate. Every design decision flows from how cleanly those two contracts are split.

The correct fix for both is **supertrait separation, not contract modification**. `ChromosomeT` becomes a minimal core (fitness, age) from which `LinearChromosome` and `TreeChromosome` extend independently. Multi-case fitness for lexicase lives on an opt-in `MultiCaseFitness: ChromosomeT` supertrait. This keeps all 30+ existing operator implementations untouched while enabling the new representations. This audit must land as Phase 1 before any feature branch modifies the trait — merging order is non-negotiable.

The feature set divides cleanly into low-risk additive work (UniqueT, MultiRangeT, Strategy trait, HillClimb, Permutate) that ships early and high-risk structural work (Variable-Length, Tree Chromosome) that ships last. The middle tier (Lexicase, Multi-Parent Crossover, Self-Adaptive Mutation) each require one new parallel trait but do not disrupt existing operator contracts. GP tree bloat, serde stack overflow on deep trees, and parallel-branch merge conflicts on `ChromosomeT` are the three concrete failure modes most likely to cause rework if not addressed by design.

---

## Stack Additions

**New external dependencies: effectively zero.**

| Decision | Verdict | Reason |
|----------|---------|--------|
| `indextree` / `ego-tree` / `slab` for GP trees | **Rejected** | Arena clone is O(arena), not O(subtree); subtree crossover requires index-remapping across arenas. Internal `Box<N>` recursive enum is simpler and clones exactly what is needed. |
| `nalgebra` for UNDX/SPX/PCX math | **Rejected** | Only centroid + normal sampling needed; `rand_distr::Normal` (already in `Cargo.toml`) + raw f64 arithmetic is sufficient. |
| `num-traits` for generic float crossover | **Conditional skip** | Only needed if multi-parent crossover is made generic over `T: Float`. All practical cases are `Range<f64>`; use concrete f64 arithmetic and avoid the dep. Revisit only if a user request surfaces. |
| `serde_stacker` | **Likely needed** | Recursive `Box<Node>` trees + serde derive = stack overflow at depth >= 50 in evolved populations. Gate behind existing `serde` feature flag. Verify wasm32 compatibility before committing. |

**Build: no new feature flags required** unless `TreeChromosome` pulls `serde_stacker` (add to existing `serde` flag). No standalone GP flag.

---

## Feature Table Stakes

What "done" means for each v3.0.0 feature:

### Architecture Audit (Phase 1 prerequisite)
- `ChromosomeT` reduced to: `fitness()`, `set_fitness()`, `age()`, `set_age()`, `calculate_fitness()`, `fitness_distance()`
- `LinearChromosome: ChromosomeT` owns: `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`, `set_gene()`
- All existing operator bounds mechanically updated from `U: ChromosomeT` to `U: LinearChromosome`
- `Reporter<U>` trait removed entirely (soft-deprecated since v2.2.0)
- `GaConfiguration` fields privatized (`pub(crate)` + read accessors)
- `needs_unique_ids` / `alleles_can_be_repeated` removed from `LimitConfiguration`
- `genes_per_chromosome: usize` replaced by `ChromosomeLength` enum (`Fixed(usize)` | `Variable { min, max }`)
- All 10 runnable examples compile and run in CI after the audit PR

### UniqueT Genotype (#174)
- Initialization guarantees a valid permutation (no duplicates, all elements present)
- Alphabet stored in chromosome (enables MultiUniqueT reuse)
- `is_valid()` check as debug assertion
- PMX and OX documented as the safe crossover operators; non-permutation operators must return `GaError`
- Completes the `job_scheduling` example migration from `RangeChromosome<i32>` hack

### MultiRangeT Genotype (#175)
- Per-gene `(lo_i, hi_i)` bounds stored as `Arc<[(T,T)]>` (same zero-copy pattern as `Range<T>`)
- Per-gene mutation rate `p_i: Vec<f64>` in chromosome metadata
- Initialization samples each gene from its own range
- Gaussian mutation respects per-gene bounds (clamp or reflect)

### MultiUniqueT Genotype (#176)
- `GroupSpec` metadata (alphabet + start index + length) per permutation group
- PMX/OX applied within each group boundary, never across boundaries
- Per-group initialization from its own alphabet
- Crossover preserves all group boundaries

### Unified Strategy Trait + HillClimb + Permutate (#172, #173, #177)
- `Strategy` trait: `run() -> Result<StrategyResult<U>, GaError>` and `best() -> Option<&U>`
- `Ga<U>`, `HillClimbEngine<U>`, `PermutateEngine<U>` all implement `Strategy`
- HillClimb Stochastic: accept any uphill neighbor, stop on no improvement or iteration limit
- HillClimb SteepestAscent: user-provided `neighbor_fn: Fn(&U) -> Vec<U>`, evaluate all, accept best
- Permutate: exhaustive enumeration with a `u64` saturating safety gate on permutation count; emits warning when count exceeds configurable limit
- `GaObserver` hooks present in HillClimb and Permutate (consistent observability)
- `Strategy` is a trait, not an enum — third-party strategy implementations remain possible

### Lexicase Selection (#220)
- New `MultiCaseFitness: ChromosomeT` supertrait with `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)`
- `LexicaseSelection` has bound `U: ChromosomeT + MultiCaseFitness`
- Scalar `fitness()` set to aggregate (mean or sum of case errors) for survivor/stopping compatibility
- Random case shuffle per selection event; reproducible under seeded RNG
- Epsilon-lexicase variant included (filters within epsilon of best per case, required for continuous scores)
- Behavioral diversity test in CI: lexicase must produce measurably higher specialist count than tournament under equivalent conditions

### Multi-Parent Crossover: UNDX, SPX, PCX (#221)
- New `MultiParentCrossoverOperator` trait: `crossover_n(parents: &[&U]) -> Result<Vec<U>, GaError>`
- New `MultiParentSelectionOperator` trait: `select_groups(chromosomes, group_size, n_groups) -> Vec<Vec<usize>>`
- `SelectionOperator` gains optional `select_n` default method (calls `select()` repeatedly) — existing signature unchanged
- `RealValued` marker trait bound on UNDX/SPX/PCX — binary/permutation chromosomes are a compile error
- Configurable `num_parents` (default 3); offspring always returns 2 per call
- Gene bounds enforcement post-crossover (clamp or reflect)
- `ga.rs` loop detects multi-parent crossover variant by enum match and takes the `select_groups` + `crossover_n` path

### Self-Adaptive Mutation (#222)
- New `SelfAdaptive: ChromosomeT` supertrait: `strategy_params() -> &[f64]`, `set_strategy_params(Vec<f64>)`, `adapt_strategy_params(tau, tau_prime)`
- New `Mutation::SelfAdaptiveGaussian` enum variant; bound `U: ChromosomeT + SelfAdaptive`
- Log-normal sigma update: `sigma_i' = sigma_i * exp(tau' * N(0,1) + tau * N_i(0,1))`; tau from Back 1996 rule of thumb
- Sigma lower bound enforced (`sigma_min = 1e-5` default — configurable)
- Sigmas initialized to 0.3; recombined via intermediate (averaged) crossover
- Benchmark gate: throughput vs plain `GaussianMutation` must be within 2x, not 10x
- `cargo check --target wasm32-unknown-unknown` required before PR

### Variable-Length Chromosomes (#224)
- `ChromosomeLength::Variable { min, max }` in `LimitConfiguration` (Phase 1 delivers this enum)
- New `Mutation::Insertion` (add gene at random position) and `Mutation::Deletion` (remove gene at random position)
- All 9 existing crossover operators return `GaError::IncompatibleChromosomeLength` for unequal-length parents (not silent truncation)
- New `Crossover::VariableLength(AlignmentStrategy)` variant handles variable-length parents explicitly
- `ExtensionOperator` receives `LengthDistribution` sampled from current population — no fixed-length regrowth
- Optional `length_penalty: f64` (parsimony pressure) in survivor config
- Fitness sharing Hamming distance and `DeterministicCrowding` survivor audited for equal-length assumption

### Tree Chromosome / GP Engine (#223)
- `TreeChromosome: ChromosomeT` supertrait: `root()`, `root_mut()`, `set_tree()`, `set_fitness_fn(Fn(&TreeNode<G>) -> f64)`, `depth()`, `node_count()`
- User-defined node enum implements `GpNode` marker trait (arity, children, depth, size)
- Separate `GpGa<U: TreeChromosome>` engine — NOT `Ga<U>` — with ramped half-and-half initialization
- Primitive set with function/terminal registration; built-in arithmetic and boolean primitives
- Subtree crossover + subtree/point/hoist mutation
- `max_depth` and `max_node_count` required config fields; both enforced post-crossover and post-mutation; return `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded`
- `Display`/`Debug` renders tree as expression string
- Ephemeral random constants (ERC) as leaf nodes
- Serde serialization gated behind `serde` feature; uses `serde_stacker` to prevent stack overflow
- CI test: serialize/deserialize a tree of depth >= 64

---

## Architecture Decisions

These decisions are resolved by research and should not be re-litigated during planning:

### 1. ChromosomeT splits into three levels
```
ChromosomeT (core: fitness, age)
+-- LinearChromosome: ChromosomeT  -- all current operators use this bound
+-- TreeChromosome: ChromosomeT   -- GpGa uses this bound
```
All existing chromosome types (`Binary`, `Range<T>`, `ListChromosome<T>`) implement `LinearChromosome`. Migration is mechanical: `U: ChromosomeT` -> `U: LinearChromosome` in operator bounds. This is the highest-touch change (~30 files) and must be a single PR that CI validates end-to-end.

### 2. Parallel trait pattern for new operator classes
Do not modify existing `CrossoverOperator` or `SelectionOperator` signatures. Add:
- `MultiParentCrossoverOperator` alongside `CrossoverOperator`
- `MultiParentSelectionOperator` alongside `SelectionOperator`
- `SelfAdaptiveMutationOperator` alongside `MutationOperator`

This preserves all 9 existing crossover implementations and every existing operator bound.

### 3. Opt-in supertrait pattern for opt-in fitness contracts
- `MultiCaseFitness: ChromosomeT` for lexicase (not an associated type on `ChromosomeT`)
- `SelfAdaptive: ChromosomeT` for self-adaptive mutation

Neither modifies the existing 30+ operator implementations.

### 4. No fitness function inside chromosomes
`set_fitness_fn` and `calculate_fitness` are removed from `ChromosomeT`/`LinearChromosome`. The engine calls `chromosome.set_fitness(fitness_fn(chromosome.dna()))` directly. This eliminates N `Arc<FitnessFn>` clones for a population of size N and unblocks serde checkpoint (closures are not serializable, fitness values are). Impact on users who call `chromosome.calculate_fitness()` directly must be validated in the audit phase.

### 5. Two separate engines, not one unified engine
`Ga<U: LinearChromosome>` and `GpGa<U: TreeChromosome>` are separate engine types. The GP execution loop differs fundamentally (ramped half-and-half init, subtree crossover, bloat control, depth limits) — forcing it into `Ga<U>` adds conditional logic throughout the hot path.

### 6. `ChromosomeLength` enum replaces `genes_per_chromosome: usize`
```rust
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}
```
Fixed-length users set `ChromosomeLength::Fixed(n)` — zero behavior change.

### 7. Internal `Box<N>` recursive enum for tree nodes
```rust
pub trait GpNode: Clone + Send + Sync + 'static {
    fn arity(&self) -> usize;
    fn children(&self) -> &[Box<Self>];
    fn children_mut(&mut self) -> &mut Vec<Box<Self>>;
    fn depth(&self) -> usize;
    fn size(&self) -> usize;
}
```
Tree clone during rayon offspring generation clones exactly the subtree, not a whole arena.

### 8. Six API simplifications included in Phase 1 (architecture audit)
| Item | Change |
|------|--------|
| `Reporter<U>` | Removed entirely |
| `GaConfiguration` fields | `pub(crate)` + read accessors |
| `needs_unique_ids` / `alleles_can_be_repeated` | Removed from `LimitConfiguration` |
| Fitness function | Moved out of chromosomes into engine call sites |
| `LocalSearchOperator` | `Arc<dyn ...>` -> `Option<LocalSearch>` enum |
| `StoppingCriteria` struct | Flattened to builder methods |

---

## Top Watch-Outs

### 1. Phase 1 must finish before any feature branch touches ChromosomeT
**Risk:** Concurrent branches modifying `ChromosomeT` produce unresolvable merge conflicts.
**Prevention:** Architecture audit PR is merged and CI is green before any feature branch opens a PR that modifies trait definitions. Feature branches may open early for local work but must rebase onto the post-audit milestone branch before requesting review.

### 2. Tree chromosomes must not fake compliance with LinearChromosome
**Risk:** Implementing `dna()` as `self.tree.traverse().collect::<Vec<_>>()` silently allows positional crossover operators to run on tree chromosomes, producing structurally corrupt offspring with no error.
**Prevention:** `TreeChromosome: ChromosomeT` is a distinct trait, not a subtrait of `LinearChromosome`. There is no path for a `TreeChromosome` implementor to accidentally satisfy `LinearChromosome`. GP operators implement against `TreeChromosome`; linear operators compile only for `LinearChromosome`.

### 3. Serde stack overflow on deep evolved trees
**Risk:** `serde` derive on recursive `Box<Node>` panics at checkpoint time on trees with depth >= 50 (typical after 30+ GP generations).
**Prevention:** CI test that serializes a tree of depth >= 64 in the `serde` test suite. Use `serde_stacker` at the tree chromosome serde impl call site. Verify `serde_stacker` compiles for `wasm32-unknown-unknown` before committing to it.

### 4. Lexicase averaging into scalar fitness silently degenerates to tournament behavior
**Risk:** Setting `fitness() = mean(case_scores)` and then running lexicase produces selection behavior identical to scalar tournament — diversity is not preserved, but tests pass.
**Prevention:** Behavioral diversity CI test: a population evolved under `LexicaseSelection` must show measurably higher specialist count (individuals excelling on distinct case subsets) than under `TournamentSelection` with matched effort. If this test cannot be written, the implementation is wrong.

### 5. Self-adaptive mutation chromosomes revert to uniform sigma after crossover
**Risk:** Crossover operators clone parent chromosomes and produce offspring. If the sigma vectors are cloned identically (not averaged), all offspring share the same sigma as one parent — self-adaptation does not function.
**Prevention:** Crossover of `SelfAdaptive` chromosomes must invoke intermediate recombination of `strategy_params` (element-wise average of parent sigmas). Add a test verifying sigma divergence: after N generations with two parents initialized at sigma=0.1 and sigma=0.9, offspring sigma distribution should span the range, not cluster at one endpoint.

### 6. Variable-length crossover silently truncating to shorter parent
**Risk:** Existing crossover operators applied to unequal-length chromosomes silently take `min(len_a, len_b)` genes and discard the tail of the longer parent. No error is raised.
**Prevention:** Fixed-length operators return `GaError::IncompatibleChromosomeLength` when parent lengths differ. Only `Crossover::VariableLength(AlignmentStrategy)` handles unequal parents. Audit all 9 existing crossover operators for equal-length assumptions as part of the Variable-Length phase.

### 7. GP bloat without enforced limits
**Risk:** Subtree crossover without depth and size limits produces population-wide bloat within 20-30 generations. Memory usage grows exponentially; fitness plateau is reached before any useful program evolves.
**Prevention:** `max_depth` and `max_node_count` are required fields on `GpConfiguration` (not optional). Both enforced in subtree crossover and mutation post-operation. Violating operators return `GaError` — they do not silently accept the oversized tree. Population average node count tracked in `GenerationStats::tree_stats: Option<TreeStats>`.

---

## Recommended Phase Ordering

This ordering respects the hard dependency graph identified across all four research files.

### Phase 1: Architecture Audit + ChromosomeT Split
**Must be first. Blocks all other phases.**

Deliver: `ChromosomeT` reduced to minimal core; `LinearChromosome` supertrait owns all flat-slice methods; `ChromosomeLength` enum; `Reporter<U>` removed; 6 API simplifications applied; all 10 examples compile and run in CI.

Covers: Architecture audit, API simplification, `Reporter<U>` removal, `ChromosomeLength` enum (prerequisite for #224).

Research flag: Standard patterns well-documented. Risk is execution volume (~30 files), not design uncertainty. No research phase needed during planning.

---

### Phase 2: New Genotypes + Unified Strategy
**Additive, no cross-feature dependencies. Runs immediately after Phase 1.**

Deliver: `Unique<T>` chromosome, `MultiRange<T>` chromosome, `Strategy` trait, `HillClimbEngine` (Stochastic + SteepestAscent), `PermutateEngine`. Updates `job_scheduling` example to use `UniqueChromosome`.

Covers issues: #174, #175, #177, #172, #173.

Research flag: Well-precedented. No research phase needed.

---

### Phase 3: MultiUniqueT Genotype
**Depends on Phase 2 (UniqueT infrastructure and GroupSpec design).**

Deliver: `MultiUnique<T>` chromosome with `GroupSpec` metadata; group-boundary-aware PMX/OX; per-group initialization.

Covers issue: #176.

Research flag: Straightforward extension of UniqueT. No research phase needed.

---

### Phase 4: Lexicase Selection
**Depends on Phase 1 only. Trait design is resolved. Must precede Phase 7 (TreeChromosome will reuse MultiCaseFitness).**

Deliver: `MultiCaseFitness: ChromosomeT` trait; `LexicaseSelection` operator; epsilon-lexicase variant; behavioral diversity CI test; `CaseFitnessFn<G>` type alias.

Covers issue: #220.

Research flag: The `MultiCaseFitness` trait interface must be locked in requirements before implementation — it will be reused by Tree Chromosome in Phase 7 for program synthesis use cases. Resolving it in Phase 4 prevents rework.

---

### Phase 5: Self-Adaptive Mutation + Multi-Parent Crossover
**Both depend on Phase 1 only. Can be developed as parallel PRs.**

Self-Adaptive delivers: `SelfAdaptive: ChromosomeT` trait; `SelfAdaptiveMutationOperator`; `Mutation::SelfAdaptiveGaussian` variant; `SelfAdaptiveRangeChromosome<T>`; sigma recombination in crossover path; benchmark CI gate.

Multi-Parent Crossover delivers: `MultiParentCrossoverOperator` trait; `MultiParentSelectionOperator` trait; `Crossover::Undx`, `Spx`, `Pcx` variants; `RealValued` marker trait bound; `select_n` default on `SelectionOperator`; gene bounds enforcement.

Covers issues: #221, #222.

Research flag: `cargo check --target wasm32-unknown-unknown` is a required gate for both. Math and trait design fully resolved. No research phase needed.

---

### Phase 6: Variable-Length Chromosomes
**Depends on Phase 1 (`ChromosomeLength` enum). Benefits from Phase 5 patterns but not a hard dependency.**

Deliver: Variable-length initialization using `ChromosomeLength::Variable { min, max }`; `Mutation::Insertion` and `Mutation::Deletion`; `Crossover::VariableLength(AlignmentStrategy)`; audit of all 9 existing crossover operators; `ExtensionOperator` length distribution; optional parsimony pressure in survivor config.

Covers issue: #224.

Research flag: Homologous crossover algorithm choice (SVLC vs simpler truncation+error) may need a targeted design decision during planning. Flag as explicit decision point in requirements.

---

### Phase 7: Tree Chromosome + GpGa Engine
**Must be last. Highest complexity. Reuses MultiCaseFitness from Phase 4.**

Deliver: `GpNode` marker trait; `TreeNode<G>` struct; `GpChromosome<G>: ChromosomeT + TreeChromosome`; `GpGa<U: TreeChromosome>` engine; ramped half-and-half initialization; `PrimitiveSet` with arithmetic/boolean built-ins + ERC support; subtree crossover + subtree/point/hoist mutation; `max_depth` and `max_node_count` enforcement; serde checkpoint with `serde_stacker`; `Display` as expression string; `GpChromosome` implementing `MultiCaseFitness`; `Option<TreeStats>` in `GenerationStats`.

Covers issue: #223.

Research flag: GpObserver sub-trait scope and `Strategy` trait extensibility to GpGa should be decided in requirements before implementation begins.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|-----------|-------|
| Stack (no new deps) | HIGH | Direct crate inspection; rand_distr already present; serde_stacker is the only conditional addition |
| ChromosomeT split design | HIGH | Research directly read trait source; migration path is mechanical |
| Feature trait designs | HIGH | MultiCaseFitness, SelfAdaptive, MultiParentCrossoverOperator all fully specified |
| GP tree implementation | MEDIUM-HIGH | Box<N> pattern is well-understood; GP bloat countermeasures are well-documented; serde_stacker wasm32 compatibility is unverified |
| Variable-length crossover semantics | MEDIUM | Homologous crossover algorithms are complex; simpler truncation+error approach decided but not benchmarked |
| Phase ordering | HIGH | Dependency graph is explicit and unambiguous |
| Self-adaptive sigma recombination | MEDIUM | Intermediate recombination requirement specified; interaction with existing crossover clone paths needs careful implementation |

**Gaps requiring attention during planning:**

1. `serde_stacker` + wasm32: Verify before committing to it in Tree Chromosome checkpoint design. If it does not compile for wasm32, an alternative iterative serde approach is needed.
2. Fitness function removal from chromosomes: Users who call `chromosome.calculate_fitness()` in isolation will break. Scope must be validated in audit phase — a migration path or engine-exposed utility function may be needed.
3. `Strategy` trait scope vs. alt-metaheuristic engines: Does `Strategy` wrap DE, Scatter, Cellular, ALPS? Resolve in Phase 2 requirements to avoid a breaking interface change in a future milestone.
4. GpObserver sub-trait: `GaObserver` hooks carry `population_size: usize` but not tree depth or bloat metrics. Decide whether `GpGa` needs `GpObserver: GaObserver` (like `IslandGaObserver`) before Phase 7 implementation begins.
5. Variable-length + self-adaptive sigma vectors: Document explicitly in Phase 6 that per-gene sigma is out of scope for v3.0.0 (scalar sigma only). Prevents scope creep during implementation.

---

## Sources

Aggregated from research files. Full citations in individual files.

- crates.io / docs.rs: `indextree` 4.8.1, `num-traits` 0.2
- Context7: `/saschagrunert/indextree`, `/websites/rs_genetic_algorithm_0_27_1_genetic_algorithm`
- Lexicase: arXiv 1709.05394, ryanboldi/lexicase, DEAP `selLexicase`
- Multi-parent crossover: ACM 2933986, Water Programming Blog (SPX), Deb et al. 2002
- Self-adaptation: PubMed 11382357, Back 1996
- GP bloat: arxiv 1806.02112, Springer 10.1007/0-387-28111-8_15
- Variable-length crossover: PLOS ONE journal.pone.0209712
- Direct source reading: `src/traits/chromosome.rs`, `src/traits/operators.rs`, `src/ga.rs`, `Cargo.toml`
