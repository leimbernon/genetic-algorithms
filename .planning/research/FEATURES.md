# Features Research: v3.0.0

**Project:** genetic-algorithms v3.0.0
**Mode:** Ecosystem — Feature landscape for advanced EC capabilities
**Confidence:** HIGH (verified against DEAP docs, MOEA Framework, EC literature, existing `ChromosomeT`/operator trait signatures)

---

## Key Findings

1. **Tree chromosome is the most architecturally disruptive feature.** `ChromosomeT`'s `dna() -> &[Self::Gene]` contract is fundamentally incompatible with tree-structured DNA. A separate `GpEngine` and new `TreeChromosomeT` trait (not implementing `ChromosomeT`) is required.

2. **Lexicase selection must not touch `ChromosomeT::fitness() -> f64`.** The scalar fitness contract is depended on by survivor selection, stopping criteria, observer hooks, and `GenerationStats`. The correct approach is a secondary `MultiCaseFitnessT` trait that chromosomes can optionally implement, storing case scores alongside (not replacing) scalar fitness.

3. **Multi-parent crossover (UNDX/SPX/PCX) requires a new trait.** The existing `CrossoverOperator::crossover(&U, &U)` is hardwired for two parents. A `MultiParentCrossoverOperator::crossover_multi(&self, parents: &[&U])` trait is needed.

4. **Self-adaptive mutation is cleanest as a new `SelfAdaptiveT` trait** (`sigmas() -> &[f64]`, `sigmas_mut() -> &mut [f64]`). The existing `MutationOperator::mutate()` signature already accepts `sigma: Option<f64>` — self-adaptive mutation simply reads sigma from the chromosome via this trait. Full CMA-ES is out of scope.

5. **UniqueT, MultiRangeT, and Unified Strategy are low-risk additions** — entirely additive, no existing operator or trait changes needed.

---

## Per-Feature Analysis

---

### 1. Tree Chromosome for Genetic Programming (GP)

**Table Stakes (required for any GP system)**

| Behavior | Why Required |
|----------|-------------|
| PrimitiveSet: typed function and terminal registration | Users must declare what operations and inputs exist. Every mature GP library (DEAP, ECJ, MOEA Framework) provides this. |
| Ramped half-and-half initialization | Standard tree seeder alternating `grow` and `full`. Canonical since Koza 1992. |
| Subtree crossover | Swap randomly selected subtrees between two parents. The dominant GP recombination operator. |
| Subtree mutation (point, hoist, expansion, shrink) | Point replaces one node's primitive. Hoist replaces subtree with one of its own subtrees (size reduction). |
| Maximum depth limit | Without it, bloat makes trees non-evaluable. |
| Maximum node-count limit | Depth alone allows wide shallow trees that exhaust memory. |
| Tree evaluation: returns f64 scalar | The output for symbolic regression. |
| `Display`/`Debug` as expression string | Users need to read what evolved. |

**Differentiators**

| Behavior | Value |
|----------|-------|
| Ephemeral random constants (ERC) | Leaf nodes whose value is sampled once at creation and fixed. Essential for symbolic regression. |
| One-point crossover (respects shared tree structure) | Documented to significantly reduce bloat vs subtree crossover. |
| Strongly typed primitive set (return/arg types match) | Prevents type-invalid tree generation. Loosely-typed (homogeneous f64) as default; strongly-typed as opt-in. |
| `serde` serialization of TreeChromosome | Checkpoint/resume for long GP runs. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Strongly typed GP as the only option | Arity/type annotation on every primitive. Loosely typed GP covers symbolic regression much more simply. |
| Automatic type derivation / reflection | Requires runtime type info that does not exist cleanly in Rust. Too complex for v3.0.0. |
| Grammar-guided or grammatical evolution | Separate representation paradigm. Out of scope. |

**Complexity Notes:** Most architecturally disruptive feature. A separate `TreeChromosomeT` trait that does NOT require `ChromosomeT`'s linear DNA interface. Tree chromosomes opt out of all slice-based operators and get their own operator set. A dedicated `GpEngine` is the appropriate container. Bloat is the defining risk — all three countermeasures (depth limit + node-count limit + size-fair crossover) are table stakes.

Reference: DEAP `deap.gp` module, MOEA Framework `Program` chromosome type, `evco` Rust crate.

---

### 2. Variable-Length Chromosomes

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Insertion mutation (add a gene at a random position) | Primary growth mechanism. |
| Deletion mutation (remove a gene at a random position) | Primary shrink mechanism. |
| Min/max length bounds enforced by configuration | Without bounds, chromosomes degenerate to length 0 or unbounded growth. |
| Crossover handles unequal parent lengths without panic | Standard single-point crossover on unequal lengths is valid; must not index out of bounds. |

**Differentiators**

| Behavior | Value |
|----------|-------|
| Homologous crossover (aligns parents by positional similarity before crossing) | Prevents "building block disruption" that naive variable-length crossover causes. |
| Parsimony pressure configuration option | Without it, chromosomes grow without fitness gain (analogous to GP bloat). |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Automatic alignment via sequence-alignment algorithms (Smith-Waterman) | O(n²) with high constant. Prohibitively expensive in hot evaluation loop. |
| Intron sequences (non-coding alignment helpers, NEAT-style) | Adds encoding complexity users must manage. NEAT introns are specific to neuro-evolution. |

**Complexity Notes:** No new trait required. `VariableLengthConfig` fields (min_length, max_length) in configuration are sufficient. New operators needed: `Mutation::Insertion` and `Mutation::Deletion`. The primary challenge is operator correctness: all 9 existing crossover operators need an audit for equal-length assumptions.

---

### 3. Lexicase Selection

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Per-individual, per-case score vector (not a scalar fitness) | The defining characteristic. Without this, lexicase degenerates to random selection. |
| Random case shuffling per selection event | The shuffle is what produces the diversity-preserving behavior. |
| Elite filtering (keep only those equal to best on each case) | The canonical algorithm. |
| Returns a single selected individual index per selection event | Conforms to `SelectionOperator` pattern. |

**Differentiators**

| Behavior | Value |
|----------|-------|
| Epsilon-lexicase (ε-lexicase) | Filter keeps all individuals within ε of best per case. Required for continuous-valued scores. GECCO 2015. Highly recommended to implement alongside base. |
| Down-sampled lexicase | Use a random subset of cases per generation. Reduces cost from O(N×C) to O(N×k). |
| Weighted shuffle | Weight test cases by difficulty to bias early-order filtering toward informative cases. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Modifying `ChromosomeT::fitness()` to return `Vec<f64>` | BREAKS the scalar contract depended on by survivor selection, stopping criteria, GaObserver, and GenerationStats. Correct approach: secondary `MultiCaseFitnessT` trait. |
| Storing case scores inside the chromosome struct itself | Bloats every chromosome type with data only used by one selection operator. Case scores belong in a side-table keyed by population index. |

**Complexity Notes:** The correct design: a `MultiCaseFitnessT` trait (`case_scores() -> &[f64]`, `set_case_scores(Vec<f64>)`) that chromosomes can optionally implement. `LexicaseSelection` requires `U: MultiCaseFitnessT`. The scalar `fitness()` can be set to the mean case score for compatibility with survivor selection and stopping criteria.

Reference: `ryanboldi/lexicase`, DEAP `selLexicase`/`selEpsilonLexicase`, MIT Press analysis.

---

### 4. Multi-Parent Crossover: UNDX, SPX, PCX

**Operator summaries:**
- **UNDX:** Offspring centered at centroid of primary parent pair, normally distributed along the inter-parent direction, perpendicular variance scaled by remaining parents. Minimum 3 parents.
- **SPX:** Parents define a simplex. Offspring sampled uniformly from within the simplex expanded by ε. Arbitrary parent count k, default k = n_dimensions+1.
- **PCX:** Offspring centered around a specific parent (not centroid), with normal perturbation in direction of other parents. More exploitative than UNDX/SPX.

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Accept `&[&U]` (slice of parent references) rather than exactly two | The defining structural requirement. |
| Configurable parent count `k` | SPX and UNDX benefit from k > 3. Default k=3 acceptable; must be user-configurable. |
| Real-valued domain only | These operators are geometrically defined for real-valued (f64) chromosomes only. |
| Gene bounds enforcement post-crossover (clamp or reflect) | Offspring values must stay within domain. |
| Return 2 offspring per call | Consistent with existing crossover behavior. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Reusing the existing two-parent `CrossoverOperator::crossover(&U, &U)` signature | Cannot accommodate 3+ parents. Must introduce a new trait in v3.0.0. |
| Applying to binary or permutation chromosomes | Geometrically undefined. Must return a clear error, not silently produce garbage. |

**Complexity Notes:** A new `MultiParentCrossoverOperator::crossover_multi(&self, parents: &[&U]) -> Result<Vec<U>, GaError>` trait is needed. Existing two-parent operators stay on the old trait. The engine's crossover step requires selecting parent tuples of size k — touches `SelectionOperator::select()` return type (currently `Vec<(usize, usize)>`).

Reference: MOEA Framework UNDX and SPX Javadocs, Deb et al. 2002.

---

### 5. Self-Adaptive Mutation

**Update rule:** `σ'_i = σ_i × exp(τ × N(0,1) + τ' × N_i(0,1))` where τ = 1/sqrt(2n), τ' = 1/sqrt(2·sqrt(n)) for per-gene sigmas (Bäck 1996 rule of thumb).

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Per-chromosome sigma vector (one σ per gene, or one global σ) | The defining characteristic. |
| Log-normal mutation of sigma before gene mutation | The canonical update rule from Evolution Strategies. |
| Sigma lower bound (σ_min > 0, e.g. 1e-5) | Without a minimum, sigma collapses to 0, halting all mutation. |
| Strategy parameters inherited via crossover (intermediate recombination of sigmas) | When two parents cross, offspring sigmas are averaged. |
| Sigma stored in the chromosome, not in global config | Architectural requirement that distinguishes self-adaptation from library-level adaptive GA. |

**Differentiators**

| Behavior | Value |
|----------|-------|
| Per-gene sigma (n sigmas) vs. single global sigma (1 sigma) | Per-gene more powerful but n extra floats per chromosome. Offer both; default to global sigma. |
| Observer reporting of per-generation sigma statistics (mean/min/max) | Useful diagnostic for convergence analysis. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Full CMA-ES (covariance matrix adaptation) | CMA-ES is a standalone algorithm, not an operator. O(n²) storage and update. Future milestone scope. |
| Encoding sigma outside the chromosome (parallel Vec) | Breaks encapsulation. Sigma must be in the chromosome to be naturally inherited via crossover. |

**Complexity Notes:** Best implemented as a new `SelfAdaptiveT` trait: `sigmas() -> &[f64]` and `sigmas_mut() -> &mut [f64]`. The new `Mutation::SelfAdaptiveGaussian` variant requires `U: ChromosomeT + SelfAdaptiveT`. No breaking change to `MutationOperator` needed.

---

### 6. UniqueT Genotype (Permutation Representation)

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Initialization guarantees a valid permutation (no duplicates) | Fundamental invariant. |
| PMX and OX documented as the safe crossover operators | Both already in the library and correct for permutations. |
| Swap and Inversion mutation documented as safe | Both already in the library and permutation-preserving. |
| Validation: `is_valid()` check (no duplicates) | Useful for debug assertions and testing. |

**Differentiators**

| Behavior | Value |
|----------|-------|
| ERX (Edge Recombination Crossover) explicitly documented for TSP | Already in library (v2.4.0). Documentation clarification. |
| `Unique<T>` stores an explicit alphabet (arbitrary valid values, not just 0..n) | Allows non-contiguous value sets (e.g., job IDs are not necessarily 0..n). |
| Insertion mutation (move one element to a different position) | Different from swap; preserves relative order of unaffected elements. Useful for scheduling. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Allowing non-permutation crossovers (single-point, uniform, arithmetic) on UniqueT | These break the permutation invariant. Runtime error or trait-bound rejection required. |

**Complexity Notes:** Low complexity. The existing `job_scheduling` example uses `RangeChromosome<i32>` with unique initialization — flagged in PROJECT.md Key Decisions as "revisit." UniqueT is the semantic and enforced replacement.

Reference: Cicirello's comprehensive review of permutation genetic operators.

---

### 7. MultiRangeT Genotype

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Per-gene `(lo_i, hi_i)` bounds stored in the chromosome/configuration | Defining requirement. |
| Per-gene mutation rate `p_i` | Enables partial-chromosome mutation with per-gene probability. |
| Initialization samples each gene from its own range | Required for valid initial population. |
| Gaussian mutation respects per-gene bounds (clamp or reflect) | Without bounds enforcement, offspring violate domain constraints. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Mixed integer/float genes at the type level | Requires `dyn Any` — expensive and breaks rayon `Send+Sync`. Keep all genes as `f64` or `T: Copy + From<f64>`. |

**Complexity Notes:** `Range<T>` already uses `Arc<[(T,T)]>` shared bounds slice — `MultiRange<T>` extends this to per-gene distinct bounds. Per-gene mutation rates (`p_i: Vec<f64>`) stored in chromosome metadata. Medium complexity.

---

### 8. MultiUniqueT Genotype

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| Each sub-sequence independently valid (no duplicates within each group) | Defining invariant. |
| Crossover preserves sub-sequence group boundaries | Operator must know where each permutation segment begins/ends in the flat DNA. |
| Per-group initialization from its own alphabet | Each group initializes independently. |
| PMX/OX applied per-group, not across group boundaries | The dominant correct behavior for multi-group permutations. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| Flattening multiple permutations into a single Vec and trusting operators not to cross group boundaries | Fragile. Every operator must know the group boundaries explicitly. `GroupSpec` metadata is required. |

**Complexity Notes:** Follows the same pattern as UniqueT but adds `groups: Vec<GroupSpec>` metadata (alphabet + start index + length per group). Dependency: builds on UniqueT infrastructure. Should be implemented after UniqueT.

---

### 9. Unified Strategy Trait

**Table Stakes**

| Behavior | Why Required |
|----------|-------------|
| `run() -> Result<StrategyResult<U>, GaError>` common method | A unified entrypoint returning best individual and statistics. |
| `best() -> Option<&U>` to retrieve current best | Standard query on all engines post-run. |
| HillClimb Stochastic variant: accept any uphill move, probabilistic | Canonical Stochastic Hill Climbing. |
| HillClimb SteepestAscent variant: evaluate all neighbors, accept best | Requires user-provided `neighbor_fn: Fn(&U) -> Vec<U>`. |
| Permutate: exhaustive enumeration over finite search space | Returns globally optimal individual. Useful only for small search spaces. |
| GaObserver hooks in HillClimb and Permutate | Consistent observability with all other engines. |

**Anti-Features**

| Anti-Feature | Why Avoid |
|--------------|-----------|
| A `Strategy` enum (GA/HillClimb/Permutate variants) | Breaks extensibility — third-party strategy implementations would be impossible. Trait is the right abstraction. |
| Forcing `GenerationStats` onto HillClimb iterations | HillClimb has no crossover or generations — pushing GA stats struct onto it produces confusing empty fields. |

---

## Feature Dependencies

```
TreeChromosome (#223)
  — BREAKS ChromosomeT linear DNA interface
  — Requires: new GpEngine (cannot use Ga<U> directly)
  — Requires: new TreeCrossoverOperator, TreeMutationOperator traits
  — Requires: new PrimitiveSet / TerminalSet registration API
  — Must ship last — highest complexity, no dependencies on other v3 features

VariableLengthChromosome (#224)
  — Requires: audit of all 9 crossover operators for equal-length assumption
  — Requires: new Mutation::Insertion, Mutation::Deletion operators
  — Requires: VariableLengthConfig (min_length, max_length) in configuration
  — Compatible with UniqueT (variable-length permutations)

LexicaseSelection (#220)
  — BREAKS scalar fitness if misdesigned; correct approach = MultiCaseFitnessT trait
  — Requires: new MultiCaseFitnessT trait (case_scores() / set_case_scores())
  — Does NOT require any other v3 feature

MultiParentCrossover (#221) — UNDX, SPX, PCX
  — Requires: new MultiParentCrossoverOperator trait (existing 2-parent signature insufficient)
  — SelectionOperator::select() return type (Vec<(usize,usize)>) may need evolution to Vec<Vec<usize>>
  — Real-valued chromosomes only (RangeChromosome<f64>)

SelfAdaptiveMutation (#222)
  — Requires: new SelfAdaptiveT trait (sigmas() / sigmas_mut())
  — Requires: new Mutation::SelfAdaptiveGaussian enum variant
  — Real-valued chromosomes only

UniqueT (#174)
  — Additive: new chromosome type only
  — Completes: job_scheduling example migration

MultiRangeT (#175)
  — Additive: new chromosome type
  — Can share per-gene metadata infrastructure with SelfAdaptiveT

MultiUniqueT (#176)
  — Depends on UniqueT (shares permutation infrastructure)
  — Requires: GroupSpec metadata + group-aware operator variants

UnifiedStrategy (#177)
  — Additive: new Strategy<U> trait + HillClimbEngine + PermutateEngine
  — SteepestAscent requires: user-provided neighbor_fn callback
```

**Recommended phase ordering:**

1. UniqueT, MultiRangeT — no dependencies, low risk, ships first
2. Unified Strategy (HillClimb, Permutate) — independent, additive
3. MultiUniqueT — depends on UniqueT infrastructure
4. Variable-Length Chromosomes — requires operator audit
5. Lexicase Selection — requires MultiCaseFitnessT design decision
6. Multi-Parent Crossover + Self-Adaptive Mutation — pair together; require new traits
7. Tree Chromosome — most complex, ships last; requires dedicated GpEngine design

---

## Open Questions

1. **LexicaseSelection + SelectionOperator signature:** Does adding `MultiCaseFitnessT` as a bound on `U` in the lexicase impl require changing the `Selection` enum's factory function, or can it remain generic with a runtime check?

2. **MultiParentCrossover + engine plumbing:** Does `SelectionOperator::select()` return `Vec<Vec<usize>>` or does the engine make multiple `select()` calls to assemble k parents?

3. **TreeChromosome + GaObserver:** Which existing `GaObserver` hooks are meaningful for GP? Should `GpEngine` implement `GaObserver` hooks or a separate `GpObserver` sub-trait?

4. **UniqueT alphabet storage:** Should the alphabet be stored in the chromosome or in the configuration? Chromosome storage allows each individual to carry its own alphabet (useful for MultiUniqueT); configuration storage is cheaper but less flexible.

5. **SteepestAscent neighbor generation:** Should `HillClimbEngine` require a user-provided `neighbor_fn`, or should the library provide default neighbor generators for `UniqueChromosome` (all single swaps) and `RangeChromosome` (grid perturbations)?
