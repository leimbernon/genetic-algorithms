# Architecture Research: v3.0.0

**Project:** genetic_algorithms v3.0.0
**Mode:** Architecture Research — Integration points for breaking-change features
**Confidence:** HIGH (direct source reading of all affected traits and execution files)

---

## Key Findings

1. **`ChromosomeT` must split into two traits.** The flat-slice contract (`dna() -> &[Gene]`, `set_dna(Cow<...>)`) is the load-bearing assumption of every operator in the library. Tree chromosomes make it structurally impossible. The fix: keep `ChromosomeT` as a minimal core (fitness, age) and create `LinearChromosome: ChromosomeT` that owns all flat-slice methods. All existing operators, engines, and chromosomes move to `LinearChromosome`. This is Phase 1 and must land before any other feature work.

2. **Phase ordering has a hard dependency graph.** Phase 1 (ChromosomeT split) is a prerequisite for ALL other phases. Phase 2 (ChromosomeLength/variable-length) is a prerequisite for Phase 7 (multi-parent crossover), because UNDX/SPX/PCX must handle variable-length parents.

3. **Lexicase fitness should be an opt-in supertrait, not an associated type.** The alternative (associated `type Fitness`) would require changing all 30+ operator implementations. The opt-in `MultiCaseFitness: ChromosomeT` approach isolates the change to selection and fitness evaluation.

4. **Multi-parent crossover needs two new parallel traits** (`MultiParentCrossoverOperator`, `MultiParentSelectionOperator`), not modifications to the existing 2-parent traits.

5. **Six API simplifications are justified under the v3.0.0 break:** remove `Reporter<U>`, privatize `GaConfiguration` fields, remove `needs_unique_ids`/`alleles_can_be_repeated` from config, move fitness function OUT of chromosomes, change `LocalSearchOperator` from `Arc<dyn ...>` to `LocalSearch` enum, consolidate `StoppingCriteria` into flat builder methods.

---

## Breaking Change Analysis

### Feature 1: Tree Chromosome (#223)

What breaks:
- `ChromosomeT::dna() -> &[Self::Gene]` — flat slice assumption; trees cannot be a contiguous slice without losing structure
- `ChromosomeT::dna_mut() -> &mut [Self::Gene]` — same
- `ChromosomeT::set_dna(Cow<'a, [Self::Gene]>)` — meaningless for tree structure
- `set_fitness_fn<F>` where `F: Fn(&[Self::Gene]) -> f64` — tree evaluation requires traversal
- `FitnessFn<G> = dyn Fn(&[G]) -> f64` — hardcodes flat-slice evaluation
- `LimitConfiguration::genes_per_chromosome: usize` — meaningless for trees
- `MutationOperator::mutate(&mut U, step, sigma)` — tree mutations have no step/sigma analog

Proposed fix — Representation-agnostic ChromosomeT v3:

```rust
// v3 ChromosomeT — minimal core only
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    type Gene: GeneT;
    fn fitness(&self) -> f64;
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;
    fn age(&self) -> usize;
    fn set_age(&mut self, age: usize) -> &mut Self;
    fn calculate_fitness(&mut self);
    fn fitness_distance(&self, target: &f64) -> f64 { (target - self.fitness()).abs() }
}

// Flat-slice supertrait — all existing operators require this
pub trait LinearChromosome: ChromosomeT {
    fn dna(&self) -> &[Self::Gene];
    fn dna_mut(&mut self) -> &mut [Self::Gene];
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;
    fn set_fitness_fn<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;
    fn set_gene(&mut self, index: usize, gene: Self::Gene) -> &mut Self { ... }
}

// Tree-specific supertrait
pub trait TreeChromosome: ChromosomeT {
    fn root(&self) -> &TreeNode<Self::Gene>;
    fn root_mut(&mut self) -> &mut TreeNode<Self::Gene>;
    fn set_tree(&mut self, root: TreeNode<Self::Gene>) -> &mut Self;
    fn set_fitness_fn<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&TreeNode<Self::Gene>) -> f64 + Send + Sync + 'static;
    fn depth(&self) -> usize;
    fn node_count(&self) -> usize;
}
```

Migration path: All existing chromosome types (`Binary`, `Range<T>`, `ListChromosome<T>`) implement `LinearChromosome`. Operator bounds change from `U: ChromosomeT` to `U: LinearChromosome` (mechanical substitution). `Ga<U>` bound becomes `Ga<U: LinearChromosome>`.

Recommendation: Two separate engines — `Ga<U: LinearChromosome>` and `GpGa<U: TreeChromosome>`. The execution loops diverge enough (GP uses ramped half-and-half init, subtree crossover, bloat control, depth limits) that unifying them adds complexity.

---

### Feature 2: Variable-Length Chromosomes (#224)

What breaks:
- `LimitConfiguration::genes_per_chromosome: usize` — single fixed value used across initialization AND operators
- `initialize_chromosomes` / `initialize_chromosomes_par` — both receive `genes_per_chromosome: usize`
- All crossover operators that assume `parent_1.dna().len() == parent_2.dna().len()`
- Fitness sharing Hamming distance in `niching/` — assumes equal length
- `DeterministicCrowding` survivor — Hamming distance assumes equal length

Proposed fix:

```rust
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}
// Replaces genes_per_chromosome: usize in LimitConfiguration
```

Fixed-length users set `ChromosomeLength::Fixed(n)` — zero change in behavior.

---

### Feature 3: Lexicase Selection (#220)

Two options evaluated:

**Option A — opt-in supertrait `MultiCaseFitness: ChromosomeT` (RECOMMENDED):**
```rust
pub trait MultiCaseFitness: ChromosomeT {
    fn case_fitness(&self) -> &[f64];
    fn set_case_fitness(&mut self, scores: Vec<f64>) -> &mut Self;
    fn case_count(&self) -> usize { self.case_fitness().len() }
}
pub type CaseFitnessFn<G> = dyn Fn(&[G]) -> Vec<f64> + Send + Sync;
```
Scalar `fitness()` returns aggregate (sum of case errors) for stats/observers. Lexicase selection has bound `U: ChromosomeT + MultiCaseFitness`. No changes to existing 30+ operator implementations.

**Option B — associated `type Fitness` on `ChromosomeT`:** More type-theoretically correct but requires changing every operator bound and the entire stats subsystem. Blast radius too large.

---

### Feature 4: Multi-Parent Crossover (#221)

What breaks:
- `CrossoverOperator::crossover(&U, &U) -> Result<Vec<U>, GaError>` — 2-ary
- `SelectionOperator::select(...) -> Vec<(usize, usize)>` — returns pairs
- `ga.rs` generation loop call site is 2-parent

Proposed fix — two new parallel traits (existing traits unchanged):
```rust
pub trait MultiParentCrossoverOperator {
    fn crossover_n<U: ChromosomeT>(
        &self,
        parents: &[&U],
    ) -> Result<Vec<U>, GaError>;
}

pub trait MultiParentSelectionOperator {
    fn select_groups<U: ChromosomeT>(
        &self,
        chromosomes: &[U],
        group_size: usize,
        number_of_groups: usize,
    ) -> Vec<Vec<usize>>;
}
```

New `Crossover` enum variants: `Undx { num_parents: usize }`, `Spx { num_parents: usize }`, `Pcx { num_parents: usize }`. The `ga.rs` loop detects multi-parent variant by enum match before the couple loop and takes the `select_groups` + `crossover_n` path. Default `num_parents = 3`.

---

### Feature 5: Self-Adaptive Mutation (#222)

Proposed fix — opt-in supertrait:
```rust
pub trait SelfAdaptive: ChromosomeT {
    fn strategy_params(&self) -> &[f64];
    fn set_strategy_params(&mut self, params: Vec<f64>) -> &mut Self;
    fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64);
}

pub trait SelfAdaptiveMutationOperator {
    fn mutate_adaptive<U: ChromosomeT + SelfAdaptive>(
        &self,
        individual: &mut U,
    ) -> Result<(), GaError>;
}
```

New `Mutation` variants: `SelfAdaptiveGaussian`, `SelfAdaptiveEsStrategy`. New chromosome type: `SelfAdaptiveRangeChromosome<T>`. Log-normal sigma update: `σ_i' = σ_i * exp(τ' * N(0,1) + τ * N_i(0,1))`. Initialization: sigmas default to 0.3.

---

## Trait Evolution Plan

| Trait | v2 Status | v3 Change |
|-------|-----------|-----------|
| `ChromosomeT` | All-in-one | Reduced to core: fitness, age, calculate_fitness |
| `LinearChromosome` | Does not exist | New supertrait of ChromosomeT; owns all flat-slice methods |
| `TreeChromosome` | Does not exist | New supertrait for tree-structured chromosomes |
| `GeneT` | Unchanged | Unchanged |
| `SelectionOperator` | 2-ary pairs | Unchanged; new `MultiParentSelectionOperator` added |
| `CrossoverOperator` | 2-ary | Unchanged; new `MultiParentCrossoverOperator` added |
| `MutationOperator` | global params | Unchanged; new `SelfAdaptiveMutationOperator` added |
| `SurvivorOperator` | Unchanged | Unchanged |
| `MultiCaseFitness` | Does not exist | New opt-in supertrait for lexicase |
| `SelfAdaptive` | Does not exist | New opt-in supertrait for self-adaptive mutation |
| `FitnessFn<G>` | `dyn Fn(&[G]) -> f64` | Unchanged; add `CaseFitnessFn<G>` and `TreeFitnessFn<G>` |
| `Reporter<U>` | Existing | REMOVED — soft-deprecated since v2.2.0, removed in v3.0.0 |

---

## New Components Needed

`src/types/tree/` module:
- `node.rs` — `TreeNode<G: GeneT>` with `value: G`, `node_type: NodeType`, `children: Vec<TreeNode<G>>`
- `chromosome.rs` — `GpChromosome<G>` implementing `ChromosomeT` + `TreeChromosome`
- `primitives.rs` — built-in function sets (arithmetic: +, -, *, %, sin, cos; boolean: and, or, not, if)

`src/operations/crossover/subtree.rs` — GP subtree crossover with max_depth enforcement

`src/operations/mutation/` tree variants:
- `subtree_mutation.rs`, `point_mutation.rs`, `hoist_mutation.rs`

`src/operations/crossover/multiparent/` — `undx.rs`, `spx.rs`, `pcx.rs`

`src/traits/multi_case_fitness.rs` — `MultiCaseFitness` trait + `CaseFitnessFn` type alias

`src/traits/self_adaptive.rs` — `SelfAdaptive` trait + `SelfAdaptiveMutationOperator`

`src/traits/tree_chromosome.rs` — `TreeChromosome` trait + `TreeFitnessFn` type alias

`src/operations/selection/lexicase.rs` — lexicase + epsilon-lexicase

`src/engines/strategy.rs` — `Strategy` trait unifying `Ga`, `HillClimb`, `Permutate`

`src/engines/hill_climb.rs` and `src/engines/permutate.rs` — new engines

`src/engines/gp_ga.rs` — GP engine (`GpGa<U: TreeChromosome>`)

`src/types/genotypes/unique.rs`, `multi_range.rs`, `multi_unique.rs` — new gene types

---

## Suggested Build Order

```
Phase 1: ChromosomeT split + LinearChromosome + API audit [MUST BE FIRST — blocks everything]
    |
    +-- Phase 2: ChromosomeLength enum + variable-length init infrastructure
    |       |
    |       +-- Phase 7: Multi-parent crossover (UNDX, SPX, PCX)
    |
    +-- Phase 3: New genotypes (Unique, MultiRange, MultiUnique)
    |
    +-- Phase 4: Strategy trait + HillClimb + Permutate engines
    |
    +-- Phase 5: Lexicase + MultiCaseFitness
    |
    +-- Phase 6: Self-adaptive mutation
    |
    +-- Phase 8: Tree Chromosome + GpGa engine [most novel, last]
```

Phase 1 is the highest-risk change — ~30 files modified mechanically. Strategy: single large PR doing `ChromosomeT` → `LinearChromosome` in operator bounds (mechanical), then the `ChromosomeT` trait reduction itself. Run CI after each commit. Phases 3-8 can be executed in parallel by different PRs once Phase 1 merges.

---

## API Simplification Opportunities

1. **Remove `Reporter<U>`** — soft-deprecated since v2.2.0; `GaObserver<U>` is the replacement. Clean removal in v3.0.0.

2. **Privatize `GaConfiguration` fields** — currently all `pub`, allowing bypass of builder validation. Apply `#[non_exhaustive]` and `pub(crate)` to fields; expose read accessors where needed.

3. **Remove `needs_unique_ids` / `alleles_can_be_repeated` from `LimitConfiguration`** — these are initialization concerns that belong in the initialization function signature, not in the engine configuration.

4. **Move fitness function out of chromosomes** — currently each chromosome stores an `Arc<FitnessFn>` clone (N copies for population_size N). Remove `set_fitness_fn` and `calculate_fitness` from `ChromosomeT`/`LinearChromosome`; engine calls `chromosome.set_fitness(fitness_fn(chromosome.dna()))` directly. This also enables checkpoint serialization (closures are not serializable, but fitness values are).

5. **Change `LocalSearchOperator` from `Arc<dyn ...>` to enum** — inconsistent with every other operator which uses enum + factory. `LocalSearch` enum already exists; use it as `Option<LocalSearch>` in config.

6. **Flatten `StoppingCriteria` into builder methods** — replace the separate `StoppingCriteria` struct field with direct builder methods: `.with_stagnation_limit(50)`, `.with_convergence_threshold(0.001)`.

---

## WASM Compatibility

| Component | Concern | Mitigation |
|-----------|---------|------------|
| `TreeNode` recursion | Stack depth for deep trees | Enforce max_depth ≤ 17 in config; iterative evaluation via explicit stack |
| Multi-parent crossover | `par_iter` in parent grouping | Gate `#[cfg(not(target_arch = "wasm32"))]` |
| HillClimb/Permutate time limit | `Instant::now()` | Gate same as `ga.rs` existing pattern |
| Variable-length init | `par_iter` already gated | New code follows existing pattern |
| Self-adaptive sigma | `rand` — wasm-safe | No action needed |

---

## Open Questions

- **Fitness function removal from chromosomes (API item 4):** Users who call `chromosome.calculate_fitness()` in isolation (outside a GA run, e.g., in tests or custom selection callbacks) would need to receive the fitness fn externally. Impact scope needs validation before committing.
- **Variable-length crossover semantics:** The SVLC and SAGA homologous crossover algorithms exist but are complex. The simpler approach (truncate to shorter parent length) loses genetic material. Needs a concrete recommendation validated against benchmark problems.
- **Tree bloat control beyond max_depth:** Parsimony pressure (adding tree size to fitness penalty) is a common complement. Whether this should be built-in to `GpGa` or delegated to the user fitness function needs a decision.
- **`GpGa` observer hooks:** `GaObserver` events like `on_crossover_complete` carry `population_size: usize` but not tree structure. A separate `GpObserver` sub-trait (like `IslandGaObserver`) may be needed for GP-specific events (`on_bloat_detected`, `on_depth_exceeded`).
- **`Strategy` trait scope vs alt-metaheuristic engines:** If `Strategy` wraps GA, HillClimb, and Permutate, does it also wrap DE, Scatter, Cellular, ALPS? This scope question should be resolved in the audit phase design document.

---

## Sources

- Direct source reading: `src/traits/chromosome.rs`, `src/traits/operators.rs`, `src/ga.rs`
- Lexicase selection analysis: PMC9453780, arXiv 1905.13266
- Multi-parent recombination: ACM 2933986, Water Programming Blog (SPX)
- Self-adaptation in ES: PubMed 11382357, algorithmafternoon.com
- Variable-length crossover: PLOS ONE journal.pone.0209712, ResearchGate 3418918
